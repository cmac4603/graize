use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::worker::generated::{
    SessionCreateRequest, SessionMessageAssistantContentItemUnion, SessionMessageInfo,
    SessionPromptRequest,
};

use super::generated::client::HttpClient;

pub struct WorkerOpencodeSession {
    client: HttpClient,
    session_id: String,
}

pub trait AgentSession: Sized {
    async fn new(base_url: String) -> Result<Self>;
    async fn prompt(&mut self, text: String) -> Result<()>;
    async fn wait_for_response(self) -> Result<String>;
}

impl AgentSession for WorkerOpencodeSession {
    async fn new(base_url: String) -> Result<Self> {
        let mut client = HttpClient::with_client(crate::http_client::shared_client().clone())
            .with_base_url(base_url);
        if let Ok(password) = std::env::var("OPENCODE_SERVER_PASSWORD") {
            let username =
                std::env::var("OPENCODE_SERVER_USERNAME").unwrap_or_else(|_| "opencode".to_owned());
            let credentials = STANDARD.encode(format!("{username}:{password}"));
            client = client.with_header("Authorization", format!("Basic {credentials}"));
        }
        let session_id = client
            .session_create(SessionCreateRequest::default())
            .await?
            .data
            .id;
        Ok(Self { client, session_id })
    }

    async fn prompt(&mut self, text: String) -> Result<()> {
        let request = SessionPromptRequest::new(text);
        self.client
            .session_prompt(&self.session_id, request)
            .await?;
        Ok(())
    }

    async fn wait_for_response(self) -> Result<String> {
        self.client
            .experimental_session_wait(&self.session_id)
            .await?;

        let context = self.client.session_context(&self.session_id).await?;
        let latest_assistant = context
            .data
            .into_iter()
            .rev()
            .find_map(|message| match message {
                SessionMessageInfo::SessionMessageAssistant(message) => Some(message),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("session context has no assistant response"))?;

        let response = latest_assistant
            .content
            .into_iter()
            .filter_map(|item| match item {
                SessionMessageAssistantContentItemUnion::SessionMessageAssistantText(text) => {
                    Some(text.text)
                }
                _ => None,
            })
            .collect::<String>();

        if response.is_empty() {
            anyhow::bail!("latest assistant message has no text response");
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::Mutex,
        thread,
    };

    use super::{AgentSession, WorkerOpencodeSession};
    use crate::worker::generated::client::HttpClient;

    static WORKER_AUTH_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn session_creation_uses_password_env_and_default_username() {
        let _env_lock = WORKER_AUTH_ENV_LOCK.lock().unwrap();
        let _auth_env = WorkerAuthEnv::with_password("secret");

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            assert!(request.starts_with("POST /api/session "));
            let auth_header = request
                .lines()
                .find(|line| line.to_ascii_lowercase().starts_with("authorization:"))
                .unwrap();
            assert_eq!(
                auth_header.split_once(':').unwrap().1.trim(),
                "Basic b3BlbmNvZGU6c2VjcmV0"
            );
            let body = r#"{"data":{"cost":0.0,"id":"session-id","location":{"directory":"/workspace"},"projectID":"project-id","time":{"created":1.0,"updated":1.0},"tokens":{"cache":{"read":0.0,"write":0.0},"input":0.0,"output":0.0,"reasoning":0.0}}}"#;
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                    .as_bytes(),
                )
                .unwrap();
        });

        let session = WorkerOpencodeSession::new(format!("http://{address}"))
            .await
            .unwrap();

        assert_eq!(session.session_id, "session-id");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn wait_for_response_returns_latest_assistant_text_after_waiting() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut wait_stream, _) = listener.accept().unwrap();
            let wait_request = read_request(&mut wait_stream);
            assert!(wait_request.starts_with("POST /api/experimental/session/session-id/wait "));
            wait_stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();

            let (mut context_stream, _) = listener.accept().unwrap();
            let context_request = read_request(&mut context_stream);
            assert!(context_request.starts_with("GET /api/session/session-id/context "));
            let body = r#"{"data":[{"agent":"build","content":[{"text":"Done.","type":"text"}],"id":"message-id","model":{"id":"model-id","providerID":"provider-id"},"time":{"created":1.0,"completed":2.0},"type":"assistant"}]}"#;
            write!(
                context_stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });

        let session = WorkerOpencodeSession {
            client: HttpClient::new().with_base_url(format!("http://{address}")),
            session_id: "session-id".to_owned(),
        };

        let response = session.wait_for_response().await.unwrap();

        assert_eq!(response, "Done.");
        server.join().unwrap();
    }

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();
            assert_ne!(bytes_read, 0, "request ended before headers completed");
            request.extend_from_slice(&buffer[..bytes_read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                return String::from_utf8(request).unwrap();
            }
        }
    }

    struct WorkerAuthEnv {
        original_password: Option<std::ffi::OsString>,
        original_username: Option<std::ffi::OsString>,
    }

    impl WorkerAuthEnv {
        fn with_password(password: &str) -> Self {
            let original_password = std::env::var_os("OPENCODE_SERVER_PASSWORD");
            let original_username = std::env::var_os("OPENCODE_SERVER_USERNAME");
            unsafe {
                std::env::set_var("OPENCODE_SERVER_PASSWORD", password);
                std::env::remove_var("OPENCODE_SERVER_USERNAME");
            }
            Self {
                original_password,
                original_username,
            }
        }
    }

    impl Drop for WorkerAuthEnv {
        fn drop(&mut self) {
            unsafe {
                match self.original_password.take() {
                    Some(value) => std::env::set_var("OPENCODE_SERVER_PASSWORD", value),
                    None => std::env::remove_var("OPENCODE_SERVER_PASSWORD"),
                }
                match self.original_username.take() {
                    Some(value) => std::env::set_var("OPENCODE_SERVER_USERNAME", value),
                    None => std::env::remove_var("OPENCODE_SERVER_USERNAME"),
                }
            }
        }
    }
}

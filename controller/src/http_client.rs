use std::{sync::OnceLock, time::Duration};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Return the controller's process-wide HTTP client.
///
/// Cloned clients share reqwest's connection pool. The overall request timeout
/// is intentionally unset because the worker API includes long-lived streams.
pub(crate) fn shared_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .user_agent(concat!("graize-controller/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("controller HTTP client configuration is valid")
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use super::shared_client;

    #[test]
    fn returns_the_same_shared_client() {
        assert!(std::ptr::eq(shared_client(), shared_client()));
    }

    #[tokio::test]
    async fn sends_the_controller_user_agent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let bytes_read = stream.read(&mut buffer).unwrap();
                assert_ne!(bytes_read, 0, "request ended before headers completed");
                request.extend_from_slice(&buffer[..bytes_read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
            String::from_utf8(request).unwrap()
        });

        let response = shared_client()
            .get(format!("http://{address}"))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let request = server.join().unwrap().to_ascii_lowercase();
        let expected_user_agent = format!(
            "user-agent: graize-controller/{}",
            env!("CARGO_PKG_VERSION")
        );
        assert!(request.contains(&expected_user_agent));
    }
}

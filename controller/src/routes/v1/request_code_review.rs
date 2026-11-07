use axum::http::StatusCode;

/// Get status of a code review.
pub async fn get() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

/// Request a code review.
/// Deploys a worker container with the repo contents on git_ref,
/// starts the agent with a prompt, and returns the session_id.
pub async fn post() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    #[tokio::test]
    async fn get_request_code_review() {
        assert_eq!(
            super::get().await,
            StatusCode::NOT_IMPLEMENTED
        );
    }

    #[tokio::test]
    async fn post_request_code_review() {
        assert_eq!(
            super::post().await,
            StatusCode::NOT_IMPLEMENTED
        );
    }
}

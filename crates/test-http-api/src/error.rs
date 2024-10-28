use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// radicle_types error.
    #[error(transparent)]
    Types(#[from] radicle_types::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            other => {
                if cfg!(debug_assertions) {
                    (StatusCode::INTERNAL_SERVER_ERROR, Some(other.to_string()))
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, None)
                }
            }
        };

        let body = Json(json!({
            "error": msg.or_else(|| status.canonical_reason().map(|r| r.to_string())),
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

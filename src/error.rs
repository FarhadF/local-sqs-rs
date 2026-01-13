use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub enum SqsError {
    QueueNameExists,
    QueueDoesNotExist,
    InvalidParameterValue(String),
    InvalidAction(String),
    // ... other errors
}

impl IntoResponse for SqsError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            SqsError::QueueNameExists => (
                StatusCode::BAD_REQUEST,
                "QueueNameExists",
                "A queue with this name already exists.".to_string(),
            ),
            SqsError::QueueDoesNotExist => (
                StatusCode::BAD_REQUEST,
                "QueueDoesNotExist",
                "The specified queue does not exist.".to_string(),
            ),
            SqsError::InvalidParameterValue(msg) => {
                (StatusCode::BAD_REQUEST, "InvalidParameterValue", msg)
            }
            SqsError::InvalidAction(action) => (
                StatusCode::BAD_REQUEST,
                "InvalidAction",
                format!("Invalid action: {}", action),
            ),
        };

        let body = Json(json!({
            "__type": error_code,
            "message": message,
        }));

        (status, body).into_response()
    }
}

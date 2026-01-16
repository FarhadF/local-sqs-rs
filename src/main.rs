use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{extract::State, Json, Router};
use tracing::info;

mod error;
mod queue;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/", post(handler))
        .with_state(state.clone());

    let addr = format!("{}:{}", state.host, state.port);
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    let target = headers
        .get("X-Amz-Target")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    info!(target);
    info!(body);

    match target {
        "AmazonSQS.CreateQueue" => {
            let request: queue::CreateQueueRequest = serde_json::from_str(&body).unwrap();
            match queue::create_queue(State(state), Json(request)).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.GetQueueUrl" => {
            let request: queue::GetQueueUrlRequest = serde_json::from_str(&body).unwrap();
            match queue::get_queue_url(State(state), Json(request)).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.ListQueues" => {
            let request: queue::ListQueuesRequest = serde_json::from_str(&body).unwrap();
            let response = queue::list_queues(State(state), Json(request)).await;
            Json(response).into_response()
        }
        "AmazonSQS.DeleteQueue" => {
            let request: queue::DeleteQueueRequest = serde_json::from_str(&body).unwrap();
            match queue::delete_queue(State(state), Json(request)).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.PurgeQueue" => {
            let request: queue::PurgeQueueRequest = serde_json::from_str(&body).unwrap();
            match queue::purge_queue(State(state), Json(request)).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.GetQueueAttributes" => {
            let request: queue::GetQueueAttributesRequest = serde_json::from_str(&body).unwrap();
            match queue::get_queue_attributes(State(state), Json(request)).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.SendMessage" => {
            let request: queue::SendMessageRequest = serde_json::from_str(&body).unwrap();
            match queue::send_message(State(state), Json(request)).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.ReceiveMessage" => {
            let request: queue::ReceiveMessageRequest = serde_json::from_str(&body).unwrap();
            match queue::receive_message(State(state), Json(request)).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.DeleteMessage" => {
            let request: queue::DeleteMessageRequest = serde_json::from_str(&body).unwrap();
            match queue::delete_message(State(state), Json(request)).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.SetQueueAttributes" => {
            let request: queue::SetQueueAttributesRequest = serde_json::from_str(&body).unwrap();
            match queue::set_queue_attributes(State(state), Json(request)).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.AddPermission" => {
            let request: queue::AddPermissionRequest = serde_json::from_str(&body).unwrap();
            match queue::add_permission(State(state), Json(request)).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        _ => {
            let error = error::SqsError::InvalidAction(target.to_string());
            error.into_response()
        }
    }
}
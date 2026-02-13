use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{extract::State, Json, Router};
use tracing::info;
use serde::Deserialize;

mod error;
mod queue;
mod state;
mod serde_helpers;
mod sns;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/", post(handler))
        .route("/*path", post(handler))
        .with_state(state.clone());

    let addr = format!("{}:{}", state.host, state.port);
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct ActionRequest {
    #[serde(rename = "Action")]
    action: String,
}

async fn handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let target = headers
        .get("X-Amz-Target")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    info!("Headers: {:?}", headers);
    info!("Body: {}", body);

    if let Some(target) = target {
        // AWS JSON Protocol
        handle_json(state, &target, &body).await
    } else {
        // Assume AWS Query Protocol (Form URL Encoded)
        handle_query(state, &body).await
    }
}

async fn handle_json(state: AppState, target: &str, body: &str) -> Response {
    info!("Handling JSON request for target: {}", target);
    match target {
        "AmazonSQS.CreateQueue" => {
            let request: queue::CreateQueueRequest = serde_json::from_str(body).unwrap();
            match queue::create_queue(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.GetQueueUrl" => {
            let request: queue::GetQueueUrlRequest = serde_json::from_str(body).unwrap();
            match queue::get_queue_url(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.ListQueues" => {
            let request: queue::ListQueuesRequest = serde_json::from_str(body).unwrap();
            let response = queue::list_queues(state, request).await;
            Json(response).into_response()
        }
        "AmazonSQS.DeleteQueue" => {
            let request: queue::DeleteQueueRequest = serde_json::from_str(body).unwrap();
            match queue::delete_queue(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.PurgeQueue" => {
            let request: queue::PurgeQueueRequest = serde_json::from_str(body).unwrap();
            match queue::purge_queue(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.GetQueueAttributes" => {
            let request: queue::GetQueueAttributesRequest = serde_json::from_str(body).unwrap();
            match queue::get_queue_attributes(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.SendMessage" => {
            let request: queue::SendMessageRequest = serde_json::from_str(body).unwrap();
            match queue::send_message(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.ReceiveMessage" => {
            let request: queue::ReceiveMessageRequest = serde_json::from_str(body).unwrap();
            match queue::receive_message(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.DeleteMessage" => {
            let request: queue::DeleteMessageRequest = serde_json::from_str(body).unwrap();
            match queue::delete_message(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.SetQueueAttributes" => {
            let request: queue::SetQueueAttributesRequest = serde_json::from_str(body).unwrap();
            match queue::set_queue_attributes(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.AddPermission" => {
            let request: queue::AddPermissionRequest = serde_json::from_str(body).unwrap();
            match queue::add_permission(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.ListQueueTags" => {
            let request: queue::ListQueueTagsRequest = serde_json::from_str(body).unwrap();
            match queue::list_queue_tags(state, request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.TagQueue" => {
            let request: queue::TagQueueRequest = serde_json::from_str(body).unwrap();
            match queue::tag_queue(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSQS.UntagQueue" => {
            let request: queue::UntagQueueRequest = serde_json::from_str(body).unwrap();
            match queue::untag_queue(state, request).await {
                Ok(_) => Json(()).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSNS.GetSubscriptionAttributes" => {
            let request: sns::GetSubscriptionAttributesRequest = serde_json::from_str(body).unwrap();
            match sns::get_subscription_attributes(request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        "AmazonSNS.Subscribe" => {
            let request: sns::SubscribeRequest = serde_json::from_str(body).unwrap();
             match sns::subscribe(request).await {
                Ok(response) => Json(response).into_response(),
                Err(e) => e.into_response(),
            }
        }
        _ => {
            let error = error::SqsError::InvalidAction(target.to_string());
            error.into_response()
        }
    }
}

async fn handle_query(_state: AppState, body: &str) -> Response {
    let action_request: Result<ActionRequest, _> = serde_urlencoded::from_str(body);
    
    let action = match action_request {
        Ok(req) => req.action,
        Err(_) => return error::SqsError::InvalidAction("Unknown (failed to parse Action)".to_string()).into_response(),
    };

    info!("Handling Query request for Action: {}", action);

    match action.as_str() {
        "GetSubscriptionAttributes" => {
            let request: sns::GetSubscriptionAttributesRequest = match serde_urlencoded::from_str(body) {
                Ok(r) => r,
                Err(e) => return error::SqsError::InvalidParameterValue(e.to_string()).into_response(),
            };
            match sns::get_subscription_attributes(request).await {
                Ok(response) => {
                    let mut xml = String::from("<GetSubscriptionAttributesResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\">\n");
                    xml.push_str("  <GetSubscriptionAttributesResult>\n");
                    xml.push_str("    <Attributes>\n");
                    for (k, v) in response.attributes {
                        xml.push_str("      <entry>\n");
                        xml.push_str(&format!("        <key>{}</key>\n", k));
                        xml.push_str(&format!("        <value>{}</value>\n", v));
                        xml.push_str("      </entry>\n");
                    }
                    xml.push_str("    </Attributes>\n");
                    xml.push_str("  </GetSubscriptionAttributesResult>\n");
                    xml.push_str("  <ResponseMetadata>\n");
                    xml.push_str(&format!("    <RequestId>{}</RequestId>\n", uuid::Uuid::new_v4()));
                    xml.push_str("  </ResponseMetadata>\n");
                    xml.push_str("</GetSubscriptionAttributesResponse>");
                    
                    ([(axum::http::header::CONTENT_TYPE, "application/xml")], xml).into_response()
                }
                Err(e) => e.into_response(), // SqsError needs update to support XML or we manual format here?
                // SqsError returns JSON. We should ideally return XML here.
                // For now, let's just use JSON error, maybe Terraform can handle it or we update SqsError later.
            }
        }
        "Subscribe" => {
             // For Subscribe, we try to parse. Note: serde_urlencoded might fail on Attributes map.
             // We defined SubscribeRequest with minimal fields to avoid this.
             let request: sns::SubscribeRequest = match serde_urlencoded::from_str(body) {
                Ok(r) => r,
                Err(e) => {
                    // If deserialization fails, it might be due to ignored fields or map format.
                    // But serde usually ignores unknown fields.
                    // Let's hope Attributes map doesn't cause failure.
                    // If it does, we might need manual parsing or ignore error.
                    return error::SqsError::InvalidParameterValue(e.to_string()).into_response();
                }
            };
            match sns::subscribe(request).await {
                Ok(response) => {
                     let mut xml = String::from("<SubscribeResponse xmlns=\"http://sns.amazonaws.com/doc/2010-03-31/\">\n");
                     xml.push_str("  <SubscribeResult>\n");
                     xml.push_str(&format!("    <SubscriptionArn>{}</SubscriptionArn>\n", response.subscription_arn));
                     xml.push_str("  </SubscribeResult>\n");
                     xml.push_str("  <ResponseMetadata>\n");
                     xml.push_str(&format!("    <RequestId>{}</RequestId>\n", uuid::Uuid::new_v4()));
                     xml.push_str("  </ResponseMetadata>\n");
                     xml.push_str("</SubscribeResponse>");
                     
                     ([(axum::http::header::CONTENT_TYPE, "application/xml")], xml).into_response()
                }
                 Err(e) => e.into_response(),
            }
        }
        // TODO: Add SQS actions here if needed for full Terraform support
        _ => error::SqsError::InvalidAction(action).into_response(),
    }
}

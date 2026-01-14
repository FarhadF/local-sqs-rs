use crate::error::SqsError;
use crate::state::{AppState, Queue};
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueRequest {
    pub queue_name: String,
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateQueueResponse {
    pub queue_url: String,
}

pub async fn create_queue(
    State(state): State<AppState>,
    Json(request): Json<CreateQueueRequest>,
) -> Result<CreateQueueResponse, SqsError> {
    let queue_name = request.queue_name;
    let queue_url = format!("http://{}:{}/{}", state.host, state.port, &queue_name);

    if let Some(existing_queue) = state.queues.get(&queue_url) {
        if existing_queue.attributes != request.attributes {
            return Err(SqsError::QueueNameExists);
        } else {
            return Ok(CreateQueueResponse {
                queue_url: existing_queue.url.clone(),
            });
        }
    }

    let new_queue = Queue {
        name: queue_name,
        url: queue_url.clone(),
        messages: Default::default(),
        attributes: request.attributes,
        created_timestamp: Utc::now(),
    };

    state.queues.insert(queue_url.clone(), new_queue);
    Ok(CreateQueueResponse { queue_url })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueUrlRequest {
    pub queue_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueUrlResponse {
    pub queue_url: String,
}

pub async fn get_queue_url(
    State(state): State<AppState>,
    Json(request): Json<GetQueueUrlRequest>,
) -> Result<GetQueueUrlResponse, SqsError> {
    let queue_name = request.queue_name;
    let queue_url = format!("http://{}:{}/{}", state.host, state.port, queue_name);

    if state.queues.contains_key(&queue_url) {
        Ok(GetQueueUrlResponse { queue_url })
    } else {
        Err(SqsError::QueueDoesNotExist)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueuesRequest {
    pub queue_name_prefix: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListQueuesResponse {
    pub queue_urls: Vec<String>,
}

pub async fn list_queues(
    State(state): State<AppState>,
    Json(request): Json<ListQueuesRequest>,
) -> ListQueuesResponse {
    let mut queue_urls: Vec<String> = Vec::new();
    for queue in state.queues.iter() {
        if let Some(prefix) = &request.queue_name_prefix {
            if queue.value().name.starts_with(prefix) {
                queue_urls.push(queue.value().url.clone());
            }
        } else {
            queue_urls.push(queue.value().url.clone());
        }
    }


    ListQueuesResponse { queue_urls }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueAttributesRequest {
    pub queue_url: String,
    pub attribute_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetQueueAttributesResponse {
    pub attributes: HashMap<String, String>,
}

pub async fn get_queue_attributes(
    State(state): State<AppState>,
    Json(request): Json<GetQueueAttributesRequest>,
) -> Result<GetQueueAttributesResponse, SqsError> {
    match state.queues.get(&request.queue_url) {
        Some(queue) => {
            let mut attributes = HashMap::new();
            if let Some(attribute_names) = &request.attribute_names {
                for attr_name in attribute_names {
                    if let Some(value) = queue.attributes.get(attr_name) {
                        attributes.insert(attr_name.clone(), value.clone());
                    }
                }
            } else {
                attributes = queue.attributes.clone();
            }
            Ok(GetQueueAttributesResponse { attributes })
        }
        None => Err(SqsError::QueueDoesNotExist),
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteQueueRequest {
    pub queue_url: String,
}

pub async fn delete_queue(
    State(state): State<AppState>,
    Json(request): Json<DeleteQueueRequest>,
) -> Result<(), SqsError> {
    if state.queues.remove(&request.queue_url).is_none() {
        return Err(SqsError::QueueDoesNotExist);
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PurgeQueueRequest {
    pub queue_url: String,
}

pub async fn purge_queue(
    State(state): State<AppState>,
    Json(request): Json<PurgeQueueRequest>,
) -> Result<(), SqsError> {
    match state.queues.get_mut(&request.queue_url) {
        Some(mut queue) => {
            queue.messages.clear();
            Ok(())
        }
        None => Err(SqsError::QueueDoesNotExist),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageRequest {
    pub queue_url: String,
    pub message_body: String,
    #[serde(default)]
    pub message_attributes: HashMap<String, crate::state::MessageAttributeValue>,
    #[serde(default)]
    pub delay_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SendMessageResponse {
    pub message_id: String,
    pub md5_of_message_body: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    Json(request): Json<SendMessageRequest>,
) -> Result<SendMessageResponse, SqsError> {
    match state.queues.get_mut(&request.queue_url) {
        Some(mut queue) => {
            let message = crate::state::Message::new(
                request.message_body,
                HashMap::new(),
                request.message_attributes,
                request.delay_seconds,
            );

            let resp = SendMessageResponse {
                message_id: message.id.clone(),
                md5_of_message_body: message.md5_of_body.clone(),
            };
            queue.messages.push_back(message);

            Ok(resp)
        }
        None => Err(SqsError::QueueDoesNotExist),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteMessageRequest {
    pub queue_url: String,
    pub receipt_handle: String,
}

pub async fn delete_message(
    State(state): State<AppState>,
    Json(request): Json<DeleteMessageRequest>,
) -> Result<(), SqsError> {
    match state.queues.get_mut(&request.queue_url) {
        Some(mut queue) => {
            let mut message_found = false;
            queue.messages.retain(|m| {
                if m.receipt_handle == Some(request.receipt_handle.clone()) {
                    message_found = true;
                    false
                } else {
                    true
                }
            });

            if message_found {
                Ok(())
            } else {
                Err(SqsError::MessageNotInflight)
            }
        }
        None => Err(SqsError::QueueDoesNotExist),
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiveMessageRequest {
    pub queue_url: String,
    #[serde(default = "default_max_number_of_messages")]
    pub max_number_of_messages: u32,
    #[serde(default)]
    pub visibility_timeout: Option<u32>,
    #[serde(default)]
    pub wait_time_seconds: Option<u32>,
}

fn default_max_number_of_messages() -> u32 {
    1
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReceiveMessageResponse {
    #[serde(rename = "Messages")]
    pub messages: Vec<crate::state::Message>,
}

pub async fn receive_message(
    State(state): State<AppState>,
    Json(request): Json<ReceiveMessageRequest>,
) -> Result<ReceiveMessageResponse, SqsError> {
    let wait_time = request.wait_time_seconds.unwrap_or(0);
    let start_time = Utc::now();

    loop {
        match state.queues.get_mut(&request.queue_url) {
            Some(mut queue) => {
                let now = Utc::now();

                // Reset visibility for messages that have timed out
                for message in queue.messages.iter_mut() {
                    if message.receipt_handle.is_some() && now >= message.visible_from {
                        message.receipt_handle = None;
                    }
                }

                let mut messages_to_return = Vec::new();
                let visibility_timeout_attr = queue
                    .attributes
                    .get("VisibilityTimeout")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30);

                for message in queue.messages.iter_mut() {
                    println!(
                        "Checking message {}: receipt_handle={:?}, visible_from={}",
                        message.id, message.receipt_handle, message.visible_from
                    );
                    if messages_to_return.len() >= request.max_number_of_messages as usize {
                        break;
                    }

                    if message.receipt_handle.is_none() && now >= message.visible_from {
                        let mut message_clone = message.clone();

                        let visibility_timeout =
                            request.visibility_timeout.unwrap_or(visibility_timeout_attr);

                        message.visible_from =
                            Utc::now() + chrono::Duration::seconds(visibility_timeout as i64);
                        
                        let receipt_handle = Uuid::new_v4().to_string();
                        message.receipt_handle = Some(receipt_handle.clone());
                        message_clone.receipt_handle = Some(receipt_handle);

                        messages_to_return.push(message_clone);
                    }
                }

                if !messages_to_return.is_empty() {
                    return Ok(ReceiveMessageResponse {
                        messages: messages_to_return,
                    });
                }
            }
            None => return Err(SqsError::QueueDoesNotExist),
        }

        if (Utc::now() - start_time).num_seconds() >= wait_time as i64 {
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(ReceiveMessageResponse {
        messages: Vec::new(),
    })
}
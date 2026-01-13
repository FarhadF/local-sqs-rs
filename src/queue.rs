use crate::error::SqsError;
use crate::state::{AppState, Queue};
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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


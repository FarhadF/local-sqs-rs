use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use uuid::Uuid;

use std::env;

#[derive(Debug, Clone)]
pub struct AppState {
    pub queues: Arc<DashMap<String, Queue>>,
    pub host: String,
    pub port: u16,
}

impl AppState {
    pub fn new() -> Self {
        let host = env::var("LOCAL_SQS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("LOCAL_SQS_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9324);
        Self {
            queues: Arc::new(DashMap::new()),
            host,
            port,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub url: String,
    pub messages: VecDeque<Message>,
    pub attributes: HashMap<String, String>,
    pub created_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub receipt_handle: String,
    pub body: String,
    pub md5_of_body: String,
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub message_attributes: HashMap<String, MessageAttributeValue>,
    pub md5_of_message_attributes: String,
    pub visibility_timeout: DateTime<Utc>,
    pub sent_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageAttributeValue {
    pub string_value: Option<String>,
    pub binary_value: Option<String>, // Representing binary as base64 encoded string
    pub data_type: String,
}

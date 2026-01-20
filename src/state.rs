use chrono::{DateTime, Utc};
use dashmap::DashMap;
use md5;
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
pub struct RedrivePolicy {
    #[serde(rename = "deadLetterTargetArn")]
    pub dead_letter_target_arn: String,
    #[serde(rename = "maxReceiveCount")]
    pub max_receive_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub url: String,
    pub messages: VecDeque<Message>,
    pub attributes: HashMap<String, String>,
    pub created_timestamp: i64,
    pub last_modified_timestamp: i64,
    #[serde(default)]
    pub redrive_policy: Option<RedrivePolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    #[serde(rename = "MessageId")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_handle: Option<String>,
    pub body: String,
    #[serde(rename = "MD5OfBody")]
    pub md5_of_body: String,
    pub attributes: HashMap<String, String>,
    #[serde(default)]
    pub message_attributes: HashMap<String, MessageAttributeValue>,
    pub md5_of_message_attributes: String,
    #[serde(skip)]
    pub visible_from: DateTime<Utc>,
    pub sent_timestamp: DateTime<Utc>,
    #[serde(default)]
    pub receive_count: u32,
}


impl Message {
    pub fn new(
        body: String,
        attributes: HashMap<String, String>,
        message_attributes: HashMap<String, MessageAttributeValue>,
        delay_seconds: Option<u32>,
    ) -> Self {
        let md5_of_body = format!("{:x}", md5::compute(body.as_bytes()));
        let md5_of_message_attributes = if message_attributes.is_empty() {
            "".to_string()
        } else {
            // A real implementation would serialize and hash the attributes
            "".to_string()
        };

        let visible_from = if let Some(delay) = delay_seconds {
            Utc::now() + chrono::Duration::seconds(delay as i64)
        } else {
            Utc::now()
        };

        Self {
            id: Uuid::new_v4().to_string(),
            receipt_handle: None,
            body,
            md5_of_body,
            attributes,
            message_attributes,
            md5_of_message_attributes,
            visible_from,
            sent_timestamp: Utc::now(),
            receive_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageAttributeValue {
    pub string_value: Option<String>,
    pub binary_value: Option<String>, // Representing binary as base64 encoded string
    pub data_type: String,
}

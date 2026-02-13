use crate::error::SqsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubscribeRequest {
    pub topic_arn: String,
    pub protocol: String,
    pub endpoint: Option<String>,
    pub return_subscription_arn: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SubscribeResponse {
    #[serde(rename = "SubscriptionArn")]
    pub subscription_arn: String,
}

pub async fn subscribe(
    request: SubscribeRequest,
) -> Result<SubscribeResponse, SqsError> {
    Ok(SubscribeResponse {
        subscription_arn: format!("{}:{}", request.topic_arn, Uuid::new_v4()),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSubscriptionAttributesRequest {
    pub subscription_arn: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GetSubscriptionAttributesResponse {
    pub attributes: HashMap<String, String>,
}

pub async fn get_subscription_attributes(
    _request: GetSubscriptionAttributesRequest,
) -> Result<GetSubscriptionAttributesResponse, SqsError> {
    let mut attributes = HashMap::new();
    attributes.insert("ConfirmationWasAuthenticated".to_string(), "true".to_string());
    attributes.insert("DeliveryPolicy".to_string(), "{}".to_string());
    attributes.insert("EffectiveDeliveryPolicy".to_string(), "{}".to_string());
    attributes.insert("Owner".to_string(), "000000000000".to_string());
    attributes.insert("PendingConfirmation".to_string(), "false".to_string());
    // We could parse the ARN to get the topic ARN if needed, but this might be enough.
    // The error message showed: arn:aws:sns:us-east-1:000000000000:DEV_CampaignInstallEvent:1f39da08-b5a9-4dc0-8df6-cf97c4bb36ee
    // So the topic ARN is likely arn:aws:sns:us-east-1:000000000000:DEV_CampaignInstallEvent
    
    // Let's try to extract it from the subscription ARN just in case
    let parts: Vec<&str> = _request.subscription_arn.split(':').collect();
    if parts.len() > 1 {
        // Reconstruct topic ARN (remove the last part which is the subscription ID)
         let topic_arn = parts[0..parts.len()-1].join(":");
         attributes.insert("TopicArn".to_string(), topic_arn);
    } else {
        attributes.insert("TopicArn".to_string(), "arn:aws:sns:local:000000000000:Topic".to_string());
    }

    attributes.insert("SubscriptionArn".to_string(), _request.subscription_arn.clone());

    Ok(GetSubscriptionAttributesResponse { attributes })
}

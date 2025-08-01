use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The payload to be sent to the central notification lambda.
#[derive(Serialize, Deserialize)]
pub struct NotificationPayload {
    /// The API key to retrieve from Parameter Store, which determines which Pushover channel to use.
    pub api_key_name: String,
    /// The title of the notification, to appear in the push notification.
    pub title: String,
    /// The message of the notification, to appear in the push notification.
    pub message: String,
}

/// Load a SNS client from the environment, using eu-west-1, which is where the topic is.
pub async fn load_sns_client_from_env() -> aws_sdk_sns::Client {
    let config = aws_config::from_env().region("eu-west-1").load().await;
    aws_sdk_sns::Client::new(&config)
}

/// Pushover notifications have a maximum length of 512 characters for the title and message combined.
/// Note that SNS also has a maximum message size of 256KB, so this is not a hard limit for SNS,
/// but it is a hard limit for Pushover.
fn truncate_payload(payload: &NotificationPayload) -> NotificationPayload {
    const MAX_LENGTH: usize = 512;

    if payload.title.len() + payload.message.len() <= MAX_LENGTH {
        return NotificationPayload {
            api_key_name: payload.api_key_name.clone(),
            title: payload.title.clone(),
            message: payload.message.clone(),
        };
    } else if payload.title.len() > MAX_LENGTH {
        return NotificationPayload {
            api_key_name: payload.api_key_name.clone(),
            title: payload.title.chars().take(MAX_LENGTH).collect(),
            message: String::new(),
        };
    } else {
        return NotificationPayload {
            api_key_name: payload.api_key_name.clone(),
            title: payload.title.clone(),
            message: payload
                .message
                .chars()
                .take(MAX_LENGTH - payload.title.len())
                .collect(),
        };
    }
}

/// Send a message to the pushover-notifications SNS topic, which will get picked up by the central notification lambda
/// and routed to the appropriate Pushover channel.
/// Note that the title + message will be truncated to 512 characters.
pub async fn publish_sns_message(
    client: &aws_sdk_sns::Client,
    payload: &NotificationPayload,
) -> Result<()> {
    let payload = truncate_payload(payload);
    let message = serde_json::to_string(&payload)?;

    client
        .publish()
        .topic_arn("arn:aws:sns:eu-west-1:982932998640:pushover-notifications")
        .message(&message)
        .send()
        .await?;

    Ok(())
}

/// Loads a SNS client and sends a message to the pushover-notifications SNS topic, which will get picked up
/// by the central notification lambda and routed to the appropriate Pushover channel.
/// Note that the title + message will be truncated to 512 characters.
pub async fn load_client_and_send_notification(payload: &NotificationPayload) -> Result<()> {
    let client = load_sns_client_from_env().await;
    publish_sns_message(&client, &payload).await
}

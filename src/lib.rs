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

pub async fn load_sns_client_from_env() -> aws_sdk_sns::Client {
    let config = aws_config::load_from_env().await;
    aws_sdk_sns::Client::new(&config)
}

/// Send a message to the pushover-notifications SNS topic, which will get picked up by the central notification lambda
/// and routed to the appropriate Pushover channel.
pub async fn publish_sns_message(
    client: &aws_sdk_sns::Client,
    payload: &NotificationPayload,
) -> Result<()> {
    let message = serde_json::to_string(payload)?;

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
pub async fn load_client_and_send_notification(payload: &NotificationPayload) -> Result<()> {
    let client = load_sns_client_from_env().await;
    publish_sns_message(&client, &payload).await
}

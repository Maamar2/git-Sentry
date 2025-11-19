use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize)]
struct SendMessagePayload {
    chat_id: i64,
    text: String,
    parse_mode: String,
}

#[derive(Debug, Serialize)]
struct AnswerCallbackQueryPayload {
    callback_query_id: String,
    text: String,
    show_alert: bool,
}

#[derive(Debug, Deserialize)]
pub struct TelegramResponse {
    pub ok: bool,
    pub result: serde_json::Value,
}

pub async fn send_message(bot_token: &str, chat_id: i64, text: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let payload = SendMessagePayload {
        chat_id,
        text: text.to_string(),
        parse_mode: "Markdown".to_string(),
    };

    let response = client.post(&url).json(&payload).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Telegram API error: {} - {}",
            response.status(),
            response.text().await?
        );
    }

    let data: TelegramResponse = response.json().await?;
    if !data.ok {
        anyhow::bail!("Telegram API returned error: {:?}", data.result);
    }

    debug!("Message sent successfully");
    Ok(())
}

pub async fn send_approval_request(
    bot_token: &str,
    chat_id: i64,
    request_id: &str,
    command: &str,
) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let keyboard = serde_json::json!({
        "inline_keyboard": [[
            {
                "text": "✅ Approve",
                "callback_data": format!("approve:{}", request_id)
            },
            {
                "text": "❌ Deny",
                "callback_data": format!("deny:{}", request_id)
            }
        ]]
    });

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": format!("🔐 *Git-Sentry Security Request*\n\n*Command:* `{}`\n\nDo you approve this operation?", command),
        "parse_mode": "Markdown",
        "reply_markup": keyboard
    });

    let response = client.post(&url).json(&payload).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Telegram API error: {} - {}",
            response.status(),
            response.text().await?
        );
    }

    Ok(())
}

pub async fn test_connection(bot_token: &str, chat_id: i64) -> Result<()> {
    send_message(
        bot_token,
        chat_id,
        "🔐 Git-Sentry connection test successful!",
    )
    .await
}

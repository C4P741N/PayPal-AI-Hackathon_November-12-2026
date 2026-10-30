use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, eprint, format, print, time::Duration, vec};

#[derive(Deserialize)]
struct ResponseContent {
    content: String,
}
#[derive(Deserialize)]
struct ResponseMessage {
    message: ResponseContent,
}
#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ResponseMessage>,
}
#[derive(Serialize)]
struct ImageUrl {
    url: String,
}
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}
#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

async fn image_to_text(
    mime_type: &str,
    base64_image: &str,
    api_key: &str,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner} {msg}")?,
    );
    spinner.set_message("Waiting for API response...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let data_url = format!("data:{};base64,{}", mime_type, base64_image);

    let payload = ChatRequest {
        model: "deepseek-chat".to_string(),
        max_tokens: 1024,
        messages: vec![Message {
            role: "user".to_string(),
            content: vec![
                ContentPart::ImageUrl {
                    image_url: ImageUrl { url: data_url },
                },
                ContentPart::Text {
                    text: prompt.to_string(),
                },
            ],
        }],
    };

    //let json_payload = serde_json::to_string_pretty(&payload).expect("Failed to serialize payload");

    //print!("Generated payload: {}", json_payload);

    let client = Client::new();
    let res = client
        .post("https://api.deepseek.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await?;

    spinner.finish_and_clear();

    if !res.status().is_success() {
        let err_body = res.text().await?;
        return Err(format!("Api error: {}", err_body).into());
    }

    let data: ChatResponse = res.json().await?;

    Ok(data
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_else(|| "No response".to_string()))
}

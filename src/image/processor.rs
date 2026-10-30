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


pub async fn image_to_text(
    mime_type: &str,
    base64_image: &str,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let spinner = ProgressBar::new_spinner();

    let prompt = r#"
Extract all available information from this image.
Return ONLY valid JSON (no Markdown code fences or additional commentary) using the following structure:

{
  "items": [
    {
      "quantity": 2,
      "description": "Laptop Charger",
      "unit_price": 1500.00,
      "total_price": 3000.00,
      "tax_type": "T"
    },
  ],
  "store_name": "The Clurb"
  "subtotal": 4900.00,
  "tax": 784.00,
  "total": 5684.00,
  "payment": {
    "method": "VISA",
    "card_number": "XXXXXXXXXXXX4242",
    "transaction_type": "CARD PRESENT",
    "tendered": 5684.00,
    "change": 0.00
  },
  "metadata": {
    "date": "10/30/2026",
    "time": "01:30 PM",
    "transaction_number": "72987869213046141"
  }
}

Rules:
- Extract the actual values from the image; do not use the sample values.
- Include all line items visible in the receipt.
- Use numeric values for quantities and monetary amounts.
- Preserve tax_type as shown on the receipt.
- Use null for missing or unreadable values.
- Do not invent information.
- Return only the JSON object.
"#;
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

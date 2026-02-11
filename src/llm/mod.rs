use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

use crate::config::Settings;

pub async fn run_instruction(settings: &Settings, instruction: &str) -> Result<String> {
    if settings.claude_api_key == "test_key" || settings.claude_api_key.trim().is_empty() {
        return Ok(format!(
            "Simulated LLM response. Instruction received: {}",
            instruction
        ));
    }

    let client = Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &settings.claude_api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": settings.claude_model,
            "max_tokens": 512,
            "messages": [{"role": "user", "content": instruction}]
        }))
        .send()
        .await?;

    let body: Value = response.json().await?;
    let content = body
        .get("content")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.get("text"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Invalid LLM response payload"))?;

    Ok(content.to_string())
}

use crate::settings::AppConfig;
use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde_json::json;

pub fn generate_summary(config: &AppConfig, commit_logs: String) -> Result<String> {
    let client = Client::new();

    let final_prompt = config
        .ai
        .prompt
        .replace("{LANGUAGE}", &config.ai.language)
        .replace("{LOGS}", &commit_logs);

    let payload = json!({
        "model": config.ai.model,
        "messages": [
        {
            "role": "system",
            "content": "You are a helpful assistant."
        },
        {
            "role": "user",
            "content": final_prompt
        }
    ],
    "stream": false
    });

    let res = client
        .post(&config.ai.api_url)
        .header("Authorization", format!("Bearer {}", config.ai.api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .context("Api request failed. check url.")?;

    let status = res.status();

    if !res.status().is_success() {
        let error_text = res.text().unwrap_or_default();
        bail!("Api failed. (Status {}): {}", status, error_text);
    }

    let response_json: serde_json::Value = res.json().context("Json parse error")?;

    response_json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .context("Api result is not expected format.")
}

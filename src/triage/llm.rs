use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::security;

#[derive(Debug, Clone)]
pub struct LlmClient {
    model: String,
    api_key_service: String,
    api_base: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

impl LlmClient {
    pub fn new(model: String, api_key_service: String, api_base: Option<String>) -> Self {
        Self {
            model,
            api_key_service,
            api_base: api_base.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            client: Client::new(),
        }
    }

    /// Generate a better filename for a file based on context
    pub async fn suggest_filename(&self, original_name: &str, context: &str) -> Result<String> {
        let api_key = match security::retrieve_api_key(&self.api_key_service) {
            Ok(key) => key,
            Err(e) => {
                warn!("Could not retrieve API key: {}", e);
                return Ok(original_name.to_string());
            }
        };

        let prompt = format!(
            "You are a file organization assistant. Suggest a clean, descriptive filename for a downloaded file.\n\n\
             Original name: {}\n\
             Context: {}\n\n\
             Respond with ONLY the suggested filename (no explanation, no quotes).",
            original_name, context
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 50,
        };

        let url = format!("{}/chat/completions", self.api_base);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            warn!("LLM API returned error: {} - {}", status, text);
            return Ok(original_name.to_string());
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")?;

        let suggestion = chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_else(|| original_name.to_string());

        info!("LLM suggested filename: {}", suggestion);

        Ok(suggestion.trim().to_string())
    }
}

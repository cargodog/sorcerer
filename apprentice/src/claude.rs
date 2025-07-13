use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: i32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

pub struct ClaudeClient {
    client: Client,
    api_key: String,
}

impl ClaudeClient {
    pub fn new() -> Self {
        let api_key = if let Ok(key_file) = std::env::var("ANTHROPIC_API_KEY_FILE") {
            std::fs::read_to_string(&key_file).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to read API key file {}: {}. API calls will fail.",
                    key_file, e
                );
                "".to_string()
            })
        } else {
            std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| {
                    eprintln!("Warning: Neither ANTHROPIC_API_KEY_FILE nor ANTHROPIC_API_KEY set. API calls will fail.");
                    "".to_string()
                })
        };

        Self {
            client: Client::new(),
            api_key: api_key.trim().to_string(),
        }
    }

    pub async fn send_message(
        &self,
        message: &str,
        conversation_history: &[String],
    ) -> Result<String> {
        self.send_message_with_system(message, conversation_history, None)
            .await
    }

    pub async fn send_message_with_system(
        &self,
        message: &str,
        conversation_history: &[String],
        system_prompt: Option<&str>,
    ) -> Result<String> {
        debug!("Sending message to Claude: {}", message);

        if self.api_key.is_empty() {
            return Err(anyhow!("ANTHROPIC_API_KEY not set"));
        }

        // Build messages array from conversation history
        let mut messages = Vec::new();

        // Add conversation history
        for hist_msg in conversation_history.iter() {
            if let Some(content) = hist_msg.strip_prefix("Sorcerer: ") {
                messages.push(Message {
                    role: "user".to_string(),
                    content: content.to_string(),
                });
            } else if let Some(colon_pos) = hist_msg.find(": ") {
                // This is an assistant message (format: "ApprenticeNname: response")
                let content = &hist_msg[colon_pos + 2..];
                messages.push(Message {
                    role: "assistant".to_string(),
                    content: content.to_string(),
                });
            }
        }

        // Add the current message
        messages.push(Message {
            role: "user".to_string(),
            content: message.to_string(),
        });

        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages,
            system: system_prompt.map(|s| s.to_string()),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Claude API error: {}", error_text);
            return Err(anyhow!("Claude API error: {}", error_text));
        }

        let claude_response: ClaudeResponse = response.json().await?;

        Ok(claude_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

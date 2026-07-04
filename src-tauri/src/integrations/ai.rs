//! AI Integration Service supporting OpenAI-compatible APIs.
//!
//! This service works with:
//!  - OpenAI (api.openai.com)
//!  - Cerebras (api.cerebras.ai)
//!  - NVIDIA NIM (integrate.api.nvidia.com)
//!  - Local servers (vllm, llama.cpp, Ollama, etc.)
//!
//! Configuration is read from environment variables:
//!  - `AI_ENABLED` (set to "true" to enable)
//!  - `AI_API_URL` (defaults to "https://api.openai.com/v1")
//!  - `AI_API_KEY` (API key for authentication)
//!  - `AI_MODEL` (model name, e.g., "llama3-8b-8192" or "gpt-4o-mini")
//!  - `AI_SYSTEM_PROMPT` (optional override)

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("AI_ENABLED").map(|v| v == "true").unwrap_or(false),
            api_url: std::env::var("AI_API_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            api_key: std::env::var("AI_API_KEY").unwrap_or_default(),
            model: std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            system_prompt: std::env::var("AI_SYSTEM_PROMPT").unwrap_or_else(|_| {
                "You are the professional AI Assistant for the ARK: Survival Ascended Dedicated Server Configuration Manager. \
                Your job is to assist the administrator in managing the game cluster. \
                You can answer general questions about ARK server settings, configuration multipliers, and network setups. \
                \
                Additionally, if the user asks you to perform an action (like starting, stopping, restarting, showing status, viewing logs, or getting server IP), you MUST output a structured command at the end of your response inside a special bracket. \
                Available commands: \
                 - Start server: [COMMAND: {\"kind\": \"start\", \"map_index\": 0}] (where map_index is optional or index of the map, default 0) \
                 - Stop server: [COMMAND: {\"kind\": \"stop\", \"map_index\": 0}] \
                 - Restart server: [COMMAND: {\"kind\": \"restart\", \"map_index\": 0}] \
                 - Check status: [COMMAND: {\"kind\": \"status\"}] \
                 - View logs: [COMMAND: {\"kind\": \"logs\", \"tail\": 20}] \
                 - Get connection IP: [COMMAND: {\"kind\": \"ip\"}] \
                \
                Be concise and professional. Do not invent commands. If the user commands something ambiguous, ask for clarification first.".to_string()
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatCompletionMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to build AI reqwest client"),
        }
    }

    /// Load config from environment
    pub fn from_env() -> Self {
        Self::new(AiConfig::default())
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled && !self.config.api_url.is_empty()
    }

    /// Query the AI assistant
    pub async fn query(&self, prompt: &str) -> Result<String, String> {
        if !self.enabled() {
            return Err("AI service is disabled or unconfigured".to_string());
        }

        let url = format!("{}/chat/completions", self.config.api_url.trim_end_matches('/'));
        let body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatCompletionMessage {
                    role: "system".to_string(),
                    content: self.config.system_prompt.clone(),
                },
                ChatCompletionMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.2,
        };

        let mut req = self.client.post(&url);
        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }

        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("AI returned error status {}: {}", status, error_text));
        }

        let parsed: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse AI JSON response: {}", e))?;

        if parsed.choices.is_empty() {
            return Err("AI response contained empty choices".to_string());
        }

        Ok(parsed.choices[0].message.content.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_falls_back_to_defaults() {
        let cfg = AiConfig::default();
        // Always have a non-empty api_url default
        assert!(!cfg.api_url.is_empty());
        // model is non-empty default
        assert!(!cfg.model.is_empty());
        // system prompt includes the COMMAND tag guide
        assert!(cfg.system_prompt.contains("[COMMAND:"));
    }

    #[test]
    fn disabled_when_flag_is_false_or_key_missing() {
        let mut cfg = AiConfig::default();
        cfg.enabled = false;
        cfg.api_key = "".into();
        let client = AiClient::new(cfg);
        assert!(!client.enabled());

        let cfg2 = AiConfig {
            enabled: true,
            api_url: "https://api.openai.com/v1".into(),
            api_key: "test-key".into(),
            model: "gpt-4o-mini".into(),
            system_prompt: "x".into(),
        };
        let client2 = AiClient::new(cfg2);
        assert!(client2.enabled());
    }

    #[test]
    fn builds_chat_completions_url_correctly() {
        let cfg = AiConfig {
            enabled: true,
            api_url: "https://api.openai.com/v1/".into(),
            api_key: "x".into(),
            model: "gpt-4o-mini".into(),
            system_prompt: "y".into(),
        };
        let client = AiClient::new(cfg);
        let expected = "https://api.openai.com/v1/chat/completions";
        let actual = format!("{}/chat/completions", client.config.api_url.trim_end_matches('/'));
        assert_eq!(actual, expected);
    }
}

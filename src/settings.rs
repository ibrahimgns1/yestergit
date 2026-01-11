use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiConfig {
    pub api_url: String,
    pub model: String,
    pub api_key: String,
    pub language: String,
    pub prompt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AiConfig {
                api_url: "http://localhost:11434/v1/chat/completions".to_string(),
                model: "llama3".to_string(),
                api_key: "".to_string(),
                language: "English".to_string(),
                prompt: r#"Act as a software developer giving a quick verbal update at a Daily Scrum meeting.
Based on the commit logs below, draft a ** short, conversational summary**  in {LANGUAGE}.
Use the first person ("I finished...", "I fixed...").
Group related tasks together. Mention the project name when describing the work.
Do NOT use bullet points, commit hashes, or technical jargon. Keep it casual and compact.

Commit Logs:
{LOGS}"#.to_string(),
            },
        }
    }
}

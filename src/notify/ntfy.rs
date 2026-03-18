// ntfy.sh notification backend
//
// Single HTTP POST per message. No SDK, no account required.
// Subscribe to your topic at https://ntfy.sh/<topic> or in the ntfy app.

use reqwest::Client;
use std::error::Error;

#[derive(Clone)]
pub struct NtfyConfig {
    pub url: String,           // e.g. "https://ntfy.sh/openclaw-abc123"
    pub token: Option<String>, // for self-hosted instances with auth
}

pub struct Message {
    pub title: String,
    pub body: String,
    pub priority: u8,          // 1=min 2=low 3=default 4=high 5=max
    pub tags: Vec<&'static str>,
    pub click_url: Option<String>,
}

pub async fn send(client: &Client, cfg: &NtfyConfig, msg: Message) -> Result<(), Box<dyn Error>> {
    let mut req = client
        .post(&cfg.url)
        .header("Title", &msg.title)
        .header("Priority", msg.priority.to_string())
        .header("Tags", msg.tags.join(","))
        .body(msg.body);

    if let Some(url) = &msg.click_url {
        req = req.header("Click", url);
    }
    if let Some(token) = &cfg.token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    req.send().await?.error_for_status()?;
    Ok(())
}

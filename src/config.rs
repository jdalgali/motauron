use crate::notify::{Backend, NotifyConfig};
use crate::notify::ntfy::NtfyConfig;
use serde::Deserialize;
use std::error::Error;

const CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub notify: NotifySection,
}

#[derive(Deserialize)]
pub struct AgentConfig {
    /// Hours between scrape cycles in daemon mode.
    pub interval_hours: u64,
    /// Minimum price_score to trigger a high-priority push (default 7 = "good").
    pub min_alert_score: i32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            interval_hours: 4,
            min_alert_score: 7,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct NotifySection {
    pub ntfy: Option<NtfySection>,
}

#[derive(Deserialize)]
pub struct NtfySection {
    pub url: String,
    pub token: Option<String>,
}

pub fn load() -> Result<Config, Box<dyn Error>> {
    if !std::path::Path::new(CONFIG_PATH).exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(CONFIG_PATH)?;
    let cfg: Config = toml::from_str(&raw)?;
    Ok(cfg)
}

pub fn notify_config(cfg: &Config) -> NotifyConfig {
    let backend = match &cfg.notify.ntfy {
        Some(n) => Backend::Ntfy(NtfyConfig {
            url: n.url.clone(),
            token: n.token.clone(),
        }),
        None => Backend::Disabled,
    };
    NotifyConfig {
        backend,
        min_alert_score: cfg.agent.min_alert_score,
    }
}

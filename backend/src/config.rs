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
    pub interval_hours: u64,
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

use anyhow::{Context, Result};

pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub session_string: String,
    pub mongo_uri: String,
    pub log_channel_id: i64,
    pub owner_id: i64,
    pub support_group: String,
    pub support_channel: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_id = std::env::var("API_ID")
            .context("API_ID not set")?
            .parse()
            .context("Invalid API_ID")?;
        let api_hash = std::env::var("API_HASH").context("API_HASH not set")?;
        let session_string = std::env::var("SESSION_STRING").context("SESSION_STRING not set")?;
        let mongo_uri = std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let log_channel_id = std::env::var("LOG_CHANNEL_ID")
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);
        let owner_id = std::env::var("OWNER_ID")
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);
        let support_group = std::env::var("SUPPORT_GROUP").unwrap_or_else(|_| "https://t.me/OliviaSupportChat".to_string());
        let support_channel = std::env::var("SUPPORT_CHANNEL").unwrap_or_else(|_| "https://t.me/OliviaBots".to_string());

        Ok(Config {
            api_id,
            api_hash,
            session_string,
            mongo_uri,
            log_channel_id,
            owner_id,
            support_group,
            support_channel,
        })
    }
}

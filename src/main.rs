mod bot;
mod config;
mod database;
mod jiosaavn;
mod utils;
mod voice_chat;

use std::sync::Arc;
use anyhow::Result;
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    info!("🎵 Starting SecretMusicBot...");

    let config = config::Config::from_env()?;
    let db = database::Db::new(&config.mongo_uri).await?;
    let ntgcalls = ntgcalls::NTgCall::new();
    let vc = voice_chat::VoiceChatManager::new(ntgcalls);

    let client = ferogram::Client::builder()
        .api_id(config.api_id)
        .api_hash(&config.api_hash)
        .session_string(&config.session_string)
        .low_memory_mode()
        .connect()
        .await?;

    let state = Arc::new(bot::AppState {
        config,
        db,
        vc,
        client,
        queues: dashmap::DashMap::new(),
        current: dashmap::DashMap::new(),
        cancel_tokens: dashmap::DashMap::new(),
        connected: dashmap::DashMap::new(),
    });

    bot::run(state).await
}

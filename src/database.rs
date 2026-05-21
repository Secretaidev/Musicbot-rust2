use mongodb::{Client as MongoClient, Database, bson::doc};
use anyhow::Result;

pub struct Db {
    db: Database,
}

impl Db {
    pub async fn new(uri: &str) -> Result<Self> {
        let client = MongoClient::with_uri_str(uri).await?;
        let db = client.database("secret_music_bot");
        Ok(Self { db })
    }

    pub async fn log_command(&self, chat_id: i64, user_id: i64, command: &str, query: &str) {
        let collection = self.db.collection("logs");
        let doc = doc! {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "chat_id": chat_id,
            "user_id": user_id,
            "command": command,
            "query": query,
        };
        let _ = collection.insert_one(doc, None).await;
    }

    pub async fn increment_stat(&self, field: &str) {
        let collection = self.db.collection("stats");
        let _ = collection
            .update_one(
                doc! { "key": "global" },
                doc! { "$inc": { field: 1 } },
                mongodb::options::UpdateOptions::builder().upsert(true).build(),
            )
            .await;
    }
}

use anyhow::{Context, Result};
use cipher::KeyInit;
use std::time::Duration;

const DES_KEY: &[u8; 8] = b"38346591";

#[derive(Debug, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artists: String,
    pub album: String,
    pub duration: u64,
    pub thumbnail: String,
    pub media_url: String,
}

pub struct JioSaavnClient {
    http: reqwest::Client,
}

impl JioSaavnClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Song>> {
        let url = format!(
            "https://www.jiosaavn.com/api.php?__call=search.getResults&q={}&ctx=web6dot0&api_version=4&_format=json",
            urlencoding::encode(query)
        );
        let resp = self.http.get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?
            .text()
            .await?;

        let resp = Self::clean_jsonp(&resp);
        let data: serde_json::Value = serde_json::from_str(&resp).context("Invalid JSON")?;
        let results = data.get("results").and_then(|r| r.as_array()).unwrap_or(&vec![]);

        let mut songs = Vec::new();
        for item in results.iter().take(10) {
            if let Some(song) = self.parse_song(item).await {
                songs.push(song);
            }
        }
        Ok(songs)
    }

    async fn parse_song(&self, item: &serde_json::Value) -> Option<Song> {
        let id = item.get("id")?.as_str()?.to_string();
        let title = item.get("title")?.as_str()?.to_string();
        let more_info = item.get("more_info")?;
        let album = more_info.get("album")?.as_str()?.unwrap_or("Unknown").to_string();
        let duration = more_info.get("duration")?.as_str()?.parse::<u64>().ok().unwrap_or(0);
        let encrypted_url = more_info.get("encrypted_media_url")?.as_str()?.to_string();
        let thumbnail = item.get("image")?.as_str()?.unwrap_or("").to_string();
        let media_url = decrypt_url(&encrypted_url).ok()?;
        let media_url = media_url.replace("_96", "_320");

        let artist_map = more_info.get("artistMap")?;
        let primary = artist_map.get("primary_artists")?.as_array()?;
        let artists: Vec<String> = primary.iter().filter_map(|a| a.get("name")?.as_str().map(String::from)).collect();
        let artists = if artists.is_empty() { "Unknown".to_string() } else { artists.join(", ") };

        Some(Song { id, title, artists, album, duration, thumbnail, media_url })
    }

    fn clean_jsonp(resp: &str) -> String {
        let trimmed = resp.trim();
        if trimmed.starts_with("})") {
            trimmed[2..].to_string()
        } else if trimmed.starts_with("}});") {
            trimmed[4..].to_string()
        } else {
            trimmed.to_string()
        }
    }
}

fn decrypt_url(encrypted: &str) -> Result<String> {
    let ciphertext = base64::decode(encrypted).context("Base64 decode failed")?;
    let mut decryptor = ecb::Decryptor::<des::Des>::new(DES_KEY.into());
    let plaintext = decryptor.decrypt_padded_vec_mut::<cipher::block_padding::Pkcs7>(&ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
    Ok(String::from_utf8(plaintext).context("UTF-8 decode failed")?)
}

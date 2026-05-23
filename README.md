#notworkinginvc



# 🎵 SecretMusicBot

[![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange?logo=rust)](https://www.rust-lang.org)
[![MongoDB](https://img.shields.io/badge/MongoDB-4.4%2B-green?logo=mongodb)](https://www.mongodb.com)
[![Telegram](https://img.shields.io/badge/Telegram-@OliviaBots-blue?logo=telegram)](https://t.me/OliviaBots)
[![Docker](https://img.shields.io/badge/Docker-Supported-blue?logo=docker)](https://www.docker.com)

> Premium Telegram Music Bot powered by JioSaavn. Play high-quality music in voice chats with minimal server load.

## ✨ Features
- 🎶 Play music from JioSaavn in Telegram voice chats
- 🔥 High-quality 320kbps audio streaming
- ⏭️ Skip, Pause, Resume, Volume control
- 📜 Queue management
- 📝 Logs to private channel
- 🗄️ MongoDB stats
- 🚀 Ultra-low memory footprint

## ⚙️ Configuration
| Variable | Description |
|----------|-------------|
| `API_ID` | Telegram API ID |
| `API_HASH` | Telegram API Hash |
| `SESSION_STRING` | String session for assistant |
| `MONGO_URI` | MongoDB connection URI |
| `LOG_CHANNEL_ID` | Channel ID for logs |
| `OWNER_ID` | Owner user ID |
| `SUPPORT_GROUP` | Support group link |
| `SUPPORT_CHANNEL` | Support channel link |

## 🚀 Deployment
### Docker
```bash
docker build -t secret-music-bot .
docker run --env-file .env secret-music-bot
```

### Railway
1. Connect your repo to Railway
2. Add environment variables in Railway dashboard
3. Deploy

### Render
1. Connect your repo to Render
2. Select **Docker** environment
3. Add environment variables
4. Deploy

## 📞 Support
- Support Group: [@OliviaSupportChat](https://t.me/OliviaSupportChat)
- Support Channel: [@OliviaBots](https://t.me/OliviaBots)
- Owner: [@its_me_secret](https://t.me/its_me_secret)

## 📜 License
MIT

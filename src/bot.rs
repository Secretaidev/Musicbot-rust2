use crate::{
    config::Config,
    database::Db,
    jiosaavn::{JioSaavnClient, Song},
    utils,
    voice_chat::VoiceChatManager,
};
use dashmap::DashMap;
use futures::StreamExt;
use std::sync::Arc;
use std::collections::VecDeque;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub config: Config,
    pub db: Db,
    pub vc: VoiceChatManager,
    pub client: ferogram::Client,
    pub queues: DashMap<i64, VecDeque<Song>>,
    pub current: DashMap<i64, Song>,
    pub cancel_tokens: DashMap<i64, CancellationToken>,
    pub connected: DashMap<i64, bool>,
}

pub async fn run(state: Arc<AppState>) -> anyhow::Result<()> {
    let client = state.client.clone();
    let mut stream = client.stream_updates();

    let mut dp = ferogram::filters::Dispatcher::new();

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("play"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_play(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("skip"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_skip(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("stop"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_stop(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("pause"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_pause(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("resume"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_resume(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("volume"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_volume(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("queue"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_queue(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("nowplaying"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_nowplaying(msg, state).await; }
    });

    let s = Arc::clone(&state);
    dp.on_message(ferogram::filters::command("help"), move |msg| {
        let state = Arc::clone(&s);
        async move { handle_help(msg, state).await; }
    });

    while let Some(upd) = stream.next().await {
        dp.dispatch(upd).await;
    }

    Ok(())
}

async fn handle_play(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let text = match msg.text() {
        Some(t) => t,
        None => return,
    };
    let args = text.splitn(2, ' ').nth(1);
    if args.is_none() {
        let _ = msg.reply(&utils::format_simple_error("Please provide a song name")).await;
        return;
    }
    let query = args.unwrap().trim();
    if query.is_empty() {
        let _ = msg.reply(&utils::format_simple_error("Please provide a song name")).await;
        return;
    }

    let chat_id = msg.chat.id;
    let (user_id, username) = match &msg.from {
        Some(user) => (user.id, user.username.clone().unwrap_or_else(|| user.first_name.clone())),
        None => return,
    };

    tokio::spawn({
        let db = state.db.clone();
        let q = query.to_string();
        async move {
            db.log_command(chat_id, user_id, "play", &q).await;
            db.increment_stat("total_songs").await;
        }
    });

    let js = JioSaavnClient::new();
    let songs = match js.search(query).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Search error: {}", e);
            let _ = msg.reply(&utils::format_simple_error("Failed to search song")).await;
            return;
        }
    };

    if songs.is_empty() {
        let _ = msg.reply(&utils::format_simple_error("No results found")).await;
        return;
    }

    let song = songs.into_iter().next().unwrap();

    if state.current.contains_key(&chat_id) {
        let mut queue = state.queues.entry(chat_id).or_insert_with(VecDeque::new);
        queue.push_back(song.clone());
        let text = format!(
            "🎵 {}\n➕ {}\n{}",
            utils::to_bold("SecretMusicBot"),
            utils::to_bold("Added to Queue"),
            utils::to_italic(&song.title)
        );
        let _ = msg.reply(&text).await;
        return;
    }

    state.current.insert(chat_id, song.clone());

    let reply_text = utils::format_playing_message(
        "SecretMusicBot",
        &song.title,
        &song.artists,
        &song.album,
        song.duration,
        &format!("@{}", username),
    );
    let _ = msg.reply(&reply_text).await;

    if state.connected.contains_key(&chat_id) {
        if let Err(e) = state.vc.change_stream(chat_id, &song.media_url) {
            log::error!("Change stream error: {}", e);
            state.current.remove(&chat_id);
            let _ = msg.reply(&utils::format_simple_error("Failed to play song.")).await;
            return;
        }
    } else {
        if let Err(e) = state.vc.join_and_play(chat_id, &song.media_url, &state.client).await {
            log::error!("Join VC error: {}", e);
            state.current.remove(&chat_id);
            let _ = msg.reply(&utils::format_simple_error("Failed to join voice chat. Is there an active VC?")).await;
            return;
        }
        state.connected.insert(chat_id, true);
    }

    if state.config.log_channel_id != 0 {
        let log_text = format!(
            "📝 {}\nChat: {}\nUser: {}\nAction: play\nQuery: {}",
            utils::to_bold("Log"),
            chat_id,
            user_id,
            query
        );
        let _ = state.client.send_message(state.config.log_channel_id, &log_text).await;
    }

    let state_monitor = Arc::clone(&state);
    tokio::spawn(async move {
        spawn_monitor(chat_id, song.duration, state_monitor).await;
    });
}

async fn handle_skip(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    let user_id = match &msg.from {
        Some(user) => user.id,
        None => return,
    };

    tokio::spawn({
        let db = state.db.clone();
        async move { db.log_command(chat_id, user_id, "skip", "").await; db.increment_stat("total_skips").await; }
    });

    if let Some((_, token)) = state.cancel_tokens.remove(&chat_id) {
        token.cancel();
    }

    if let Err(e) = state.vc.stop(chat_id) {
        log::error!("Stop error: {}", e);
    }
    state.current.remove(&chat_id);

    if let Some((_, mut queue)) = state.queues.remove(&chat_id) {
        if let Some(next_song) = queue.pop_front() {
            state.queues.insert(chat_id, queue);
            play_next(chat_id, next_song, Arc::clone(&state)).await;
        } else {
            state.connected.remove(&chat_id);
            let _ = msg.reply(&utils::format_simple_info("Queue finished. Left voice chat.")).await;
        }
    } else {
        state.connected.remove(&chat_id);
        let _ = msg.reply(&utils::format_simple_info("Skipped. No more songs in queue.")).await;
    }
}

async fn handle_stop(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    let user_id = match &msg.from {
        Some(user) => user.id,
        None => return,
    };

    tokio::spawn({
        let db = state.db.clone();
        async move { db.log_command(chat_id, user_id, "stop", "").await; }
    });

    if let Some((_, token)) = state.cancel_tokens.remove(&chat_id) {
        token.cancel();
    }

    if let Err(e) = state.vc.stop(chat_id) {
        log::error!("Stop error: {}", e);
    }
    state.current.remove(&chat_id);
    state.queues.remove(&chat_id);
    state.connected.remove(&chat_id);

    let _ = msg.reply(&utils::format_simple_info("Stopped and left voice chat.")).await;
}

async fn handle_pause(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    match state.vc.pause(chat_id) {
        Ok(true) => { let _ = msg.reply(&utils::format_simple_info("Paused.")).await; }
        Ok(false) => { let _ = msg.reply(&utils::format_simple_error("Not playing.")).await; }
        Err(e) => { log::error!("Pause error: {}", e); }
    }
}

async fn handle_resume(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    match state.vc.resume(chat_id) {
        Ok(true) => { let _ = msg.reply(&utils::format_simple_info("Resumed.")).await; }
        Ok(false) => { let _ = msg.reply(&utils::format_simple_error("Not paused.")).await; }
        Err(e) => { log::error!("Resume error: {}", e); }
    }
}

async fn handle_volume(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    let text = msg.text().unwrap_or_default();
    let args = text.splitn(2, ' ').nth(1);
    if let Some(vol_str) = args {
        if let Ok(vol) = vol_str.trim().parse::<i32>() {
            if vol < 1 || vol > 200 {
                let _ = msg.reply(&utils::format_simple_error("Volume must be between 1 and 200.")).await;
                return;
            }
            let vol_tg = vol * 100;
            if let Err(e) = state.vc.set_volume(chat_id, vol_tg, &state.client).await {
                log::error!("Set volume error: {}", e);
                let _ = msg.reply(&utils::format_simple_error("Failed to set volume.")).await;
                return;
            }
            let _ = msg.reply(&utils::format_simple_info(&format!("Volume set to {}.", vol))).await;
            return;
        }
    }
    let _ = msg.reply(&utils::format_simple_error("Usage: /volume <1-200>")).await;
}

async fn handle_queue(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    if let Some(queue) = state.queues.get(&chat_id) {
        if queue.is_empty() {
            let _ = msg.reply(&utils::format_simple_info("Queue is empty.")).await;
            return;
        }
        let songs: Vec<(usize, String, String)> = queue.iter().enumerate().map(|(i, s)| (i + 1, s.title.clone(), s.artists.clone())).collect();
        let text = utils::format_queue("SecretMusicBot", &songs);
        let _ = msg.reply(&text).await;
    } else {
        let _ = msg.reply(&utils::format_simple_info("Queue is empty.")).await;
    }
}

async fn handle_nowplaying(msg: ferogram::types::IncomingMessage, state: Arc<AppState>) {
    let chat_id = msg.chat.id;
    if let Some(song) = state.current.get(&chat_id) {
        let text = utils::format_playing_message(
            "SecretMusicBot",
            &song.title,
            &song.artists,
            &song.album,
            song.duration,
            "Current",
        );
        let _ = msg.reply(&text).await;
    } else {
        let _ = msg.reply(&utils::format_simple_info("Nothing is playing.")).await;
    }
}

async fn handle_help(msg: ferogram::types::IncomingMessage, _state: Arc<AppState>) {
    let text = utils::format_help("SecretMusicBot");
    let _ = msg.reply(&text).await;
}

async fn play_next(chat_id: i64, song: Song, state: Arc<AppState>) {
    state.current.insert(chat_id, song.clone());

    if let Err(e) = state.vc.change_stream(chat_id, &song.media_url) {
        log::error!("Change stream error: {}", e);
        state.current.remove(&chat_id);
        return;
    }

    let reply_text = utils::format_playing_message(
        "SecretMusicBot",
        &song.title,
        &song.artists,
        &song.album,
        song.duration,
        "Auto",
    );
    let _ = state.client.send_message(chat_id, &reply_text).await;

    let state_monitor = Arc::clone(&state);
    tokio::spawn(async move {
        spawn_monitor(chat_id, song.duration, state_monitor).await;
    });
}

async fn spawn_monitor(chat_id: i64, duration: u64, state: Arc<AppState>) {
    let token = CancellationToken::new();
    state.cancel_tokens.insert(chat_id, token.clone());

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(duration)) => {
            state.cancel_tokens.remove(&chat_id);
            state.current.remove(&chat_id);

            if let Some((_, mut queue)) = state.queues.remove(&chat_id) {
                if let Some(next_song) = queue.pop_front() {
                    state.queues.insert(chat_id, queue);
                    play_next(chat_id, next_song, Arc::clone(&state)).await;
                } else {
                    state.connected.remove(&chat_id);
                    let _ = state.vc.stop(chat_id);
                }
            } else {
                state.connected.remove(&chat_id);
                let _ = state.vc.stop(chat_id);
            }
        }
        _ = token.cancelled() => {
            state.cancel_tokens.remove(&chat_id);
        }
    }
}

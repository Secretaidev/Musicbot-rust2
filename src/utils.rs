pub fn to_bold(text: &str) -> String {
    text.chars().map(|c| match c {
        'A'..='Z' => char::from_u32(0x1D400 + (c as u32 - 'A' as u32)).unwrap_or(c),
        'a'..='z' => char::from_u32(0x1D41A + (c as u32 - 'a' as u32)).unwrap_or(c),
        '0'..='9' => char::from_u32(0x1D7CE + (c as u32 - '0' as u32)).unwrap_or(c),
        _ => c,
    }).collect()
}

pub fn to_italic(text: &str) -> String {
    text.chars().map(|c| match c {
        'A'..='Z' => char::from_u32(0x1D434 + (c as u32 - 'A' as u32)).unwrap_or(c),
        'a'..='z' => char::from_u32(0x1D44E + (c as u32 - 'a' as u32)).unwrap_or(c),
        _ => c,
    }).collect()
}

pub fn format_duration(seconds: u64) -> String {
    let m = seconds / 60;
    let s = seconds % 60;
    format!("{:02}:{:02}", m, s)
}

pub fn format_playing_message(
    bot_name: &str,
    song_title: &str,
    artists: &str,
    album: &str,
    duration: u64,
    requested_by: &str,
) -> String {
    format!(
        "🎵 {}\n🎶 {}\n├─ {}: {}\n├─ {}: {}\n├─ {}: {}\n├─ {}: {}\n└─ {}: {}",
        to_bold(bot_name),
        to_bold("Now Playing"),
        to_bold("Song"),
        to_italic(song_title),
        to_bold("Artist"),
        to_italic(artists),
        to_bold("Album"),
        to_italic(album),
        to_bold("Duration"),
        to_italic(&format_duration(duration)),
        to_bold("Requested by"),
        to_italic(requested_by),
    )
}

pub fn format_queue(bot_name: &str, songs: &[(usize, String, String)]) -> String {
    let mut text = format!("🎵 {}\n📜 {}\n", to_bold(bot_name), to_bold("Queue"));
    for (i, title, artists) in songs {
        text.push_str(&format!("{}. {} - {}\n", i, to_italic(title), to_italic(artists)));
    }
    text
}

pub fn format_help(bot_name: &str) -> String {
    format!(
        "🎵 {}\n📖 {}\n\
        /play <song> - {}\n\
        /skip - {}\n\
        /stop - {}\n\
        /pause - {}\n\
        /resume - {}\n\
        /volume <1-200> - {}\n\
        /queue - {}\n\
        /nowplaying - {}\n\
        /help - {}",
        to_bold(bot_name),
        to_bold("Help"),
        to_italic("Play a song"),
        to_italic("Skip current song"),
        to_italic("Stop and leave VC"),
        to_italic("Pause playback"),
        to_italic("Resume playback"),
        to_italic("Set volume"),
        to_italic("Show queue"),
        to_italic("Show current song"),
        to_italic("Show this message"),
    )
}

pub fn format_simple_error(msg: &str) -> String {
    format!("❌ {}", to_bold(msg))
}

pub fn format_simple_info(msg: &str) -> String {
    format!("✅ {}", to_bold(msg))
}

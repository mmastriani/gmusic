use crate::models::Song;
use std::fs;
use std::path::Path;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;

pub fn scan_songs_recursive(dir: &Path) -> Vec<Song> {
    let mut songs = Vec::new();
    scan_songs_helper(dir, &mut songs);
    songs.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    songs
}

fn scan_songs_helper(dir: &Path, songs: &mut Vec<Song>) {
    let audio_extensions = ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus", "wma"];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') {
                        scan_songs_helper(&path, songs);
                    }
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if audio_extensions.contains(&ext.to_lowercase().as_str()) {
                        songs.push(get_basic_song_info(&path));
                    }
                }
            }
        }
    }
}

pub fn get_basic_song_info(path: &Path) -> Song {
    let default_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Song")
        .to_string();

    Song {
        title: default_title,
        artist: "Loading...".to_string(),
        album: "Loading...".to_string(),
        duration_secs: 0,
        path: path.to_path_buf(),
    }
}

pub fn extract_song_metadata(path: &Path) -> Song {
    let default_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Song")
        .to_string();

    let mut title = default_title;
    let mut artist = "Unknown Artist".to_string();
    let mut album = "Unknown Album".to_string();
    let mut duration_secs = 0u64;

    if let Ok(mut file) = std::fs::File::open(path) {
        if let Ok(probe) = Probe::new(&mut file).guess_file_type() {
            let probe = probe.options(lofty::config::ParseOptions::new().read_properties(true));
            if let Ok(tagged_file) = probe.read() {
                duration_secs = tagged_file.properties().duration().as_secs();
            if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                if let Some(t) = tag.title() {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        title = trimmed.to_string();
                    }
                }
                if let Some(a) = tag.artist() {
                    let trimmed = a.trim();
                    if !trimmed.is_empty() {
                        artist = trimmed.to_string();
                    }
                }
                if let Some(al) = tag.album() {
                    let trimmed = al.trim();
                    if !trimmed.is_empty() {
                        album = trimmed.to_string();
                    }
                }
            }
        }
        }
    }

    Song {
        title,
        artist,
        album,
        duration_secs,
        path: path.to_path_buf(),
    }
}

pub fn format_duration(secs: u64) -> String {
    if secs == 0 {
        return "--:--".to_string();
    }
    let mins = secs / 60;
    let remaining_secs = secs % 60;
    format!("{}:{:02}", mins, remaining_secs)
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use gtk4 as gtk;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub music_directories: Vec<PathBuf>,
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let mut path = gtk::glib::user_config_dir();
        path.push("gmusic");
        path.push("config.json");
        path
    }

    pub fn default_music_dir() -> PathBuf {
        gtk::glib::user_special_dir(gtk::glib::UserDirectory::Music).unwrap_or_else(|| {
            let mut home = gtk::glib::home_dir();
            home.push("Music");
            home
        })
    }

    pub fn default_config() -> Self {
        Self {
            music_directories: vec![Self::default_music_dir()],
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    if !config.music_directories.is_empty() {
                        return config;
                    }
                }
            }
        }
        let config = Self::default_config();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }
}

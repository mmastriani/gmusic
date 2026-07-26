use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use gtk4 as gtk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

pub mod folder_item {
    use super::*;
    use glib::subclass::prelude::*;

    pub mod imp {
        use super::*;

        #[derive(Default)]
        pub struct FolderItem {
            pub name: RefCell<String>,
            pub path: RefCell<PathBuf>,
            pub children: RefCell<Option<gio::ListStore>>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for FolderItem {
            const NAME: &'static str = "GMusicFolderItem";
            type Type = super::FolderItem;
        }

        impl ObjectImpl for FolderItem {}
    }

    glib::wrapper! {
        pub struct FolderItem(ObjectSubclass<imp::FolderItem>);
    }

    impl FolderItem {
        pub fn new(name: &str, path: PathBuf, children: Option<gio::ListStore>) -> Self {
            let obj: Self = glib::Object::builder().build();
            let imp = obj.imp();
            *imp.name.borrow_mut() = name.to_string();
            *imp.path.borrow_mut() = path;
            *imp.children.borrow_mut() = children;
            obj
        }

        pub fn name(&self) -> String {
            self.imp().name.borrow().clone()
        }

        pub fn path(&self) -> PathBuf {
            self.imp().path.borrow().clone()
        }

        pub fn children(&self) -> Option<gio::ListStore> {
            self.imp().children.borrow().clone()
        }

        pub fn has_children(&self) -> bool {
            if let Some(ref c) = *self.imp().children.borrow() {
                c.n_items() > 0
            } else {
                false
            }
        }
    }
}

pub use folder_item::FolderItem;

pub mod song_item {
    use super::*;
    use glib::subclass::prelude::*;

    pub mod imp {
        use super::*;

        #[derive(Default)]
        pub struct SongItem {
            pub title: RefCell<String>,
            pub artist: RefCell<String>,
            pub album: RefCell<String>,
            pub duration_secs: RefCell<u64>,
            pub path: RefCell<PathBuf>,
            pub index: RefCell<usize>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for SongItem {
            const NAME: &'static str = "GMusicSongItem";
            type Type = super::SongItem;
        }

        impl ObjectImpl for SongItem {}
    }

    glib::wrapper! {
        pub struct SongItem(ObjectSubclass<imp::SongItem>);
    }

    impl SongItem {
        pub fn new(song: &Song, index: usize) -> Self {
            let obj: Self = glib::Object::builder().build();
            let imp = obj.imp();
            *imp.title.borrow_mut() = song.title.clone();
            *imp.artist.borrow_mut() = song.artist.clone();
            *imp.album.borrow_mut() = song.album.clone();
            *imp.duration_secs.borrow_mut() = song.duration_secs;
            *imp.path.borrow_mut() = song.path.clone();
            *imp.index.borrow_mut() = index;
            obj
        }

        pub fn title(&self) -> String {
            self.imp().title.borrow().clone()
        }

        pub fn artist(&self) -> String {
            self.imp().artist.borrow().clone()
        }

        pub fn album(&self) -> String {
            self.imp().album.borrow().clone()
        }

        pub fn duration_secs(&self) -> u64 {
            *self.imp().duration_secs.borrow()
        }

        pub fn index(&self) -> usize {
            *self.imp().index.borrow()
        }

        pub fn update_metadata(&self, title: &str, artist: &str, album: &str, duration_secs: u64) {
            let imp = self.imp();
            *imp.title.borrow_mut() = title.to_string();
            *imp.artist.borrow_mut() = artist.to_string();
            *imp.album.borrow_mut() = album.to_string();
            *imp.duration_secs.borrow_mut() = duration_secs;
        }

        pub fn path(&self) -> std::path::PathBuf {
            self.imp().path.borrow().clone()
        }
    }
}

pub use song_item::SongItem;

#[derive(Clone, Debug)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u64,
    pub path: PathBuf,
}

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

pub fn build_folder_item(name: &str, path: PathBuf) -> FolderItem {
    let mut subdirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                if let Some(file_name) = sub_path.file_name().and_then(|n| n.to_str()) {
                    if !file_name.starts_with('.') {
                        subdirs.push((file_name.to_string(), sub_path));
                    }
                }
            }
        }
    }

    subdirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    let children = if !subdirs.is_empty() {
        let children_store = gio::ListStore::new::<FolderItem>();
        for (sub_name, sub_path) in subdirs {
            let child_item = build_folder_item(&sub_name, sub_path);
            children_store.append(&child_item);
        }
        Some(children_store)
    } else {
        None
    };

    FolderItem::new(name, path, children)
}

pub fn build_root_folder_store(config: &AppConfig) -> gio::ListStore {
    let root_store = gio::ListStore::new::<FolderItem>();

    if config.music_directories.len() == 1 {
        let root_path = &config.music_directories[0];
        let mut subdirs = Vec::new();
        if let Ok(entries) = fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    if let Some(file_name) = sub_path.file_name().and_then(|n| n.to_str()) {
                        if !file_name.starts_with('.') {
                            subdirs.push((file_name.to_string(), sub_path));
                        }
                    }
                }
            }
        }
        subdirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        for (sub_name, sub_path) in subdirs {
            let child_item = build_folder_item(&sub_name, sub_path);
            root_store.append(&child_item);
        }
    }
 else if config.music_directories.len() > 1 {
        for dir in &config.music_directories {
            let folder_name = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_else(|| dir.to_str().unwrap_or("Music"));
            let folder_item = build_folder_item(folder_name, dir.clone());
            root_store.append(&folder_item);
        }
    }

    root_store
}

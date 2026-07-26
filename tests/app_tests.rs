use gmusic::{
    build_root_folder_store, format_duration, scan_songs_recursive, AppConfig, FolderItem,
};
use gtk4::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(0), "--:--");
    assert_eq!(format_duration(65), "1:05");
    assert_eq!(format_duration(307), "5:07");
}

#[test]
fn test_scan_songs_recursive() {
    let tmp_dir = std::env::temp_dir().join("gmusic_test_scan");
    let _ = fs::remove_dir_all(&tmp_dir);
    let sub_dir = tmp_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    let dummy_mp3 = tmp_dir.join("track1.mp3");
    let dummy_flac = sub_dir.join("track2.flac");
    let dummy_txt = tmp_dir.join("notes.txt");

    fs::write(&dummy_mp3, b"dummy audio content").unwrap();
    fs::write(&dummy_flac, b"dummy audio content").unwrap();
    fs::write(&dummy_txt, b"not audio").unwrap();

    let songs = scan_songs_recursive(&tmp_dir);
    assert_eq!(songs.len(), 2);
    let titles: Vec<String> = songs.iter().map(|s| s.title.clone()).collect();
    assert!(titles.contains(&"track1".to_string()));
    assert!(titles.contains(&"track2".to_string()));

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_default_config() {
    let config = AppConfig::default_config();
    assert!(!config.music_directories.is_empty());
    assert_eq!(config.music_directories[0], AppConfig::default_music_dir());
}

#[test]
fn test_config_save_load() {
    let tmp_dir = std::env::temp_dir().join("gmusic_test_config");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let test_file = tmp_dir.join("config.json");
    let config = AppConfig {
        music_directories: vec![PathBuf::from("/tmp/music1"), PathBuf::from("/tmp/music2")],
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&test_file, json).unwrap();

    let read_content = fs::read_to_string(&test_file).unwrap();
    let loaded: AppConfig = serde_json::from_str(&read_content).unwrap();

    assert_eq!(loaded.music_directories.len(), 2);
    assert_eq!(loaded.music_directories[0], PathBuf::from("/tmp/music1"));
    assert_eq!(loaded.music_directories[1], PathBuf::from("/tmp/music2"));

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_build_root_folder_store_single_vs_multiple() {
    let tmp_root = std::env::temp_dir().join("gmusic_test_tree");
    let _ = fs::remove_dir_all(&tmp_root);

    let dir1 = tmp_root.join("MusicDir1");
    let dir1_sub = dir1.join("Rock");
    let dir2 = tmp_root.join("MusicDir2");

    fs::create_dir_all(&dir1_sub).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    let config_single = AppConfig {
        music_directories: vec![dir1.clone()],
    };
    let store_single = build_root_folder_store(&config_single);
    assert_eq!(store_single.n_items(), 1);
    let item = store_single
        .item(0)
        .unwrap()
        .downcast::<FolderItem>()
        .unwrap();
    assert_eq!(item.name(), "Rock");


    let config_multi = AppConfig {
        music_directories: vec![dir1, dir2],
    };
    let store_multi = build_root_folder_store(&config_multi);
    assert_eq!(store_multi.n_items(), 2);

    let _ = fs::remove_dir_all(&tmp_root);
}

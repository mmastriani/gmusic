use crate::models::{FolderItem, Song, SongItem};
use crate::player::play_track;
use crate::state::SharedPlayerState;
use crate::utils::{extract_song_metadata, format_duration, scan_songs_recursive};
use gtk4 as gtk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::{Button, Box as GtkBox, Label, NoSelection, Orientation, SizeGroup, SizeGroupMode};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static CURRENT_FOLDER_VERSION: AtomicUsize = AtomicUsize::new(0);

pub fn build_main_panel(
    player_state: SharedPlayerState,
    play_btn: Button,
) -> (GtkBox, Rc<dyn Fn(&FolderItem)>) {
    let main_panel = GtkBox::new(Orientation::Vertical, 0);
    main_panel.set_hexpand(true);
    main_panel.set_vexpand(true);
    main_panel.add_css_class("main-panel-box");

    let header_box = GtkBox::new(Orientation::Vertical, 4);

    let folder_title_label = Label::builder()
        .label("Songs")
        .halign(gtk::Align::Start)
        .css_classes(vec!["folder-title"])
        .build();

    let folder_subtitle_label = Label::builder()
        .label("0 Music")
        .halign(gtk::Align::Start)
        .css_classes(vec!["folder-subtitle"])
        .build();

    let action_bar = GtkBox::new(Orientation::Horizontal, 12);
    action_bar.set_margin_bottom(20);

    let more_btn = Button::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("Options")
        .css_classes(vec!["action-circle-btn"])
        .build();

    action_bar.append(&more_btn);

    header_box.append(&folder_title_label);
    header_box.append(&folder_subtitle_label);
    header_box.append(&action_bar);

    main_panel.append(&header_box);

    let songs_store = gio::ListStore::new::<SongItem>();
    let selection_model = NoSelection::new(Some(songs_store.clone()));

    let title_sg = SizeGroup::new(SizeGroupMode::Horizontal);
    let artist_sg = SizeGroup::new(SizeGroupMode::Horizontal);
    let album_sg = SizeGroup::new(SizeGroupMode::Horizontal);
    let duration_sg = SizeGroup::new(SizeGroupMode::Horizontal);

    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup({
        let title_sg = title_sg.clone();
        let artist_sg = artist_sg.clone();
        let album_sg = album_sg.clone();
        let duration_sg = duration_sg.clone();
        move |_, list_item| {
            let row_box = GtkBox::new(Orientation::Horizontal, 16);
            row_box.add_css_class("song-row");

            let title_lbl = Label::builder()
                .halign(gtk::Align::Start)
                .xalign(0.0)
                .hexpand(true)
                .width_request(240)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .css_classes(vec!["song-title"])
                .build();

            let artist_lbl = Label::builder()
                .halign(gtk::Align::Start)
                .xalign(0.0)
                .hexpand(true)
                .width_request(160)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .css_classes(vec!["song-artist"])
                .build();

            let album_lbl = Label::builder()
                .halign(gtk::Align::Start)
                .xalign(0.0)
                .hexpand(true)
                .width_request(160)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .css_classes(vec!["song-album"])
                .build();

            let duration_lbl = Label::builder()
                .halign(gtk::Align::End)
                .xalign(1.0)
                .width_request(50)
                .css_classes(vec!["song-duration"])
                .build();

            let star_btn = Button::builder()
                .icon_name("non-starred-symbolic")
                .css_classes(vec!["flat", "star-btn"])
                .tooltip_text("Favoritar")
                .build();

            let more_btn = Button::builder()
                .icon_name("view-more-symbolic")
                .css_classes(vec!["flat"])
                .tooltip_text("Options")
                .build();

            title_sg.add_widget(&title_lbl);
            artist_sg.add_widget(&artist_lbl);
            album_sg.add_widget(&album_lbl);
            duration_sg.add_widget(&duration_lbl);

            row_box.append(&title_lbl);
            row_box.append(&artist_lbl);
            row_box.append(&album_lbl);
            row_box.append(&duration_lbl);
            row_box.append(&star_btn);
            row_box.append(&more_btn);

            list_item.set_child(Some(&row_box));
        }
    });

    struct MetadataUpdate {
        folder_version: usize,
        index: usize,
        meta: Song,
    }

    struct MetadataRequest {
        folder_version: usize,
        index: usize,
        path: std::path::PathBuf,
    }

    let updates = std::sync::Arc::new(std::sync::Mutex::new(Vec::<MetadataUpdate>::new()));
    let updates_ui = updates.clone();
    let store_channel = songs_store.clone();
    let player_state_idle = player_state.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        let mut queued = Vec::new();
        if let Ok(mut guard) = updates_ui.lock() {
            std::mem::swap(&mut queued, &mut *guard);
        }
        for update in queued {
            if CURRENT_FOLDER_VERSION.load(Ordering::SeqCst) == update.folder_version {
                if let Some(_item_obj) = store_channel.item(update.index as u32) {
                    let new_song = Song {
                        title: update.meta.title.clone(),
                        artist: update.meta.artist.clone(),
                        album: update.meta.album.clone(),
                        duration_secs: update.meta.duration_secs,
                        path: update.meta.path.clone(),
                    };
                    let new_song_item = SongItem::new(&new_song, update.index);
                    store_channel.splice(
                        update.index as u32,
                        1,
                        &[new_song_item.upcast::<glib::Object>()],
                    );
                }
                let mut st = player_state_idle.borrow_mut();
                if st.playlist.len() > update.index {
                    st.playlist[update.index] = update.meta.clone();
                }
            }
        }
        glib::ControlFlow::Continue
    });

    let (req_tx, req_rx) = std::sync::mpsc::channel::<MetadataRequest>();
    let updates_bg = updates.clone();

    std::thread::spawn(move || {
        while let Ok(req) = req_rx.recv() {
            if CURRENT_FOLDER_VERSION.load(Ordering::SeqCst) != req.folder_version {
                continue;
            }
            let meta = extract_song_metadata(&req.path);
            if let Ok(mut guard) = updates_bg.lock() {
                guard.push(MetadataUpdate {
                    folder_version: req.folder_version,
                    index: req.index,
                    meta,
                });
            }
        }
    });

    let req_tx_bind = req_tx.clone();
    factory.connect_bind(move |_, list_item| {
        if let Some(row_box) = list_item.child().and_downcast::<GtkBox>() {
            if let Some(item_obj) = list_item.item() {
                if let Some(song_item) = item_obj.downcast_ref::<SongItem>() {
                    if song_item.artist() == "Loading..." {
                        println!("Requesting index {}", song_item.index());
                        let _ = req_tx_bind.send(MetadataRequest {
                            folder_version: CURRENT_FOLDER_VERSION.load(Ordering::SeqCst),
                            index: song_item.index(),
                            path: song_item.path(),
                        });
                    }

                    let mut child = row_box.first_child();

                    if let Some(ref c) = child {
                        if let Some(lbl) = c.downcast_ref::<Label>() {
                            let idx = song_item.index();
                            let raw_title = song_item.title();
                            let track_title = if raw_title
                                .chars()
                                .next()
                                .map_or(false, |ch| ch.is_ascii_digit())
                            {
                                raw_title
                            } else {
                                format!("{:02} {}", idx + 1, raw_title)
                            };
                            lbl.set_text(&track_title);
                        }
                    }
                    child = child.and_then(|c| c.next_sibling());

                    if let Some(ref c) = child {
                        if let Some(lbl) = c.downcast_ref::<Label>() {
                            lbl.set_text(&song_item.artist());
                        }
                    }
                    child = child.and_then(|c| c.next_sibling());

                    if let Some(ref c) = child {
                        if let Some(lbl) = c.downcast_ref::<Label>() {
                            lbl.set_text(&song_item.album());
                        }
                    }
                    child = child.and_then(|c| c.next_sibling());

                    if let Some(ref c) = child {
                        if let Some(lbl) = c.downcast_ref::<Label>() {
                            lbl.set_text(&format_duration(song_item.duration_secs()));
                        }
                    }
                }
            }
        }
    });

    let list_view = gtk::ListView::new(Some(selection_model), Some(factory));
    list_view.add_css_class("songs-listview");

    let player_state_act = player_state.clone();
    let play_btn_act = play_btn.clone();
    list_view.connect_activate(move |_, position| {
        play_track(&player_state_act, position as usize, &play_btn_act);
    });

    let card_box = GtkBox::new(Orientation::Vertical, 0);
    card_box.add_css_class("songs-card");
    card_box.set_overflow(gtk::Overflow::Hidden);

    let scrolled_songs = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .child(&list_view)
        .build();

    card_box.append(&scrolled_songs);
    main_panel.append(&card_box);

    let title_lbl_clone = folder_title_label;
    let subtitle_lbl_clone = folder_subtitle_label;
    let store_clone = songs_store;
    let player_state_update = player_state;

    let update_panel = Rc::new(move |folder_item: &FolderItem| {
        let name = folder_item.name();
        let path = folder_item.path();

        title_lbl_clone.set_text(&name);

        let _folder_version = CURRENT_FOLDER_VERSION.fetch_add(1, Ordering::SeqCst) + 1;

        let songs = scan_songs_recursive(&path);
        subtitle_lbl_clone.set_text(&format!("{} Music", songs.len()));

        player_state_update.borrow_mut().playlist = songs.clone();

        store_clone.remove_all();

        let items: Vec<SongItem> = songs
            .iter()
            .enumerate()
            .map(|(idx, song)| SongItem::new(song, idx))
            .collect();

        store_clone.extend_from_slice(&items);
    });

    (main_panel, update_panel)
}

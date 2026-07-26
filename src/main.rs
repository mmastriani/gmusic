use gmusic::{
    AppConfig, FolderItem, Song, SongItem, build_root_folder_store, extract_song_metadata,
    format_duration, scan_songs_recursive,
};
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk4 as gtk;
mod audio;
use audio::AudioEngine;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Image, Label, ListBox,
    ListBoxRow, NoSelection, Orientation, Paned, Scale, SizeGroup, SizeGroupMode,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static CURRENT_FOLDER_VERSION: AtomicUsize = AtomicUsize::new(0);

fn setup_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        r#"
        window {
            background-color: #fafafa;
            color: #2e3436;
            font-family: cantarell, "Segoe UI", sans-serif;
        }

        window, window.background {
            border-radius: 12px;
            background-color: #fafafa;
        }

        headerbar, .titlebar {
            background-color: #fafafa;
            background-image: none;
            border: none;
            box-shadow: none;
            padding: 4px 8px;
        }

        headerbar windowcontrols button, .titlebar windowcontrols button, .titlebutton {
            background-color: rgba(0, 0, 0, 0.08);
            background-image: none;
            border-radius: 9999px;
            min-width: 20px;
            min-height: 20px;
            padding: 0;
            margin: 12px 4px;
            border: none;
        }

        headerbar windowcontrols button:hover, .titlebar windowcontrols button:hover, .titlebutton:hover {
            background-color: rgba(0, 0, 0, 0.15);
        }
        
        .pref-title, window.aboutdialog headerbar .title {
            font-weight: bold;
            font-size: 12pt;
        }

        window.aboutdialog label selection {
            background-color: transparent;
            color: inherit;
        }

        .version-pill {
            background-color: #e0eaf5;
            color: #1c71d8;
            border-radius: 9999px;
            padding: 4px 14px;
            font-weight: 700;
            font-size: 14px;
        }

        .about-title {
            font-weight: 800;
            font-size: 24pt;
        }

        .about-subtitle {
            font-size: 12pt;
            color: #555555;
        }

        .title-4 {
            font-weight: bold;
            font-size: 12pt;
        }

        .main-headerbar {
            border-bottom: 1px solid rgba(0, 0, 0, 0.08);
        }

        .navigation-sidebar {
            background-color: #ededed;
        }

        paned > separator {
            background-image: none;
            background-color: rgba(0, 0, 0, 0.08);
            min-width: 1px;
        }

        .navigation-sidebar row {
            border-radius: 8px;
            margin: 2px 6px;
            padding: 6px 10px;
            color: #2e3436;
        }

        .navigation-sidebar row:selected {
            background-color: #dedbd8;
            color: #000000;
            font-weight: 600;
        }

        .main-panel-box {
            background-color: #fafafa;
            padding: 28px 40px;
        }

        .folder-title {
            font-size: 28px;
            font-weight: 800;
            color: #1e1e1e;
        }

        .folder-subtitle {
            font-size: 14px;
            color: #777777;
            margin-top: 2px;
            margin-bottom: 16px;
        }

        .play-all-btn {
            border-radius: 50%;
            min-width: 44px;
            min-height: 44px;
            padding: 0;
            background-color: #e6e6e8;
            border: none;
            color: #2e3436;
        }

        .play-all-btn:hover {
            background-color: #d8d8dc;
        }

        .action-circle-btn {
            border-radius: 50%;
            min-width: 38px;
            min-height: 38px;
            padding: 0;
            background-color: #e6e6e8;
            border: none;
            color: #2e3436;
        }

        .action-circle-btn:hover {
            background-color: #d8d8dc;
        }

        .songs-card {
            background-color: #ffffff;
            border: 1px solid rgba(0, 0, 0, 0.08);
            border-radius: 12px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.03);
        }

        .songs-listview row {
            padding: 0;
            margin: 0;
            background-color: transparent;
        }

        .songs-listview row:hover {
            background-color: transparent;
        }

        .song-row {
            padding: 10px 18px;
            border-bottom: 1px solid #f0f0f0;
        }

        .song-row:first-child {
            border-top-left-radius: 12px;
            border-top-right-radius: 12px;
        }

        .song-row:last-child {
            border-bottom: none;
            border-bottom-left-radius: 12px;
            border-bottom-right-radius: 12px;
        }

        .song-row:hover {
            background-color: #f8f8f9;
        }

        .song-title {
            font-size: 14px;
            font-weight: 500;
            color: #2c2c2c;
        }

        .song-artist {
            font-size: 13px;
            color: #666666;
        }

        .song-album {
            font-size: 13px;
            color: #777777;
        }

        .song-duration {
            font-size: 13px;
            color: #888888;
            font-variant-numeric: tabular-nums;
        }

        .star-btn {
            color: #999999;
        }

        .star-btn:hover {
            color: #f6d32d;
        }

        .empty-state-label {
            font-size: 15px;
            color: #888888;
            padding: 40px;
        }

        .lcd-display {
            background-color: #ededed;
            border: 1px solid rgba(0, 0, 0, 0.08);
            border-radius: 6px;
            min-width: 550px;
            padding: 4px 12px;
            box-shadow: none;
        }

        .lcd-title {
            font-weight: 700;
            font-size: 13px;
            color: #2c2c2c;
        }

        .lcd-artist {
            font-size: 12px;
            color: #666666;
        }

        .lcd-time {
            font-size: 11px;
            color: #888888;
            font-variant-numeric: tabular-nums;
        }

        .lcd-scale {
            padding: 0 8px;
        }
        
        .lcd-scale slider {
            min-width: 0px;
            min-height: 0px;
            margin: 0px;
            padding: 0px;
            border: none;
            background-color: transparent;
            background-image: none;
            box-shadow: none;
            opacity: 0;
        }

        .lcd-scale trough {
            min-height: 3px;
            border: none;
            background-color: rgba(0, 0, 0, 0.15);
            border-radius: 3px;
        }

        .lcd-scale highlight {
            min-height: 3px;
            border-radius: 3px;
        }

        .media-btn {
            min-width: 50px;
            min-height: 50px;
            border-radius: 50%;
            background-color: transparent;
            background-image: none;
            border: none;
            padding: 0px;
            margin: 0px;
            box-shadow: none;
        }

        .media-btn:hover {
            background-color: rgba(0, 0, 0, 0.1);
        }

        .media-btn-play image {
            -gtk-icon-size: 30px;
        }

        .media-btn-skip image {
            -gtk-icon-size: 20px;
        }
        "#,
    );

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn main() {
    if std::env::var("GST_USE_DECODEBIN3").unwrap_or_default() != "0" {
        unsafe {
            std::env::set_var("GST_USE_PLAYBIN3", "0");
            std::env::set_var("GST_USE_DECODEBIN3", "0");
        }
        let exe = std::env::current_exe().expect("Failed to get current exe");
        let status = std::process::Command::new(exe)
            .status()
            .expect("Failed to spawn process");
        std::process::exit(status.code().unwrap_or(1));
    }

    gstreamer::init().expect("Failed to initialize gstreamer");
    let app = Application::builder()
        .application_id("org.example.GMusic")
        .build();

    app.connect_startup(setup_actions);
    app.connect_activate(build_ui);
    app.run();
}

thread_local! {
    static RELOAD_SIDEBAR: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };
}

fn trigger_sidebar_reload() {
    RELOAD_SIDEBAR.with(|r| {
        if let Some(ref reload) = *r.borrow() {
            reload();
        }
    });
}

fn setup_actions(app: &Application) {
    setup_css();
    let prefs_action = gio::SimpleAction::new("preferences", None);
    let app_clone = app.clone();
    prefs_action.connect_activate(move |_, _| {
        show_preferences(&app_clone);
    });
    app.add_action(&prefs_action);

    let about_action = gio::SimpleAction::new("about", None);
    let app_clone2 = app.clone();
    about_action.connect_activate(move |_, _| {
        show_about(&app_clone2);
    });
    app.add_action(&about_action);
}

fn choose_folder<F: Fn(PathBuf) + 'static>(parent: &gtk::Window, title: &str, on_chosen: F) {
    let chooser = gtk::FileChooserNative::new(
        Some(title),
        Some(parent),
        gtk::FileChooserAction::SelectFolder,
        Some("Select"),
        Some("Cancel"),
    );

    chooser.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    on_chosen(path);
                }
            }
        }
        dialog.destroy();
    });

    chooser.show();
}

fn show_preferences(app: &Application) {
    let window = app
        .active_window()
        .expect("No active window for Preferences");

    let header_bar = HeaderBar::new();
    let title_label = Label::builder()
        .label("Preferences")
        .css_classes(vec!["title", "pref-title"])
        .build();
    header_bar.set_title_widget(Some(&title_label));

    let prefs_window = gtk::Window::builder()
        .transient_for(&window)
        .modal(true)
        .title("Preferences")
        .default_width(540)
        .default_height(420)
        .build();

    prefs_window.set_titlebar(Some(&header_bar));

    let config = Rc::new(RefCell::new(AppConfig::load()));

    let main_box = GtkBox::new(Orientation::Vertical, 16);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);

    let section_title = Label::builder()
        .label("Music Directories")
        .halign(gtk::Align::Start)
        .css_classes(vec!["title-4"])
        .build();

    let section_desc = Label::builder()
        .label("Select the directories where the application will look for music (minimum 1).")
        .halign(gtk::Align::Start)
        .css_classes(vec!["dim-label"])
        .wrap(true)
        .build();

    let header_box = GtkBox::new(Orientation::Vertical, 4);
    header_box.append(&section_title);
    header_box.append(&section_desc);

    let list_box = ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .css_classes(vec!["boxed-list"])
        .build();

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(180)
        .vexpand(true)
        .child(&list_box)
        .build();

    let add_btn = Button::builder()
        .label("Add Directory")
        .icon_name("list-add-symbolic")
        .css_classes(vec!["suggested-action"])
        .build();

    let reset_btn = Button::builder()
        .label("Reset")
        .icon_name("edit-clear-all-symbolic")
        .build();

    let action_box = GtkBox::new(Orientation::Horizontal, 10);
    action_box.append(&add_btn);
    action_box.append(&reset_btn);

    main_box.append(&header_box);
    main_box.append(&scrolled_window);
    main_box.append(&action_box);

    prefs_window.set_child(Some(&main_box));

    let notify_config_changed = move || {
        trigger_sidebar_reload();
    };

    fn render_list(
        list_box: &ListBox,
        config: &Rc<RefCell<AppConfig>>,
        prefs_window: &gtk::Window,
        on_change: &Rc<dyn Fn()>,
    ) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        let cfg = config.borrow();
        let total_dirs = cfg.music_directories.len();

        for (index, dir_path) in cfg.music_directories.iter().enumerate() {
            let row = ListBoxRow::new();
            let row_box = GtkBox::new(Orientation::Horizontal, 10);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);

            let icon = Image::from_icon_name("folder-music-symbolic");
            let path_str = dir_path.to_string_lossy().to_string();
            let label = Label::builder()
                .label(&path_str)
                .halign(gtk::Align::Start)
                .hexpand(true)
                .ellipsize(gtk::pango::EllipsizeMode::Middle)
                .tooltip_text(&path_str)
                .build();

            let edit_btn = Button::builder()
                .icon_name("document-edit-symbolic")
                .tooltip_text("Change this directory")
                .css_classes(vec!["flat"])
                .build();

            let config_edit = config.clone();
            let win_edit = prefs_window.clone();
            let list_box_edit = list_box.clone();
            let on_change_edit = on_change.clone();

            edit_btn.connect_clicked(move |_| {
                let config_c = config_edit.clone();
                let win_c = win_edit.clone();
                let list_box_c = list_box_edit.clone();
                let on_change_c = on_change_edit.clone();

                choose_folder(&win_edit, "Change Music Directory", move |new_path| {
                    let mut cfg = config_c.borrow_mut();
                    if index < cfg.music_directories.len() {
                        cfg.music_directories[index] = new_path;
                        let _ = cfg.save();
                    }
                    drop(cfg);
                    on_change_c();
                    render_list(&list_box_c, &config_c, &win_c, &on_change_c);
                });
            });

            let remove_btn = Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Remove this directory")
                .css_classes(vec!["flat", "destructive-action"])
                .sensitive(total_dirs > 1)
                .build();

            let config_rm = config.clone();
            let win_rm = prefs_window.clone();
            let list_box_rm = list_box.clone();
            let on_change_rm = on_change.clone();

            remove_btn.connect_clicked(move |_| {
                let mut cfg = config_rm.borrow_mut();
                if cfg.music_directories.len() > 1 && index < cfg.music_directories.len() {
                    cfg.music_directories.remove(index);
                    let _ = cfg.save();
                }
                drop(cfg);
                on_change_rm();
                render_list(&list_box_rm, &config_rm, &win_rm, &on_change_rm);
            });

            row_box.append(&icon);
            row_box.append(&label);
            row_box.append(&edit_btn);
            row_box.append(&remove_btn);

            row.set_child(Some(&row_box));
            list_box.append(&row);
        }
    }

    let notify_cb: Rc<dyn Fn()> = Rc::new(notify_config_changed);
    render_list(&list_box, &config, &prefs_window, &notify_cb);

    let config_add = config.clone();
    let win_add = prefs_window.clone();
    let list_box_add = list_box.clone();
    let notify_cb_add = notify_cb.clone();

    add_btn.connect_clicked(move |_| {
        let config_c = config_add.clone();
        let win_c = win_add.clone();
        let list_box_c = list_box_add.clone();
        let notify_c = notify_cb_add.clone();

        choose_folder(&win_add, "Add Music Directory", move |new_path| {
            let mut cfg = config_c.borrow_mut();
            if !cfg.music_directories.contains(&new_path) {
                cfg.music_directories.push(new_path);
                let _ = cfg.save();
            }
            drop(cfg);
            notify_c();
            render_list(&list_box_c, &config_c, &win_c, &notify_c);
        });
    });

    let config_reset = config.clone();
    let win_reset = prefs_window.clone();
    let list_box_reset = list_box.clone();
    let notify_cb_reset = notify_cb.clone();

    reset_btn.connect_clicked(move |_| {
        let mut cfg = config_reset.borrow_mut();
        *cfg = AppConfig::default_config();
        let _ = cfg.save();
        drop(cfg);
        notify_cb_reset();
        render_list(&list_box_reset, &config_reset, &win_reset, &notify_cb_reset);
    });

    prefs_window.present();
}

fn show_about(app: &Application) {
    let window = app.active_window().expect("No window found for About");

    let about_win = gtk::Window::builder()
        .transient_for(&window)
        .modal(true)
        .title("")
        .default_width(360)
        .resizable(false)
        .hide_on_close(true)
        .build();

    let header_bar = gtk::HeaderBar::new();
    header_bar.set_show_title_buttons(true);
    about_win.set_titlebar(Some(&header_bar));

    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.set_margin_top(24);
    main_box.set_margin_bottom(32);
    main_box.set_margin_start(32);
    main_box.set_margin_end(32);
    main_box.set_halign(gtk::Align::Fill);

    let icon = gtk::Image::from_icon_name("gmusic");
    icon.set_pixel_size(128);
    icon.set_margin_bottom(24);

    let title_label = gtk::Label::new(Some("gMusic"));
    title_label.add_css_class("about-title");
    title_label.set_margin_bottom(8);

    let subtitle_label = gtk::Label::new(Some("The GNOME Project"));
    subtitle_label.add_css_class("about-subtitle");
    subtitle_label.set_margin_bottom(12);

    let version_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    version_box.set_halign(gtk::Align::Center);
    let version_label = gtk::Label::new(Some("0.1.0"));
    version_label.add_css_class("version-pill");
    version_box.append(&version_label);
    version_box.set_margin_bottom(32);

    let links_list = gtk::ListBox::new();
    links_list.add_css_class("boxed-list");
    links_list.set_selection_mode(gtk::SelectionMode::None);
    
    let website_row = gtk::ListBoxRow::new();
    let web_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    web_box.set_margin_top(14);
    web_box.set_margin_bottom(14);
    web_box.set_margin_start(14);
    web_box.set_margin_end(14);
    let web_lbl = gtk::Label::new(Some("Website"));
    web_lbl.set_hexpand(true);
    web_lbl.set_halign(gtk::Align::Start);
    let web_icon = gtk::Image::from_icon_name("external-link-symbolic");
    web_box.append(&web_lbl);
    web_box.append(&web_icon);
    website_row.set_child(Some(&web_box));
    links_list.append(&website_row);

    let issue_row = gtk::ListBoxRow::new();
    let issue_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    issue_box.set_margin_top(14);
    issue_box.set_margin_bottom(14);
    issue_box.set_margin_start(14);
    issue_box.set_margin_end(14);
    let issue_lbl = gtk::Label::new(Some("Report an Issue"));
    issue_lbl.set_hexpand(true);
    issue_lbl.set_halign(gtk::Align::Start);
    let issue_icon = gtk::Image::from_icon_name("external-link-symbolic");
    issue_box.append(&issue_lbl);
    issue_box.append(&issue_icon);
    issue_row.set_child(Some(&issue_box));
    links_list.append(&issue_row);

    let info_list = gtk::ListBox::new();
    info_list.add_css_class("boxed-list");
    info_list.set_selection_mode(gtk::SelectionMode::None);
    info_list.set_margin_top(16);

    let credits_row = gtk::ListBoxRow::new();
    let cred_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    cred_box.set_margin_top(14);
    cred_box.set_margin_bottom(14);
    cred_box.set_margin_start(14);
    cred_box.set_margin_end(14);
    let cred_lbl = gtk::Label::new(Some("Credits"));
    cred_lbl.set_hexpand(true);
    cred_lbl.set_halign(gtk::Align::Start);
    let cred_icon = gtk::Image::from_icon_name("go-next-symbolic");
    cred_box.append(&cred_lbl);
    cred_box.append(&cred_icon);
    credits_row.set_child(Some(&cred_box));
    info_list.append(&credits_row);

    let legal_row = gtk::ListBoxRow::new();
    let legal_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    legal_box.set_margin_top(14);
    legal_box.set_margin_bottom(14);
    legal_box.set_margin_start(14);
    legal_box.set_margin_end(14);
    let legal_lbl = gtk::Label::new(Some("Legal"));
    legal_lbl.set_hexpand(true);
    legal_lbl.set_halign(gtk::Align::Start);
    let legal_icon = gtk::Image::from_icon_name("go-next-symbolic");
    legal_box.append(&legal_lbl);
    legal_box.append(&legal_icon);
    legal_row.set_child(Some(&legal_box));
    info_list.append(&legal_row);

    let center_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    center_box.set_halign(gtk::Align::Center);
    center_box.append(&icon);
    center_box.append(&title_label);
    center_box.append(&subtitle_label);
    center_box.append(&version_box);

    main_box.append(&center_box);
    main_box.append(&links_list);
    main_box.append(&info_list);

    about_win.set_child(Some(&main_box));
    about_win.present();
}

fn create_sidebar_tree_view(
    config: &Rc<RefCell<AppConfig>>,
    on_folder_selected: Rc<dyn Fn(FolderItem)>,
) -> (gtk::ScrolledWindow, gtk::ListView, Rc<dyn Fn()>) {
    let list_view = gtk::ListView::new(None::<gtk::SelectionModel>, None::<gtk::ListItemFactory>);
    list_view.add_css_class("navigation-sidebar");

    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let expander = gtk::TreeExpander::new();
        let box_widget = GtkBox::new(Orientation::Horizontal, 8);
        box_widget.set_margin_start(4);
        box_widget.set_margin_end(4);
        box_widget.set_margin_top(4);
        box_widget.set_margin_bottom(4);

        let icon = Image::from_icon_name("folder-music-symbolic");
        let label = Label::builder()
            .halign(gtk::Align::Start)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();

        box_widget.append(&icon);
        box_widget.append(&label);
        expander.set_child(Some(&box_widget));

        list_item.set_child(Some(&expander));
    });

    factory.connect_bind(|_, list_item| {
        let expander = list_item
            .child()
            .and_downcast::<gtk::TreeExpander>()
            .expect("Expected TreeExpander");

        let row = list_item
            .item()
            .and_downcast::<gtk::TreeListRow>()
            .expect("Expected TreeListRow");

        expander.set_list_row(Some(&row));

        if let Some(item_obj) = row.item() {
            if let Some(folder_item) = item_obj.downcast_ref::<FolderItem>() {
                if let Some(box_widget) = expander.child().and_downcast::<GtkBox>() {
                    if let Some(label) = box_widget.last_child().and_downcast::<Label>() {
                        label.set_text(&folder_item.name());
                    }
                }
            }
        }
    });

    list_view.set_factory(Some(&factory));

    let config_clone = config.clone();
    let list_view_clone = list_view.clone();

    let update_sidebar = Rc::new(move || {
        *config_clone.borrow_mut() = AppConfig::load();
        let cfg = config_clone.borrow();
        let root_store = build_root_folder_store(&cfg);

        let tree_model = gtk::TreeListModel::new(root_store, false, false, |item| {
            let folder_item = item.downcast_ref::<FolderItem>()?;
            folder_item.children().map(|c| c.upcast())
        });

        let selection_model = gtk::SingleSelection::new(Some(tree_model));
        selection_model.set_autoselect(true);

        let on_select = on_folder_selected.clone();
        let on_select_init = on_folder_selected.clone();
        selection_model.connect_selection_changed(move |model, _pos, _n| {
            if let Some(item_obj) = model.selected_item() {
                if let Some(tree_row) = item_obj.downcast_ref::<gtk::TreeListRow>() {
                    if let Some(folder_obj) = tree_row.item() {
                        if let Some(folder_item) = folder_obj.downcast_ref::<FolderItem>() {
                            on_select(folder_item.clone());
                        }
                    }
                }
            }
        });

        list_view_clone.set_model(Some(&selection_model));

        if selection_model.n_items() > 0 {
            selection_model.select_item(0, true);
            if let Some(item_obj) = selection_model.selected_item() {
                if let Some(tree_row) = item_obj.downcast_ref::<gtk::TreeListRow>() {
                    if let Some(folder_obj) = tree_row.item() {
                        if let Some(folder_item) = folder_obj.downcast_ref::<FolderItem>() {
                            on_select_init(folder_item.clone());
                        }
                    }
                }
            }
        }
    });

    update_sidebar();

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .child(&list_view)
        .css_classes(vec!["navigation-sidebar"])
        .build();
    (scrolled_window, list_view, update_sidebar)
}

#[derive(Clone)]
pub struct LcdWidgets {
    pub stack: gtk::Stack,
    pub title_label: gtk::Label,
    pub artist_label: gtk::Label,
    pub time_label: gtk::Label,
    pub duration_label: gtk::Label,
    pub progress_scale: gtk::Scale,
}

#[derive(Default)]
pub struct PlayerState {
    pub playlist: Vec<Song>,
    pub current_index: Option<usize>,
    pub engine: std::rc::Rc<AudioEngine>,
    pub current_media: Option<PathBuf>,
    pub volume: f64,
    pub lcd_widgets: Option<LcdWidgets>,
}

impl PlayerState {
    pub fn default_with_vol() -> Self {
        Self {
            playlist: Vec::new(),
            current_index: None,
            engine: std::rc::Rc::new(AudioEngine::default()),
            current_media: None,
            volume: 0.5,
            lcd_widgets: None,
        }
    }
}

pub type SharedPlayerState = Rc<RefCell<PlayerState>>;

fn play_track(state: &SharedPlayerState, index: usize, play_btn: &Button) {
    let song = {
        let st = state.borrow();
        if index >= st.playlist.len() {
            return;
        }
        st.playlist[index].clone()
    };

    let vol = state.borrow().volume;

    state.borrow().engine.play_file(song.path.clone());
    state.borrow().engine.set_volume(vol);

    state.borrow_mut().current_index = Some(index);
    state.borrow_mut().current_media = Some(song.path.clone());

    if let Some(ref lcd) = state.borrow().lcd_widgets {
        lcd.stack.set_visible_child_name("player");
        lcd.title_label.set_label(&song.title);
        lcd.artist_label.set_label(&song.artist);

        let duration = song.duration_secs;
        let mins = duration / 60;
        let secs = duration % 60;
        lcd.duration_label
            .set_label(&format!("{}:{:02}", mins, secs));

        lcd.progress_scale.set_range(0.0, duration as f64);
        lcd.progress_scale.set_value(0.0);
        lcd.time_label.set_label("0:00");
    }

    play_btn.set_icon_name("media-playback-pause-symbolic");
}

fn toggle_play_pause(state: &SharedPlayerState, play_btn: &Button) {
    let has_media = state.borrow().current_media.is_some();
    let playlist_empty = state.borrow().playlist.is_empty();

    if playlist_empty {
        return;
    }

    if has_media {
        let is_playing = state
            .borrow()
            .engine
            .is_playing
            .load(std::sync::atomic::Ordering::SeqCst);
        if is_playing {
            state.borrow().engine.pause();
            play_btn.set_icon_name("media-playback-start-symbolic");
        } else {
            state.borrow().engine.resume();
            play_btn.set_icon_name("media-playback-pause-symbolic");
        }
    } else {
        play_track(state, 0, play_btn);
    }
}

fn play_previous_track(state: &SharedPlayerState, play_btn: &Button) {
    let (has_media, current_idx, playlist_len) = {
        let st = state.borrow();
        (
            st.current_media.is_some(),
            st.current_index,
            st.playlist.len(),
        )
    };

    if playlist_len == 0 {
        return;
    }

    if has_media {
        let timestamp = state
            .borrow()
            .engine
            .timestamp_us
            .load(std::sync::atomic::Ordering::SeqCst);
        if timestamp > 3_000_000 {
            state.borrow().engine.seek(0);
            return;
        }
    }

    if let Some(idx) = current_idx {
        if idx > 0 {
            play_track(state, idx - 1, play_btn);
        } else {
            if has_media {
                state.borrow().engine.seek(0);
            } else {
                play_track(state, 0, play_btn);
            }
        }
    } else {
        play_track(state, 0, play_btn);
    }
}

fn play_next_track(state: &SharedPlayerState, play_btn: &Button) {
    let (current_idx, playlist_len) = {
        let st = state.borrow();
        (st.current_index, st.playlist.len())
    };

    if playlist_len == 0 {
        return;
    }

    if let Some(idx) = current_idx {
        if idx + 1 < playlist_len {
            play_track(state, idx + 1, play_btn);
        } else {
            state.borrow().engine.stop();
            play_btn.set_icon_name("media-playback-start-symbolic");
        }
    } else {
        play_track(state, 0, play_btn);
    }
}

fn build_main_panel(
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

fn build_ui(app: &Application) {
    gtk::Window::set_default_icon_name("gmusic");
    let config = Rc::new(RefCell::new(AppConfig::load()));
    let player_state: SharedPlayerState = Rc::new(RefCell::new(PlayerState::default_with_vol()));

    let prev_btn = Button::builder()
        .icon_name("media-skip-backward-symbolic")
        .valign(gtk::Align::Center)
        .has_frame(false)
        .css_classes(vec!["media-btn", "media-btn-skip"])
        .build();
    let play_btn = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .valign(gtk::Align::Center)
        .has_frame(false)
        .css_classes(vec!["media-btn", "media-btn-play"])
        .build();
    let next_btn = Button::builder()
        .icon_name("media-skip-forward-symbolic")
        .valign(gtk::Align::Center)
        .has_frame(false)
        .css_classes(vec!["media-btn", "media-btn-skip"])
        .build();

    let volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_value(50.0);
    volume_scale.set_width_request(150);
    volume_scale.set_valign(gtk::Align::Center);

    let player_state_vol = player_state.clone();
    volume_scale.connect_value_changed(move |scale| {
        let vol = scale.value() / 100.0;
        let mut st = player_state_vol.borrow_mut();
        st.volume = vol;
        st.engine.set_volume(vol);
    });

    let player_state_play = player_state.clone();
    let play_btn_clone = play_btn.clone();
    play_btn.connect_clicked(move |_| {
        toggle_play_pause(&player_state_play, &play_btn_clone);
    });

    let player_state_prev = player_state.clone();
    let play_btn_prev = play_btn.clone();
    prev_btn.connect_clicked(move |_| {
        play_previous_track(&player_state_prev, &play_btn_prev);
    });

    let player_state_next = player_state.clone();
    let play_btn_next = play_btn.clone();
    next_btn.connect_clicked(move |_| {
        play_next_track(&player_state_next, &play_btn_next);
    });

    let left_box = GtkBox::new(Orientation::Horizontal, 5);
    left_box.append(&prev_btn);
    left_box.append(&play_btn);
    left_box.append(&next_btn);
    left_box.append(&volume_scale);

    let center_image = Image::from_icon_name("start-here-symbolic");
    center_image.set_pixel_size(24);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.add_named(&center_image, Some("logo"));

    let lcd_box = GtkBox::new(Orientation::Vertical, 0);
    lcd_box.add_css_class("lcd-display");

    let title_label = Label::builder()
        .label("")
        .css_classes(vec!["lcd-title"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .max_width_chars(30)
        .build();

    let artist_label = Label::builder()
        .label("")
        .css_classes(vec!["lcd-artist"])
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .max_width_chars(30)
        .build();

    let labels_box = GtkBox::new(Orientation::Vertical, 0);
    labels_box.append(&title_label);
    labels_box.append(&artist_label);

    let time_label = Label::builder()
        .label("0:00")
        .css_classes(vec!["lcd-time"])
        .build();

    let duration_label = Label::builder()
        .label("0:00")
        .css_classes(vec!["lcd-time"])
        .build();

    let progress_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    progress_scale.set_draw_value(false);
    progress_scale.set_hexpand(true);
    progress_scale.add_css_class("lcd-scale");

    let player_state_seek = player_state.clone();
    progress_scale.connect_change_value(move |scale, _scroll_type, value| {
        let st = player_state_seek.borrow();
        st.engine.seek(value as u64 * 1_000_000);
        scale.set_value(value);
        glib::Propagation::Stop
    });

    let bottom_box = GtkBox::new(Orientation::Horizontal, 4);
    bottom_box.append(&time_label);
    bottom_box.append(&progress_scale);
    bottom_box.append(&duration_label);

    lcd_box.append(&labels_box);
    lcd_box.append(&bottom_box);

    stack.add_named(&lcd_box, Some("player"));
    stack.set_halign(gtk::Align::Center);

    let center_box = GtkBox::new(Orientation::Horizontal, 0);
    center_box.set_hexpand(true);
    center_box.set_margin_start(40);
    center_box.set_margin_end(40);
    center_box.append(&stack);

    let lcd_widgets = LcdWidgets {
        stack,
        title_label,
        artist_label,
        time_label,
        duration_label,
        progress_scale,
    };
    player_state.borrow_mut().lcd_widgets = Some(lcd_widgets);

    let timer_state = player_state.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let st = timer_state.borrow();
        if let Some(ref lcd) = st.lcd_widgets {
            let is_playing = st.engine.is_playing.load(Ordering::SeqCst);
            if is_playing {
                let us = st.engine.timestamp_us.load(Ordering::SeqCst);
                let current_secs = us / 1_000_000;
                let mins = current_secs / 60;
                let secs = current_secs % 60;

                lcd.time_label.set_label(&format!("{}:{:02}", mins, secs));
                lcd.progress_scale.set_value(current_secs as f64);
            }
        }
        glib::ControlFlow::Continue
    });

    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("About"), Some("app.about"));

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .valign(gtk::Align::Center)
        .has_frame(false)
        .popover(&popover)
        .build();

    let header_bar = HeaderBar::new();
    header_bar.add_css_class("main-headerbar");
    header_bar.pack_start(&left_box);
    header_bar.pack_end(&menu_button);
    header_bar.set_title_widget(Some(&center_box));

    let (main_panel, update_main_panel) = build_main_panel(player_state, play_btn);

    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.set_width_request(240);
    sidebar.add_css_class("navigation-sidebar");

    let sidebar_header = Label::builder()
        .label("Library")
        .halign(gtk::Align::Start)
        .margin_start(14)
        .margin_top(14)
        .margin_bottom(8)
        .css_classes(vec!["title-4", "dim-label"])
        .build();
    sidebar.append(&sidebar_header);

    let on_folder_selected: Rc<dyn Fn(FolderItem)> = Rc::new(move |folder_item| {
        update_main_panel(&folder_item);
    });

    let (sidebar_tree_view, _, update_sidebar) =
        create_sidebar_tree_view(&config, on_folder_selected);
    sidebar.append(&sidebar_tree_view);

    RELOAD_SIDEBAR.with(|r| {
        *r.borrow_mut() = Some(update_sidebar);
    });

    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&main_panel));
    paned.set_position(350);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("gmusic")
        .default_width(1050)
        .default_height(720)
        .child(&paned)
        .build();

    window.set_titlebar(Some(&header_bar));

    let app_clone_close = app.clone();
    window.connect_close_request(move |_| {
        app_clone_close.quit();
        glib::Propagation::Proceed
    });

    window.present();
}

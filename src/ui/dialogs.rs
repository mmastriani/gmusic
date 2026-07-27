use crate::config::AppConfig;
use crate::ui::sidebar::trigger_sidebar_reload;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Button, Box as GtkBox, HeaderBar, Image, Label, ListBox, ListBoxRow, Orientation};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn choose_folder<F: Fn(PathBuf) + 'static>(parent: &gtk::Window, title: &str, on_chosen: F) {
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

pub fn show_preferences(app: &Application) {
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

pub fn show_about(app: &Application) {
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

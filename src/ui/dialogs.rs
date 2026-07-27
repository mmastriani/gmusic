use crate::config::AppConfig;
use crate::ui::sidebar::trigger_sidebar_reload;
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, Button, Box as GtkBox, Image, Label, ListBox, ListBoxRow, Orientation};
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

    let builder = gtk::Builder::from_string(include_str!("../../resources/ui/preferences.ui"));
    
    let prefs_window: gtk::Window = builder.object("prefs_window").expect("Could not get prefs_window");
    prefs_window.set_transient_for(Some(&window));
    
    let list_box: ListBox = builder.object("prefs_list_box").expect("Could not get prefs_list_box");
    let add_btn: Button = builder.object("add_btn").expect("Could not get add_btn");
    let reset_btn: Button = builder.object("reset_btn").expect("Could not get reset_btn");

    let config = Rc::new(RefCell::new(AppConfig::load()));

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

    let builder = gtk::Builder::from_string(include_str!("../../resources/ui/about.ui"));
    
    let about_win: gtk::Window = builder.object("about_window").expect("Could not get about_window");
    about_win.set_transient_for(Some(&window));

    about_win.present();
}

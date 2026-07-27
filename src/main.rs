mod config;
mod models;
mod utils;
mod state;
mod player;
mod ui;
pub mod audio;
use gstreamer as gst;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gio;
use gtk::Application;
use ui::dialogs::{show_about, show_preferences};
use ui::window::{build_ui, setup_css};

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

fn main() {
    gst::init().expect("Failed to initialize gstreamer");

    let app = Application::builder()
        .application_id("com.github.mastriani.gmusic")
        .build();

    app.connect_startup(|app| {
        setup_actions(app);
    });

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}

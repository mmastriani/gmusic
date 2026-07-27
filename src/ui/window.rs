use crate::config::AppConfig;
use crate::models::FolderItem;
use crate::player::{play_next_track, play_previous_track, toggle_play_pause};
use crate::state::{LcdWidgets, PlayerState, SharedPlayerState};
use crate::ui::main_panel::build_main_panel;
use crate::ui::sidebar::{create_sidebar_tree_view, RELOAD_SIDEBAR};
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use gtk::{
    Application, ApplicationWindow, Button, Box as GtkBox, Label,
    Paned, Scale,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;

pub fn setup_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}


pub fn build_ui(app: &Application) {
    gtk::Window::set_default_icon_name("gmusic");
    let config = Rc::new(RefCell::new(AppConfig::load()));
    let player_state: SharedPlayerState = Rc::new(RefCell::new(PlayerState::default_with_vol()));

    let builder = gtk::Builder::from_string(include_str!("../../resources/ui/main_window.ui"));
    
    let window: ApplicationWindow = builder.object("main_window").expect("Could not get main_window");
    window.set_application(Some(app));
    
    let prev_btn: Button = builder.object("prev_btn").expect("Could not get prev_btn");
    let play_btn: Button = builder.object("play_btn").expect("Could not get play_btn");
    let next_btn: Button = builder.object("next_btn").expect("Could not get next_btn");
    
    let volume_scale: Scale = builder.object("volume_scale").expect("Could not get volume_scale");
    volume_scale.set_range(0.0, 100.0);
    volume_scale.set_value(50.0);

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

    let stack: gtk::Stack = builder.object("stack").expect("Could not get stack");
    let title_label: Label = builder.object("title_label").expect("Could not get title_label");
    let artist_label: Label = builder.object("artist_label").expect("Could not get artist_label");
    let time_label: Label = builder.object("time_label").expect("Could not get time_label");
    let duration_label: Label = builder.object("duration_label").expect("Could not get duration_label");
    let progress_scale: Scale = builder.object("progress_scale").expect("Could not get progress_scale");
    
    progress_scale.set_range(0.0, 100.0);
    let player_state_seek = player_state.clone();
    progress_scale.connect_change_value(move |scale, _scroll_type, value| {
        let st = player_state_seek.borrow();
        st.engine.seek(value as u64 * 1_000_000);
        scale.set_value(value);
        glib::Propagation::Stop
    });

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
    let timer_play_btn = play_btn.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        let is_ended = timer_state.borrow().engine.is_ended.load(Ordering::SeqCst);
        if is_ended {
            timer_state.borrow().engine.is_ended.store(false, Ordering::SeqCst);
            play_next_track(&timer_state, &timer_play_btn);
        }

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

    let (main_panel, update_main_panel) = build_main_panel(player_state, play_btn);

    let sidebar: GtkBox = builder.object("sidebar").expect("Could not get sidebar");
    
    let on_folder_selected: Rc<dyn Fn(FolderItem)> = Rc::new(move |folder_item| {
        update_main_panel(&folder_item);
    });

    let (sidebar_tree_view, _, update_sidebar) =
        create_sidebar_tree_view(&config, on_folder_selected);
    
    sidebar.append(&sidebar_tree_view);

    RELOAD_SIDEBAR.with(|r| {
        *r.borrow_mut() = Some(update_sidebar);
    });

    let paned: Paned = builder.object("paned").expect("Could not get paned");
    paned.set_end_child(Some(&main_panel));

    let app_clone_close = app.clone();
    window.connect_close_request(move |_| {
        app_clone_close.quit();
        glib::Propagation::Proceed
    });

    window.present();
}


use crate::config::AppConfig;
use crate::models::FolderItem;
use crate::player::{play_next_track, play_previous_track, toggle_play_pause};
use crate::state::{LcdWidgets, PlayerState, SharedPlayerState};
use crate::ui::main_panel::build_main_panel;
use crate::ui::sidebar::{create_sidebar_tree_view, RELOAD_SIDEBAR};
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::glib;
use gtk::gio;
use gtk::{
    Application, ApplicationWindow, Button, Box as GtkBox, HeaderBar, Image, Label,
    Orientation, Paned, Scale,
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


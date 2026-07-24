use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gio;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Image, Label, Orientation,
    Scale, Separator,
};

fn main() {
    let app = Application::builder()
        .application_id("org.example.GMusic")
        .build();

    app.connect_startup(setup_actions);
    app.connect_activate(build_ui);
    app.run();
}

fn setup_actions(app: &Application) {
    // Ação: Preferências
    let prefs_action = gio::SimpleAction::new("preferences", None);
    let app_clone = app.clone();
    prefs_action.connect_activate(move |_, _| {
        show_preferences(&app_clone);
    });
    app.add_action(&prefs_action);

    // Ação: Sobre
    let about_action = gio::SimpleAction::new("about", None);
    let app_clone2 = app.clone();
    about_action.connect_activate(move |_, _| {
        show_about(&app_clone2);
    });
    app.add_action(&about_action);
}

fn show_preferences(app: &Application) {
    let window = app.active_window().expect("Sem janela ativa para Preferences");

    // Cria a HeaderBar para ter o visual "GNOME" (Client-Side Decorations)
    let header_bar = gtk::HeaderBar::new();
    let title_label = Label::builder()
        .label("Preferências")
        .css_classes(vec!["title"])
        .build();
    header_bar.set_title_widget(Some(&title_label));
    
    let prefs_window = gtk::Window::builder()
        .transient_for(&window)
        .modal(true)
        .title("Preferências")
        .default_width(400)
        .default_height(300)
        .build();

    prefs_window.set_titlebar(Some(&header_bar));

    let label = Label::builder()
        .label("Opções de preferências aparecerão aqui.")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    prefs_window.set_child(Some(&label));
    prefs_window.present();
}

fn show_about(app: &Application) {
    let window = app.active_window().expect("Sem janela ativa para About");
    
    let about = gtk::AboutDialog::builder()
        .transient_for(&window)
        .modal(true)
        .program_name("gmusic")
        .version("0.1.0")
        .comments("Um player de música simples em GTK4.")
        .logo_icon_name("audio-x-generic")
        .build();

    about.present();
}

fn build_ui(app: &Application) {
    // --- Top Bar ---

    // Left side: Prev, Play, Next buttons + Volume scale
    let prev_btn = Button::builder()
        .icon_name("media-skip-backward-symbolic")
        .build();
    let play_btn = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .build();
    let next_btn = Button::builder()
        .icon_name("media-skip-forward-symbolic")
        .build();

    // Volume scale
    let volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_value(50.0);
    volume_scale.set_width_request(100);
    volume_scale.set_valign(gtk::Align::Center); // Vertically center the scale

    let left_box = GtkBox::new(Orientation::Horizontal, 5);
    left_box.append(&prev_btn);
    left_box.append(&play_btn);
    left_box.append(&next_btn);
    left_box.append(&volume_scale);

    // Center: Gnome Icon (when not playing)
    let center_image = Image::from_icon_name("start-here-symbolic");
    center_image.set_pixel_size(24);
    let center_box = GtkBox::new(Orientation::Horizontal, 0);
    center_box.append(&center_image);

    // Menu Button (Hamburger)
    let menu = gio::Menu::new();
    menu.append(Some("Preferências"), Some("app.preferences"));
    menu.append(Some("Sobre"), Some("app.about"));

    let popover = gtk::PopoverMenu::from_model(Some(&menu));
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .popover(&popover)
        .build();

    // Create HeaderBar (the top bar)
    let header_bar = HeaderBar::new();
    header_bar.pack_start(&left_box);
    header_bar.pack_end(&menu_button);
    header_bar.set_title_widget(Some(&center_box));

    // --- Main Window Content ---

    // Left panel (Sidebar)
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.set_width_request(200);
    let sidebar_label = Label::builder()
        .label("Painel Esquerdo (Biblioteca, etc.)")
        .vexpand(true) // Center vertically
        .build();
    sidebar.append(&sidebar_label);

    // Main panel
    let main_panel = GtkBox::new(Orientation::Vertical, 0);
    main_panel.set_hexpand(true);
    let main_label = Label::builder()
        .label("Painel Principal (Músicas, Álbuns)")
        .vexpand(true) // Center vertically
        .build();
    main_panel.append(&main_label);

    // Split View
    let content_box = GtkBox::new(Orientation::Horizontal, 0);
    content_box.append(&sidebar);
    content_box.append(&Separator::new(Orientation::Vertical));
    content_box.append(&main_panel);

    // --- Window ---
    let window = ApplicationWindow::builder()
        .application(app)
        .title("gmusic")
        .default_width(1000)
        .default_height(700)
        .child(&content_box)
        .build();

    window.set_titlebar(Some(&header_bar));
    window.present();
}

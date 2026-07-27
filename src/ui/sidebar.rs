use crate::config::AppConfig;
use crate::models::{build_root_folder_store, FolderItem};
use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Image, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    pub static RELOAD_SIDEBAR: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };
}

pub fn trigger_sidebar_reload() {
    RELOAD_SIDEBAR.with(|r| {
        if let Some(ref reload) = *r.borrow() {
            reload();
        }
    });
}

pub fn create_sidebar_tree_view(
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

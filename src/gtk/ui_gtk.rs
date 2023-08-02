use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender, time::Duration};


use crate::{wallet_event::WalletEvent, gtk::functions::show_dialog_message_pop_up};

use gtk::{
    gdk,
    glib::{self, Priority},
    prelude::*,
    Application, CssProvider, StyleContext, Window,
};

use super::functions::{
    handle_ui_event, login_button_clicked, send_button_clicked, start_button_clicked, hex_string_to_bytes,
};
use super::ui_events::UIEvent;

/// Recibe un sender para enviarle el sender que envia eventos a la UI y un sender para enviarle eventos al nodo
/// Crea la UI y la ejecuta
pub fn run_ui(ui_sender: Sender<glib::Sender<UIEvent>>, sender_to_node: Sender<WalletEvent>) {
    let app = Application::builder()
        .application_id("org.gtk-rs.bitcoin")
        .build();
    app.connect_activate(move |_| {
        build_ui( &ui_sender, &sender_to_node);
    });
    let args: Vec<String> = vec![]; // necessary to not use main program args
    app.run_with_args(&args);
}

/// Recibe un sender para enviarle el sender que envia eventos a la UI y un sender para enviarle eventos al nodo
/// Inicializa la UI, carga el archivo glade y conecta los callbacks de los botones. Envia el sender que envia eventos a la UI al nodo y 
/// muestra la ventana inicial
fn build_ui(
    ui_sender: &Sender<glib::Sender<UIEvent>>,
    sender_to_node: &Sender<WalletEvent>,
) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("resources/interfaz.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let css_provider: CssProvider = CssProvider::new();
    css_provider
        .load_from_path("src/gtk/resources/styles.css")
        .expect("Failed to load CSS file.");
    let screen: gdk::Screen = gdk::Screen::default().expect("Failed to get default screen.");
    StyleContext::add_provider_for_screen(
        &screen,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );

    let initial_window: Window = builder.object("initial-window").unwrap();

    // login elements

    let status_login: gtk::Label = builder.object("status-login").unwrap();
    let ref_to_status_login = status_login.clone();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_to_loading_account_label = Rc::new(RefCell::new(loading_account_label.clone()));
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let ref2_to_dropdown = dropdown.clone();

    // send tab elements
    let send_balance: gtk::Label = builder.object("send-balance").unwrap();

    // overview tab elements
    let available_label: gtk::Label = builder.object("available label").unwrap();
    let ref_to_available_label = available_label.clone();
    // cuando cambia uno, cambia el otro
    ref_to_available_label.connect_notify_local(Some("label"), move |label, _| {
        let new_text = label.text().to_string();
        send_balance.set_label(new_text.as_str());
    });
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender.send(tx).expect("could not send sender to client");

    //initial_window.show();
    let main_window: gtk::Window = builder.object("main-window").unwrap();

    main_window.show();
    // SEARCH ENTRIES
    let search_blocks_entry: gtk::SearchEntry = builder.object("search-block").unwrap();
    let search_headers_entry: gtk::SearchEntry = builder.object("search-block-headers").unwrap();
    let search_blocks_button: gtk::Button = builder.object("search-blocks-button").unwrap();
    let search_headers_button: gtk::Button = builder.object("search-header-button").unwrap();
    let sender_to_find_block = sender_to_node.clone();

    search_blocks_button.connect_clicked(move |_| {
        let text = search_blocks_entry.text().to_string();
        if let Some(block_hash) = hex_string_to_bytes(text.as_str()) {
            println!("searching block {}", text);
            sender_to_find_block
                .send(WalletEvent::SearchBlock(block_hash))
                .unwrap();
        } else {
            show_dialog_message_pop_up(format!("Error {text} is not a valid block hash").as_str(), "Error searching block")
        }
        search_blocks_entry.set_text("");

    });
    let sender_to_find_header = sender_to_node.clone();
    search_headers_button.connect_clicked(move |_| {
        let text = search_headers_entry.text().to_string();
        if let Some(block_hash) = hex_string_to_bytes(text.as_str()) {
            println!("searching header {}", text);
            sender_to_find_header
                .send(WalletEvent::SearchHeader(block_hash))
                .unwrap();
        } else {
            show_dialog_message_pop_up(format!("Error {text} is not a valid block hash").as_str(), "Error searching header")
        }
        search_headers_entry.set_text("");
    });
    
    

    let sender_to_get_account = sender_to_node.clone();

    /*
        for i in 0..50 {
            let row = liststore_blocks.append();
            liststore_blocks.set(
                &row,
                &[
                    (0, &i.to_value()),
                    (1, &"new id"),
                    (2, &"new merkle root"),
                    (3, &50.to_value()),
                ],
            );
        }
        let row = liststore_blocks.append();
        liststore_blocks.set(
            &row,
            &[
                (0, &2001.to_value()),
                (1, &"new id"),
                (2, &"new merkle root"),
                (3, &50.to_value()),
            ],
        );
    */
    let ref_to_builder = builder.clone();
    rx.attach(None, move |msg| {
        handle_ui_event(ref_to_builder.clone(), msg, sender_to_get_account.clone());
        Continue(true)
    });
    gtk::glib::timeout_add_local(Duration::from_secs(5), move || {
        update_label(ref_to_loading_account_label.clone());
        Continue(true)
    });

    start_button_clicked(&builder.clone(), sender_to_node.clone());
    login_button_clicked(&builder.clone(), sender_to_node.clone());
    send_button_clicked(&builder.clone(), sender_to_node.clone());

    let sender_to_change_account = sender_to_node.clone();
    ref2_to_dropdown.connect_changed(move |combobox| {
        // Obtener el texto de la opción seleccionada
        if let Some(selected_text) = combobox.active_text() {
            if selected_text != ref_to_status_login.text() {
                ref_to_status_login.set_label(selected_text.as_str());
                ref_to_status_login.set_visible(true);
                if let Some(new_index) = combobox.active() {
                    sender_to_change_account
                        .send(WalletEvent::ChangeAccount(new_index as usize))
                        .unwrap();
                }
            }
        }
    });
    gtk::main();
}

fn update_label(label: Rc<RefCell<gtk::Label>>) -> Continue {
    let waiting_labels = [
        "Hold tight! Setting up your Bitcoin account...",
        "We're ensuring your account's security...",
        "Be patient! Your Bitcoin account is being created...",
    ];
    let current_text = label.borrow().text().to_string();
    for i in 0..waiting_labels.len() {
        if current_text == waiting_labels[i] {
            let next_text = waiting_labels[(i + 1) % waiting_labels.len()];
            label.borrow().set_text(next_text);
            break;
        }
    }
    Continue(true)
}

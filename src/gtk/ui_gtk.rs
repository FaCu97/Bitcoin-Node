use std::{cell::RefCell, rc::Rc, sync::mpsc::{Sender, self}, time::Duration};

use crate::{gtk::ui_functions::show_dialog_message_pop_up, wallet_event::WalletEvent};

use gtk::{
    glib::{self, Priority},
    prelude::*,
    Application, Window, Builder,
};

use super::ui_functions::{
    handle_ui_event, hex_string_to_bytes, login_button_clicked, send_button_clicked,
    start_button_clicked, update_label, add_css_to_screen,
};
use super::ui_events::UIEvent;

const GLADE_FILE: &str = include_str!("resources/interfaz.glade");

/// Recibe un sender para enviarle el sender que envia eventos a la UI y un sender para enviarle eventos al nodo
/// Crea la UI y la ejecuta
pub fn run_ui(ui_sender: Sender<glib::Sender<UIEvent>>, sender_to_node: Sender<WalletEvent>) {
    let app = Application::builder()
        .application_id("org.gtk-rs.bitcoin")
        .build();
    app.connect_activate(move |_| {
        build_ui(&ui_sender, &sender_to_node);
    });
    let args: Vec<String> = vec![]; // necessary to not use main program args
    app.run_with_args(&args);
}

/// Recibe un sender para enviarle el sender que envia eventos a la UI y un sender para enviarle eventos al nodo
/// Inicializa la UI, carga el archivo glade y conecta los callbacks de los botones. Envia el sender que envia eventos a la UI al nodo y
/// muestra la ventana inicial
fn build_ui(ui_sender: &Sender<glib::Sender<UIEvent>>, sender_to_node: &Sender<WalletEvent>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    // envio sender de eventos a la UI al thread del nodo
    ui_sender.send(tx).expect("could not send sender to client");
    let builder = gtk::Builder::from_string(GLADE_FILE);
    add_css_to_screen();
    let initial_window: Window = builder.object("initial-window").unwrap();
    initial_window.show();
    //let main_window: gtk::Window = builder.object("main-window").unwrap();
    //main_window.show();
    let tx_to_node = sender_to_node.clone();
    let builder_clone = builder.clone();
    rx.attach(None, move |msg| {
        handle_ui_event(builder_clone.clone(), msg, tx_to_node.clone());
        Continue(true)
    });
    handle_dynamic_ui(&builder, sender_to_node);
    gtk::main();
}



/// Recibe un builder y un sender para enviarle eventos al nodo
/// Conecta los callbacks de los botones y elementos dinamicos de la UI
pub fn handle_dynamic_ui(builder: &Builder, sender_to_node: &Sender<WalletEvent>) {
    start_button_clicked(builder, sender_to_node.clone());
    send_button_clicked(builder, sender_to_node.clone());
    sync_balance_labels(builder);
    search_blocks_button_clicked(builder, sender_to_node.clone());
    search_headers_button_clicked(builder, sender_to_node.clone());
    login_button_clicked(builder, sender_to_node.clone());
    dropdown_accounts_changed(builder, sender_to_node.clone());
    change_loading_account_label_periodically(builder); 
}

/// Realiza la accion correspondiente a apretar el boton de buscar bloques. Envia un evento al nodo para que busque el bloque
/// en caso de que el hash ingresado sea valido. En caso contrario muestra un mensaje de error
pub fn search_blocks_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let search_blocks_entry: gtk::SearchEntry = builder.object("search-block").unwrap();
    let search_blocks_button: gtk::Button = builder.object("search-blocks-button").unwrap();
    search_blocks_button.connect_clicked(move |_| {
        let text = search_blocks_entry.text().to_string();
        if let Some(block_hash) = hex_string_to_bytes(text.as_str()) {
            println!("searching block {}", text);
            sender
                .send(WalletEvent::SearchBlock(block_hash))
                .unwrap();
        } else {
            show_dialog_message_pop_up(
                format!("Error {text} is not a valid block hash").as_str(),
                "Error searching block",
            )
        }
        search_blocks_entry.set_text("");
    });
}

/// Realiza la accion correspondiente a apretar el boton de buscar headers. Envia un evento al nodo para que busque el header
/// en caso de que el hash ingresado sea valido. En caso contrario muestra un mensaje de error
pub fn search_headers_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let search_headers_entry: gtk::SearchEntry = builder.object("search-block-headers").unwrap();
    let search_headers_button: gtk::Button = builder.object("search-header-button").unwrap();
    search_headers_button.connect_clicked(move |_| {
        let text = search_headers_entry.text().to_string();
        if let Some(block_hash) = hex_string_to_bytes(text.as_str()) {
            println!("searching header {}", text);
            sender
                .send(WalletEvent::SearchHeader(block_hash))
                .unwrap();
        } else {
            show_dialog_message_pop_up(
                format!("Error {text} is not a valid block hash").as_str(),
                "Error searching header",
            )
        }
        search_headers_entry.set_text("");
    });
}

/// Realiza la accion correspondiente a apretar una opcion del dropdown de cuentas. Envia un evento al nodo para que cambie de cuenta
/// y muestra el address de la cuenta seleccionada
pub fn dropdown_accounts_changed(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let status_login: gtk::Label = builder.object("status-login").unwrap();
    dropdown.connect_changed(move |combobox| {
        // Obtener el texto de la opción seleccionada
        if let Some(selected_text) = combobox.active_text() {
            status_login.set_label(selected_text.as_str());
            status_login.set_visible(true);
            if let Some(new_index) = combobox.active() {
                sender
                    .send(WalletEvent::ChangeAccount(new_index as usize))
                    .unwrap();
            }
        }
    });
}

/// Sinconiza los labels de balance de la pestaña Overview y Send para que muestren el mismo balance
pub fn sync_balance_labels(builder: &Builder) {
    let available_label: gtk::Label = builder.object("available label").unwrap();
    let send_balance: gtk::Label = builder.object("send-balance").unwrap();
    let ref_to_available_label = available_label;
    // cuando cambia uno, cambia el otro automaticamente
    ref_to_available_label.connect_notify_local(Some("label"), move |label, _| {
        let new_text = label.text().to_string();
        send_balance.set_label(new_text.as_str());
    });
}

/// Cambia el label de loading account cada 5 segundos
pub fn change_loading_account_label_periodically(builder: &Builder) {
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_to_loading_account_label = Rc::new(RefCell::new(loading_account_label));
    gtk::glib::timeout_add_local(Duration::from_secs(5), move || {
        update_label(ref_to_loading_account_label.clone());
        Continue(true)
    });
}
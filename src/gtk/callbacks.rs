use super::ui_functions::{
    disable_buttons_and_entries, get_buttons, get_entries, hex_string_to_bytes,
    show_dialog_message_pop_up,
};
use crate::wallet_event::WalletEvent;
use gtk::{prelude::*, Builder, Spinner};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Sender},
    time::Duration,
};

/// Recibe un builder y un sender para enviarle eventos al nodo
/// Conecta los callbacks de los botones y elementos dinamicos de la UI
pub fn connect_ui_callbacks(builder: &Builder, sender_to_node: &Sender<WalletEvent>) {
    start_button_clicked(builder, sender_to_node.clone());
    send_button_clicked(builder, sender_to_node.clone());
    sync_balance_labels(builder);
    search_blocks_button_clicked(builder, sender_to_node.clone());
    search_headers_button_clicked(builder, sender_to_node.clone());
    login_button_clicked(builder, sender_to_node.clone());
    dropdown_accounts_changed(builder, sender_to_node.clone());
    change_loading_account_label_periodically(builder);
    search_tx_poi_button_clicked(builder, sender_to_node.clone());
}

/// Esta funcion realiza la accion que corresponde al presionar el boton de start
pub fn start_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let start_button: gtk::Button = builder.object("start-button").unwrap();
    let ref_start_btn = start_button.clone();
    start_button.connect_clicked(move |_| {
        sender.send(WalletEvent::Start).unwrap();
        ref_start_btn.set_visible(false);
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

/// Esta funcion realiza la accion que corresponde al presionar el boton de send creando una nueva
/// transaccion en caso de que los datos ingresados sean validos, la informacion de la transaccion
/// es mostrada en la interfaz a traves de un pop up
pub fn send_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let send_button: gtk::Button = builder.object("send-button").unwrap();
    let pay_to_entry: gtk::Entry = builder.object("pay to entry").unwrap();
    let fee_entry: gtk::Entry = builder.object("fee").unwrap();
    let amount_entry: gtk::Entry = builder.object("amount-entry").unwrap();
    send_button.connect_clicked(move |_| {
        let address_to_send = String::from(pay_to_entry.text());
        let amount = String::from(amount_entry.text());
        let fee: String = String::from(fee_entry.text());
        pay_to_entry.set_text("");
        amount_entry.set_text("");
        fee_entry.set_text("");
        if let Some((valid_amount, valid_fee)) = validate_amount_and_fee(amount, fee) {
            sender
                .send(WalletEvent::MakeTransaction(
                    address_to_send,
                    valid_amount,
                    valid_fee,
                ))
                .unwrap();
        }
    });
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
            sender.send(WalletEvent::SearchBlock(block_hash)).unwrap();
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
            sender.send(WalletEvent::SearchHeader(block_hash)).unwrap();
        } else {
            show_dialog_message_pop_up(
                format!("Error {text} is not a valid block hash").as_str(),
                "Error searching header",
            )
        }
        search_headers_entry.set_text("");
    });
}

/// Esta funcion realiza la accion que corresponde al presionar el boton de login
pub fn login_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    // elementos de la interfaz
    let login_button: gtk::Button = builder.object("login-button").unwrap();
    let address_entry: gtk::Entry = builder.object("address").unwrap();
    let private_key_entry: gtk::Entry = builder.object("private-key").unwrap();
    let account_loading_spinner: Spinner = builder.object("account-spin").unwrap();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_account_spin = account_loading_spinner;
    let ref_loading_account_label = loading_account_label;
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let ref_to_dropdown = dropdown;
    let ref_to_buttons = get_buttons(builder);
    let ref_to_entries = get_entries(builder);
    // accion al clickearse el boton de login
    login_button.connect_clicked(move |_| {
        disable_buttons_and_entries(&ref_to_buttons, &ref_to_entries);
        ref_to_dropdown.set_sensitive(false);
        ref_account_spin.set_visible(true);
        ref_loading_account_label.set_visible(true);

        let address = String::from(address_entry.text());
        let private_key = String::from(private_key_entry.text());
        address_entry.set_text("");
        private_key_entry.set_text("");
        sender
            .send(WalletEvent::AddAccountRequest(private_key, address))
            .unwrap();
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

/// Cambia el label de loading account cada 5 segundos
pub fn change_loading_account_label_periodically(builder: &Builder) {
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_to_loading_account_label = Rc::new(RefCell::new(loading_account_label));
    gtk::glib::timeout_add_local(Duration::from_secs(5), move || {
        update_label(ref_to_loading_account_label.clone());
        Continue(true)
    });
}

pub fn search_tx_poi_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let search_poi_tx_entry: gtk::SearchEntry = builder.object("search-tx").unwrap();
    let search_poi_block_entry: gtk::SearchEntry = builder.object("search-poi-block").unwrap();
    let search_tx_poi_button: gtk::Button = builder.object("search-tx-button").unwrap();

    search_tx_poi_button.connect_clicked(move |_| {
        let tx_hash_string = search_poi_tx_entry.text().to_string();
        let block_hash_string = search_poi_block_entry.text().to_string();
        let mut tx_is_valid = true;
        let mut block_is_valid = true;
        if hex_string_to_bytes(tx_hash_string.as_str()).is_none() {
            tx_is_valid = false;
        }
        if hex_string_to_bytes(tx_hash_string.as_str()).is_none() {
            block_is_valid = false;
        }

        if tx_is_valid && block_is_valid {
            sender
                .send(WalletEvent::PoiOfTransactionRequest(
                    block_hash_string,
                    tx_hash_string,
                ))
                .unwrap();
        } else if !tx_is_valid && !block_is_valid {
            show_dialog_message_pop_up(
                format!("Error {tx_hash_string} and {block_hash_string} are not valid hashes")
                    .as_str(),
                "Error searching POI",
            )
        } else if !tx_is_valid {
            show_dialog_message_pop_up(
                format!("Error {tx_hash_string} is not a valid tx hash").as_str(),
                "Error searching tx",
            )
        } else if !block_is_valid {
            show_dialog_message_pop_up(
                format!("Error {block_hash_string} is not a valid block hash").as_str(),
                "Error searching block",
            )
        }
        search_poi_tx_entry.set_text("");
        search_poi_block_entry.set_text("");
    });
}

/*
***************************************************************************
************************ AUXILIAR FUNCTIONS *******************************
***************************************************************************
*/

/// esta funcion chequea si el usuario ingreso un amount y un fee validos
/// en caso de que no sea asi, se muestra un pop up con un mensaje de error
fn validate_amount_and_fee(amount: String, fee: String) -> Option<(i64, i64)> {
    let valid_amount = match amount.parse::<i64>() {
        Ok(amount) => amount,
        Err(_) => {
            show_dialog_message_pop_up(
                "Error, please enter a valid amount of Satoshis",
                "Failed to make transaction",
            );
            return None;
        }
    };
    let valid_fee = match fee.parse::<i64>() {
        Ok(fee) => fee,
        Err(_) => {
            show_dialog_message_pop_up(
                "Error, please enter a valid fee of Satoshis",
                "Failed to make transaction",
            );
            return None;
        }
    };

    Some((valid_amount, valid_fee))
}

/// Recibe un Label y cambia su texto por el siguiente en la lista de waiting_labels
pub fn update_label(label: Rc<RefCell<gtk::Label>>) -> Continue {
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

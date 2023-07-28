use std::{cell::RefCell, rc::Rc, sync::mpsc};

use gtk::{prelude::*, Builder, ProgressBar, Spinner};

use crate::wallet_event::WalletEvent;

use super::ui_gtk::{disable_buttons_and_entries, show_dialog_message_pop_up};

/// Esta funcion renderiza la barra de carga de bloques descargados
pub fn render_progress_bar(builder: &Builder, blocks_downloaded: usize, blocks_to_download: usize) {
    let progress_bar: ProgressBar = builder.object("block-bar").unwrap();
    progress_bar.set_fraction(blocks_downloaded as f64 / blocks_to_download as f64);
    progress_bar.set_text(Some(
        format!(
            "Blocks downloaded: {}/{}",
            blocks_downloaded, blocks_to_download
        )
        .as_str(),
    ));
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
/// Esta funcion realiza la accion que corresponde al presionar el boton de login
pub fn login_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    // elementos de la interfaz
    let login_button: gtk::Button = builder.object("login-button").unwrap();
    let address_entry: gtk::Entry = builder.object("address").unwrap();
    let private_key_entry: gtk::Entry = builder.object("private-key").unwrap();
    let account_loading_spinner: Spinner = builder.object("account-spin").unwrap();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_account_spin = account_loading_spinner.clone();
    let ref_loading_account_label = loading_account_label.clone();
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let ref_to_dropdown = dropdown.clone();
    let ref_to_buttons = get_buttons(&builder);
    let ref_to_entries = get_entries(&builder);
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

///Esta funcion realiza la accion que corresponde al presionar el boton de send creando una nueva
/// transaccion en caso de que los datos ingresados sean validos, la informacion de la transaccion
/// es mostrada en la interfaz a traves de un pop up
pub fn send_button_clicked(builder: &Builder, sender: mpsc::Sender<WalletEvent>) {
    let send_button: gtk::Button = builder.object("send-button").unwrap();
    let pay_to_entry: gtk::Entry = builder.object("pay to entry").unwrap();
    let fee_entry: gtk::Entry = builder.object("fee").unwrap();
    let amount_entry: gtk::Entry = builder.object("amount-entry").unwrap();
    let send_balance: gtk::Label = builder.object("send-balance").unwrap();
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

/// Esta funcion obtiene los botones de la interfaz
pub fn get_buttons(builder: &Builder) -> Vec<gtk::Button> {
    let buttons = vec![
        builder.object("send-button").unwrap(),
        builder.object("search-tx-button").unwrap(),
        builder.object("search-blocks-button").unwrap(),
        builder.object("search-header-button").unwrap(),
        builder.object("login-button").unwrap(),
    ];
    buttons
}
/// Esta funcion obtiene los entries de la interfaz
pub fn get_entries(builder: &Builder) -> Vec<gtk::Entry> {
    let entries = vec![
        builder.object("pay to entry").unwrap(),
        builder.object("amount-entry").unwrap(),
        builder.object("fee").unwrap(),
        builder.object("search-tx").unwrap(),
        builder.object("search-block").unwrap(),
        builder.object("search-block-headers").unwrap(),
        builder.object("address").unwrap(),
        builder.object("private-key").unwrap(),
    ];
    entries
}
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

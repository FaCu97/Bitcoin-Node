use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock},
};

use gtk::{prelude::*, Builder, ProgressBar, Spinner, TreeView, Window};

use crate::{
    account::Account,
    blocks::{
        block::{self, Block},
        block_header::BlockHeader,
    },
    wallet_event::WalletEvent,
};

use super::ui_events::UIEvent;

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
type Headers = Arc<RwLock<Vec<BlockHeader>>>;

const AMOUNT_TO_SHOW: usize = 500;

pub fn handle_ui_event(
    builder: Builder,
    ui_event: UIEvent,
    sender_to_get_account: mpsc::Sender<WalletEvent>,
) {
    match ui_event {
        UIEvent::ActualizeBlocksDownloaded(blocks_downloaded, blocks_to_download) => {
            actualize_progress_bar(&builder, blocks_downloaded, blocks_to_download);
        }
        UIEvent::StartHandshake => {
            actualize_message_header(&builder, "Making handshake with nodes...");
        }
        UIEvent::ActualizeHeadersDownloaded(headers_downloaded) => {
            actualize_message_header(
                &builder,
                format!("Headers downloaded: {}", headers_downloaded).as_str(),
            );
        }
        UIEvent::InitializeUITabs((headers, blocks)) => {
            render_main_window(&builder, &headers, &blocks);
        }
        UIEvent::StartDownloadingHeaders => {
            let message_header: gtk::Label = builder.object("message-header").unwrap();
            let spinner: Spinner = builder.object("header-spin").unwrap();
            message_header.set_visible(true);
            spinner.set_visible(true);
        }
        UIEvent::FinsihDownloadingHeaders(headers) => {
            actualize_message_and_spinner(
                &builder,
                false,
                format!("TOTAL HEADERS DOWNLOADED : {}", headers).as_str(),
            );
        }
        UIEvent::StartDownloadingBlocks => {
            render_progress_bar(&builder);
        }
        UIEvent::AccountAddedSuccesfully(account) => {
            update_account_tab(&builder, account);
        }
        UIEvent::AddAccountError(error) => {
            render_account_tab(&builder);
            show_dialog_message_pop_up(error.as_str(), "Error trying to add account");
        }
        UIEvent::ChangeAccountError(error) => {
            show_dialog_message_pop_up(error.as_str(), "Error trying to change account");
        }
        UIEvent::AccountChanged(account) => {
            println!("Account changed to: {}", account.address);
            let available_label = builder.object("available label").unwrap();
            update_overview(&account, &available_label);
            // TODO: Actualizar Overview --> Balance y recent transactions y pestana transactions
        }
        UIEvent::MakeTransactionStatus(status) => {
            show_dialog_message_pop_up(status.as_str(), "transaction's status");
        }

        UIEvent::AddBlock(block) => {
            let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
            let liststore_headers: gtk::ListStore = builder.object("liststore-headers").unwrap();

            add_row_first_to_liststore_block(&liststore_blocks, &block);
            add_row_first_to_liststore_headers(
                &liststore_headers,
                &block.block_header,
                block.get_height(),
            );

            sender_to_get_account
                .send(WalletEvent::GetAccountRequest)
                .unwrap();
        }
        UIEvent::BlockFound(block) => {
            show_dialog_message_pop_up(format!("encontre al bloque {}", block.hex_hash()).as_str(), "Block found");
        }
        UIEvent::HeaderFound(header) => {
            show_dialog_message_pop_up(format!("encontre al header {}", header.hex_hash()).as_str(), "Header found");
        }
        UIEvent::NotFound => {
            show_dialog_message_pop_up("Not found", "Not found");
        }
        _ => (),
    }
}

/// Esta funcion renderiza la barra de carga de bloques descargados
fn actualize_progress_bar(builder: &Builder, blocks_downloaded: usize, blocks_to_download: usize) {
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
fn actualize_message_header(builder: &Builder, msg: &str) {
    let message_header: gtk::Label = builder.object("message-header").unwrap();
    message_header.set_label(msg);
}

fn actualize_message_and_spinner(builder: &Builder, visible: bool, msg: &str) {
    let message_header: gtk::Label = builder.object("message-header").unwrap();
    let spinner: Spinner = builder.object("header-spin").unwrap();
    message_header.set_label(msg);
    spinner.set_visible(visible);
}

fn render_progress_bar(builder: &Builder) {
    let progress_bar: ProgressBar = builder.object("block-bar").unwrap();
    progress_bar.set_visible(true);
    progress_bar.set_text(Some("Blocks downloaded: 0"));
}

fn render_main_window(builder: &Builder, headers: &Headers, blocks: &Blocks) {
    let initial_window: gtk::Window = builder.object("initial-window").unwrap();
    let main_window: gtk::Window = builder.object("main-window").unwrap();
    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
    let liststore_headers: gtk::ListStore = builder.object("liststore-headers").unwrap();
    let header_table: TreeView = builder.object("header_table").unwrap();
    let block_table: TreeView = builder.object("block_table").unwrap();

    initial_window.close();
    main_window.show();
    initialize_headers_tab(&liststore_headers, &header_table, &headers);
    initialize_blocks_tab(&liststore_blocks, &block_table, &headers, &blocks);
}

fn update_account_tab(builder: &Builder, account: Account) {
    let account_loading_spinner: Spinner = builder.object("account-spin").unwrap();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    account_loading_spinner.set_visible(false);
    loading_account_label.set_visible(false);
    let buttons = get_buttons(builder);
    let entries = get_entries(builder);
    enable_buttons_and_entries(&buttons, &entries);
    dropdown.set_sensitive(true);
    show_dialog_message_pop_up(
        format!("Account {} added to wallet!", account.address).as_str(),
        "Account added succesfully",
    );
    dropdown.append_text(account.address.as_str());
}

fn render_account_tab(builder: &Builder) {
    let account_loading_spinner: Spinner = builder.object("account-spin").unwrap();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let buttons = get_buttons(builder);
    let entries = get_entries(builder);
    enable_buttons_and_entries(&buttons, &entries);
    account_loading_spinner.set_visible(false);
    loading_account_label.set_visible(false);
    dropdown.set_sensitive(true);
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

fn initialize_blocks_tab(
    liststore_blocks: &gtk::ListStore,
    block_table: &TreeView,
    headers: &Headers,
    blocks: &Blocks,
) {
    // temporal tree model
    let tree_model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    block_table.set_model(Some(&tree_model));
    let mut block_hash: Vec<[u8; 32]> = Vec::new();
    for header in headers.read().unwrap().iter().rev().take(AMOUNT_TO_SHOW) {
        block_hash.push(header.hash());
    }

    for hash in block_hash {
        let blocks_lock = blocks.read().unwrap();
        let block = blocks_lock.get(&hash).unwrap();
        add_row_last_to_liststore_block(liststore_blocks, block)
    }
    block_table.set_model(Some(liststore_blocks));
}

fn initialize_headers_tab(
    liststore_headers: &gtk::ListStore,
    header_table: &TreeView,
    headers: &Headers,
) {
    // temporal tree model
    let tree_model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    header_table.set_model(Some(&tree_model));

    for (index, header) in headers
        .read()
        .unwrap()
        .iter()
        .enumerate()
        .rev()
        .take(AMOUNT_TO_SHOW / 2)
    {
        add_row_last_to_liststore_headers(liststore_headers, header, index as u32);
    }

    for (index, header) in headers
        .read()
        .unwrap()
        .iter()
        .enumerate()
        .take(AMOUNT_TO_SHOW / 2)
        .rev()
    {
        add_row_last_to_liststore_headers(liststore_headers, header, index as u32);
    }

    header_table.set_model(Some(liststore_headers));
}

/// Agrega una fila al final de la lista de bloques
fn add_row_last_to_liststore_block(liststore_blocks: &gtk::ListStore, block: &Block) {
    let row = liststore_blocks.append();
    add_block_row(liststore_blocks, row, block);
}

/// Agrega una fila al principio de la lista de bloques
fn add_row_first_to_liststore_block(liststore_blocks: &gtk::ListStore, block: &Block) {
    let row = liststore_blocks.prepend();
    add_block_row(liststore_blocks, row, block);
}
/// Agrega una fila liststore de headers
fn add_block_row(liststore_blocks: &gtk::ListStore, row: gtk::TreeIter, block: &Block) {
    liststore_blocks.set(
        &row,
        &[
            (0, &block.get_height().to_value()),
            (1, &block.hex_hash()),
            (2, &block.utc_time()),
            (3, &block.txn_count.decoded_value().to_value()),
        ],
    );
}

/// Agrega una fila al final de la lista de bloques
fn add_row_last_to_liststore_headers(
    liststore_headers: &gtk::ListStore,
    header: &BlockHeader,
    height: u32,
) {
    let row = liststore_headers.append();
    add_header_row(liststore_headers, row, header, height);
}

/// Agrega una fila al principio de la lista de bloques
fn add_row_first_to_liststore_headers(
    liststore_headers: &gtk::ListStore,
    header: &BlockHeader,
    height: u32,
) {
    let row = liststore_headers.prepend();
    add_header_row(liststore_headers, row, header, height);
}
/// Agrega una fila liststore de headers
fn add_header_row(
    liststore_headers: &gtk::ListStore,
    row: gtk::TreeIter,
    header: &BlockHeader,
    height: u32,
) {
    liststore_headers.set(
        &row,
        &[
            (0, &height.to_value()),
            (1, &header.hex_hash()),
            (2, &header.utc_time()),
        ],
    );
}

fn update_overview(account: &Account, available_label: &gtk::Label) {
    available_label.set_label(format!("{}", account.balance()).as_str());
}

pub fn enable_buttons_and_entries(buttons: &Vec<gtk::Button>, entries: &Vec<gtk::Entry>) {
    for button in buttons {
        button.set_sensitive(true);
    }
    for entry in entries {
        entry.set_sensitive(true);
    }
}

pub fn disable_buttons_and_entries(buttons: &Vec<gtk::Button>, entries: &Vec<gtk::Entry>) {
    for button in buttons {
        button.set_sensitive(false);
    }
    for entry in entries {
        entry.set_sensitive(false);
    }
}

pub fn show_dialog_message_pop_up(message: &str, title: &str) {
    let dialog = gtk::MessageDialog::new(
        None::<&Window>,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::Ok,
        message,
    );
    dialog.set_title(title);
    dialog.set_keep_above(true);
    let content_area = dialog.content_area();
    content_area.style_context().add_class("dialog");
    dialog.run();
    dialog.close();
}


/// Convierte un string hexadecimal a un array de bytes que representa el hash
/// Recibe un string hexadecimal de 64 caracteres
/// Devuelve un array de bytes de 32 bytes
/// Si el string no es hexadecimal o no tiene 64 caracteres, devuelve None
pub fn hex_string_to_bytes(hex_string: &str) -> Option<[u8; 32]> {
    if hex_string.len() != 64 {
        return None; // La longitud del string hexadecimal debe ser de 64 caracteres (32 bytes en hexadecimal)
    }
    let mut result = [0u8; 32];
    let hex_chars: Vec<_> = hex_string.chars().collect();
    for i in 0..32 {
        let start = i * 2;
        let end = start + 2;
        if let Ok(byte) = u8::from_str_radix(&hex_chars[start..end].iter().collect::<String>(), 16) {
            result[31 - i] = byte; // Invertimos el orden de asignación para obtener el resultado invertido
        } else {
            return None; // La cadena contiene caracteres no hexadecimales
        }
    }
    Some(result)
}
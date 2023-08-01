use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock},
};

use gtk::{prelude::*, Builder, ProgressBar, Spinner, TreeView, Window};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    wallet_event::WalletEvent,
};

use super::ui_events::UIEvent;

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
type Headers = Arc<RwLock<Vec<BlockHeader>>>;

pub fn handle_ui_event(
    builder: Builder,
    ui_event: UIEvent,
    sender_to_get_account: mpsc::Sender<WalletEvent>,
) {
    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
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
            let row = liststore_blocks.append();
            liststore_blocks.set(
                &row,
                &[
                    (0, &block.get_height().to_value()),
                    (1, &block.hex_hash()),
                    (2, &block.utc_time()),
                    (3, &block.txn_count.decoded_value().to_value()),
                ],
            );
            sender_to_get_account
                .send(WalletEvent::GetAccountRequest)
                .unwrap();
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
    initialize_blocks_tab(&liststore_blocks, &block_table, &blocks);
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
    blocks: &Blocks,
) {
    // temporal tree model
    let tree_model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    block_table.set_model(Some(&tree_model));

    for block in blocks.read().unwrap().values().take(100) {
        let row = liststore_blocks.append();
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

    for (index, header) in headers.read().unwrap().iter().enumerate().rev().take(100) {
        let row = liststore_headers.append();
        liststore_headers.set(
            &row,
            &[
                (0, &((index) as u32).to_value()),
                (1, &header.hex_hash()),
                (2, &header.utc_time()),
            ],
        );
    }

    for (index, header) in headers.read().unwrap().iter().enumerate().take(100).rev() {
        let row = liststore_headers.append();
        liststore_headers.set(
            &row,
            &[
                (0, &(index as u32).to_value()),
                (1, &header.hex_hash()),
                (2, &header.utc_time()),
            ],
        );
    }

    header_table.set_model(Some(liststore_headers));
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

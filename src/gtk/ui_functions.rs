use std::{
    collections::HashMap,
    sync::{
        mpsc::{self},
        Arc, RwLock,
    },
};

use gtk::{
    gdk,
    gdk_pixbuf::{self, Pixbuf},
    prelude::*,
    Builder, CssProvider, ProgressBar, Spinner, StyleContext, TreeView, Window,
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    transactions::transaction::Transaction,
    wallet_event::WalletEvent,
};

use super::ui_events::UIEvent;

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
type Headers = Arc<RwLock<Vec<BlockHeader>>>;

const AMOUNT_TO_SHOW: usize = 500;

pub fn handle_ui_event(
    builder: Builder,
    ui_event: UIEvent,
    sender_to_node: mpsc::Sender<WalletEvent>,
) {
    let tx_table: TreeView = builder.object("tx_table").unwrap();
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
            update_overview(&account, &available_label, &builder);

            // actualiza la pestana de transacciones
            sender_to_node
                .send(WalletEvent::GetTransactionsRequest)
                .unwrap();
            // TODO: Actualizar Overview --> Balance y recent transactions y pestana transactions
        }
        UIEvent::MakeTransactionStatus(status) => {
            show_dialog_message_pop_up(status.as_str(), "transaction's status");
        }
        UIEvent::AddBlock(block) => {
            handle_add_block(sender_to_node, &builder, &block);
        }
        UIEvent::ShowPendingTransaction(account, transaction) => {
            show_dialog_message_pop_up(
                format!(
                    "New incoming pending transaction: {} received for account: {}",
                    transaction.hex_hash(),
                    account.address
                )
                .as_str(),
                "Account added succesfully",
            );
            sender_to_node
                .send(WalletEvent::GetTransactionsRequest)
                .unwrap();
        }

        UIEvent::UpdateTransactions(transactions) => {
            render_transactions(&transactions, tx_table);
            render_recent_transactions(&transactions, &builder);
        }

        UIEvent::NewPendingTx() => {
            sender_to_node
                .send(WalletEvent::GetTransactionsRequest)
                .unwrap();
        }
        UIEvent::ShowConfirmedTransaction(block, account, transaction) => {
            show_dialog_message_pop_up(
                format!(
                    "Transaction confirmed: {} for account: {} in block: {}",
                    transaction.hex_hash(),
                    account.address,
                    block.hex_hash()
                )
                .as_str(),
                "Account added succesfully",
            );
            sender_to_node
                .send(WalletEvent::GetTransactionsRequest)
                .unwrap();
        }
        UIEvent::BlockFound(block) => {
            show_dialog_message_pop_up(
                format!(
                    "Height: {} \nHash: {} \nTime (UTC): {} \nTx Count: {}",
                    block.get_height(),
                    block.hex_hash(),
                    block.utc_time(),
                    block.txn_count.decoded_value()
                )
                .as_str(),
                "Block found",
            );
        }
        UIEvent::HeaderFound(header, height) => {
            show_dialog_message_pop_up(
                format!(
                    "Height: {} \nHash: {} \nTime (UTC): {}",
                    height,
                    header.hex_hash(),
                    header.utc_time()
                )
                .as_str(),
                "Header found",
            );
        }
        UIEvent::NotFound => {
            show_dialog_message_pop_up("Not found", "Not found");
        }
    }
}

/// Muestra las transacciones en la pestana de transacciones
fn render_transactions(transactions: &Vec<(String, Transaction)>, tx_table: TreeView) {
    let tree_model = gtk::ListStore::new(&[
        gdk_pixbuf::Pixbuf::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        i32::static_type(),
    ]);

    for tx in transactions {
        let status_icon_pixbuf = if tx.0 == "Pending" {
            // Cargar la imagen "Pending.png" y convertirla en un GdkPixbuf
            Pixbuf::from_file("src/gtk/resources/pending.png").ok()
        } else {
            // Cargar la imagen "Confirmed.png" y convertirla en un GdkPixbuf
            Pixbuf::from_file("src/gtk/resources/confirmed.png").ok()
        };

        let row = tree_model.append();
        if let Some(pixbuf) = status_icon_pixbuf {
            tree_model.set(
                &row,
                &[
                    (0, &pixbuf.to_value()),
                    (1, &tx.0.to_value()),
                    (2, &tx.1.hex_hash().to_value()),
                    (3, &"P2PKH".to_value()),
                    (4, &tx.1.amount().to_value()),
                ],
            );
        }
    }
    tx_table.set_model(Some(&tree_model));
}

/// Shows the recent transactions in the overview tab
fn render_recent_transactions(transactions: &Vec<(String, Transaction)>, builder: &Builder) {
    // Get the last five elements or all elements if there are fewer than five
    let recent_transactions = if transactions.len() <= 5 {
        &transactions[..]
    } else {
        &transactions[transactions.len() - 5..]
    };
    let amount_labels = [
        "amount-tx-1",
        "amount-tx-2",
        "amount-tx-3",
        "amount-tx-4",
        "amount-tx-5",
    ];
    let icons = [
        "icon-tx-1",
        "icon-tx-2",
        "icon-tx-3",
        "icon-tx-4",
        "icon-tx-5",
    ];
    let type_labels = [
        "type-tx-1",
        "type-tx-2",
        "type-tx-3",
        "type-tx-4",
        "type-tx-5",
    ];
    let recent_tx = [
        "recent-tx-1",
        "recent-tx-2",
        "recent-tx-3",
        "recent-tx-4",
        "recent-tx-5",
    ];
    for (i, tx) in recent_transactions.iter().enumerate() {
        let hash: gtk::AccelLabel = builder.object(recent_tx[i]).unwrap();
        hash.set_label(&tx.1.hex_hash());
        hash.set_visible(true);
        let amount_label: gtk::AccelLabel = builder.object(amount_labels[i]).unwrap();
        amount_label.set_label(format!("{}", tx.1.amount()).as_str());
        amount_label.set_visible(true);
        let icon: gtk::Image = builder.object(icons[i]).unwrap();
        if tx.0 == "Pending" {
            icon.set_from_file(Some("src/gtk/resources/ov_pending.png"));
        } else {
            icon.set_from_file(Some("src/gtk/resources/ov_confirmed.png"));
        }
        icon.set_visible(true);
        let type_label: gtk::AccelLabel = builder.object(type_labels[i]).unwrap();
        type_label.set_visible(true);
    }
}

/// Agrega el bloque y header a las pestañas.
/// Solicita a la wallet la cuenta para actualizar la información
fn handle_add_block(sender_to_node: mpsc::Sender<WalletEvent>, builder: &Builder, block: &Block) {
    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
    let liststore_headers: gtk::ListStore = builder.object("liststore-headers").unwrap();

    add_row_first_to_liststore_block(&liststore_blocks, block);
    add_row_first_to_liststore_headers(&liststore_headers, &block.block_header, block.get_height());

    sender_to_node.send(WalletEvent::GetAccountRequest).unwrap();
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
    initialize_headers_tab(&liststore_headers, &header_table, headers);
    initialize_blocks_tab(&liststore_blocks, &block_table, headers, blocks);
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

fn update_overview(account: &Account, available_label: &gtk::Label, builder: &Builder) {
    available_label.set_label(format!("{}", account.balance()).as_str());
    let amount_labels = [
        "amount-tx-1",
        "amount-tx-2",
        "amount-tx-3",
        "amount-tx-4",
        "amount-tx-5",
    ];
    let icons = [
        "icon-tx-1",
        "icon-tx-2",
        "icon-tx-3",
        "icon-tx-4",
        "icon-tx-5",
    ];
    let type_labels = [
        "type-tx-1",
        "type-tx-2",
        "type-tx-3",
        "type-tx-4",
        "type-tx-5",
    ];
    let recent_tx = [
        "recent-tx-1",
        "recent-tx-2",
        "recent-tx-3",
        "recent-tx-4",
        "recent-tx-5",
    ];
    for i in 0..5 {
        let hash: gtk::AccelLabel = builder.object(recent_tx[i]).unwrap();
        hash.set_visible(false);
        let amount_label: gtk::AccelLabel = builder.object(amount_labels[i]).unwrap();
        amount_label.set_visible(false);
        let icon: gtk::Image = builder.object(icons[i]).unwrap();
        icon.set_visible(false);
        let type_label: gtk::AccelLabel = builder.object(type_labels[i]).unwrap();
        type_label.set_visible(false);
    }
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
        if let Ok(byte) = u8::from_str_radix(&hex_chars[start..end].iter().collect::<String>(), 16)
        {
            result[31 - i] = byte; // Invertimos el orden de asignación para obtener el resultado invertido
        } else {
            return None; // La cadena contiene caracteres no hexadecimales
        }
    }
    Some(result)
}

/// Le agrega el estilo del archivo css a la pantalla
pub fn add_css_to_screen() {
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
}

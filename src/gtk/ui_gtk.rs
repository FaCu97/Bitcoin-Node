use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc, RwLock},
    time::Duration,
};

use crate::{
    account::Account,
    blocks::{block::Block, block_header::BlockHeader},
    wallet_event::WalletEvent,
};
use gtk::{
    gdk,
    glib::{self, Priority},
    prelude::*,
    Application, Builder, CssProvider, ProgressBar, Spinner, StyleContext, Window,
};

use super::ui_events::UIEvent;

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
type Headers = Arc<RwLock<Vec<BlockHeader>>>;

pub fn run_ui(ui_sender: Sender<glib::Sender<UIEvent>>, sender_to_node: Sender<WalletEvent>) {
    let app = Application::builder()
        .application_id("org.gtk-rs.bitcoin")
        .build();
    app.connect_activate(move |app| {
        println!("UI thread");
        build_ui(app, &ui_sender, &sender_to_node);
    });
    let args: Vec<String> = vec![]; // necessary to not use main program args
    app.run_with_args(&args);
}

fn build_ui(
    _app: &Application,
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
    // accounts
    let accounts: Rc<RefCell<Vec<Account>>> = Rc::new(RefCell::new(Vec::new()));
    // buttons and entries
    let buttons = get_buttons(&builder);
    let ref_to_buttons = buttons.clone();
    let entries = get_entries(&builder);
    let ref_to_entries = entries.clone();
    let dropdown: gtk::ComboBoxText = builder.object("dropdown-menu").unwrap();
    let ref_to_dropdown = dropdown.clone();
    let ref2_to_dropdown = dropdown.clone();
    // windows
    let initial_window: Window = builder.object("initial-window").unwrap();
    let main_window: Window = builder.object("main-window").unwrap();
    // login elements
    let login_button: gtk::Button = builder.object("login-button").unwrap();
    let address_entry: gtk::Entry = builder.object("address").unwrap();
    let private_key_entry: gtk::Entry = builder.object("private-key").unwrap();
    let status_login: gtk::Label = builder.object("status-login").unwrap();
    let ref_to_status_login = status_login.clone();
    let account_loading_spinner: Spinner = builder.object("account-spin").unwrap();
    let loading_account_label: gtk::Label = builder.object("load-account").unwrap();
    let ref_to_loading_account_label = Rc::new(RefCell::new(loading_account_label.clone()));

    let ref_account_spin = account_loading_spinner.clone();
    let ref_loading_account_label = loading_account_label.clone();
    // labels
    let message_header: gtk::Label = builder.object("message-header").unwrap();
    // initial window load elements
    let start_button: gtk::Button = builder.object("start-button").unwrap();
    let progress_bar: ProgressBar = builder.object("block-bar").unwrap();
    let spinner: Spinner = builder.object("header-spin").unwrap();
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender.send(tx).expect("could not send sender to client");
    //initial_window.show();
    main_window.show();
    let ref_main_window = main_window.clone();

    // models
    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
    let liststore_headers: gtk::ListStore = builder.object("liststore-headers").unwrap();
    let liststore_transactions: gtk::ListStore = builder.object("liststore-transactions").unwrap();

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
    rx.attach(None, move |msg| {
        match msg {
            UIEvent::ActualizeBlocksDownloaded(blocks_downloaded, blocks_to_download) => {
                progress_bar.set_fraction(blocks_downloaded as f64 / blocks_to_download as f64);
                progress_bar.set_text(Some(
                    format!(
                        "Blocks downloaded: {}/{}",
                        blocks_downloaded, blocks_to_download
                    )
                    .as_str(),
                ));
            }
            UIEvent::StartHandshake => {
                message_header.set_label("Making handshake with nodes...");
            }
            UIEvent::ActualizeHeadersDownloaded(headers_downloaded) => {
                message_header
                    .set_label(format!("Headers downloaded: {}", headers_downloaded).as_str());
            }
            UIEvent::LoadingUtxoSet => {
                spinner.set_visible(true);
                message_header.set_label("Loading utxo set...");
            }
            UIEvent::InitializeUITabs((headers, blocks)) => {
                initial_window.close();
                main_window.show();
                initialize_headers_tab(&liststore_headers, &headers);
                initialize_blocks_tab(&liststore_blocks, &blocks);
            }
            UIEvent::StartDownloadingHeaders => {
                message_header.set_visible(true);
                spinner.set_visible(true);
            }
            UIEvent::FinsihDownloadingHeaders(headers) => {
                spinner.set_visible(false);
                message_header
                    .set_label(format!("TOTAL HEADERS DOWNLOADED : {}", headers).as_str());
            }
            UIEvent::StartDownloadingBlocks => {
                progress_bar.set_visible(true);
                progress_bar.set_text(Some("Blocks downloaded: 0"));
            }
            UIEvent::AccountAddedSuccesfully(account) => {
                account_loading_spinner.set_visible(false);
                loading_account_label.set_visible(false);
                enable_buttons_and_entries(&buttons, &entries);
                dropdown.set_sensitive(true);
                show_dialog_message_pop_up(
                    format!("Account {} added to wallet!", account.address).as_str(),
                    "Account added succesfully",
                );
                dropdown.append_text(account.address.as_str());
                accounts.borrow_mut().push(account);
            }
            UIEvent::AddAccountError(error) => {
                account_loading_spinner.set_visible(false);
                loading_account_label.set_visible(false);
                enable_buttons_and_entries(&buttons, &entries);
                dropdown.set_sensitive(true);
                show_dialog_message_pop_up(error.as_str(), "Error trying to add account");
            }
            UIEvent::ChangeAccountError(error) => {
                show_dialog_message_pop_up(error.as_str(), "Error trying to change account");
            }
            UIEvent::AccountChanged(account) => {
                println!("Account changed to: {}", account.address);
                // TODO: Actualizar Overview --> Balance y recent transactions y pestana transactions
            }

            UIEvent::ShowPendingTransaction(account, transaction) => {
                let row = liststore_transactions.append();
                liststore_transactions.set(
                    &row,
                    &[
                        (0, &"Pending".to_value()),
                        (1, &transaction.hex_hash().to_value()),
                        (2, &"P2PKH".to_value()),
                        (3, &transaction.amount().to_value()),
                    ],
                );

                show_dialog_message_pop_up(
                    format!(
                        "New incoming pending transaction: {} received for account: {}",
                        transaction.hex_hash(),
                        account.address
                    )
                    .as_str(),
                    "Account added succesfully",
                );
            }

            UIEvent::NewPendingTx(transaction) => {
                let row = liststore_transactions.append();
                liststore_transactions.set(
                    &row,
                    &[
                        (0, &"Pending".to_value()),
                        (1, &transaction.hex_hash().to_value()),
                        (2, &"P2PKH".to_value()),
                        (3, &transaction.amount().to_value()),
                    ],
                );
            }
            UIEvent::ShowConfirmedTransaction(block, account, transaction) => {
                let row = liststore_transactions.append();
                liststore_transactions.set(
                    &row,
                    &[
                        (0, &"Confirmed".to_value()),
                        (1, &transaction.hex_hash().to_value()),
                        (2, &"P2PKH".to_value()),
                        (3, &transaction.amount().to_value()),
                    ],
                );

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
            }
            _ => (),
        }
        Continue(true)
    });
    gtk::glib::timeout_add_local(Duration::from_secs(5), move || {
        update_label(ref_to_loading_account_label.clone());
        Continue(true)
    });
    let sender_to_start = sender_to_node.clone();
    let ref_start_btn = start_button.clone();
    start_button.connect_clicked(move |_| {
        sender_to_start.send(WalletEvent::Start).unwrap();
        ref_start_btn.set_visible(false);
    });
    let sender_to_login = sender_to_node.clone();
    login_button.connect_clicked(move |_| {
        disable_buttons_and_entries(&ref_to_buttons, &ref_to_entries);
        ref_to_dropdown.set_sensitive(false);
        ref_account_spin.set_visible(true);
        ref_loading_account_label.set_visible(true);

        let address = String::from(address_entry.text());
        let private_key = String::from(private_key_entry.text());
        sender_to_login
            .send(WalletEvent::AddAccountRequest(private_key, address))
            .unwrap();
    });
    let sender_to_finish = sender_to_node.clone();
    ref_main_window.connect_delete_event(move |_, _| {
        sender_to_finish.send(WalletEvent::Finish).unwrap();
        gtk::main_quit();
        Inhibit(false)
    });
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

/// Initializa la pestaña de bloques
fn initialize_blocks_tab(liststore_blocks: &gtk::ListStore, blocks: &Blocks) {
    println!("INICIALIZO TAB BLOQUESSSSS");
    let mut i = 0;
    for block in blocks.read().unwrap().values() {
        i += 1;
        let row = liststore_blocks.append();
        liststore_blocks.set(
            &row,
            &[
                (0, &i.to_value()), // a comletar
                (1, &block.hex_hash()),
                (2, &block.utc_time()),
                (3, &block.txn_count.decoded_value().to_value()),
            ],
        );
        if i == 50 {
            break;
        }
    }
}

fn initialize_headers_tab(liststore_headers: &gtk::ListStore, headers: &Headers) {
    println!("INICIALIZO TAB HEADERS");
    let mut i = 0;
    for (index, header) in headers.read().unwrap().iter().enumerate() {
        i += 1;
        let row = liststore_headers.append();
        liststore_headers.set(
            &row,
            &[
                (0, &(index as u32).to_value()),
                (1, &header.hex_hash()),
                (2, &header.utc_time()),
            ],
        );
        if i == 50 {
            break;
        }
    }
}
fn get_buttons(builder: &Builder) -> Vec<gtk::Button> {
    let buttons = vec![
        builder.object("send-button").unwrap(),
        builder.object("search-tx-button").unwrap(),
        builder.object("search-blocks-button").unwrap(),
        builder.object("search-header-button").unwrap(),
        builder.object("login-button").unwrap(),
    ];
    buttons
}

fn get_entries(builder: &Builder) -> Vec<gtk::Entry> {
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

fn enable_buttons_and_entries(buttons: &Vec<gtk::Button>, entries: &Vec<gtk::Entry>) {
    for button in buttons {
        button.set_sensitive(true);
    }
    for entry in entries {
        entry.set_sensitive(true);
    }
}

fn disable_buttons_and_entries(buttons: &Vec<gtk::Button>, entries: &Vec<gtk::Entry>) {
    for button in buttons {
        button.set_sensitive(false);
    }
    for entry in entries {
        entry.set_sensitive(false);
    }
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

fn show_dialog_message_pop_up(message: &str, title: &str) {
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

/*

pub struct UIContainer {
    pub main_window: MainNotebook,
    pub builder: Builder,
}


pub struct InitialWindow {
    pub window: Window,
}
impl InitialWindow {
    pub fn new(builder: Builder) -> Self {
        Self { window }
    }
    pub fn upadte(&self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUITabs(_) => {
                self.window.close();
            }
            UIEvent::ActualizeBlocksDownloaded(blocks_downloaded) => {
                println!("Actualize blocks downloaded: {}", blocks_downloaded);
            }
            UIEvent::ActualizeHeadersDownloaded(headers_downloaded) => {
                println!("Actualize headers downloaded: {}", headers_downloaded);
            }
            _ => (),
        }
    }
}


pub struct MainNotebook {
    pub notebook: Notebook,
}


pub struct Notebook {
    pub notebook: gtk::Notebook,
    pub initial_window: InitialWindow,
    overview_tab: OverViewTab,
    send_tab: SendTab,
    transactions_tab: TransactionsTab,
    blocks_tab: BlocksTab,
}

impl Notebook {
    pub fn new(initial_window: &Window, main_window: &Window) -> Self {
        let notebook = Notebook {
            notebook: gtk::Notebook::new(),
            initial_window: InitialWindow::new(initial_window),
            overview_tab: OverViewTab::new(main_window),
            send_tab: SendTab::new(main_window),
            transactions_tab: TransactionsTab::new(main_window),
            blocks_tab: BlocksTab::new(main_window),
        };
        Self::create_tab("Overview", &notebook, &notebook.overview_tab.container);
        Self::create_tab("Send", &notebook, &notebook.send_tab.container);
        Self::create_tab(
            "Transactions",
            &notebook,
            &notebook.transactions_tab.container,
        );
        Self::create_tab("Blocks", &notebook, &notebook.blocks_tab.container);
        notebook
    }
    pub fn update(&mut self, event: UIEvent) {
        match event {
            UIEvent::InitializeUITabs(_) => {
                self.notebook.show_all();
            }
            _ => (),
        }
        self.initial_window.upadte(&event);
        self.overview_tab.update(&event);
        self.send_tab.update(&event);
        self.transactions_tab.update(&event);
        self.blocks_tab.update(&event);
    }
    fn create_tab(title: &str, notebook: &Notebook, container: &gtk::Box) -> u32 {
        let label = gtk::Label::new(Some(title));
        notebook.notebook.append_page(container, Some(&label))
    }
}


pub struct OverViewTab {
    pub container: gtk::Box,
}

impl OverViewTab {
    pub fn new(main_window: &Window) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUITabs(_) => {
                self.initialize();
            }
            _ => (),
        }
    }
    fn initialize(&self) {
        println!("Initialize Overview tab");
    }
}

pub struct SendTab {
    pub container: gtk::Box,
}

impl SendTab {
    pub fn new(main_window: &Window) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUITabs(_) => {
                self.initialize();
            }
            _ => (),
        }
    }
    fn initialize(&self) {
        println!("Initialize send tab");
    }
}

pub struct TransactionsTab {
    pub container: gtk::Box,
}

impl TransactionsTab {
    pub fn new(main_window: &Window) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUITabs(_) => {
                self.initialize();
            }
            UIEvent::ShowPendingTransaction(account, tx) => {
                self.show_pending_transaction(account, tx);
            }
            UIEvent::ShowConfirmedTransaction(block, account, tx) => {
                self.show_confirmed_transaction(block, account, tx);
            }
            _ => (),
        }
    }
    fn initialize(&self) {
        println!("Initialize transactions tab");
    }
    fn show_confirmed_transaction(&self, block: &Block, account: &Account, tx: &Transaction) {
        println!(
            "Confirmed transaction: {:?} involves account: {}",
            tx, account.address
        );
    }
    fn show_pending_transaction(&self, account: &Account, tx: &Transaction) {
        println!(
            "Pending transaction: {:?} involves account: {}",
            tx, account.address
        );
    }
}
pub struct BlocksTab {
    pub container: gtk::Box,
}

impl BlocksTab {
    pub fn new(main_window: &Window) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }

    pub fn update(&mut self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUITabs(blocks) => {
                self.initialize(blocks);
            }
            UIEvent::AddBlock(block) => {
                self.add_block(block);
            }
            _ => {}
        }
    }

    fn initialize(&self, blocks: &Arc<RwLock<HashMap<[u8; 32], Block>>>) {
        let blocks = blocks.read().unwrap();
        for (hash, block) in blocks.iter() {
            println!("Hash: {:?} Block: {:?}", hash, block);
        }
    }
    fn add_block(&self, block: &Block) {
        println!("Add block: {:?}", block);
    }
}
*/

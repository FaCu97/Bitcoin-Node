use std::sync::mpsc::Sender;

use crate::wallet_event::WalletEvent;
use gtk::{
    gdk,
    glib::{self, Priority},
    prelude::*,
    Application, CssProvider, ProgressBar, Spinner, StyleContext, Window,
};

use super::ui_events::UIEvent;

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
    let initial_window: Window = builder.object("initial-window").unwrap();
    let main_window: Window = builder.object("main-window").unwrap();
    let start_button: gtk::Button = builder.object("start-button").unwrap();
    let message_header: gtk::Label = builder.object("message-header").unwrap();
    let progress_bar: ProgressBar = builder.object("block-bar").unwrap();
    let spinner: Spinner = builder.object("header-spin").unwrap();
    let login_button: gtk::Button = builder.object("login").unwrap();
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender.send(tx).expect("could not send sender to client");
    //initial_window.show();
    main_window.show();
    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();
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
            UIEvent::InitializeUITabs(blocks) => {
                println!("INICIALIZO TAB BLOQUESSSSS");
                let mut i = 0;
                for block in blocks.read().unwrap().values() {
                    i += 1;
                    let row = liststore_blocks.append();
                    liststore_blocks.set(
                        &row,
                        &[
                            (0, &2001.to_value()),
                            (1, &block.hex_hash()),
                            (2, &block.utc_time()),
                            (3, &block.txn_count.decoded_value().to_value()),
                        ],
                    );
                    if i == 50 {
                        break;
                    }
                }
                initial_window.close();
                main_window.show_all();
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
            UIEvent::AddAccount(account) => {
                println!("Add account: {:?}", account);
            }
            UIEvent::AddAccountError => {
                println!("Error al agregar cuenta");
            }
            _ => (),
        }
        Continue(true)
    });
    let sender_to_start = sender_to_node.clone();
    let copy = start_button.clone();
    start_button.connect_clicked(move |_| {
        sender_to_start.send(WalletEvent::Start).unwrap();
        copy.set_visible(false);
    });
    let sender_to_add_account = sender_to_node.clone();
    login_button.connect_clicked(move |_| {
        let address_entry: gtk::Entry = builder.object("address").unwrap();
        let private_key_entry: gtk::Entry = builder.object("private key").unwrap();
        let address = address_entry.text().to_string();
        let private_key = private_key_entry.text().to_string();
        sender_to_add_account.send(WalletEvent::AddAccountRequest(private_key, address)).unwrap();
    });
    gtk::main();
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

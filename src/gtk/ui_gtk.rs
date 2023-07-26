use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{mpsc::Sender, Arc, RwLock},
    thread::sleep,
    time::Duration,
};

use crate::{
    account::Account, blocks::block::Block, transactions::transaction::Transaction,
    wallet_event::WalletEvent,
};
use gtk::{
    gdk,
    glib::{self, Priority},
    prelude::*,
    Application, ApplicationWindow, CssProvider, ProgressBar, StyleContext, TreeView, Window,
};

use super::ui_events::UIEvent;
pub struct Gtk;

impl Gtk {
    pub fn run() {
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

        let initial_window: ApplicationWindow = builder.object("initial-window").unwrap();
        let main_window: ApplicationWindow = builder.object("main-window").unwrap();
        let start_button: gtk::Button = builder.object("start-button").unwrap();
        //let progress_bar: ProgressBar = builder.object("load-bar").unwrap();
        initial_window.show_all();

        start_button.connect_clicked(move |_| {
            main_window.show_all();
            initial_window.close();
        });
        gtk::main();
    }
}

pub fn run_ui(ui_sender: Sender<glib::Sender<UIEvent>>, sender_to_node: Sender<WalletEvent>) {
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender.send(tx).expect("could not send sender to client");
    rx.attach(None, move |msg| {
        println!("New event: {:?}", msg);
        Continue(true)
    });

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
    app: &Application,
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
    //let initial_window: ApplicationWindow = gtk::ApplicationWindow::new(app);

    let main_window: Window = builder.object("main-window").unwrap();
    let start_button: gtk::Button = builder.object("start-button").unwrap();
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender.send(tx).expect("could not send sender to client");

    /*******  BLOCK TABLE  ********/

    // creo que no hace falta el block_table
    //let block_table: TreeView = builder.object("block_table").unwrap();

    let liststore_blocks: gtk::ListStore = builder.object("liststore-blocks").unwrap();

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

    /******************************/

    //let notebook = Rc::new(RefCell::new(Notebook::new(&initial_window, &main_window)));
    // let notebook_clone = notebook.clone();

    rx.attach(None, move |msg| {
        print(&msg);
        //notebook_clone.borrow_mut().update(msg);
        Continue(true)
    });
    initial_window.show_all();
    start_button.connect_clicked(move |_| {
        main_window.show_all();
        initial_window.close();
    });

    //notebook.borrow().initial_window.container.show_all();
    //initial_window.show_all();
    //println!("HOLAAAA");
    gtk::main();
}

fn print(msg: &UIEvent) {
    println!("new event: {:?}", msg);
}
/*
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
            UIEvent::InitializeUI => {
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

pub struct InitialWindow {
    pub container: Window,
}

impl InitialWindow {
    pub fn new(application_window: &Window) -> Self {
        let container = application_window.clone();
        Self { container }
    }
    pub fn upadte(&self, event: &UIEvent) {
        match event {
            UIEvent::InitializeUI => {
                self.container.close();
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

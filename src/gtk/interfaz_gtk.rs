use std::{sync::mpsc::Sender, cell::RefCell, rc::Rc};

use gtk::{gdk, prelude::*, CssProvider, ProgressBar, StyleContext, Application, ApplicationWindow, glib::{self, Priority}};
use crate::wallet_event::WalletEvent;

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
    let app = Application::builder()
        .application_id("org.gtk-rs.bitcoin")
        .build();
    app.connect_activate(move |app| {
        build_ui(app, &ui_sender, &sender_to_node);
    });
    app.run();
}

fn build_ui(app: &Application, ui_sender: &Sender<glib::Sender<UIEvent>>, sender_to_node: &Sender<WalletEvent>) {
    let glade_src = include_str!("resources/interfaz.glade");
    let builder = gtk::Builder::from_string(glade_src);
    let main_window: ApplicationWindow = builder.object("main-window").unwrap();
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
    let (tx, rx) = glib::MainContext::channel(Priority::default());
    ui_sender
        .send(tx)
        .expect("could not send sender to client");
    let notebook = Rc::new(RefCell::new(Notebook::new(&main_window)));
    let notebook_clone = notebook.clone();
    rx.attach(None, move |msg| {
        notebook_clone.borrow_mut().update(msg);
        Continue(true)
    });






    let initial_window: ApplicationWindow = builder.object("initial-window").unwrap();
    let main_window: ApplicationWindow = builder.object("main-window").unwrap();
    let start_button: gtk::Button = builder.object("start-button").unwrap();
    initial_window.show_all();
    start_button.connect_clicked(move |_| {
        main_window.show_all();
        initial_window.close();
    });
}


pub struct Notebook {
    pub notebook: gtk::Notebook,
    overview_tab: OverViewTab,
    send_tab: SendTab,
    transactions_tab: TransactionsTab,
    blocks_tab: BlocksTab,
}


impl Notebook {
    pub fn new(main_window: &ApplicationWindow) -> Self {
        let notebook = Notebook {
            notebook: gtk::Notebook::new(),
            overview_tab: OverViewTab::new(),
            send_tab: SendTab::new(),
            transactions_tab: TransactionsTab::new(),
            blocks_tab: BlocksTab::new(),
        };
        Self::create_tab("Overview", &notebook, &notebook.overview_tab.container);
        Self::create_tab("Send", &notebook, &notebook.send_tab.container);
        Self::create_tab("Transactions", &notebook, &notebook.transactions_tab.container);
        Self::create_tab("Blocks", &notebook, &notebook.blocks_tab.container);
        notebook
    }
    pub fn update(&mut self, event: UIEvent) {
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
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            _ => {}
        }
    }
}

pub struct SendTab {
    pub container: gtk::Box,
}

impl SendTab {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            _ => {}
        }
    }

}

pub struct TransactionsTab {
    pub container: gtk::Box,
}

impl TransactionsTab {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            _ => {}
        }
    }

}
pub struct BlocksTab {
    pub container: gtk::Box,
}

impl BlocksTab {
    pub fn new() -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        Self { container }
    }
    
    pub fn update(&mut self, event: &UIEvent) {
        match event {
            _ => {}
        }
    }

}



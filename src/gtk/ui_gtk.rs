use std::sync::mpsc::Sender;

use super::ui_events::UIEvent;
use super::{
    callbacks::connect_ui_callbacks,
    ui_functions::{add_css_to_screen, handle_ui_event},
};
use crate::wallet_event::WalletEvent;
use gtk::{
    glib::{self, Priority},
    prelude::*,
    Application, Window,
};

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
    //initial_window.show();
    show_tx_prueba(&builder);
    let main_window: gtk::Window = builder.object("main-window").unwrap();
    main_window.show();
    let tx_to_node = sender_to_node.clone();
    let builder_clone = builder.clone();
    rx.attach(None, move |msg| {
        handle_ui_event(builder_clone.clone(), msg, tx_to_node.clone());
        Continue(true)
    });
    connect_ui_callbacks(&builder, sender_to_node);
    gtk::main();
}


fn show_tx_prueba(builder: &gtk::Builder) {
    let tx_table: gtk::TreeView = builder.object("tx_table").unwrap();
    let tree_model = gtk::ListStore::new(&[
        gtk::gdk_pixbuf::Pixbuf::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        i32::static_type(),
    ]);

        // Cargar la imagen "Pending.png" y convertirla en un GdkPixbuf
    let pending = gtk::gdk_pixbuf::Pixbuf::from_file("src/gtk/resources/pending.png").ok();
        // Cargar la imagen "Confirmed.png" y convertirla en un GdkPixbuf
    let confirmed = gtk::gdk_pixbuf::Pixbuf::from_file("src/gtk/resources/confirmed.png").ok();

    let row = tree_model.append();
    if let Some(pixbuf) = pending {
        tree_model.set(
            &row,
            &[
                (0, &pixbuf.to_value()),
                (1, &"Pending".to_value()),
                (2, &"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_value()),
                (3, &"P2PKH".to_value()),
                (4, &2000.to_value()),
            ],
        );
    }
    let row = tree_model.append();
    if let Some(pixbuf) = confirmed {
        tree_model.set(
            &row,
            &[
                (0, &pixbuf.to_value()),
                (1, &"Confirmed".to_value()),
                (2, &"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_value()),
                (3, &"P2PKH".to_value()),
                (4, &2000.to_value()),
            ],
        );
    }
    tx_table.set_model(Some(&tree_model));
}

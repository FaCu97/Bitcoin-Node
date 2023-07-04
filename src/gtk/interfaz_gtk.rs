use gtk::{gdk, prelude::*, CssProvider, ProgressBar, StyleContext};

pub struct Gtk;

impl Gtk {
    pub fn run() {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let glade_src = include_str!("resources/interfaz.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let css_provider = CssProvider::new();
        css_provider
            .load_from_path("src/gtk/resources/styles.css")
            .expect("Failed to load CSS file.");

        let screen = gdk::Screen::default().expect("Failed to get default screen.");
        StyleContext::add_provider_for_screen(
            &screen,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        let initial_window: gtk::Window = builder.object("initial-window").unwrap();
        let main_window: gtk::Window = builder.object("main-window").unwrap();
        let start_button: gtk::Button = builder.object("start-button").unwrap();
        let progress_bar: ProgressBar = builder.object("load-bar").unwrap();
        initial_window.show_all();

        start_button.connect_clicked(move |_| {
            main_window.show_all();
            initial_window.close();
        });
        gtk::main();
    }
}

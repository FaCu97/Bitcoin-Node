use gtk::prelude::*;
use gtk::{Builder, Window};

pub struct Gtk;

impl Gtk{

    pub fn run() {
        gtk::init().expect("fail");

        let glade_src = include_str!("bitcoin.glade");
        let builder = Builder::from_string(glade_src);

        let window: Window = builder.object("window").unwrap();

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        window.show_all();
        gtk::main();
    }
}

use gtk::prelude::*;
use gtk::{Builder, Button, CheckButton, Entry, Label, SpinButton, Window};

pub struct Gtk;

impl Gtk {
    pub fn run() {
        gtk::init().expect("fail");

        let glade_src = include_str!("bitcoin.glade");
        let builder = Builder::from_string(glade_src);

        let window: Window = builder.object("window").unwrap();

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let available_balance_button: Button = builder.object("available balance button").unwrap();
        let remove_entry_button: Button = builder.object("remove entry button").unwrap();
        let clear_send_button: Button = builder.object("clear send button").unwrap();
        let available_label: Label = builder.object("available label").unwrap();
        let amount_send_spin: SpinButton = builder.object("amount send spin").unwrap();
        let pay_to_entry: Entry = builder.object("pay to entry").unwrap();
        let label_entry: Entry = builder.object("label entry").unwrap();
        let combo_entry: Entry = builder.object("combo entry").unwrap();
        let subtract_fee_checkbox: CheckButton = builder.object("subtract fee checkbox").unwrap();

        let pay_to_entry_clone = pay_to_entry.clone();
        let amount_send_spin_clone = amount_send_spin.clone();

        available_balance_button.connect_clicked(move |_| {
            let label_value: f64 = available_label.text().to_string().parse().unwrap();
            let spin_value: f64 = amount_send_spin_clone.value();
            if spin_value > label_value {
                amount_send_spin_clone.set_value(label_value);
            }
        });

        remove_entry_button.connect_clicked(move |_| {
            pay_to_entry_clone.set_text("");
        });

        clear_send_button.connect_clicked(move |_| {
            pay_to_entry.set_text("");
            label_entry.set_text("");
            combo_entry.set_text("BTC");
            subtract_fee_checkbox.set_active(false);
            amount_send_spin.set_value(0.0);
        });

        window.show_all();
        gtk::main();
    }
}

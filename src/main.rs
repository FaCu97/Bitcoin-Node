use bitcoin::blockchain_download::initial_block_download;
use bitcoin::config::Config;
use bitcoin::custom_errors::NodeCustomErrors;
use bitcoin::gtk::interfaz_gtk::run_ui;
use bitcoin::gtk::ui_events::{send_event_to_ui, UIEvent};
use bitcoin::handshake::handshake_with_nodes;
use bitcoin::logwriter::log_writer::{
    set_up_loggers, shutdown_loggers, LogSender, LogSenderHandles,
};
use bitcoin::network::get_active_nodes_from_dns_seed;
use bitcoin::node::Node;
use bitcoin::server::NodeServer;
use bitcoin::terminal_ui::terminal_ui;
use bitcoin::wallet::Wallet;
use bitcoin::wallet_event::WalletEvent;
use gtk::glib;
use std::sync::mpsc::{channel, Receiver};
use std::{env, thread};

fn main() -> Result<(), NodeCustomErrors> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[2] == *"-i" {
        run_with_ui(args)?;
    } else {
        run_without_ui(&args)?;
    }
    Ok(())
}

fn run_with_ui(mut args: Vec<String>) -> Result<(), NodeCustomErrors> {
    args.pop();
    // this channel is used to receive the UISender (glib::Sender<UIEvent>) from the ui that creates the channel
    // an sends via this channel the UISender to the node
    let (tx, rx) = channel();
    // channel to comunicate the ui with the node
    let (sender_from_ui_to_node, receiver_from_ui_to_node) = channel();
    let app_thread = thread::spawn(move || -> Result<(), NodeCustomErrors> {
        // sender to comunicate with the ui
        let ui_tx = rx.recv().map_err(|err| {
            println!("ERROR AL RECIBIR!");
            NodeCustomErrors::ThreadChannelError(err.to_string())
        })?; // receive the ui sender from the client
        run_node(&args, Some(ui_tx), Some(receiver_from_ui_to_node)) // run the node with the ui sender
    });
    run_ui(tx, sender_from_ui_to_node);
    app_thread
        .join()
        .map_err(|err| NodeCustomErrors::ThreadJoinError(format!("{:?}", err)))??;
    Ok(())
}

fn run_without_ui(args: &[String]) -> Result<(), NodeCustomErrors> {
    run_node(args, None, None)
}

fn run_node(
    args: &[String],
    ui_sender: Option<glib::Sender<UIEvent>>,
    node_rx: Option<Receiver<WalletEvent>>,
) -> Result<(), NodeCustomErrors> {
    if ui_sender.is_some() {
        println!("Ui_sender exists!\n");
    }
    let config = Config::from(args)?;
    let (log_sender, log_sender_handles) = set_up_loggers(&config)?;
    let active_nodes = get_active_nodes_from_dns_seed(&config, &log_sender)?;
    let pointer_to_nodes = handshake_with_nodes(&config, &log_sender, active_nodes)?;
    let (headers, blocks, headers_height) =
        initial_block_download(&config, &log_sender, &ui_sender, pointer_to_nodes.clone())?;
    send_event_to_ui(&ui_sender, UIEvent::InitializeUITabs(blocks.clone()));
    let mut node = Node::new(
        &log_sender,
        &ui_sender,
        pointer_to_nodes,
        headers,
        blocks,
        headers_height,
    )?;
    let mut wallet = Wallet::new(node.clone())?;
    let server = NodeServer::new(&config, &log_sender, &ui_sender, &mut node)?;
    interact_with_user(&ui_sender, &mut wallet, node_rx);
    shut_down(node, server, log_sender, log_sender_handles)?;
    Ok(())
}

/// Cierra el nodo, el server y los loggers
fn shut_down(
    node: Node,
    server: NodeServer,
    log_sender: LogSender,
    log_sender_handles: LogSenderHandles,
) -> Result<(), NodeCustomErrors> {
    node.shutdown_node()?;
    server.shutdown_server()?;
    shutdown_loggers(log_sender, log_sender_handles)?;
    Ok(())
}

fn interact_with_user(
    ui_sender: &Option<glib::Sender<UIEvent>>,
    wallet: &mut Wallet,
    node_rx: Option<Receiver<WalletEvent>>,
) {
    if let Some(rx) = node_rx {
        handle_ui_request(ui_sender, rx, wallet)
    } else {
        terminal_ui(ui_sender, wallet)
    }
}

fn handle_ui_request(
    ui_sender: &Option<glib::Sender<UIEvent>>,
    rx: Receiver<WalletEvent>,
    wallet: &mut Wallet,
) {
    for event in rx {
        match event {
            WalletEvent::AddAccountRequest(wif, address) => {
                if wallet.add_account(ui_sender, wif, address).is_err() {
                    println!("Error al agregar la cuenta");
                }
            }
            WalletEvent::MakeTransactionRequest(account_index, address, amount, fee) => {
                if wallet
                    .make_transaction(account_index, &address, amount, fee)
                    .is_err()
                {
                    println!("Error al crear la transaccion");
                }
            }
            WalletEvent::PoiOfTransactionRequest(block_hash, transaction_hash) => {
                if wallet
                    .tx_proof_of_inclusion(block_hash, transaction_hash)
                    .is_err()
                {
                    println!("Error al crear la prueba de inclusion");
                }
            }
        }
    }
    println!("TERMINA HANDLE UI REQUEST!!!! \n");
}

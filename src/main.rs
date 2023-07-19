use bitcoin::blockchain_download::initial_block_download;
use bitcoin::config::Config;
use bitcoin::custom_errors::NodeCustomErrors;
use bitcoin::gtk::app::initialize_ui;
use bitcoin::gtk::interfaz_gtk::Gtk;
use bitcoin::gtk::ui_events::UIEvent;
use bitcoin::handshake::handshake_with_nodes;
use bitcoin::logwriter::log_writer::{set_up_loggers, shutdown_loggers, LogSender, LogSenderHandles};
use bitcoin::network::get_active_nodes_from_dns_seed;
use bitcoin::node::Node;
use bitcoin::server::NodeServer;
use bitcoin::terminal_ui;
use bitcoin::wallet::Wallet;
use std::{env, thread};
use std::sync::mpsc::{self, Sender};

fn main() -> Result<(), NodeCustomErrors> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[2] == *"-i" {
        run_with_ui(args.clone())?;
    } else {
        run_without_ui(&args)?;
    }
    Ok(())
}

fn run_with_ui(args: Vec<String>) -> Result<(), NodeCustomErrors> {
    let (tx, rx) = mpsc::channel(); // channel necessary to pass the ui sender to the client
    let app_thread = thread::spawn(move || -> Result<(), NodeCustomErrors> {
        let ui_tx: Sender<UIEvent> = rx.recv().unwrap(); // receive the ui sender from the client
        run_node(&args, Some(ui_tx)) // run the client with the ui sender
    });
    //run_ui(tx);
    app_thread.join().map_err(|err| NodeCustomErrors::ThreadJoinError(format!("{:?}", err)))??;
    Ok(())
}

fn run_without_ui(args: &[String]) -> Result<(), NodeCustomErrors> {
    run_node(args, None)
}

fn run_node(args: &[String], ui_sender: Option<Sender<UIEvent>>) -> Result<(), NodeCustomErrors> {
    let config = Config::from(args)?;
    let (log_sender, log_sender_handles) = set_up_loggers(&config)?;
    let active_nodes = get_active_nodes_from_dns_seed(&config, &log_sender)?;
    let pointer_to_nodes = handshake_with_nodes(&config, &log_sender, active_nodes)?;
    let (headers, blocks) = initial_block_download(&config, &log_sender, pointer_to_nodes.clone())?;
    let mut node = Node::new(&log_sender, pointer_to_nodes, headers, blocks)?;
    let wallet = Wallet::new(node.clone())?;
    let server = NodeServer::new(&config, &log_sender, &mut node)?;
    if ui_sender.is_none() {
        terminal_ui(wallet);
    } else {
        initialize_ui(ui_sender, &log_sender, node.block_chain.read().unwrap().clone());
    }
    shut_down(node, server, log_sender, log_sender_handles)?;
    Ok(())
}

/// Cierra el nodo, el server y los loggers
fn shut_down(node: Node, server: NodeServer, log_sender: LogSender, log_sender_handles: LogSenderHandles) -> Result<(), NodeCustomErrors> {
    node.shutdown_node()?;
    server.shutdown_server()?;
    shutdown_loggers(log_sender, log_sender_handles)?;
    Ok(())
}

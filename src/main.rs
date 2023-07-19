use bitcoin::blockchain_download::initial_block_download;
use bitcoin::config::Config;
use bitcoin::custom_errors::NodeCustomErrors;
use bitcoin::gtk::interfaz_gtk::Gtk;
use bitcoin::handshake::handshake_with_nodes;
use bitcoin::logwriter::log_writer::{set_up_loggers, shutdown_loggers};
use bitcoin::network::get_active_nodes_from_dns_seed;
use bitcoin::node::Node;
use bitcoin::server::NodeServer;
use bitcoin::terminal_ui;
use bitcoin::wallet::Wallet;
use std::env;

fn main() -> Result<(), NodeCustomErrors> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[2] == *"-i" {
        run_with_ui()?;
    } else {
        run_without_ui(&args)?;
    }
    Ok(())
}

fn run_with_ui() -> Result<(), NodeCustomErrors> {
    Ok(())
}

fn run_without_ui(args: &[String]) -> Result<(), NodeCustomErrors> {
    let config = Config::from(args)?;
    let (log_sender, log_sender_handles) = set_up_loggers(&config)?;
    let active_nodes = get_active_nodes_from_dns_seed(&config, &log_sender)?;
    let pointer_to_nodes = handshake_with_nodes(&config, &log_sender, active_nodes)?;
    let (headers, blocks) = initial_block_download(&config, &log_sender, pointer_to_nodes.clone())?;
    let mut node = Node::new(&log_sender, pointer_to_nodes, headers, blocks)?;
    let wallet = Wallet::new(node.clone())?;
    let server = NodeServer::new(&config, &log_sender, &mut node)?;
    terminal_ui(wallet);
    node.shutdown_node()?;
    server.shutdown_server()?;
    shutdown_loggers(log_sender, log_sender_handles)?;
    Ok(())
}

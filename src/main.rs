use bitcoin::blockchain_download::initial_block_download;
use bitcoin::config::Config;
use bitcoin::custom_errors::NodeCustomErrors;
use bitcoin::gtk::ui_events::{send_event_to_ui, UIEvent};
use bitcoin::gtk::ui_gtk::run_ui;
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
    //wait_for_start_button(&node_rx);
    send_event_to_ui(&ui_sender, UIEvent::StartHandshake);
    let config = Config::from(args)?;
    let (log_sender, log_sender_handles) = set_up_loggers(&config)?;
    let node_ips = get_active_nodes_from_dns_seed(&config, &log_sender)?;
    let nodes = handshake_with_nodes(&config, &log_sender, node_ips)?;
    let blockchain = initial_block_download(&config, &log_sender, &ui_sender, nodes.clone())?;
    let mut node = Node::new(&log_sender, &ui_sender, nodes, blockchain.clone())?;
    send_event_to_ui(
        &ui_sender,
        UIEvent::InitializeUITabs((blockchain.headers, blockchain.blocks)),
    );
    let mut wallet = Wallet::new(node.clone())?;
    let server = NodeServer::new(&config, &log_sender, &ui_sender, &mut node)?;
    interact_with_user(&ui_sender, &mut wallet, node_rx);
    shut_down(node, server, log_sender, log_sender_handles)?;
    Ok(())
}

fn wait_for_start_button(rx: &Option<Receiver<WalletEvent>>) {
    if let Some(rx) = rx {
        for event in rx {
            if let WalletEvent::Start = event {
                break;
            }
        }
    }
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
                if let Err(NodeCustomErrors::LockError(err)) =
                    wallet.add_account(ui_sender, wif, address)
                {
                    send_event_to_ui(ui_sender, UIEvent::AddAccountError(err));
                }
            }
            WalletEvent::ChangeAccount(account_index) => {
                if let Err(err) = wallet.change_account(ui_sender, account_index) {
                    send_event_to_ui(ui_sender, UIEvent::ChangeAccountError(err.to_string()));
                }
            }
            WalletEvent::MakeTransaction(address, amount, fee) => {
                if let Err(err) = wallet.make_transaction(&address, amount, fee) {
                    send_event_to_ui(ui_sender, UIEvent::MakeTransactionStatus(err.to_string()));
                } else {
                    send_event_to_ui(
                        ui_sender,
                        UIEvent::MakeTransactionStatus(
                            "The transaction was made succesfuly!".to_string(),
                        ),
                    );
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

            WalletEvent::GetAccountRequest => {
                if let Some(account) = wallet.get_current_account() {
                    send_event_to_ui(ui_sender, UIEvent::AccountChanged(account));
                }
            }
            WalletEvent::SearchBlock(block_hash) => {
                if let Some(block) = wallet.search_block(block_hash) {
                    send_event_to_ui(ui_sender, UIEvent::BlockFound(block));
                } else {
                    send_event_to_ui(ui_sender, UIEvent::NotFound);
                }
            }
            WalletEvent::SearchHeader(block_hash) => {
                if let Some((header, height)) = wallet.search_header(block_hash) {
                    send_event_to_ui(ui_sender, UIEvent::HeaderFound(header, height));
                } else {
                    send_event_to_ui(ui_sender, UIEvent::NotFound);
                }
            }
            WalletEvent::Finish => {
                break;
            }
            _ => (),
        }
    }
}

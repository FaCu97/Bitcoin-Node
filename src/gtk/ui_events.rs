use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use gtk::glib;

use crate::{account::Account, blocks::block::Block, transactions::transaction::Transaction};

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
#[derive(Clone, Debug)]
pub enum UIEvent {
    StartDownloadingHeaders,
    FinsihDownloadingHeaders,
    StartDownloadingBlocks,
    ShowConfirmedTransaction(Block, Account, Transaction),
    AddAccount(Account),
    ShowPendingTransaction(Account, Transaction),
    AddBlock(Block),
    InitializeUITabs(Blocks),
    ActualizeHeadersDownloaded(usize),
    ActualizeBlocksDownloaded(usize),
}

pub fn send_event_to_ui(ui_sender: &Option<glib::Sender<UIEvent>>, event: UIEvent) {
    if let Some(ui_sender) = ui_sender {
        ui_sender
            .send(event.clone())
            .expect("Error al enviar el evento a la interfaz");
    }
}

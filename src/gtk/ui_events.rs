use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use gtk::glib;

use crate::{account::Account, blocks::block::Block, transactions::transaction::Transaction};

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
#[derive(Clone, Debug)]
pub enum UIEvent {
    StartHandshake,
    StartDownloadingHeaders,
    FinsihDownloadingHeaders(usize),
    StartDownloadingBlocks,
    ShowConfirmedTransaction(Block, Account, Transaction),
    AddAccount(Account),
    AddAccountError,
    ShowPendingTransaction(Account, Transaction),
    AddBlock(Block),
    InitializeUITabs(Blocks),
    ActualizeHeadersDownloaded(usize),
    ActualizeBlocksDownloaded(usize, usize),
}

pub fn send_event_to_ui(ui_sender: &Option<glib::Sender<UIEvent>>, event: UIEvent) {
    if let Some(ui_sender) = ui_sender {
        ui_sender
            .send(event)
            .expect("Error al enviar el evento a la interfaz");
    }
}

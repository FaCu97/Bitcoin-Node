use std::{collections::HashMap, sync::{RwLock, Arc}};

use gtk::glib;

use crate::{blocks::block::Block, transactions::transaction::Transaction, account::Account};

type Blocks = Arc<RwLock<HashMap<[u8; 32], Block>>>;
#[derive(Clone, Debug)]
pub enum UIEvent {
    InitializeUI(HashMap<[u8; 32], Block>),
    ShowConfirmedTransaction(),
    AddAccount(),
    ShowPendingTransaction(Account, Transaction),
    AddBlock(Block),
    InitializeUITabs(Blocks),
    ActualizeHeadersDownloaded(i32),
    ActualizeBlocksDownloaded(i32),
}

pub fn send_event_to_ui(ui_sender: &Option<glib::Sender<UIEvent>>, event: UIEvent) {
    if let Some(ui_sender) = ui_sender {
        ui_sender
            .send(event)
            .expect("Error al enviar el evento a la interfaz");
    }
}

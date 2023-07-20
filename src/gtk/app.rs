use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{blocks::block::Block, logwriter::log_writer::LogSender};

use super::ui_events::{UIEvent, UIEventSender};

pub fn initialize_ui(
    tx: Option<Sender<UIEvent>>,
    log_sender: &LogSender,
    blocks: HashMap<[u8; 32], Block>,
) -> UIEventSender {
    let ui_sender = match tx {
        Some(sender) => UIEventSender::with_ui(sender),
        None => UIEventSender::withou_ui(),
    };
    ui_sender.initialize_ui(log_sender, blocks);
    ui_sender
}

use std::{collections::HashMap, sync::mpsc::Sender};

use crate::{
    blocks::block::Block,
    logwriter::log_writer::{write_in_log, LogSender},
};

pub enum UIEvent {
    InitializeUI(HashMap<[u8; 32], Block>),
    ShowConfirmedTransaction(),
    AddAccount(),
    ShowPendingTransaction(),
}

#[derive(Debug, Clone)]

pub struct UIEventSender {
    ui_sender: Option<Sender<UIEvent>>,
}

impl UIEventSender {
    pub fn withou_ui() -> Self {
        UIEventSender { ui_sender: None }
    }

    pub fn with_ui(sender: Sender<UIEvent>) -> Self {
        UIEventSender {
            ui_sender: Some(sender),
        }
    }

    pub fn initialize_ui(&self, log_sender: &LogSender, blocks: HashMap<[u8; 32], Block>) {
        self.send_initialize_ui_event_to_ui(log_sender, blocks);
    }

    pub fn send_initialize_ui_event_to_ui(
        &self,
        log_sender: &LogSender,
        blocks: HashMap<[u8; 32], Block>,
    ) {
        self.send_event_to_ui(log_sender, UIEvent::InitializeUI(blocks));
    }

    pub fn send_event_to_ui(&self, log_sender: &LogSender, event: UIEvent) {
        if let Some(sender) = &self.ui_sender {
            if sender.send(event).is_err() {
                write_in_log(&log_sender.error_log_sender, "Error sending event to ui");
            }
        }
    }
}

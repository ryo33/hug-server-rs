use tokio::sync::mpsc::{Receiver, Sender};

use crate::payload::{HugEvent, Payload};

pub fn message_sockets() -> (MessageSocket, MessageSocket) {
    let (tx1, rx1) = tokio::sync::mpsc::channel(100);
    let (tx2, rx2) = tokio::sync::mpsc::channel(100);
    (
        MessageSocket {
            sender: tx1,
            receiver: rx2,
        },
        MessageSocket {
            sender: tx2,
            receiver: rx1,
        },
    )
}

pub struct MessageSocket {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

#[derive(Debug)]
pub enum Message {
    Payload(Payload),
    Joined,
    Leave,
}

pub type JoinNotifier = Sender<Message>;

pub type EventSender = Sender<HugEvent>;

pub fn ws_bridge() -> (EventSender, Receiver<HugEvent>) {
    tokio::sync::mpsc::channel(100)
}

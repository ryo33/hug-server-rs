use tokio::sync::mpsc::{Receiver, Sender};

use crate::payload::Payload;

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

pub enum Message {
    Payload(Payload),
    Joined,
    Leave,
}

pub type JoinNotifier = Sender<Message>;

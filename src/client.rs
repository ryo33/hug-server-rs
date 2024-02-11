use futures::{Sink, SinkExt};
use uuid::Uuid;

use crate::{
    channel::{Message, MessageSocket},
    payload::{HugEvent, Payload},
    State,
};

pub struct Client<WsSender> {
    pub state_ref: State,
    pub message_socket: Option<MessageSocket>,
    pub ws_sender: WsSender,
}

impl<WsSender: Sink<HugEvent> + Unpin> Client<WsSender>
where
    WsSender::Error: std::error::Error + Send + Sync + 'static,
{
    pub async fn join_random(&mut self) -> anyhow::Result<()> {
        self.message_socket = Some(self.state_ref.join_random().await?);
        Ok(())
    }
    pub async fn create_room(&mut self) -> anyhow::Result<()> {
        let (key, socket) = self.state_ref.create_room().await;
        self.message_socket = Some(socket);
        self.ws_sender
            .send(HugEvent::RoomCreated {
                key: key.to_string(),
            })
            .await?;
        Ok(())
    }
    pub async fn join_room(&mut self, key: Uuid) -> anyhow::Result<()> {
        if let Some(socket) = self.state_ref.find_room(key).await {
            socket.sender.send(Message::Joined).await?;
            self.ws_sender
                .send(HugEvent::Joined { is_primary: false })
                .await?;
            self.message_socket = Some(socket);
        } else {
            self.ws_sender.send(HugEvent::NotFound).await?;
        }
        Ok(())
    }
    pub async fn leave(&mut self) -> anyhow::Result<()> {
        self.message_socket = None;
        if let Some(socket) = &mut self.message_socket {
            socket.sender.send(Message::Leave).await?;
        }
        Ok(())
    }
    pub async fn push(&mut self, payload: Payload) -> anyhow::Result<()> {
        let message_socket = self
            .message_socket
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("not in a room"))?;
        message_socket
            .sender
            .send(Message::Payload(payload))
            .await?;
        Ok(())
    }

    pub async fn handle_message(&mut self, message: Option<Message>) -> anyhow::Result<()> {
        let Some(message) = message else {
            tracing::info!("the peer has closed the connection");
            return Ok(());
        };
        match message {
            Message::Payload(payload) => {
                self.ws_sender.send(HugEvent::Push { payload }).await?;
            }
            Message::Joined => {
                self.ws_sender
                    .send(HugEvent::Joined { is_primary: true })
                    .await?;
            }
            Message::Leave => {
                tracing::info!("leave message received, but not implemented yet");
            }
        }
        Ok(())
    }

    pub async fn recv_message(&mut self) -> Option<Message> {
        if let Some(message_socket) = self.message_socket.as_mut() {
            message_socket.receiver.recv().await
        } else {
            futures::future::pending().await
        }
    }
}

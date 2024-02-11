use uuid::Uuid;

use crate::{
    channel::{EventSender, Message, MessageSocket},
    payload::{HugEvent, Payload},
    State,
};

pub struct Client {
    pub state: State,
    pub message_socket: Option<MessageSocket>,
    pub event_sender: EventSender,
}

impl Client {
    #[tracing::instrument(err, skip(self))]
    pub async fn join_random(&mut self) -> anyhow::Result<()> {
        self.message_socket = Some(self.state.join_random().await?);
        Ok(())
    }
    #[tracing::instrument(err, skip(self))]
    pub async fn create_room(&mut self) -> anyhow::Result<()> {
        let (key, socket) = self.state.create_room().await;
        self.message_socket = Some(socket);
        self.event_sender
            .send(HugEvent::RoomCreated {
                key: key.to_string(),
            })
            .await?;
        Ok(())
    }
    #[tracing::instrument(err, skip(self))]
    pub async fn join_room(&mut self, key: Uuid) -> anyhow::Result<()> {
        if let Some(socket) = self.state.find_room(key).await {
            socket.sender.send(Message::Joined).await?;
            self.event_sender
                .send(HugEvent::Joined { is_primary: false })
                .await?;
            self.message_socket = Some(socket);
        } else {
            self.event_sender.send(HugEvent::NotFound).await?;
        }
        Ok(())
    }
    #[tracing::instrument(err, skip(self))]
    pub async fn leave(&mut self) -> anyhow::Result<()> {
        self.message_socket = None;
        if let Some(socket) = &mut self.message_socket {
            socket.sender.send(Message::Leave).await?;
        }
        Ok(())
    }
    #[tracing::instrument(err, skip(self))]
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

    #[tracing::instrument(err, skip(self))]
    pub async fn handle_message(&mut self, message: Option<Message>) -> anyhow::Result<()> {
        let Some(message) = message else {
            tracing::info!("the peer has closed the connection");
            return Ok(());
        };
        match message {
            Message::Payload(payload) => {
                self.event_sender.send(HugEvent::Push { payload }).await?;
            }
            Message::Joined => {
                self.event_sender
                    .send(HugEvent::Joined { is_primary: true })
                    .await?;
            }
            Message::Leave => {
                tracing::info!("leave message received, but not implemented yet");
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn recv_message(&mut self) -> Option<Message> {
        if let Some(message_socket) = self.message_socket.as_mut() {
            message_socket.receiver.recv().await
        } else {
            futures::future::pending().await
        }
    }
}

mod key_registry;

use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::channel::{message_sockets, JoinNotifier, Message, MessageSocket};

use self::key_registry::KeyRegistry;

#[derive(Clone, Default)]
pub struct State {
    random_matching: Arc<Mutex<RandomMatching>>,
    key_matching: Arc<KeyMatching>,
}

#[derive(Default)]
pub struct RandomMatching {
    pub waiting: Option<MatchWait>,
}

impl RandomMatching {
    #[tracing::instrument(err, skip(self))]
    pub async fn join_random(&mut self) -> anyhow::Result<MessageSocket> {
        if let Some(wait) = self.waiting.take() {
            if let Err(err) = wait.join_notifier.send(Message::Joined).await {
                tracing::info!("join_notifier send error: {}", err);
                // The other side has gone away, so new one be the waiting one.
            } else {
                tracing::info!("matched with random player");
                wait.socket.sender.send(Message::Joined).await?;
                return Ok(wait.socket);
            }
        }
        tracing::info!("waiting for random player");
        let (mine, theirs) = message_sockets();
        self.waiting = Some(MatchWait {
            socket: theirs,
            join_notifier: mine.sender.clone(),
        });
        Ok(mine)
    }
}

#[derive(Default)]
pub struct KeyMatching {
    pub key_registry: KeyRegistry,
}

pub struct MatchWait {
    pub socket: MessageSocket,
    pub join_notifier: JoinNotifier,
}

impl State {
    #[tracing::instrument(skip(self))]
    pub async fn find_room(&self, key: Uuid) -> Option<MessageSocket> {
        self.key_matching.key_registry.remove(&key).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_room(&self) -> (Uuid, MessageSocket) {
        let key = Uuid::new_v4();
        let (mine, theirs) = message_sockets();
        self.key_matching.key_registry.insert(key, theirs).await;
        (key, mine)
    }

    pub async fn join_random(&self) -> anyhow::Result<MessageSocket> {
        self.random_matching.lock().await.join_random().await
    }
}

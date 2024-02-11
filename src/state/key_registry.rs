use std::{sync::Arc, time::Duration};

use moka::future::{Cache, CacheBuilder};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::channel::MessageSocket;

pub struct KeyRegistry {
    /// Use Arc to make clonable.
    /// Make Option to allow Option::take.
    map: Cache<Uuid, Arc<Mutex<Option<MessageSocket>>>, ahash::RandomState>,
}

impl KeyRegistry {
    pub fn new() -> Self {
        Self {
            map: CacheBuilder::new(100_000)
                .time_to_live(Duration::from_secs(60 * 60))
                .build_with_hasher(ahash::RandomState::new()),
        }
    }

    #[tracing::instrument(skip(self, socket))]
    pub async fn insert(&self, key: Uuid, socket: MessageSocket) {
        self.map
            .insert(key, Arc::new(Mutex::new(Some(socket))))
            .await;
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove(&self, key: &Uuid) -> Option<MessageSocket> {
        self.map.remove(key).await.and_then(|arc| arc.lock().take())
    }
}

impl Default for KeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

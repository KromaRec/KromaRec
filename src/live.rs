//! Account-scoped cancellation for live transport connections. Admission and
//! invalidation run inside the serialized database operation, after validation
//! or commit respectively. No credential material lives in the registry.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

#[derive(Default)]
pub(crate) struct Registry(Mutex<HashMap<i64, watch::Sender<()>>>);

pub(crate) struct Lease {
    registry: Arc<Registry>,
    player: i64,
    receiver: Option<watch::Receiver<()>>,
}

impl Registry {
    pub(crate) fn admit(self: &Arc<Self>, player: i64) -> Lease {
        let mut players = self.0.lock().expect("lease registry");
        let receiver = players
            .entry(player)
            .or_insert_with(|| watch::channel(()).0)
            .subscribe();
        Lease {
            registry: self.clone(),
            player,
            receiver: Some(receiver),
        }
    }

    pub(crate) fn revoke(&self, player: i64) {
        self.0.lock().expect("lease registry").remove(&player);
    }
}

impl Lease {
    pub(crate) async fn revoked(&mut self) {
        let _ = self.receiver.as_mut().expect("live lease").changed().await;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut players = self.registry.0.lock().expect("lease registry");
        self.receiver.take();
        if players
            .get(&self.player)
            .is_some_and(|entry| entry.receiver_count() == 0)
        {
            players.remove(&self.player);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn old_connections_cannot_remove_a_reconnected_players_lease() {
        let registry = Arc::new(Registry::default());
        let mut first = registry.admit(1);
        let sibling = registry.admit(1);
        let other = registry.admit(2);
        registry.revoke(1);
        first.revoked().await;

        let mut reconnect = registry.admit(1);
        drop(first);
        drop(sibling);
        assert_eq!(registry.0.lock().unwrap().len(), 2);
        registry.revoke(1);
        reconnect.revoked().await;
        assert!(matches!(
            other.receiver.as_ref().unwrap().has_changed(),
            Ok(false)
        ));
        drop(other);
        assert!(registry.0.lock().unwrap().is_empty());
    }
}

use tokio::sync::broadcast;

use crate::auth::UserId;

/// Something about a user changed that their open sessions should react to.
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// The user's [`GlobalRole`](crate::authz::GlobalRole) changed.
    RoleChanged { user_id: UserId },
    DisplayNameChanged {
        user_id: UserId,
        display_name: String,
    },
}

/// Broadcasts [`UserEvent`]s, so the frontend can re-fetch what changed
/// instead of waiting for a reload.
#[derive(Clone)]
pub struct UserEvents {
    tx: broadcast::Sender<UserEvent>,
}

impl UserEvents {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UserEvent> {
        self.tx.subscribe()
    }

    pub fn role_changed(&self, user_id: UserId) {
        let _ = self.tx.send(UserEvent::RoleChanged { user_id });
    }

    pub fn display_name_changed(&self, user_id: UserId, display_name: &str) {
        let _ = self.tx.send(UserEvent::DisplayNameChanged {
            user_id,
            display_name: display_name.to_string(),
        });
    }
}

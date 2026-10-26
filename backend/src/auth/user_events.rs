use tokio::sync::broadcast;

use crate::auth::UserId;
use crate::authz::GlobalRole;

/// Something about a user changed that their open sessions should react to.
#[derive(Debug, Clone)]
pub enum UserEvent {
    RoleChanged {
        user_id: UserId,
        role: GlobalRole,
    },
    DisplayNameChanged {
        user_id: UserId,
        display_name: String,
    },
}

/// Broadcasts [`UserEvent`]s, carrying the new value so the frontend can
/// patch its state instead of re-fetching it.
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

    pub fn role_changed(&self, user_id: UserId, role: GlobalRole) {
        let _ = self.tx.send(UserEvent::RoleChanged { user_id, role });
    }

    pub fn display_name_changed(&self, user_id: UserId, display_name: &str) {
        let _ = self.tx.send(UserEvent::DisplayNameChanged {
            user_id,
            display_name: display_name.to_string(),
        });
    }
}

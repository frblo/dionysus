use crate::auth::identity::UserId;

/// The new identity that's waiting to be attached to `target_user_id`, once
/// its ownership is confirmed.
#[derive(Debug, Clone)]
pub struct PendingLink {
    pub new_provider_id: String,
    pub subject: String,
    pub email: String,
    pub target_user_id: UserId,
}

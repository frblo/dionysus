use crate::auth::identity::UserId;
use crate::auth::ttl_store::TtlStore;

/// Tracks a login whose verified email collided with an existing, different
/// account. It is tracked until the user either confirms it's them 
/// (by logging in again via one of that account's own providers) or the 
/// link expires unconfirmed.
///
/// Keyed by an opaque link token handed to the browser in place of a
/// session.
pub type PendingLinkStore = TtlStore<PendingLink>;

/// The new identity that's waiting to be attached to `target_user_id`, once
/// its ownership is confirmed.
#[derive(Debug, Clone)]
pub struct PendingLink {
    pub new_provider_id: String,
    pub subject: String,
    pub email: String,
    pub target_user_id: UserId,
}

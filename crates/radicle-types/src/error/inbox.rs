use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum InboxError {
    /// List notification error.
    #[error(transparent)]
    ListNotificationsError(
        #[from] crate::domain::inbox::models::notification::ListNotificationsError,
    ),

    /// Inbox error.
    #[error(transparent)]
    Inbox(#[from] radicle::node::notifications::Error),
}

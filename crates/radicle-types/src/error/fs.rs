use strum::{AsRefStr, EnumDiscriminants};
use ts_rs::TS;

#[derive(Debug, AsRefStr, EnumDiscriminants, thiserror::Error)]
#[strum_discriminants(derive(TS))]
#[strum_discriminants(ts(export))]
#[strum_discriminants(ts(export_to = "error/"))]
pub enum EmbedsError {
    #[error("not able to save embed.")]
    SaveEmbed,

    /// Tauri clipboard error.
    #[error("not able to read the clipboard.")]
    ClipboardError(#[from] tauri_plugin_clipboard_manager::Error),

    /// Tauri fs error.
    #[error(transparent)]
    Fs(#[from] tauri_plugin_fs::Error),
}

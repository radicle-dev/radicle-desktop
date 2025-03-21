use std::path::PathBuf;

use radicle::git;
use radicle::identity;
use radicle_types::domain::repo::models::cobs;
use radicle_types::domain::repo::service::Service;
use radicle_types::domain::repo::traits::RepoService;
use radicle_types::error::Error;
use radicle_types::outbound::radicle::Radicle as R;
use radicle_types::outbound::sqlite::Sqlite;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

pub mod issue;
pub mod patch;

#[tauri::command]
pub async fn get_embed(
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    name: Option<String>,
    oid: git::Oid,
) -> Result<cobs::EmbedWithMimeType, Error> {
    service.get_embed(rid, name, oid)
}

#[tauri::command]
pub async fn save_embed_by_path(
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    path: PathBuf,
) -> Result<git::Oid, Error> {
    service.save_embed_by_path(rid, path)
}

#[tauri::command]
pub async fn save_embed_by_clipboard(
    app_handle: tauri::AppHandle,
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    name: String,
) -> Result<git::Oid, Error> {
    let content = app_handle
        .clipboard()
        .read_image()
        .map(|i| i.rgba().to_vec())?;

    service.save_embed_by_bytes(rid, name, content)
}

#[tauri::command]
pub async fn save_embed_by_bytes(
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    name: String,
    bytes: Vec<u8>,
) -> Result<git::Oid, Error> {
    service.save_embed_by_bytes(rid, name, bytes)
}

#[tauri::command]
pub async fn save_embed_to_disk(
    app_handle: tauri::AppHandle,
    service: tauri::State<'_, Service<R, Sqlite>>,
    rid: identity::RepoId,
    oid: git::Oid,
    name: String,
) -> Result<(), Error> {
    let Some(path) = app_handle
        .dialog()
        .file()
        .set_file_name(name)
        .blocking_save_file()
    else {
        return Err(Error::SaveEmbedError);
    };
    let path = path.into_path()?;

    service.save_embed_to_disk(rid, oid, path)
}

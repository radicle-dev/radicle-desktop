use std::path::PathBuf;

use anyhow::{Context, Result};

use radicle::git;
use radicle::identity;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::thread::Thread;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

use crate::AppState;

pub mod issue;
pub mod patch;

#[tauri::command]
pub async fn get_embed(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    name: Option<String>,
    oid: git::Oid,
) -> Result<types::cobs::EmbedWithMimeType, Error> {
    ctx.get_embed(rid, name, oid)
}

#[tauri::command]
pub async fn save_embed_by_path(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    path: PathBuf,
) -> Result<git::Oid, Error> {
    ctx.save_embed_by_path(rid, path)
}

#[tauri::command]
pub async fn save_embed_by_clipboard(
    app_handle: tauri::AppHandle,
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    name: String,
) -> Result<git::Oid, Error> {
    let content = app_handle
        .clipboard()
        .read_image()
        .map(|i| i.rgba().to_vec())
        .context("Not able to read the image from the clipboard")?;

    ctx.save_embed_by_bytes(rid, name, content)
}

#[tauri::command]
pub async fn save_embed_by_bytes(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    name: String,
    bytes: Vec<u8>,
) -> Result<git::Oid, Error> {
    ctx.save_embed_by_bytes(rid, name, bytes)
}

#[tauri::command]
pub async fn save_embed_to_disk(
    app_handle: tauri::AppHandle,
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    oid: git::Oid,
    name: String,
) -> Result<(), Error> {
    let path = app_handle
        .dialog()
        .file()
        .set_file_name(name)
        .blocking_save_file()
        .context("no path defined")?;

    let path = path
        .into_path()
        .context("Not able to convert into PathBuf")?;

    ctx.save_embed_to_disk(rid, oid, path)
}

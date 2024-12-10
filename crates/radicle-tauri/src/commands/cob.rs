use radicle::cob;
use radicle::git;
use radicle::identity;
use radicle_types as types;
use radicle_types::error::Error;
use radicle_types::traits::cobs::Cobs;
use radicle_types::traits::thread::Thread;

use crate::AppState;

pub mod draft;
pub mod issue;
pub mod patch;

#[tauri::command]
pub async fn get_file_by_oid(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    oid: git::Oid,
) -> Result<String, Error> {
    ctx.get_embed(rid, oid)
}

#[tauri::command]
pub async fn process_chunk(ctx: tauri::State<'_, AppState>, chunk: Vec<u8>) -> Result<(), Error> {
    let mut buffer = ctx.buffer.lock().unwrap();
    buffer.extend_from_slice(&chunk);

    Ok(())
}

#[tauri::command]
pub async fn save_embed(
    ctx: tauri::State<'_, AppState>,
    rid: identity::RepoId,
    name: &str,
) -> Result<git::Oid, Error> {
    let buffer = ctx.buffer.lock().unwrap();

    ctx.save_embed(rid, name, &buffer.to_vec())
}

#[tauri::command]
pub fn activity_by_id(
    ctx: tauri::State<AppState>,
    rid: identity::RepoId,
    type_name: cob::TypeName,
    id: git::Oid,
) -> Result<Vec<types::cobs::issue::Operation>, Error> {
    ctx.activity_by_id(rid, type_name, id)
}

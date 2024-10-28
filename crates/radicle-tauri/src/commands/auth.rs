use radicle_types::traits::auth::Auth;

use crate::{error::Error, AppState};

#[tauri::command]
pub(crate) fn authenticate(ctx: tauri::State<AppState>) -> Result<(), Error> {
    ctx.authenticate().map_err(Error::from)
}

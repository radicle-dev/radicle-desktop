use radicle_types::error::Error;
use radicle_types::traits::auth::Auth;

use crate::AppState;

#[tauri::command]
pub(crate) fn authenticate(ctx: tauri::State<AppState>) -> Result<(), Error> {
    ctx.authenticate().map_err(Error::from)
}

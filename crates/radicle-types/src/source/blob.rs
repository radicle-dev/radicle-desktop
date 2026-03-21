use base64::{prelude::BASE64_STANDARD, Engine};
use radicle::git::Oid;
use radicle_surf as surf;
use serde::Serialize;
use ts_rs::TS;

use crate::repo::Commit;
use crate::traits::repo::MAX_BLOB_SIZE;

#[derive(TS, Serialize)]
#[ts(export)]
#[ts(export_to = "source/")]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    #[ts(as = "String")]
    id: Oid,
    binary: bool,
    commit: Commit,
    mime_type: String,
    content: String,
}

impl<T: AsRef<[u8]>> From<surf::blob::Blob<T>> for Blob {
    fn from(blob: surf::blob::Blob<T>) -> Self {
        let content = match blob.size() {
            s if s > MAX_BLOB_SIZE && blob.is_binary() => {
                String::from("Payload too large, content has been truncated")
            }
            _ => match std::str::from_utf8(blob.content()) {
                Ok(s) => s.to_owned(),
                Err(_) => BASE64_STANDARD.encode(blob.content()),
            },
        };

        let mime_type = infer::get(blob.content()).map(|i| i.mime_type().to_string());

        Blob {
            id: crate::oid::from_surf(blob.object_id()),
            binary: blob.is_binary(),
            commit: blob.commit().clone().into(),
            content,
            mime_type: mime_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        }
    }
}

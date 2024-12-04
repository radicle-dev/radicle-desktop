use radicle::git;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Options {
    pub base: git::Oid,
    pub head: git::Oid,
    pub unified: Option<u32>,
    pub highlight: Option<bool>,
}

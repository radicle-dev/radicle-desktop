use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

use radicle::identity::RepoId;

/// Repos info.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct RepoInfo {
    pub payloads: SupportedPayloads,
    #[ts(type = "({ id: string } | { id: string, alias?: string })[]")]
    pub delegates: Vec<Value>,
    pub threshold: usize,
    #[ts(type = "{ type: 'public' } | { type: 'private', allow?: string[] }")]
    pub visibility: radicle::identity::Visibility,
    #[ts(as = "String")]
    pub rid: RepoId,
    pub seeding: usize,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SupportedPayloads {
    #[serde(rename = "xyz.radicle.project")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    #[ts(type = r#"{
  data: {
    defaultBranch: string,
    description: string,
    name: string,
  },
  meta: {
    head: string,
    issues: {
      open: number,
      closed: number,
    },
    patches: {
      open: number,
      draft: number,
      archived: number,
      merged: number,
    }
    lastCommit: number,
  }
}"#)]
    pub project: Option<Value>,
}

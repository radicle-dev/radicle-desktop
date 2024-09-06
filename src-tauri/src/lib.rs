mod commands;
mod error;
mod json;
mod types;

use serde_json::json;
use tauri::Manager;

use radicle::identity::doc::PayloadId;
use radicle::identity::DocAt;
use radicle::identity::RepoId;
use radicle::issue::cache::Issues;
use radicle::node::routing::Store;
use radicle::patch::cache::Patches;
use radicle::storage::git::Repository;
use radicle::storage::{ReadRepository, ReadStorage};

use commands::{auth, profile, repos};
use types::repo::SupportedPayloads;

struct AppState {
    profile: radicle::Profile,
}

impl AppState {
    pub fn repo_info<R: ReadRepository + radicle::cob::Store>(
        &self,
        repo: &R,
        doc: DocAt,
    ) -> Result<types::repo::RepoInfo, error::Error> {
        let DocAt { doc, .. } = doc;
        let rid = repo.id();

        let aliases = self.profile.aliases();
        let delegates = doc
            .delegates
            .into_iter()
            .map(|did| json::Author::new(&did).as_json(&aliases))
            .collect::<Vec<_>>();
        let db = &self.profile.database()?;
        let seeding = db.count(&rid).unwrap_or_default();

        let project = doc.payload.get(&PayloadId::project()).and_then(|payload| {
            let (_, head) = repo.head().ok()?;
            let commit = repo.commit(head).ok()?;
            let patches = self.profile.patches(repo).ok()?;
            let patches = patches.counts().ok()?;
            let issues = self.profile.issues(repo).ok()?;
            let issues = issues.counts().ok()?;

            Some(json!({
                "data": payload,
                "meta": {
                    "issues": issues,
                    "patches": patches,
                    "head": head,
                    "lastCommit": commit.time().seconds(),
                },
            }))
        });

        Ok(types::repo::RepoInfo {
            payloads: SupportedPayloads { project },
            delegates,
            threshold: doc.threshold,
            visibility: doc.visibility,
            rid,
            seeding,
        })
    }

    /// Get a repository by RID, checking to make sure we're allowed to view it.
    pub fn repo(&self, rid: RepoId) -> Result<(Repository, DocAt), error::Error> {
        let repo = self.profile.storage.repository(rid)?;
        let doc = repo.identity_doc()?;
        Ok((repo, doc))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let builder = tauri::Builder::default().plugin(tauri_plugin_devtools::init());
    #[cfg(not(debug_assertions))]
    let builder = tauri::Builder::default();

    builder
        .setup(|app| {
            let profile: radicle::Profile = match radicle::Profile::load() {
                Ok(profile) => Ok::<radicle::Profile, error::Error>(profile),
                Err(radicle::profile::Error::NotFound(path)) => Err(error::Error::WithHint {
                    err: anyhow::anyhow!("Radicle profile not found in '{}'.", path.display()),
                    hint: "To setup your radicle profile, run `rad auth`.",
                }),
                Err(e) => Err(error::Error::WithHint {
                    err: e.into(),
                    hint: "Could not load radicle profile",
                }),
            }?;

            app.manage(AppState { profile });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            auth::authenticate,
            repos::list_repos,
            profile::config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

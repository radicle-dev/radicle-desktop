use traits::Profile;
use traits::cobs::Cobs;
use traits::issue::{Issues, IssuesMut};
use traits::job::Jobs;
use traits::patch::{Patches, PatchesMut};
use traits::release::Releases;
use traits::release_mut::ReleasesMut;
use traits::repo::Repo;
use traits::thread::Thread;

pub mod artifact;
pub mod cobs;
pub mod config;
pub mod diff;
pub mod domain;
pub mod error;
pub mod oid;
pub mod outbound;
pub mod repo;
pub mod settings;
pub mod source;
pub mod syntax;
pub mod test;
pub mod traits;

#[derive(Clone)]
pub struct AppState {
    pub profile: radicle::Profile,
}

/// Client for the external `rad-artifact` node. Held separately from
/// `AppState` so command handlers and tests that don't touch artifact
/// seeding don't need to construct it.
#[derive(Clone)]
pub struct ArtifactClient {
    pub client: radicle_artifact_client::tokio::Client,
}

impl Repo for AppState {}
impl Thread for AppState {}
impl Cobs for AppState {}
impl Issues for AppState {}
impl IssuesMut for AppState {}
impl Jobs for AppState {}
impl Patches for AppState {}
impl PatchesMut for AppState {}
impl Releases for AppState {}
impl ReleasesMut for AppState {}
impl Profile for AppState {
    fn profile(&self) -> radicle::Profile {
        self.profile.clone()
    }
}

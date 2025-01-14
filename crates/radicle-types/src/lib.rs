use traits::auth::Auth;
use traits::cobs::Cobs;
use traits::issue::{Issues, IssuesMut};
use traits::patch::{Patches, PatchesMut};
use traits::repo::Repo;
use traits::thread::Thread;
use traits::Profile;

pub mod cobs;
pub mod config;
pub mod diff;
pub mod domain;
pub mod error;
pub mod outbound;
pub mod repo;
pub mod syntax;
pub mod test;
pub mod traits;

pub struct AppState {
    pub profile: radicle::Profile,
}

impl Auth for AppState {}
impl Repo for AppState {}
impl Thread for AppState {}
impl Cobs for AppState {}
impl Issues for AppState {}
impl IssuesMut for AppState {}
impl Patches for AppState {}
impl PatchesMut for AppState {}
impl Profile for AppState {
    fn profile(&self) -> radicle::Profile {
        self.profile.clone()
    }
}

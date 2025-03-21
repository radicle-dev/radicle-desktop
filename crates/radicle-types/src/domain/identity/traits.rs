use radicle::crypto::ssh::Passphrase;

use crate::error::Error;

pub trait IdentityService {
    fn authenticate(&self, passphrase: Passphrase) -> Result<(), Error>;
}

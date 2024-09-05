use serde_json::{json, Value};

use radicle::identity;
use radicle::node::AliasStore;

pub(crate) struct Author<'a>(&'a identity::Did);

impl<'a> Author<'a> {
    pub fn new(did: &'a identity::Did) -> Self {
        Self(did)
    }

    pub fn as_json(&self, aliases: &impl AliasStore) -> Value {
        aliases.alias(self.0).map_or(
            json!({ "id": self.0 }),
            |alias| json!({ "id": self.0, "alias": alias, }),
        )
    }
}

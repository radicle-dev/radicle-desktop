use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Serde JSON error.
    #[error(transparent)]
    SerdeJSON(#[from] serde_json::error::Error),
}

#[derive(Serialize)]
struct ErrorWrapper {
    err: String,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let wrapper = ErrorWrapper {
            err: self.to_string(),
        };

        wrapper.serialize(serializer)
    }
}

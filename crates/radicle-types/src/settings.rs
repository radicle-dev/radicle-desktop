use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Error;

const SETTINGS_FILE: &str = "desktop.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Whether newly-added artifacts are automatically seeded by the
    /// rad-artifact node and registered with a `radiroh://` location.
    /// Defaults to true — turning it off keeps the create-artifact path
    /// COB-only.
    #[serde(default = "default_auto_seed")]
    pub auto_seed_artifacts: bool,
}

fn default_auto_seed() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_seed_artifacts: default_auto_seed(),
        }
    }
}

fn path(home: &Path) -> PathBuf {
    home.join(SETTINGS_FILE)
}

/// Read settings from `<home>/desktop.json`. Missing file or parse failures
/// fall back to defaults — settings are non-critical.
pub fn load(home: &Path) -> Settings {
    let p = path(home);
    let Ok(bytes) = std::fs::read(&p) else {
        return Settings::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Write settings atomically (write to temp, rename) so a crash mid-write
/// can't leave a half-written file.
pub fn save(home: &Path, settings: &Settings) -> Result<(), Error> {
    let p = path(home);
    let tmp = p.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(settings)?;
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, &p)?;
    Ok(())
}

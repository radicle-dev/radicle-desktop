use std::{ops::Range, path::PathBuf};

use radicle_surf as surf;
use serde::Serialize;
use ts_rs::TS;

use radicle::git;

#[derive(Serialize, TS)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Diff {
    pub files: Vec<FileDiff>,
    pub stats: Stats,
}

impl From<surf::diff::Diff> for Diff {
    fn from(value: surf::diff::Diff) -> Self {
        Self {
            files: value.files().cloned().map(Into::into).collect::<Vec<_>>(),
            stats: (*value.stats()).into(),
        }
    }
}

#[derive(Serialize, TS)]
#[serde(
    tag = "status",
    rename_all_fields = "camelCase",
    rename_all = "camelCase"
)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub enum FileDiff {
    Added(Added),
    Deleted(Deleted),
    Modified(Modified),
    Moved(Moved),
    Copied(Copied),
}

impl From<surf::diff::FileDiff> for FileDiff {
    fn from(value: surf::diff::FileDiff) -> Self {
        match value {
            surf::diff::FileDiff::Added(surf::diff::Added { path, diff, new }) => {
                Self::Added(Added {
                    path,
                    diff: diff.into(),
                    new: new.into(),
                })
            }
            surf::diff::FileDiff::Deleted(surf::diff::Deleted { path, diff, old }) => {
                Self::Deleted(Deleted {
                    path,
                    diff: diff.into(),
                    old: old.into(),
                })
            }
            surf::diff::FileDiff::Modified(surf::diff::Modified {
                path,
                diff,
                old,
                new,
            }) => Self::Modified(Modified {
                path,
                diff: diff.into(),
                old: old.into(),
                new: new.into(),
            }),
            surf::diff::FileDiff::Moved(surf::diff::Moved {
                old_path,
                old,
                new_path,
                new,
                diff,
            }) => Self::Moved(Moved {
                old_path,
                old: old.into(),
                new_path,
                new: new.into(),
                diff: diff.into(),
            }),
            surf::diff::FileDiff::Copied(surf::diff::Copied {
                old_path,
                new_path,
                old,
                new,
                diff,
            }) => Self::Copied(Copied {
                old_path,
                new_path,
                old: old.into(),
                new: new.into(),
                diff: diff.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(
    tag = "type",
    rename_all_fields = "camelCase",
    rename_all = "camelCase"
)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub enum DiffContent {
    Binary,
    Plain {
        hunks: Hunks,
        stats: FileStats,
        eof: EofNewLine,
    },
    Empty,
}

impl From<surf::diff::DiffContent> for DiffContent {
    fn from(value: surf::diff::DiffContent) -> Self {
        match value {
            surf::diff::DiffContent::Plain { hunks, stats, eof } => Self::Plain {
                hunks: hunks.into(),
                stats: stats.into(),
                eof: eof.into(),
            },
            surf::diff::DiffContent::Binary => Self::Binary,
            surf::diff::DiffContent::Empty => Self::Empty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct DiffFile {
    #[ts(as = "String")]
    pub oid: git::Oid,
    pub mode: FileMode,
}

impl From<surf::diff::DiffFile> for DiffFile {
    fn from(value: surf::diff::DiffFile) -> Self {
        Self {
            oid: value.oid,
            mode: value.mode.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Added {
    pub path: PathBuf,
    pub diff: DiffContent,
    pub new: DiffFile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Deleted {
    pub path: PathBuf,
    pub diff: DiffContent,
    pub old: DiffFile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Moved {
    pub old_path: PathBuf,
    pub old: DiffFile,
    pub new_path: PathBuf,
    pub new: DiffFile,
    pub diff: DiffContent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Copied {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub old: DiffFile,
    pub new: DiffFile,
    pub diff: DiffContent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Modified {
    pub path: PathBuf,
    pub diff: DiffContent,
    pub old: DiffFile,
    pub new: DiffFile,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Stats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl From<surf::diff::Stats> for Stats {
    fn from(value: surf::diff::Stats) -> Self {
        Self {
            files_changed: value.files_changed,
            insertions: value.insertions,
            deletions: value.deletions,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all_fields = "camelCase", rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub enum FileMode {
    Blob,
    BlobExecutable,
    Tree,
    Link,
    Commit,
}

impl From<surf::diff::FileMode> for FileMode {
    fn from(value: surf::diff::FileMode) -> Self {
        match value {
            surf::diff::FileMode::Blob => Self::Blob,
            surf::diff::FileMode::BlobExecutable => Self::BlobExecutable,
            surf::diff::FileMode::Tree => Self::Tree,
            surf::diff::FileMode::Link => Self::Link,
            surf::diff::FileMode::Commit => Self::Commit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all_fields = "camelCase", rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub enum EofNewLine {
    OldMissing,
    NewMissing,
    BothMissing,
    NoneMissing,
}

impl From<surf::diff::EofNewLine> for EofNewLine {
    fn from(value: surf::diff::EofNewLine) -> Self {
        match value {
            surf::diff::EofNewLine::OldMissing => Self::OldMissing,
            surf::diff::EofNewLine::NewMissing => Self::NewMissing,
            surf::diff::EofNewLine::BothMissing => Self::BothMissing,
            surf::diff::EofNewLine::NoneMissing => Self::NoneMissing,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct FileStats {
    pub additions: usize,
    pub deletions: usize,
}

impl From<surf::diff::FileStats> for FileStats {
    fn from(value: surf::diff::FileStats) -> Self {
        Self {
            additions: value.additions,
            deletions: value.deletions,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(
    tag = "type",
    rename_all_fields = "camelCase",
    rename_all = "camelCase"
)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub enum Modification {
    Addition(Addition),
    Deletion(Deletion),
    Context {
        line: String,
        line_no_old: u32,
        line_no_new: u32,
        highlight: Option<crate::syntax::Line>,
    },
}

impl From<surf::diff::Modification> for Modification {
    fn from(value: surf::diff::Modification) -> Self {
        match value {
            surf::diff::Modification::Addition(surf::diff::Addition { line, line_no }) => {
                Modification::Addition(Addition {
                    line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                    line_no,
                    highlight: None,
                })
            }
            surf::diff::Modification::Deletion(surf::diff::Deletion { line, line_no }) => {
                Modification::Deletion(Deletion {
                    line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                    line_no,
                    highlight: None,
                })
            }
            surf::diff::Modification::Context {
                line,
                line_no_old,
                line_no_new,
            } => Modification::Context {
                line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                line_no_old,
                line_no_new,
                highlight: None,
            },
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, TS)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Hunks(pub Vec<Hunk>);

impl From<Vec<Hunk>> for Hunks {
    fn from(hunks: Vec<Hunk>) -> Self {
        Self(hunks)
    }
}

impl From<surf::diff::Hunks<surf::diff::Modification>> for Hunks {
    fn from(value: surf::diff::Hunks<surf::diff::Modification>) -> Self {
        Self(value.0.into_iter().map(Into::into).collect::<Vec<_>>())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<Modification>,
    pub old: Range<u32>,
    pub new: Range<u32>,
}

impl From<surf::diff::Hunk<surf::diff::Modification>> for Hunk {
    fn from(value: surf::diff::Hunk<surf::diff::Modification>) -> Self {
        Self {
            header: String::from_utf8_lossy(value.header.as_bytes()).to_string(),
            lines: value.lines.into_iter().map(Into::into).collect::<Vec<_>>(),
            old: value.old,
            new: value.new,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Line(pub(crate) Vec<u8>);

impl Line {
    /// Create a new line.
    pub fn new(item: Vec<u8>) -> Self {
        Self(item)
    }
}

impl From<surf::diff::Line> for Line {
    fn from(value: surf::diff::Line) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Addition {
    pub line: String,
    pub line_no: u32,
    pub highlight: Option<crate::syntax::Line>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[ts(export_to = "diff/")]
pub struct Deletion {
    pub line: String,
    pub line_no: u32,
    pub highlight: Option<crate::syntax::Line>,
}

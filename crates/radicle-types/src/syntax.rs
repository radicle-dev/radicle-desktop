use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tree_sitter_highlight as ts;
use ts_rs::TS;

use radicle::git;
use radicle_surf as surf;

use crate as types;

/// Highlight groups enabled.
const HIGHLIGHTS: &[&str] = &[
    "attribute",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "declare",
    "embedded",
    "escape",
    "export",
    "float.literal",
    "function",
    "function.builtin",
    "function.macro",
    "function.method",
    "identifier",
    "indent.and",
    "indent.begin",
    "indent.branch",
    "indent.end",
    "integer_literal",
    "keyword",
    "keyword.coroutine",
    "keyword.debug",
    "keyword.exception",
    "keyword.repeat",
    "local.definition",
    "local.reference",
    "local.scope",
    "label",
    "module",
    "none",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "shorthand_property_identifier",
    "statement",
    "string",
    "string.special",
    "tag",
    "tag.delimiter",
    "tag.error",
    "text",
    "text.literal",
    "text.title",
    "type",
    "type.builtin",
    "type.qualifier",
    "type_annotation",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

/// A structure encapsulating an item and styling.
#[derive(Clone, TS, Debug, Serialize, Eq, PartialEq)]
#[ts(export)]
#[ts(export_to = "syntax/")]
pub struct Paint {
    pub item: String,
    pub style: Option<String>,
}

impl Paint {
    /// Constructs a new `Paint` structure encapsulating `item` with no set styling.
    pub fn new(item: String) -> Paint {
        Paint { item, style: None }
    }

    /// Sets the style of `self` to `style`.
    pub fn with_style(mut self, style: String) -> Paint {
        self.style = Some(style);
        self
    }
}

/// A styled string that does not contain any `'\n'`.
#[derive(Clone, Debug, Serialize, Eq, PartialEq, TS)]
#[ts(export)]
#[ts(export_to = "syntax/")]
pub struct Label(Paint);

impl Label {
    /// Create a new label.
    pub fn new(s: &str) -> Self {
        Self(Paint::new(cleanup(s)))
    }

    /// Style a label.
    pub fn style(self, style: String) -> Self {
        Self(self.0.with_style(style))
    }
}

impl From<String> for Label {
    fn from(value: String) -> Self {
        Self::new(value.as_str())
    }
}

impl From<&str> for Label {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// A line of text that has styling and can be displayed.
#[derive(Clone, Debug, Serialize, Default, PartialEq, TS, Eq)]
#[ts(export)]
#[ts(export_to = "syntax/")]
pub struct Line {
    items: Vec<Label>,
}

impl Line {
    /// Create a new line.
    pub fn new(item: impl Into<Label>) -> Self {
        Self {
            items: vec![item.into()],
        }
    }
}

impl IntoIterator for Line {
    type Item = Label;
    type IntoIter = Box<dyn Iterator<Item = Label>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.items.into_iter())
    }
}

impl<T: Into<Label>> From<T> for Line {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl From<Vec<Label>> for Line {
    fn from(items: Vec<Label>) -> Self {
        Self { items }
    }
}

/// Cleanup the input string for display as a label.
fn cleanup(input: &str) -> String {
    input.chars().filter(|c| *c != '\n' && *c != '\r').collect()
}

/// Syntax highlighted file builder.
#[derive(Default)]
struct Builder {
    /// Output lines.
    lines: Vec<Line>,
    /// Current output line.
    line: Vec<Label>,
    /// Current label.
    label: Vec<u8>,
    /// Current stack of styles.
    styles: Vec<String>,
}

impl Builder {
    /// Run the builder to completion.
    fn run(
        mut self,
        highlights: impl Iterator<Item = Result<ts::HighlightEvent, ts::Error>>,
        code: &[u8],
    ) -> Result<Vec<Line>, ts::Error> {
        for event in highlights {
            match event? {
                ts::HighlightEvent::Source { start, end } => {
                    println!("highlightEvent::source {start} {end}");
                    for (i, byte) in code.iter().enumerate().skip(start).take(end - start) {
                        if *byte == b'\n' {
                            self.advance();
                            // Start on new line.
                            self.lines.push(Line::from(self.line.clone()));
                            self.line.clear();
                        } else if i == code.len() - 1 {
                            // File has no `\n` at the end.
                            self.label.push(*byte);
                            self.advance();
                            self.lines.push(Line::from(self.line.clone()));
                        } else {
                            // Add to existing label.
                            self.label.push(*byte);
                        }
                    }
                }
                ts::HighlightEvent::HighlightStart(h) => {
                    let name = HIGHLIGHTS[h.0];
                    println!("highlightEvent::HighlightStart {name}");

                    self.advance();
                    self.styles.push(name.to_string());
                }
                ts::HighlightEvent::HighlightEnd => {
                    self.advance();
                    self.styles.pop();
                }
            }
        }
        Ok(self.lines)
    }

    /// Advance the state by pushing the current label onto the current line,
    /// using the current styling.
    fn advance(&mut self) {
        if !self.label.is_empty() {
            // Take the top-level style when there are more than one.
            let style = self.styles.first().cloned().unwrap_or_default();
            self.line
                .push(Label::new(String::from_utf8_lossy(&self.label).as_ref()).style(style));
            self.label.clear();
        }
    }
}

/// Syntax highlighter based on `tree-sitter`.
pub struct Highlighter {
    configs: std::collections::HashMap<String, ts::HighlightConfiguration>,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        let configs: std::collections::HashMap<String, ts::HighlightConfiguration> = [
            ("rust", Self::config("rust")),
            ("json", Self::config("json")),
            ("jsdoc", Self::config("jsdoc")),
            ("typescript", Self::config("typescript")),
            ("javascript", Self::config("javascript")),
            ("markdown", Self::config("markdown")),
            ("css", Self::config("css")),
            ("go", Self::config("go")),
            ("regex", Self::config("regex")),
            ("shell", Self::config("shell")),
            ("c", Self::config("c")),
            ("python", Self::config("python")),
            ("svelte", Self::config("svelte")),
            ("ruby", Self::config("ruby")),
            ("tsx", Self::config("tsx")),
            ("html", Self::config("html")),
            ("toml", Self::config("toml")),
        ]
        .into_iter()
        .filter_map(|(lang, cfg)| cfg.map(|c| (lang.to_string(), c)))
        .collect();

        Highlighter { configs }
    }

    /// Highlight a source code file.
    pub fn highlight(&mut self, path: &Path, code: &[u8]) -> Result<Vec<Line>, ts::Error> {
        let mut highlighter = ts::Highlighter::new();
        // Check for a language if none found return plain lines.
        let Some(language) = Self::detect(path, code) else {
            let Ok(code) = std::str::from_utf8(code) else {
                return Err(ts::Error::Unknown);
            };
            println!("Not able to detect language?");
            return Ok(code.lines().map(Line::new).collect());
        };

        // Check if there is a configuration if none found return plain lines.
        let Some(config) = &mut Self::config(&language) else {
            let Ok(code) = std::str::from_utf8(code) else {
                return Err(ts::Error::Unknown);
            };
            println!("Not found a configuration?");
            return Ok(code.lines().map(Line::new).collect());
        };

        config.configure(HIGHLIGHTS);

        let highlights = highlighter.highlight(config, code, None, |language| {
            let l: &'static str = std::boxed::Box::leak(language.to_string().into_boxed_str());

            self.configs.get(l)
        })?;

        Builder::default().run(highlights, code)
    }

    /// Detect language.
    fn detect(path: &Path, _code: &[u8]) -> Option<String> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => Some(String::from("rust")),
            Some("svelte") => Some(String::from("svelte")),
            Some("ts" | "js") => Some(String::from("typescript")),
            Some("json") => Some(String::from("json")),
            Some("regex") => Some(String::from("regex")),
            Some("sh" | "bash") => Some(String::from("shell")),
            Some("md" | "markdown") => Some(String::from("markdown")),
            Some("go") => Some(String::from("go")),
            Some("c") => Some(String::from("c")),
            Some("py") => Some(String::from("python")),
            Some("rb") => Some(String::from("ruby")),
            Some("tsx") => Some(String::from("tsx")),
            Some("html") | Some("htm") | Some("xml") => Some(String::from("html")),
            Some("css") => Some(String::from("css")),
            Some("toml") => Some(String::from("toml")),
            _ => None,
        }
    }

    /// Get a language configuration.
    fn config(language: &str) -> Option<ts::HighlightConfiguration> {
        match language {
            "rust" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_rust::LANGUAGE.into(),
                    language,
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "json" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_json::LANGUAGE.into(),
                    language,
                    tree_sitter_json::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "javascript" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_javascript::LANGUAGE.into(),
                    language,
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_javascript::LOCALS_QUERY,
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "typescript" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
                    language,
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                    "",
                    tree_sitter_typescript::LOCALS_QUERY,
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "markdown" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_md::LANGUAGE.into(),
                    language,
                    tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                    tree_sitter_md::INJECTION_QUERY_BLOCK,
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "css" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_css::LANGUAGE.into(),
                    language,
                    tree_sitter_css::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "go" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_go::LANGUAGE.into(),
                    language,
                    tree_sitter_go::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "shell" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_bash::LANGUAGE.into(),
                    language,
                    tree_sitter_bash::HIGHLIGHT_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "c" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_c::LANGUAGE.into(),
                    language,
                    tree_sitter_c::HIGHLIGHT_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "python" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_python::LANGUAGE.into(),
                    language,
                    tree_sitter_python::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "regex" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_regex::LANGUAGE.into(),
                    language,
                    tree_sitter_regex::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "svelte" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_svelte_ng::LANGUAGE.into(),
                    language,
                    tree_sitter_svelte_ng::HIGHLIGHTS_QUERY,
                    tree_sitter_svelte_ng::INJECTIONS_QUERY,
                    tree_sitter_svelte_ng::LOCALS_QUERY,
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "ruby" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_ruby::LANGUAGE.into(),
                    language,
                    tree_sitter_ruby::HIGHLIGHTS_QUERY,
                    "",
                    tree_sitter_ruby::LOCALS_QUERY,
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "jsdoc" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_jsdoc::LANGUAGE.into(),
                    language,
                    tree_sitter_jsdoc::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "tsx" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_typescript::LANGUAGE_TSX.into(),
                    language,
                    tree_sitter_typescript::HIGHLIGHTS_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_typescript::LOCALS_QUERY,
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "html" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_html::LANGUAGE.into(),
                    language,
                    tree_sitter_html::HIGHLIGHTS_QUERY,
                    tree_sitter_html::INJECTIONS_QUERY,
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            "toml" => Some(
                ts::HighlightConfiguration::new(
                    tree_sitter_toml_ng::LANGUAGE.into(),
                    language,
                    tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .expect("Highlighter::config: highlight configuration must be valid"),
            ),
            _ => None,
        }
    }
}

/// Blob returned by the [`Repo`] trait.
#[derive(PartialEq, Eq, Debug)]
pub enum Blob {
    Binary,
    Empty,
    Plain(Vec<u8>),
}

/// A repository of Git blobs.
pub trait Repo {
    /// Lookup a blob from the repo.
    fn blob(&self, oid: git::Oid) -> Result<Blob, git::raw::Error>;
    /// Lookup a file in the workdir.
    fn file(&self, path: &Path) -> Option<Blob>;
}

impl Repo for git::raw::Repository {
    fn blob(&self, oid: git::Oid) -> Result<Blob, git::raw::Error> {
        let blob = self.find_blob(*oid)?;

        if blob.is_binary() {
            Ok(Blob::Binary)
        } else {
            let content = blob.content();

            if content.is_empty() {
                Ok(Blob::Empty)
            } else {
                Ok(Blob::Plain(blob.content().to_vec()))
            }
        }
    }

    fn file(&self, path: &Path) -> Option<Blob> {
        self.workdir()
            .and_then(|dir| fs::read(dir.join(path)).ok())
            .map(|content| {
                // A file is considered binary if there is a zero byte in the first 8 kilobytes
                // of the file. This is the same heuristic Git uses.
                let binary = content.iter().take(8192).any(|b| *b == 0);
                if binary {
                    Blob::Binary
                } else {
                    Blob::Plain(content)
                }
            })
    }
}

/// Blobs passed down to the hunk renderer.
#[derive(Debug)]
pub struct Blobs<T> {
    pub old: Option<T>,
    pub new: Option<T>,
}

impl<T> Blobs<T> {
    pub fn new(old: Option<T>, new: Option<T>) -> Self {
        Self { old, new }
    }
}

impl Blobs<(PathBuf, Blob)> {
    pub fn highlight(&self, hi: &mut Highlighter) -> Blobs<Vec<Line>> {
        let mut blobs = Blobs::default();
        if let Some((path, Blob::Plain(content))) = &self.old {
            blobs.old = hi.highlight(path, content).ok();
        }
        if let Some((path, Blob::Plain(content))) = &self.new {
            blobs.new = hi.highlight(path, content).ok();
        }
        blobs
    }

    pub fn from_paths<R: Repo>(
        old: Option<(&Path, git::Oid)>,
        new: Option<(&Path, git::Oid)>,
        repo: &R,
    ) -> Blobs<(PathBuf, Blob)> {
        Blobs::new(
            old.and_then(|(path, oid)| {
                repo.blob(oid)
                    .ok()
                    .or_else(|| repo.file(path))
                    .map(|blob| (path.to_path_buf(), blob))
            }),
            new.and_then(|(path, oid)| {
                repo.blob(oid)
                    .ok()
                    .or_else(|| repo.file(path))
                    .map(|blob| (path.to_path_buf(), blob))
            }),
        )
    }
}

impl<T> Default for Blobs<T> {
    fn default() -> Self {
        Self {
            old: None,
            new: None,
        }
    }
}

pub trait ToPretty {
    /// The output of the render process.
    type Output: Serialize;
    /// Context that can be passed down from parent objects during rendering.
    type Context;

    /// Render to pretty diff output.
    fn pretty<R: Repo>(
        &self,
        hi: &mut Highlighter,
        context: &Self::Context,
        repo: &R,
    ) -> Self::Output;
}

impl ToPretty for surf::diff::Diff {
    type Output = types::diff::Diff;
    type Context = ();

    fn pretty<R: Repo>(
        &self,
        hi: &mut Highlighter,
        context: &Self::Context,
        repo: &R,
    ) -> Self::Output {
        let files = self
            .files()
            .map(|f| f.pretty(hi, context, repo))
            .collect::<Vec<_>>();

        types::diff::Diff {
            files,
            stats: (*self.stats()).into(),
        }
    }
}

impl ToPretty for surf::diff::FileDiff {
    type Output = types::diff::FileDiff;
    type Context = ();

    fn pretty<R: Repo>(
        &self,
        hi: &mut Highlighter,
        _context: &Self::Context,
        repo: &R,
    ) -> Self::Output {
        match self {
            surf::diff::FileDiff::Added(f) => types::diff::FileDiff::Added(f.pretty(hi, &(), repo)),
            surf::diff::FileDiff::Deleted(f) => {
                types::diff::FileDiff::Deleted(f.pretty(hi, &(), repo))
            }
            surf::diff::FileDiff::Modified(f) => {
                types::diff::FileDiff::Modified(f.pretty(hi, &(), repo))
            }
            surf::diff::FileDiff::Moved(f) => types::diff::FileDiff::Moved(f.pretty(hi, &(), repo)),
            surf::diff::FileDiff::Copied(f) => {
                types::diff::FileDiff::Copied(f.pretty(hi, &(), repo))
            }
        }
    }
}

impl ToPretty for surf::diff::DiffContent {
    type Output = types::diff::DiffContent;
    type Context = Blobs<(PathBuf, Blob)>;

    fn pretty<R: Repo>(
        &self,
        hi: &mut Highlighter,
        blobs: &Self::Context,
        repo: &R,
    ) -> Self::Output {
        match self {
            surf::diff::DiffContent::Plain {
                hunks: surf::diff::Hunks(hunks),
                eof,
                stats,
            } => {
                let blobs = blobs.highlight(hi);

                let hunks = hunks
                    .iter()
                    .map(|h| h.pretty(hi, &blobs, repo))
                    .collect::<Vec<_>>();

                types::diff::DiffContent::Plain {
                    hunks: hunks.into(),
                    stats: (*stats).into(),
                    eof: (*eof).clone().into(),
                }
            }
            surf::diff::DiffContent::Binary => types::diff::DiffContent::Binary,
            surf::diff::DiffContent::Empty => types::diff::DiffContent::Empty,
        }
    }
}

impl ToPretty for surf::diff::Moved {
    type Output = types::diff::Moved;
    type Context = ();

    fn pretty<R: Repo>(&self, hi: &mut Highlighter, _: &Self::Context, repo: &R) -> Self::Output {
        let old = Some((self.old_path.as_path(), self.old.oid));
        let new = Some((self.new_path.as_path(), self.new.oid));
        let blobs = Blobs::from_paths(old, new, repo);

        types::diff::Moved {
            old_path: self.old_path.clone(),
            old: self.old.clone().into(),
            new_path: self.new_path.clone(),
            new: self.new.clone().into(),
            diff: self.diff.pretty(hi, &blobs, repo),
        }
    }
}

impl ToPretty for surf::diff::Added {
    type Output = types::diff::Added;
    type Context = ();

    fn pretty<R: Repo>(&self, hi: &mut Highlighter, _: &Self::Context, repo: &R) -> Self::Output {
        let old = None;
        let new = Some((self.path.as_path(), self.new.oid));
        let blobs = Blobs::from_paths(old, new, repo);

        types::diff::Added {
            path: self.path.clone(),
            diff: self.diff.pretty(hi, &blobs, repo),
            new: self.new.clone().into(),
        }
    }
}

impl ToPretty for surf::diff::Deleted {
    type Output = types::diff::Deleted;
    type Context = ();

    fn pretty<R: Repo>(&self, hi: &mut Highlighter, _: &Self::Context, repo: &R) -> Self::Output {
        let old = Some((self.path.as_path(), self.old.oid));
        let new = None;
        let blobs = Blobs::from_paths(old, new, repo);

        types::diff::Deleted {
            path: self.path.clone(),
            diff: self.diff.pretty(hi, &blobs, repo),
            old: self.old.clone().into(),
        }
    }
}

impl ToPretty for surf::diff::Modified {
    type Output = types::diff::Modified;
    type Context = ();

    fn pretty<R: Repo>(&self, hi: &mut Highlighter, _: &Self::Context, repo: &R) -> Self::Output {
        let old = Some((self.path.as_path(), self.old.oid));
        let new = Some((self.path.as_path(), self.new.oid));
        let blobs = Blobs::from_paths(old, new, repo);

        types::diff::Modified {
            path: self.path.clone(),
            diff: self.diff.pretty(hi, &blobs, repo),
            new: self.new.clone().into(),
            old: self.old.clone().into(),
        }
    }
}

impl ToPretty for surf::diff::Copied {
    type Output = types::diff::Copied;
    type Context = ();

    fn pretty<R: Repo>(&self, hi: &mut Highlighter, _: &Self::Context, repo: &R) -> Self::Output {
        let old = Some((self.old_path.as_path(), self.old.oid));
        let new = Some((self.new_path.as_path(), self.new.oid));
        let blobs = Blobs::from_paths(old, new, repo);

        types::diff::Copied {
            old_path: self.old_path.clone(),
            new_path: self.new_path.clone(),
            diff: self.diff.pretty(hi, &blobs, repo),
            new: self.new.clone().into(),
            old: self.old.clone().into(),
        }
    }
}

impl ToPretty for surf::diff::Hunk<surf::diff::Modification> {
    type Output = types::diff::Hunk;
    type Context = Blobs<Vec<Line>>;

    fn pretty<R: Repo>(
        &self,
        hi: &mut Highlighter,
        blobs: &Self::Context,
        repo: &R,
    ) -> Self::Output {
        let lines = self
            .lines
            .clone()
            .into_iter()
            .map(|l| l.pretty(hi, blobs, repo))
            .collect::<Vec<_>>();

        types::diff::Hunk {
            header: String::from_utf8_lossy(self.header.as_bytes()).to_string(),
            new: self.new.clone(),
            old: self.old.clone(),
            lines,
        }
    }
}

impl ToPretty for surf::diff::Modification {
    type Output = types::diff::Modification;
    type Context = Blobs<Vec<Line>>;

    fn pretty<R: Repo>(
        &self,
        _hi: &mut Highlighter,
        blobs: &<radicle_surf::diff::Modification as ToPretty>::Context,
        _repo: &R,
    ) -> Self::Output {
        match self {
            surf::diff::Modification::Deletion(surf::diff::Deletion { line, line_no }) => {
                if let Some(lines) = &blobs.old.as_ref() {
                    types::diff::Modification::Deletion(types::diff::Deletion {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        highlight: Some(lines[*line_no as usize - 1].clone()),
                        line_no: *line_no,
                    })
                } else {
                    types::diff::Modification::Deletion(types::diff::Deletion {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        line_no: *line_no,
                        highlight: None,
                    })
                }
            }
            surf::diff::Modification::Addition(surf::diff::Addition { line, line_no }) => {
                if let Some(lines) = &blobs.new.as_ref() {
                    types::diff::Modification::Addition(types::diff::Addition {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        line_no: *line_no,
                        highlight: Some(lines[*line_no as usize - 1].clone()),
                    })
                } else {
                    types::diff::Modification::Addition(types::diff::Addition {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        line_no: *line_no,
                        highlight: None,
                    })
                }
            }
            surf::diff::Modification::Context {
                line,
                line_no_new,
                line_no_old,
            } => {
                // Nb. we can check in the old or the new blob, we choose the new.
                if let Some(lines) = &blobs.new.as_ref() {
                    types::diff::Modification::Context {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        line_no_new: *line_no_new,
                        line_no_old: *line_no_old,
                        highlight: Some(lines[*line_no_new as usize - 1].clone()),
                    }
                } else {
                    types::diff::Modification::Context {
                        line: String::from_utf8_lossy(line.as_bytes()).to_string(),
                        line_no_new: *line_no_new,
                        line_no_old: *line_no_old,
                        highlight: None,
                    }
                }
            }
        }
    }
}

//! Locating the external binaries the app shells out to.
//!
//! An app launched from the desktop — Finder, the Dock, a `.desktop` entry —
//! inherits its environment from the OS session manager rather than from a
//! shell, so `PATH` holds none of the user's own install locations: on macOS
//! launchd hands out `/usr/bin:/bin:/usr/sbin:/sbin`, which covers neither
//! Homebrew nor Nix nor anything under the home directory. So `PATH` is where
//! we look first, and the usual install locations after it.
//!
//! Nothing here is load-bearing. Everything the app runs `git` for it can also
//! do through libgit2, only slower, and `rad` is looked for to tell the user
//! whether they have the CLI rather than to run it.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
// See <https://learn.microsoft.com/windows/win32/procthread/process-creation-flags#flags>.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Environment variable that pins the git binary, skipping detection
/// entirely. An escape hatch for debugging a misdetection.
pub const GIT_ENV: &str = "RAD_DESKTOP_GIT";

/// How long a candidate gets to answer `--version`. Git answers immediately,
/// so this is only here to keep a candidate that never exits from taking the
/// resolution — and every caller waiting on it — with it.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// Apple's `git` at `/usr/bin/git` is a shim that exists on every macOS
/// install whether or not git does. With the Command Line Developer Tools
/// missing, running it does nothing but open a system dialog offering to
/// install them — once per invocation. See [`command_line_tools_installed`].
#[cfg(target_os = "macos")]
const APPLE_GIT_SHIM: &str = "/usr/bin/git";

/// The resolved git binary. `None` means "searched and found nothing", which
/// is cached like any other answer so that a machine without git does not
/// rescan on every diff.
static GIT: OnceLock<Option<PathBuf>> = OnceLock::new();

/// The git binary to shell out to, or `None` when the machine has none that
/// works. Callers are expected to have a fallback: everything the app runs
/// git for is an optimization over the same work done through libgit2.
pub fn git() -> Option<PathBuf> {
    GIT.get_or_init(|| {
        let git = search();
        match &git {
            Some(path) => log::info!("Using git binary {}", path.display()),
            None => log::warn!(
                "No working git binary found; falling back to libgit2 for diff stats, \
                 commit counts and file history, which is slower on large repositories"
            ),
        }

        git
    })
    .clone()
}

/// A [`std::process::Command`] for that binary. The one place that knows how
/// git is spawned, `CREATE_NO_WINDOW` on Windows included, so that no call
/// site can forget it and flash up a console window.
pub fn git_command() -> Option<std::process::Command> {
    // `mut` only for the Windows line below.
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut command = std::process::Command::new(git()?);
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    Some(command)
}

/// The `rad` CLI, if it is installed somewhere we can see.
///
/// Deliberately uncached: the app polls for the CLI so that installing it
/// while the app is running lifts the "Radicle CLI not installed" warning
/// without a restart.
pub fn rad() -> Option<PathBuf> {
    candidates("rad").find(|path| path.is_file())
}

/// Everywhere a binary called `name` might be, `PATH` first so that the user's
/// own choice wins, then the locations a desktop launch's `PATH` tends to be
/// missing.
fn candidates(name: &str) -> impl Iterator<Item = PathBuf> {
    // Homebrew on Apple silicon and on Intel, MacPorts, a Nix profile (per
    // user and system-wide), and the directory the Radicle installer uses.
    #[cfg(unix)]
    const DIRS: &[&str] = &[
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/local/bin",
        "~/.nix-profile/bin",
        "/nix/var/nix/profiles/default/bin",
        "/run/current-system/sw/bin",
        "~/.local/bin",
        "~/.radicle/bin",
    ];
    #[cfg(windows)]
    const DIRS: &[&str] = &[];

    let name = name.to_owned();
    // Collected because the iterator `which_all` hands back borrows the name.
    let path = which::which_all(&name)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let known = DIRS.iter().filter_map(move |dir| {
        let dir = match dir.strip_prefix("~/") {
            Some(rest) => PathBuf::from(std::env::var_os("HOME")?).join(rest),
            None => PathBuf::from(dir),
        };

        Some(dir.join(&name))
    });

    path.into_iter().chain(known)
}

fn search() -> Option<PathBuf> {
    if let Some(pinned) = std::env::var_os(GIT_ENV) {
        let pinned = PathBuf::from(pinned);
        match validate(&pinned) {
            Some(version) => {
                log::info!("{GIT_ENV} pins git to {}: {version}", pinned.display());

                return Some(pinned);
            }
            None => log::warn!(
                "{GIT_ENV} is set to {}, which does not run git; detecting instead",
                pinned.display()
            ),
        }
    }

    detect()
}

/// Also looks in the 64-bit, 32-bit and per-user program files directories,
/// the same ones VS Code's git extension does. `Git\cmd\git.exe` is the
/// wrapper Git for Windows means for others to call, as opposed to the bare
/// executable under `Git\mingw64\bin`.
#[cfg(windows)]
fn detect() -> Option<PathBuf> {
    const PROGRAM_FILES: &[&str] = &["ProgramW6432", "ProgramFiles(x86)", "ProgramFiles"];

    let program_files = PROGRAM_FILES
        .iter()
        .filter_map(|var| Some(PathBuf::from(std::env::var_os(var)?).join(r"Git\cmd\git.exe")));
    let local = std::env::var_os("LocalAppData")
        .map(|base| PathBuf::from(base).join(r"Programs\Git\cmd\git.exe"));

    candidates("git.exe")
        .chain(program_files)
        .chain(local)
        .find(|candidate| validate(candidate).is_some())
}

#[cfg(not(windows))]
fn detect() -> Option<PathBuf> {
    for candidate in candidates("git") {
        // Skipping the shim has to leave a Homebrew or Nix git further down
        // the line still reachable, which is why this is a whole list of
        // candidates rather than the first `git` on `PATH`.
        #[cfg(target_os = "macos")]
        if candidate == Path::new(APPLE_GIT_SHIM) && !command_line_tools_installed() {
            log::info!(
                "Skipping {APPLE_GIT_SHIM}: the Command Line Developer Tools are not installed, \
                 so running it would only prompt to install them"
            );
            continue;
        }
        if validate(&candidate).is_some() {
            return Some(candidate);
        }
    }

    None
}

/// Whether Apple's `git` shim has a real git to hand off to.
///
/// `xcode-select -p` answers with the active developer directory — the Command
/// Line Developer Tools' or a full Xcode's, and either of those ships git. It
/// exits with 2 when there is none, which is exactly the case where the shim
/// prompts instead of running git. It also answers with a directory that has
/// since been deleted, an Xcode moved to the bin leaving the setting behind,
/// so the answer has to still be there as well. `xcode-select` is a real
/// binary rather than a shim, so asking is free of side effects.
#[cfg(target_os = "macos")]
fn command_line_tools_installed() -> bool {
    let Ok(output) = std::process::Command::new("/usr/bin/xcode-select")
        .arg("-p")
        .output()
        .inspect_err(|e| log::warn!("Could not run xcode-select: {e}"))
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let developer_dir = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    !developer_dir.is_empty() && Path::new(&developer_dir).is_dir()
}

/// The version a candidate reports, or `None` if it is not a git we can use.
fn validate(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let version = probe(path)?;

    // Exiting cleanly is not enough: plenty of binaries answer `--version`,
    // and accepting one of those would mean parsing another program's output
    // as rev-lists and diffs. Git says "git version 2.51.0" — untranslated,
    // and the same wording from Apple's, Homebrew's, Nix's and Git for
    // Windows' builds alike.
    if !version.starts_with("git version ") {
        log::warn!(
            "{} answers `--version` with {version:?} rather than a git version",
            path.display()
        );

        return None;
    }

    Some(version)
}

/// Asks a candidate for its version, giving up after [`PROBE_TIMEOUT`].
fn probe(path: &Path) -> Option<String> {
    use std::io::Read;

    let mut command = std::process::Command::new(path);
    command
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command.spawn().ok()?;

    // Waiting for the exit before reading: a candidate that prints more than
    // the pipe holds would block on us while we block on it, and the deadline
    // is what breaks that as well.
    let deadline = std::time::Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) => {}
        }
        if std::time::Instant::now() >= deadline {
            log::warn!(
                "{} did not answer `--version` within {}s, taking it as not being git",
                path.display(),
                PROBE_TIMEOUT.as_secs()
            );
            let _ = child.kill();
            let _ = child.wait();

            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // The process is gone and `git --version` is a single line, far short of
    // the pipe's capacity, so this reads what is already buffered and sees the
    // end of input straight after.
    let mut version = Vec::new();
    child.stdout.take()?.read_to_end(&mut version).ok()?;

    Some(String::from_utf8_lossy(&version).trim().to_string())
}

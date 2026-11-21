//! Codedeck's domain: finds projects, reads agent sessions, matches them. No UI.

pub mod avatar;
pub mod git;
pub mod index;
pub mod launch;
pub mod license;
pub mod matcher;
pub mod model;
pub mod scan;
pub mod settings;
pub mod sources;
pub mod stack;
pub mod watch;

pub use model::{Agent, Project, Session, Snapshot};
pub use settings::Settings;

use std::time::{SystemTime, UNIX_EPOCH};

/// The one call the app makes: walk the roots, read every agent, match them up.
pub fn scan_all(settings: &Settings) -> Snapshot {
    let roots = scan::find_projects(&settings.roots, settings.depth);
    let read = sources::read_all(&settings.home(), &settings.agents);
    let (projects, loose) = matcher::assign(roots, read.sessions);
    Snapshot { projects, loose, skipped: read.skipped, scanned_ms: now_ms() }
}

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

pub(crate) fn mtime_ms(path: &std::path::Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as i64)
}

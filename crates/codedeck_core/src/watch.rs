//! FSEvents on scan roots and agent folders. The app debounces and rescans.

use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;

/// Keep the returned watcher alive; dropping it stops watching.
pub fn watch(
    paths: Vec<PathBuf>,
    on_change: impl Fn() + Send + 'static,
) -> anyhow::Result<notify::RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        // Our own `git status` touches .git; build output churns. Neither is news.
        let Ok(event) = event else { return };
        let noise = |p: &PathBuf| {
            let p = p.to_string_lossy();
            p.contains("/.git/") || p.contains("/node_modules/") || p.contains("/target/")
        };
        if !event.kind.is_access() && !event.paths.iter().all(noise) {
            on_change();
        }
    })?;
    for path in paths.iter().filter(|p| p.exists()) {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }
    Ok(watcher)
}

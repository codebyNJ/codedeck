use repobox_core::{Settings, Snapshot, index, watch};
use repobox_ui::prelude::*;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Most a watcher burst can make us rescan. Unchanged session files are cached,
/// so a rescan mostly costs a walk, a stat per file and one `git status` per project.
const MIN_RESCAN_GAP: Duration = Duration::from_secs(10);

/// Everything the screens show, and the only bridge to repobox_core.
pub struct AppState {
    pub settings: Settings,
    pub snapshot: Snapshot,
    pub scanning: bool,
    last_scan: Option<Instant>,
    changed: Arc<AtomicBool>,
    _watcher: Option<notify::RecommendedWatcher>,
    _poll: Task<()>,
    scan_task: Option<Task<anyhow::Result<()>>>,
}

impl AppState {
    pub fn new(settings: Settings, cx: &mut Context<Self>) -> Self {
        let changed = Arc::new(AtomicBool::new(false));
        let flag = changed.clone();
        let watcher = watch::watch(settings.watch_paths(), move || flag.store(true, Ordering::Relaxed))
            .inspect_err(|error| log::warn!("watcher: {error:#}"))
            .ok();
        // FSEvents arrive on another thread; poll the flag here and rescan on the UI side.
        let poll = cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(3)).await;
            let alive = this.update(cx, |state, cx| {
                let due = state.last_scan.is_none_or(|at| at.elapsed() >= MIN_RESCAN_GAP);
                if due && state.changed.swap(false, Ordering::Relaxed) {
                    state.rescan(cx);
                }
            });
            if alive.is_err() {
                break;
            }
        });
        let mut state = Self {
            settings,
            // Render the last scan instantly; the fresh one replaces it.
            snapshot: index::load().unwrap_or_default(),
            scanning: false,
            last_scan: None,
            changed,
            _watcher: watcher,
            _poll: poll,
            scan_task: None,
        };
        state.rescan(cx);
        state
    }

    pub fn rescan(&mut self, cx: &mut Context<Self>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.last_scan = Some(Instant::now());
        cx.notify();
        let settings = self.settings.clone();
        self.scan_task = Some(cx.spawn(async move |this, cx| {
            let snapshot = cx
                .background_spawn(async move {
                    let snapshot = repobox_core::scan_all(&settings);
                    if let Err(error) = index::save(&snapshot) {
                        log::warn!("cache: {error:#}");
                    }
                    snapshot
                })
                .await;
            let names: Vec<String> = snapshot.projects.iter().map(|p| p.name.clone()).collect();
            this.update(cx, |state, cx| {
                state.snapshot = snapshot;
                state.scanning = false;
                cx.notify();
            })?;
            // Avatars after the cards are up; cards draw a bot until each one lands.
            cx.background_spawn(async move { repobox_core::avatar::fetch_missing(names.iter().map(String::as_str)) }).await;
            this.update(cx, |_, cx| cx.notify())
        }));
    }

    /// Change settings, save them, rescan if what we scan changed.
    pub fn update_settings(&mut self, cx: &mut Context<Self>, change: impl FnOnce(&mut Settings)) {
        let before = (self.settings.roots.clone(), self.settings.agents.clone());
        change(&mut self.settings);
        if let Err(error) = self.settings.save() {
            log::warn!("settings: {error:#}");
        }
        if before != (self.settings.roots.clone(), self.settings.agents.clone()) {
            self.rescan(cx);
        }
        cx.notify();
    }
}

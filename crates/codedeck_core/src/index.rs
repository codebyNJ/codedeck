//! Last snapshot on disk, so launch can render before the rescan finishes.

use crate::{Snapshot, settings::support_dir};
use std::path::PathBuf;

fn path() -> PathBuf {
    support_dir().join("cache.json")
}

pub fn load() -> Option<Snapshot> {
    let bytes = std::fs::read(path()).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn save(snapshot: &Snapshot) -> anyhow::Result<()> {
    std::fs::create_dir_all(support_dir())?;
    let tmp = path().with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(snapshot)?)?;
    std::fs::rename(tmp, path())?; // atomic: never a half-written cache
    Ok(())
}

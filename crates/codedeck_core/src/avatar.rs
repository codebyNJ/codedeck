//! Project avatars from DiceBear (bottts-neutral), fetched once and cached on disk.
//! The project name is the seed, so it is sent to api.dicebear.com.

use crate::settings::support_dir;
use std::{path::PathBuf, process::Command};

pub fn path(name: &str) -> PathBuf {
    support_dir().join("avatars").join(format!("{}.svg", encode(name)))
}

/// Downloads any missing avatars. Uses curl: every Mac has it, and it keeps an
/// HTTP stack out of the binary. Failures are fine; the UI draws a fallback bot.
pub fn fetch_missing<'a>(names: impl IntoIterator<Item = &'a str>) {
    let dir = support_dir().join("avatars");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    for name in names {
        let target = path(name);
        if target.exists() {
            continue;
        }
        let url = format!("https://api.dicebear.com/10.x/bottts-neutral/svg?seed={}&radius=22", encode(name));
        let ok = Command::new("curl")
            .args(["-sf", "--max-time", "8", "-o"])
            .arg(&target)
            .arg(&url)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            let _ = std::fs::remove_file(&target); // no half-written files; missing is expected
        }
    }
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| if b.is_ascii_alphanumeric() || b"-_.".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect()
}

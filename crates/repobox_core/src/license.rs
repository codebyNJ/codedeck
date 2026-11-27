//! One-time license. Activation and signed-token checks land with the store
//! integration (Polar / Lemon Squeezy).
// ponytail: format check only, no activation or signature; add both before selling.

use crate::settings::support_dir;

pub fn load() -> Option<String> {
    std::fs::read_to_string(support_dir().join("license")).ok().map(|k| k.trim().to_string())
}

pub fn save(key: &str) -> anyhow::Result<()> {
    anyhow::ensure!(looks_valid(key), "that doesn't look like a Codedeck key");
    std::fs::create_dir_all(support_dir())?;
    std::fs::write(support_dir().join("license"), key.trim())?;
    Ok(())
}

pub fn looks_valid(key: &str) -> bool {
    let key = key.trim();
    key.len() >= 16 && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// "CDK-7F3A-••••-••••-91B2" for display.
pub fn masked(key: &str) -> String {
    key.chars()
        .enumerate()
        .map(|(i, c)| if c != '-' && i > 8 && i + 4 < key.len() { '•' } else { c })
        .collect()
}

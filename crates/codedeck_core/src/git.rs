use std::{path::Path, process::Command};

/// Branch name from .git/HEAD; detached heads show the short hash.
pub fn branch(root: &Path) -> Option<String> {
    let head = std::fs::read_to_string(root.join(".git/HEAD")).ok()?;
    let head = head.trim();
    Some(match head.strip_prefix("ref: refs/heads/") {
        Some(name) => name.to_string(),
        None => head.chars().take(7).collect(),
    })
}

/// Number of changed + untracked files. Uses the git CLI: reimplementing the
/// index format isn't worth it, and every developer Mac has git.
pub fn dirty_count(root: &Path) -> u32 {
    if !root.join(".git").exists() {
        return 0;
    }
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map(|out| out.stdout.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count() as u32)
        .unwrap_or(0)
}

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
        // No index refresh: otherwise every scan rewrites .git/index and every
        // project looks like it was just touched.
        .args(["--no-optional-locks", "status", "--porcelain"])
        .current_dir(root)
        .output()
        .map(|out| out.stdout.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count() as u32)
        .unwrap_or(0)
}

/// Days shown in the project page's commit grid.
pub const HISTORY_DAYS: usize = 42;

pub struct History {
    pub last_subject: String,
    pub last_ms: i64,
    /// Commits per day, oldest first; the last entry is today.
    pub per_day: Vec<u32>,
}

/// Last commit plus a per-day commit count. Two `git log` calls; run on demand, not per scan.
pub fn history(root: &Path) -> Option<History> {
    let log = |args: &[&str]| -> Option<String> {
        let out = Command::new("git").arg("--no-optional-locks").arg("log").args(args).current_dir(root).output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let last = log(&["-1", "--format=%ct%x00%s"])?;
    let (stamp, subject) = last.trim_end().split_once('\0')?;
    let since = format!("--since={HISTORY_DAYS}.days");
    let stamps = log(&[&since, "--format=%ct"]).unwrap_or_default();
    Some(History {
        last_subject: subject.to_string(),
        last_ms: stamp.parse::<i64>().ok()? * 1000,
        per_day: per_day(stamps.lines().filter_map(|l| l.parse::<i64>().ok()), crate::now_ms() / 1000),
    })
}

/// Buckets unix seconds into HISTORY_DAYS rolling 24h windows ending now.
fn per_day(stamps: impl Iterator<Item = i64>, now_s: i64) -> Vec<u32> {
    let mut days = vec![0; HISTORY_DAYS];
    for s in stamps {
        let back = ((now_s - s).max(0) / 86_400) as usize;
        if back < HISTORY_DAYS {
            days[HISTORY_DAYS - 1 - back] += 1;
        }
    }
    days
}

#[cfg(test)]
mod tests {
    #[test]
    fn buckets_by_day() {
        let now = 100 * 86_400;
        let days = super::per_day([now, now - 10, now - 86_400 - 5, now - 50 * 86_400].into_iter(), now);
        assert_eq!(days[super::HISTORY_DAYS - 1], 2);
        assert_eq!(days[super::HISTORY_DAYS - 2], 1);
        assert_eq!(days.iter().sum::<u32>(), 3);
    }
}

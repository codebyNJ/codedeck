use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Agent {
    Claude,
    OpenCode,
    Codex,
}

impl Agent {
    pub fn label(self) -> &'static str {
        match self {
            Agent::Claude => "Claude Code",
            Agent::OpenCode => "OpenCode",
            Agent::Codex => "Codex",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub agent: Agent,
    pub id: String,
    pub cwd: PathBuf,
    pub title: String,
    pub updated_ms: i64,
    /// Prompts the user typed (not tool results, not streamed chunks).
    pub prompts: u32,
}

/// A session touched within this window counts as running.
pub const LIVE_MS: i64 = 2 * 60 * 1000;

impl Session {
    pub fn is_live(&self, now_ms: i64) -> bool {
        now_ms - self.updated_ms < LIVE_MS
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub root: PathBuf,
    pub name: String,
    /// simple-icons slugs, e.g. "rust", "typescript", "nextdotjs".
    pub stacks: Vec<String>,
    pub branch: Option<String>,
    pub dirty: u32,
    /// Newest first.
    pub sessions: Vec<Session>,
    /// Latest of: session activity, project folder mtime.
    pub updated_ms: i64,
}

impl Project {
    pub fn is_live(&self, now_ms: i64) -> bool {
        self.sessions.first().is_some_and(|s| s.is_live(now_ms))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    /// Most recently active first.
    pub projects: Vec<Project>,
    /// Sessions run outside any project (from ~, a scan root, /tmp).
    pub loose: Vec<Session>,
    /// Sessions that couldn't be read.
    pub skipped: usize,
    pub scanned_ms: i64,
}

impl Snapshot {
    pub fn session_count(&self) -> usize {
        self.projects.iter().map(|p| p.sessions.len()).sum()
    }
}

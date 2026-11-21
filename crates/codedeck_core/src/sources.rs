pub mod claude;
pub mod codex;
pub mod opencode;

use crate::{Agent, Session};
use std::path::Path;

#[derive(Default)]
pub struct Read {
    pub sessions: Vec<Session>,
    pub skipped: usize,
}

impl Read {
    fn extend(&mut self, other: anyhow::Result<Read>, agent: Agent) {
        match other {
            Ok(read) => {
                self.sessions.extend(read.sessions);
                self.skipped += read.skipped;
            }
            // A missing or unreadable agent store never fails the scan.
            Err(error) => log::info!("{}: {error:#}", agent.label()),
        }
    }
}

/// Reads every enabled agent. Formats are undocumented, so each reader skips
/// what it can't parse and counts it instead of failing.
pub fn read_all(home: &Path, agents: &[Agent]) -> Read {
    let mut all = Read::default();
    for agent in agents {
        let read = match agent {
            Agent::Claude => claude::read(home),
            Agent::OpenCode => opencode::read(home),
            Agent::Codex => codex::read(home),
        };
        all.extend(read, *agent);
    }
    all
}

/// First line of a prompt, trimmed to a card-sized title.
pub(crate) fn short_title(text: &str) -> String {
    let line = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let mut title: String = line.chars().take(80).collect();
    if line.chars().count() > 80 {
        title.push('…');
    }
    title
}

use crate::Agent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)] // old files keep loading as fields are added
pub struct Settings {
    pub roots: Vec<PathBuf>,
    pub depth: usize,
    pub agents: Vec<Agent>,
    pub editor: String,
    pub terminal: String,
    pub theme: Theme,
    pub onboarded: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

impl Default for Settings {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let roots = ["codes", "code", "Developer", "Projects", "src", "dev"]
            .iter()
            .map(|d| home.join(d))
            .filter(|p| p.is_dir())
            .collect();
        Self {
            roots,
            depth: 4,
            agents: vec![Agent::Claude, Agent::OpenCode, Agent::Codex],
            editor: "Cursor".into(),
            terminal: "Terminal".into(),
            theme: Theme::Light,
            onboarded: false,
        }
    }
}

impl Settings {
    pub fn home(&self) -> PathBuf {
        dirs::home_dir().unwrap_or_default()
    }

    pub fn load() -> Self {
        std::fs::read(path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(support_dir())?;
        std::fs::write(path(), serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// Folders whose changes should trigger a rescan.
    pub fn watch_paths(&self) -> Vec<PathBuf> {
        let home = self.home();
        let mut paths = self.roots.clone();
        paths.push(home.join(".claude/projects"));
        paths.push(home.join(".local/share/opencode"));
        paths.push(home.join(".codex/sessions"));
        paths
    }
}

pub fn support_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_default().join("Codedeck")
}

fn path() -> PathBuf {
    support_dir().join("settings.json")
}

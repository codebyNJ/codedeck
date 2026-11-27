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
    pub layout: Layout,
    pub painting: Painting,
    pub onboarded: bool,
}

/// How Home lays out the project list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    /// Two columns of full cards.
    #[default]
    Cards,
    /// Three columns of small tiles.
    Compact,
    /// One row per project.
    List,
}

/// Which public-domain painting tops Home. Auto follows the time of day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Painting {
    #[default]
    Auto,
    Monet,
    Seurat,
    Inness,
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
            layout: Layout::Cards,
            painting: Painting::Auto,
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
    let data = dirs::data_dir().unwrap_or_default();
    let dir = data.join("Repobox");
    // ponytail: one-time move from the old name; drop once no Codedeck installs remain.
    let old = data.join("Codedeck");
    if !dir.exists() && old.exists() {
        let _ = std::fs::rename(&old, &dir);
    }
    dir
}

fn path() -> PathBuf {
    support_dir().join("settings.json")
}

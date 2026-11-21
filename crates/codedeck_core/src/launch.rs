//! Opening projects in editors and terminals. macOS `open` does the work.

use std::{path::Path, process::Command};

/// (Display name, .app name in /Applications, asset icon name)
pub const EDITORS: &[(&str, &str, &str)] = &[
    ("Cursor", "Cursor", "cursor"),
    ("VS Code", "Visual Studio Code", "vscode"),
    ("Zed", "Zed", "zed"),
];

pub const TERMINALS: &[&str] = &["Terminal", "iTerm", "Ghostty", "Warp"];

pub fn installed(app: &str) -> bool {
    Path::new("/Applications").join(format!("{app}.app")).exists()
}

pub fn open_in(app: &str, path: &Path) -> anyhow::Result<()> {
    let status = Command::new("open").args(["-a", app]).arg(path).status()?;
    anyhow::ensure!(status.success(), "couldn't open {app}");
    Ok(())
}

/// Editor display name → .app name.
pub fn open_in_editor(editor: &str, path: &Path) -> anyhow::Result<()> {
    let app = EDITORS.iter().find(|(name, ..)| *name == editor).map_or(editor, |(_, app, _)| *app);
    open_in(app, path)
}

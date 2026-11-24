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

/// The shell command that continues `session` in its agent's CLI.
pub fn resume_command(session: &crate::Session) -> String {
    let id = quote(&session.id);
    match session.agent {
        crate::Agent::Claude => format!("claude --resume {id}"),
        crate::Agent::OpenCode => format!("opencode --session {id}"),
        crate::Agent::Codex => format!("codex resume {id}"),
    }
}

/// Reopen an agent session in `terminal`: write a `.command` script and `open` it.
// ponytail: relies on the terminal running .command files (Terminal, iTerm do); per-terminal AppleScript if Ghostty/Warp need it.
pub fn resume(terminal: &str, session: &crate::Session) -> anyhow::Result<()> {
    let command = resume_command(session);
    let name: String = session.id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    let script = std::env::temp_dir().join(format!("codedeck-resume-{name}.command"));
    std::fs::write(&script, format!("#!/bin/zsh -l\ncd {} && exec {command}\n", quote(&session.cwd.to_string_lossy())))?;
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))?;
    open_in(terminal, &script)
}

/// Single-quote for sh.
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn quotes() {
        assert_eq!(super::quote("a'b c"), r"'a'\''b c'");
    }
}

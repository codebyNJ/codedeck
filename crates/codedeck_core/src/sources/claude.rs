//! Claude Code: ~/.claude/projects/<encoded-cwd>/<session-id>.jsonl

use super::{Read, short_title};
use crate::{Agent, Session, mtime_ms};
use anyhow::Context as _;
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::Mutex,
};

pub fn read(home: &Path) -> anyhow::Result<Read> {
    let projects = home.join(".claude/projects");
    let mut read = Read::default();
    for folder in std::fs::read_dir(&projects).context("no ~/.claude/projects")?.flatten() {
        let Ok(files) = std::fs::read_dir(folder.path()) else { continue };
        for file in files.flatten() {
            let path = file.path();
            if path.extension().is_none_or(|e| e != "jsonl") {
                continue;
            }
            match read_cached(&path) {
                Ok(Some(session)) => read.sessions.push(session),
                Ok(None) => {}
                Err(error) => {
                    log::warn!("claude {}: {error:#}", path.display());
                    read.skipped += 1;
                }
            }
        }
    }
    Ok(read)
}

/// Session files only grow, so an unchanged mtime means an unchanged session.
/// Rescans then cost one stat per file instead of a full parse.
fn read_cached(path: &Path) -> anyhow::Result<Option<Session>> {
    type Seen = HashMap<PathBuf, (i64, Option<Session>)>;
    static CACHE: Mutex<Option<Seen>> = Mutex::new(None);
    let mtime = mtime_ms(path);
    if let Ok(cache) = CACHE.lock()
        && let Some((seen, session)) = cache.as_ref().and_then(|c| c.get(path))
        && *seen == mtime
    {
        return Ok(session.clone());
    }
    let session = read_session(path)?;
    if let Ok(mut cache) = CACHE.lock() {
        cache.get_or_insert_default().insert(path.to_path_buf(), (mtime, session.clone()));
    }
    Ok(session)
}

/// One pass over the file. The cwd comes from inside the lines, because the
/// folder name's encoding (`/` and `.` → `-`) can't be reversed.
pub fn read_session(path: &Path) -> anyhow::Result<Option<Session>> {
    let file = std::fs::File::open(path)?;
    let mut cwd = None;
    let mut ai_title = None;
    let mut first_prompt = None;
    let mut prompts = 0u32;
    for line in BufReader::new(file).lines() {
        let line = line?;
        // Cheap prefilter: most lines are large tool payloads we don't need.
        if !(line.contains("\"cwd\"") || line.contains("ai-title") || line.contains("\"type\":\"user\"")) {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
        if cwd.is_none() {
            cwd = value["cwd"].as_str().map(Into::into);
        }
        match value["type"].as_str() {
            Some("ai-title") => ai_title = value["aiTitle"].as_str().map(short_title),
            Some("user") if value["isMeta"].as_bool() != Some(true) => {
                // Tool results also arrive as "user" lines; only typed text counts.
                if let Some(text) = prompt_text(&value["message"]["content"]) {
                    prompts += 1;
                    if first_prompt.is_none() {
                        first_prompt = Some(short_title(&text));
                    }
                }
            }
            _ => {}
        }
    }
    let Some(cwd) = cwd else { return Ok(None) };
    if prompts == 0 {
        return Ok(None);
    }
    let id = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(Some(Session {
        agent: Agent::Claude,
        id,
        cwd,
        title: ai_title.or(first_prompt).unwrap_or_else(|| "Untitled session".into()),
        updated_ms: mtime_ms(path),
        prompts,
    }))
}

fn prompt_text(content: &Value) -> Option<String> {
    let text = match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts.iter().find_map(|p| p["text"].as_str())?.to_string(),
        _ => return None,
    };
    // Skip harness-injected prompts like <command-name> or <local-command-stdout>.
    (!text.trim_start().starts_with('<')).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_fixture() -> anyhow::Result<()> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/claude.jsonl");
        let session = read_session(&path)?.context("expected a session")?;
        assert_eq!(session.cwd, Path::new("/Users/test/codes/blog"));
        assert_eq!(session.title, "Blog redesign");
        assert_eq!(session.prompts, 2);
        Ok(())
    }
}

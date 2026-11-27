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
    let mut last_prompt = String::new();
    let mut prompts = 0u32;
    let mut last_activity = None;
    for line in BufReader::new(file).lines() {
        let line = line?;
        // Claude Code also appends bookkeeping lines (away summaries, turn timings)
        // to old sessions, so mtime lies; only real messages count as activity.
        if line.contains("\"type\":\"user\"") || line.contains("\"type\":\"assistant\"") {
            last_activity = timestamp_ms(&line).or(last_activity);
        }
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
                    last_prompt = short_title(&text);
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
        updated_ms: last_activity.unwrap_or_else(|| mtime_ms(path)),
        prompts,
        last_prompt,
    }))
}

/// The line's `"timestamp":"…"` value, without parsing the (often huge) line.
fn timestamp_ms(line: &str) -> Option<i64> {
    const KEY: &str = "\"timestamp\":\"";
    let start = line.rfind(KEY)? + KEY.len();
    let end = start + line[start..].find('"')?;
    line[start..end].parse::<jiff::Timestamp>().ok().map(|t| t.as_millisecond())
}

fn prompt_text(content: &Value) -> Option<String> {
    let text = match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts.iter().find_map(|p| p["text"].as_str())?.to_string(),
        _ => return None,
    };
    // Skip harness-injected prompts: <command-name>, <local-command-stdout>,
    // and the summary Claude Code writes when it continues a compacted session.
    let lead = text.trim_start();
    (!lead.starts_with('<') && !lead.starts_with("[Request interrupted") && !lead.starts_with("This session is being continued from a previous conversation")).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::{prompt_text, timestamp_ms};

    #[test]
    fn reads_timestamps() {
        let line = r#"{"type":"assistant","message":{},"timestamp":"2026-11-10T14:08:50.764Z","uuid":"x"}"#;
        assert_eq!(timestamp_ms(line), Some(1_794_319_730_764));
        assert_eq!(timestamp_ms(r#"{"type":"user"}"#), None);
    }
    use serde_json::json;

    #[test]
    fn skips_injected_prompts() {
        assert_eq!(prompt_text(&json!("fix the build")).as_deref(), Some("fix the build"));
        assert_eq!(prompt_text(&json!([{ "type": "text", "text": "hi" }])).as_deref(), Some("hi"));
        assert!(prompt_text(&json!("[Request interrupted by user]")).is_none());
        assert!(prompt_text(&json!("<command-name>/clear</command-name>")).is_none());
        assert!(prompt_text(&json!("This session is being continued from a previous conversation that ran out of context.")).is_none());
    }
}

//! Codex CLI: ~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl
//! Not verified against a real install yet; parsed defensively.

use super::{Read, short_title};
use crate::{Agent, Session, mtime_ms};
use anyhow::Context as _;
use serde_json::Value;
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub fn read(home: &Path) -> anyhow::Result<Read> {
    let root = home.join(".codex/sessions");
    anyhow::ensure!(root.exists(), "no ~/.codex/sessions");
    let mut files = Vec::new();
    collect(&root, &mut files);
    let mut read = Read::default();
    for path in files {
        match read_session(&path) {
            Ok(Some(session)) => read.sessions.push(session),
            Ok(None) => {}
            Err(_) => read.skipped += 1,
        }
    }
    Ok(read)
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|e| e == "jsonl") {
            files.push(path);
        }
    }
}

fn read_session(path: &Path) -> anyhow::Result<Option<Session>> {
    let mut lines = BufReader::new(std::fs::File::open(path)?).lines();
    let first: Value = serde_json::from_str(&lines.next().context("empty")??)?;
    let meta = if first["payload"].is_object() { &first["payload"] } else { &first };
    let Some(cwd) = meta["cwd"].as_str() else { return Ok(None) };
    let id = meta["id"].as_str().map(String::from).unwrap_or_else(|| {
        path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    });
    let mut title = None;
    let mut prompts = 0u32;
    let mut last_prompt = String::new();
    for line in lines.map_while(Result::ok) {
        let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
        let item = if value["payload"].is_object() { &value["payload"] } else { &value };
        if item["type"] == "message" && item["role"] == "user" {
            prompts += 1;
            let text = item["content"][0]["text"].as_str().filter(|t| !t.starts_with('<')).map(short_title);
            if title.is_none() {
                title = text.clone();
            }
            if let Some(text) = text {
                last_prompt = text;
            }
        }
    }
    Ok(Some(Session {
        agent: Agent::Codex,
        id,
        cwd: cwd.into(),
        title: title.unwrap_or_else(|| "Untitled session".into()),
        updated_ms: mtime_ms(path),
        prompts,
        last_prompt,
    }))
}

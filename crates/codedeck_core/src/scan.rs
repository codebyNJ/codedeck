use std::path::{Path, PathBuf};

/// Folders never worth descending into.
pub const SKIP: &[&str] = &["node_modules", "target", "dist", "build", "vendor", "Library", "Applications"];

/// Every folder under `roots` that looks like a project. Stops descending at the
/// first project, so nested packages of a monorepo stay one project.
pub fn find_projects(roots: &[PathBuf], depth: usize) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in roots {
        walk(root, depth, &mut found);
    }
    found.sort();
    found.dedup();
    found
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    if is_project(dir) {
        found.push(dir.to_path_buf());
        return;
    }
    if depth == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || SKIP.contains(&name.as_ref()) {
            continue;
        }
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            walk(&entry.path(), depth - 1, found);
        }
    }
}

pub fn is_project(dir: &Path) -> bool {
    dir.join(".git").exists() || crate::stack::MARKERS.iter().any(|(file, _)| dir.join(file).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_at_first_project() -> anyhow::Result<()> {
        let tmp = std::env::temp_dir().join(format!("codedeck-scan-{}", std::process::id()));
        std::fs::create_dir_all(tmp.join("a/packages/inner"))?;
        std::fs::write(tmp.join("a/Cargo.toml"), "")?;
        std::fs::write(tmp.join("a/packages/inner/package.json"), "{}")?;
        std::fs::create_dir_all(tmp.join("notes"))?;
        let found = find_projects(std::slice::from_ref(&tmp), 4);
        std::fs::remove_dir_all(&tmp)?;
        assert_eq!(found, vec![tmp.join("a")]);
        Ok(())
    }
}

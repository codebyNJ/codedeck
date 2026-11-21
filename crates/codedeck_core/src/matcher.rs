use crate::{Project, Session, git, mtime_ms, stack};
use std::path::PathBuf;

/// Builds projects from scanned roots and gives each session to the project
/// whose root is the longest prefix of its cwd. A session outside every root
/// becomes its own project only if its cwd is a real project folder; the rest
/// (sessions run from ~, ~/codes, /tmp) come back as loose.
pub fn assign(roots: Vec<PathBuf>, mut sessions: Vec<Session>) -> (Vec<Project>, Vec<Session>) {
    let mut roots = roots;
    for session in &sessions {
        if owner(&roots, &session.cwd).is_none() && is_user_project(&session.cwd) {
            roots.push(session.cwd.clone());
        }
    }
    roots.sort();
    roots.dedup();

    let mut projects: Vec<Project> = roots.into_iter().map(project).collect();
    let roots: Vec<PathBuf> = projects.iter().map(|p| p.root.clone()).collect();
    let mut loose = Vec::new();
    sessions.sort_by_key(|s| std::cmp::Reverse(s.updated_ms));
    for session in sessions {
        match owner(&roots, &session.cwd) {
            Some(index) => projects[index].sessions.push(session),
            None => loose.push(session),
        }
    }
    for project in &mut projects {
        if let Some(latest) = project.sessions.first() {
            project.updated_ms = project.updated_ms.max(latest.updated_ms);
        }
    }
    projects.sort_by_key(|p| std::cmp::Reverse(p.updated_ms));
    (projects, loose)
}

/// App data folders can hold a .git or package.json too; those aren't the user's code.
fn is_user_project(cwd: &std::path::Path) -> bool {
    let text = cwd.to_string_lossy();
    crate::scan::is_project(cwd) && !text.contains("/Library/") && !text.contains("/.")
}

fn owner(roots: &[PathBuf], cwd: &std::path::Path) -> Option<usize> {
    roots
        .iter()
        .enumerate()
        .filter(|(_, root)| cwd.starts_with(root))
        .max_by_key(|(_, root)| root.as_os_str().len())
        .map(|(index, _)| index)
}

fn project(root: PathBuf) -> Project {
    Project {
        name: root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        stacks: stack::detect(&root),
        branch: git::branch(&root),
        dirty: git::dirty_count(&root),
        sessions: Vec::new(),
        updated_ms: mtime_ms(&root.join(".git/index")).max(mtime_ms(&root)),
        root,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_prefix_wins() {
        let roots = vec![PathBuf::from("/c/app"), PathBuf::from("/c/app/web")];
        assert_eq!(owner(&roots, "/c/app/web/src".as_ref()), Some(1));
        assert_eq!(owner(&roots, "/c/app/api".as_ref()), Some(0));
        assert_eq!(owner(&roots, "/c/application".as_ref()), None); // not a path prefix
    }
}

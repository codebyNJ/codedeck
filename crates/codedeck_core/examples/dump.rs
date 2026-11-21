//! cargo run -p codedeck_core --example dump — scans with your real settings.

fn main() {
    let settings = codedeck_core::Settings::load();
    let started = std::time::Instant::now();
    let snapshot = codedeck_core::scan_all(&settings);
    let now = codedeck_core::now_ms();
    println!("roots: {:?}", settings.roots);
    println!(
        "{} projects · {} sessions · {} loose · {} skipped · {:?}\n",
        snapshot.projects.len(),
        snapshot.session_count(),
        snapshot.loose.len(),
        snapshot.skipped,
        started.elapsed()
    );
    for project in snapshot.projects.iter().take(25) {
        println!(
            "{:<28} {:<14} dirty {:<3} {:>3} sessions {}  [{}]",
            project.name,
            project.branch.as_deref().unwrap_or("-"),
            project.dirty,
            project.sessions.len(),
            if project.is_live(now) { "● running" } else { "" },
            project.stacks.join(", ")
        );
        if let Some(session) = project.sessions.first() {
            println!("    └ {} · {}", session.agent.label(), session.title);
        }
    }
}

use std::path::Path;

/// Marker file → simple-icons slug. Order is display order.
pub const MARKERS: &[(&str, &str)] = &[
    ("Cargo.toml", "rust"),
    ("go.mod", "go"),
    ("pyproject.toml", "python"),
    ("requirements.txt", "python"),
    ("Gemfile", "ruby"),
    ("pubspec.yaml", "flutter"),
    ("Package.swift", "swift"),
    ("pom.xml", "openjdk"),
    ("build.gradle", "gradle"),
    ("build.gradle.kts", "kotlin"),
    ("composer.json", "php"),
    ("mix.exs", "elixir"),
    ("deno.json", "deno"),
    ("package.json", "javascript"),
];

/// Files that refine a JS project.
const REFINE: &[(&str, &str)] = &[
    ("tsconfig.json", "typescript"),
    ("next.config.js", "nextdotjs"),
    ("next.config.mjs", "nextdotjs"),
    ("next.config.ts", "nextdotjs"),
    ("vite.config.ts", "vite"),
    ("vite.config.js", "vite"),
    ("svelte.config.js", "svelte"),
    ("astro.config.mjs", "astro"),
    ("tailwind.config.js", "tailwindcss"),
    ("tailwind.config.ts", "tailwindcss"),
];

pub fn detect(root: &Path) -> Vec<String> {
    let mut stacks: Vec<String> = Vec::new();
    fn push(stacks: &mut Vec<String>, slug: &str) {
        if !stacks.iter().any(|s| s == slug) {
            stacks.push(slug.to_string());
        }
    }
    for (file, slug) in MARKERS {
        if root.join(file).exists() {
            push(&mut stacks, slug);
        }
    }
    if root.join("package.json").exists() {
        for (file, slug) in REFINE {
            if root.join(file).exists() {
                push(&mut stacks, slug);
            }
        }
        // TypeScript replaces plain JavaScript; it's the more useful badge.
        if stacks.iter().any(|s| s == "typescript") {
            stacks.retain(|s| s != "javascript");
        }
    }
    if has_xcodeproj(root) {
        push(&mut stacks, "swift");
    }
    stacks
}

fn has_xcodeproj(root: &Path) -> bool {
    std::fs::read_dir(root).is_ok_and(|entries| {
        entries.flatten().any(|e| e.file_name().to_string_lossy().ends_with(".xcodeproj"))
    })
}

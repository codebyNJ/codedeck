mod home;
mod project;
mod settings;

use crate::shell::Shell;
use codedeck_core::launch;
use codedeck_ui::prelude::*;
use std::path::Path;

/// "now", "4m", "2h", "3d", "5w" — the prototype's short ages.
pub fn ago(ms: i64) -> String {
    let minutes = (codedeck_core::now_ms() - ms).max(0) / 60_000;
    match minutes {
        0 => "now".into(),
        1..=59 => format!("{minutes}m"),
        60..=1439 => format!("{}h", minutes / 60),
        1440..=10079 => format!("{}d", minutes / 1440),
        _ => format!("{}w", minutes / 10080),
    }
}

pub fn tilde(path: &Path) -> String {
    match dirs_home().and_then(|home| path.strip_prefix(home).ok().map(|rest| rest.to_path_buf())) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(Into::into)
}

/// Editors that are actually installed: (display name, icon stem).
pub fn installed_editors() -> Vec<(&'static str, &'static str)> {
    launch::EDITORS.iter().filter(|(_, app, _)| launch::installed(app)).map(|(name, _, icon)| (*name, *icon)).collect()
}

/// The chosen editor if installed, else the first installed one.
pub fn default_editor(chosen: &str) -> String {
    let installed = installed_editors();
    if installed.iter().any(|(name, _)| *name == chosen) {
        chosen.to_string()
    } else {
        installed.first().map_or(chosen.to_string(), |(name, _)| name.to_string())
    }
}

pub fn stack_label(slug: &str) -> &str {
    match slug {
        "rust" => "Rust",
        "typescript" => "TypeScript",
        "javascript" => "JavaScript",
        "go" => "Go",
        "python" => "Python",
        "swift" => "Swift",
        "flutter" => "Flutter",
        "nextdotjs" => "Next.js",
        "ruby" => "Ruby",
        "kotlin" => "Kotlin",
        "vite" => "Vite",
        "tailwindcss" => "Tailwind",
        "svelte" => "Svelte",
        "astro" => "Astro",
        other => other,
    }
}

impl Shell {
    /// The "Open with" row: one jelly circle per installed editor.
    pub fn editor_buttons(&self, root: &Path, size: f32, cx: &mut Context<Self>) -> AnyElement {
        let default = default_editor(&self.state.read(cx).settings.editor);
        let theme = cx.theme().clone();
        h_flex()
            .gap(px(4.))
            .child(div().mr(px(4.)).text_size(px(11.5)).text_color(theme.ink_3).child("Open with"))
            .children(installed_editors().into_iter().map(|(name, icon)| {
                let root = root.to_path_buf();
                EditorButton::new(SharedString::from(format!("ed-{name}-{}", root.display())), icon)
                    .size(size)
                    .default_editor(name == default)
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        if let Err(error) = launch::open_in_editor(name, &root) {
                            log::warn!("{error:#}");
                        }
                    })
            }))
            .into_any_element()
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        Theme::toggle(cx);
        let dark = cx.theme().dark;
        self.state.update(cx, |state, cx| {
            state.update_settings(cx, |s| {
                s.theme = if dark { codedeck_core::settings::Theme::Dark } else { codedeck_core::settings::Theme::Light }
            })
        });
    }
}

mod home;
pub mod palette;
mod project;
mod sessions;
mod settings;

use crate::shell::{Screen, Shell};
use codedeck_core::{Agent, Session, avatar, launch};
use codedeck_ui::prelude::*;
use std::path::Path;

pub const AGENTS: [Agent; 3] = [Agent::Claude, Agent::OpenCode, Agent::Codex];

/// The agent's own logo, in its brand colour (black ones follow the ink).
pub fn agent_icon(agent: Agent, size: f32, cx: &App) -> Svg {
    let (path, color) = match agent {
        Agent::Claude => ("icons/agent/claude.svg", rgb(0xd97757).into()),
        Agent::OpenCode => ("icons/agent/opencode.svg", cx.theme().ink),
        Agent::Codex => ("icons/agent/codex.svg", cx.theme().ink),
    };
    svg().path(path).size(px(size)).flex_shrink_0().text_color(color)
}

/// The prototype's `.gl` tile: the agent logo on a soft square, a green dot when live.
pub fn agent_mark(agent: Agent, live: bool, cx: &App) -> Div {
    let theme = cx.theme();
    div()
        .relative()
        .size(px(34.))
        .flex_shrink_0()
        .rounded(px(11.))
        .bg(theme.soft)
        .flex()
        .items_center()
        .justify_center()
        .child(agent_icon(agent, 16., cx))
        .when(live, |d| {
            d.child(div().absolute().right(px(-2.)).top(px(-2.)).size(px(10.)).rounded_full().bg(theme.live).border_2().border_color(theme.bg))
        })
}

/// Today / Yesterday / This week / Earlier, by local calendar day.
pub fn day_group(updated_ms: i64) -> &'static str {
    let today = jiff::ZonedDateTime::now().date();
    let Ok(stamp) = jiff::Timestamp::from_millisecond(updated_ms) else { return "Earlier" };
    let days = (today - stamp.to_zoned_datetime(jiff::tz::TimeZone::system()).date()).get_days();
    match days {
        ..=0 => "Today",
        1 => "Yesterday",
        2..=6 => "This week",
        _ => "Earlier",
    }
}

/// Sessions (already newest first) split into consecutive day groups.
pub fn by_day<T>(items: impl IntoIterator<Item = T>, updated: impl Fn(&T) -> i64) -> Vec<(&'static str, Vec<T>)> {
    let mut groups: Vec<(&'static str, Vec<T>)> = Vec::new();
    for item in items {
        let group = day_group(updated(&item));
        match groups.last_mut() {
            Some((name, list)) if *name == group => list.push(item),
            _ => groups.push((group, vec![item])),
        }
    }
    groups
}

pub fn group_label(label: impl Into<SharedString>, cx: &App) -> Div {
    div().pt(px(18.)).pb(px(6.)).font_family(MONO).text_size(px(10.5)).text_color(cx.theme().ink_3).child(label.into())
}

pub fn prompts(n: u32) -> String {
    if n == 1 { "1 prompt".into() } else { format!("{n} prompts") }
}

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
    /// Reopens the session in the user's terminal, in its own folder.
    pub fn resume(&self, session: &Session, cx: &App) {
        let terminal = self.state.read(cx).settings.terminal.clone();
        if let Err(error) = launch::resume(&terminal, session) {
            log::warn!("{error:#}");
        }
    }

    /// The prototype's `.srow`. `project` adds the project pill (Sessions screen).
    pub fn session_row(&self, session: &Session, project: Option<(String, Option<std::path::PathBuf>)>, now: i64, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let live = session.is_live(now);
        let id = SharedString::from(format!("srow-{}", session.id));
        // The prototype's "› last message"; the agent and folder when there's nothing newer than the title.
        let subtitle = if session.last_prompt.is_empty() || session.last_prompt == session.title {
            format!("{} · {}", session.agent.label(), tilde(&session.cwd))
        } else {
            format!("› {}", session.last_prompt)
        };
        let resume = {
            let session = session.clone();
            let button = GelButton::new(SharedString::from(format!("resume-{}", session.id)), "Resume").small();
            let button = if live { button } else { button.paper() };
            let this = cx.entity().downgrade();
            button.on_click(move |_, _, cx| {
                cx.stop_propagation();
                if let Some(this) = this.upgrade() {
                    this.read(cx).resume(&session, cx);
                }
            })
        };
        h_flex()
            .id(id.clone())
            .group(id.clone())
            .gap(px(13.))
            .py(px(12.))
            .px(px(4.))
            .border_t_1()
            .border_color(theme.line)
            .child(agent_mark(session.agent, live, cx))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.))
                    .child(div().text_size(px(14.)).font_weight(FontWeight::MEDIUM).truncate().child(session.title.clone()))
                    .child(div().font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).truncate().child(subtitle)),
            )
            .when_some(project, |row, (name, root)| {
                let pill = h_flex()
                    .id(SharedString::from(format!("sp-{}", session.id)))
                    .max_w(px(180.))
                    .h(px(22.))
                    .px(px(7.))
                    .gap(px(6.))
                    .rounded(px(7.))
                    .bg(theme.soft)
                    .text_size(px(11.5))
                    .text_color(theme.ink_2)
                    .when(root.is_some(), |p| p.child(Avatar::project(name.clone()).image(avatar::path(&name)).size(14.)))
                    .child(div().truncate().child(name));
                row.child(match root {
                    Some(root) => pill
                        .hover(|p| p.bg(theme.soft_2))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.go(Screen::Project(root.clone()), cx)
                        }))
                        .into_any_element(),
                    None => pill.into_any_element(),
                })
            })
            .child(div().w(px(76.)).text_right().font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).child(prompts(session.prompts)))
            .child(div().w(px(36.)).text_right().font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).child(ago(session.updated_ms)))
            // Shown on hover, always on the running row.
            .child(div().when(!live, |d| d.opacity(0.).group_hover(id, |d| d.opacity(1.))).child(resume))
            .into_any_element()
    }

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

    /// Native folder picker; picked folders join the scan roots.
    pub fn add_folders(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: true,
            prompt: Some("Scan".into()),
        });
        let state = self.state.clone();
        cx.spawn(async move |_, cx| {
            let Ok(Ok(Some(paths))) = picked.await else { return };
            state.update(cx, |state, cx| {
                state.update_settings(cx, |s| {
                    for path in paths {
                        if !s.roots.contains(&path) {
                            s.roots.push(path);
                        }
                    }
                })
            });
        })
        .detach();
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

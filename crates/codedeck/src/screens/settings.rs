use super::{AGENTS, ago, agent_icon, installed_editors, tilde};
use crate::shell::Shell;
use codedeck_core::{Agent, index, launch, scan, settings::{Painting, support_dir}};
use codedeck_ui::prelude::*;

const PAINTINGS: [(Painting, &str, &str); 4] = [
    (Painting::Auto, "Auto", ""),
    (Painting::Monet, "Monet", "art/wheat-summer.jpg"),
    (Painting::Seurat, "Seurat", "art/jatte.jpg"),
    (Painting::Inness, "Inness", "art/farm.jpg"),
];

/// Where each agent keeps its history, relative to home.
fn history_dir(agent: Agent) -> &'static str {
    match agent {
        Agent::Claude => ".claude/projects",
        Agent::OpenCode => ".local/share/opencode",
        Agent::Codex => ".codex/sessions",
    }
}

fn size_label(bytes: u64) -> String {
    match bytes {
        0 => "empty".into(),
        b if b < 1 << 20 => format!("{} KB", b.div_ceil(1024)),
        b => format!("{:.1} MB", b as f64 / (1 << 20) as f64),
    }
}

impl Shell {
    pub fn render_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let state = self.state.read(cx);
        let settings = state.settings.clone();
        let snapshot = state.snapshot.clone();
        let editors = installed_editors();
        let terminals: Vec<&str> = launch::TERMINALS.iter().copied().filter(|t| launch::installed(t) || *t == "Terminal").collect();
        let home = settings.home();
        let mono = |text: String| div().font_family(MONO).text_size(px(12.)).text_color(theme.ink_2).child(text);

        let group = |title: &'static str| {
            v_flex()
                .gap(px(0.))
                .child(div().pb(px(8.)).font_family(MONO).text_size(px(10.5)).text_color(theme.ink_3).child(title.to_uppercase()))
        };
        let row = |label: SharedString, hint: Option<String>| {
            h_flex()
                .min_h(px(54.))
                .py(px(8.))
                .gap(px(16.))
                .border_b_1()
                .border_color(theme.line)
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(div().text_size(px(14.)).child(label))
                        .when_some(hint, |col, hint| col.child(div().text_size(px(12.)).text_color(theme.ink_3).child(hint))),
                )
        };

        let painting = settings.painting;
        let thumbs = h_flex().gap(px(10.)).py(px(12.)).border_b_1().border_color(theme.line).children(PAINTINGS.iter().skip(1).map(|(choice, name, path)| {
            let choice = *choice;
            v_flex()
                .id(SharedString::from(format!("art-{name}")))
                .gap(px(6.))
                .child(
                    div()
                        .w(px(150.))
                        .h(px(84.))
                        .rounded(px(10.))
                        .overflow_hidden()
                        .border_2()
                        .border_color(if painting == choice { theme.ink } else { theme.bg })
                        .child(img(*path).size_full().object_fit(ObjectFit::Cover)),
                )
                .child(div().text_size(px(11.5)).text_color(theme.ink_3).child(*name))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.painting = choice));
                }))
        }));
        let appearance = group("Appearance")
            .child(row("Theme".into(), Some("Light is the default; dark is built in.".into())).child(
                Segmented::new("theme", ["Light", "Dark"]).small().selected(theme.dark as usize).on_select(cx.listener(|this, index, _, cx| {
                    if (*index == 1) != cx.theme().dark {
                        this.toggle_theme(cx);
                    }
                })),
            ))
            .child(
                row("Home painting".into(), Some("Public-domain art from the Art Institute of Chicago. Auto follows the time of day.".into())).child(
                    Segmented::new("painting", PAINTINGS.iter().map(|(_, n, _)| *n))
                        .small()
                        .selected(PAINTINGS.iter().position(|(p, _, _)| *p == painting).unwrap_or(0))
                        .on_select(cx.listener(|this, index: &usize, _, cx| {
                            let choice = PAINTINGS[*index].0;
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.painting = choice));
                        })),
                ),
            )
            .child(thumbs);

        let latest = snapshot.projects.iter().flat_map(|p| &p.sessions).chain(&snapshot.loose).max_by_key(|s| s.updated_ms);
        let apps = group("Apps")
            .child(
                row("Open projects in".into(), Some("Used by every Open button. Marked with the green dot.".into())).child(
                    Segmented::new("editor", editors.iter().map(|(name, _)| *name))
                        .small()
                        .selected(editors.iter().position(|(n, _)| *n == super::default_editor(&settings.editor)).unwrap_or(0))
                        .on_select(cx.listener(move |this, index, _, cx| {
                            let Some((name, _)): Option<(&str, &str)> = installed_editors().get(*index).copied() else { return };
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.editor = name.into()));
                        })),
                ),
            )
            .child(
                row("Resume conversations in".into(), Some("Opens a new window in the session's folder.".into())).child(
                    Segmented::new("terminal", terminals.clone())
                        .small()
                        .selected(terminals.iter().position(|t| *t == settings.terminal).unwrap_or(0))
                        .on_select(cx.listener(move |this, index, _, cx| {
                            let Some(name): Option<&str> = terminals.get(*index).copied() else { return };
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.terminal = name.into()));
                        })),
                ),
            )
            .when_some(latest, |g, session| {
                g.child(
                    row("Resume command".into(), Some(format!("What Codedeck runs for your latest {} session.", session.agent.label()))).child(
                        div()
                            .max_w(px(300.))
                            .truncate()
                            .px(px(9.))
                            .py(px(4.))
                            .rounded(px(7.))
                            .bg(theme.soft)
                            .font_family(MONO)
                            .text_size(px(12.))
                            .child(launch::resume_command(session)),
                    ),
                )
            });

        let key = |label: &'static str, keys: &'static str| {
            h_flex()
                .h(px(38.))
                .border_b_1()
                .border_color(theme.line)
                .child(div().flex_1().text_size(px(14.)).child(label))
                .child(div().font_family(MONO).text_size(px(12.)).text_color(theme.ink_2).child(keys))
        };
        let shortcuts = group("Shortcuts")
            .child(key("Jump to a project or session", "⌘K"))
            .child(key("Search projects", "⌘F"))
            .child(key("Home", "⌘1"))
            .child(key("Settings", "⌘,"))
            .child(key("Rescan", "⌘R"))
            .child(key("Light / dark", "⇧⌘L"));

        let agents = group("Agents").children(AGENTS.into_iter().map(|agent| {
            let on = settings.agents.contains(&agent);
            let dir = history_dir(agent);
            let count = snapshot.projects.iter().flat_map(|p| &p.sessions).chain(&snapshot.loose).filter(|s| s.agent == agent).count();
            let found = home.join(dir).exists();
            let hint = match (found, on) {
                (false, _) => format!("~/{dir} · not found"),
                (true, false) => format!("~/{dir} · off"),
                (true, true) => format!("~/{dir} · {count} conversations"),
            };
            let control = if found {
                Toggle::new(SharedString::from(format!("agent-{agent:?}")), on)
                    .on_change(cx.listener(move |this, on, _, cx| {
                        let on = *on;
                        this.state.update(cx, |state, cx| {
                            state.update_settings(cx, |s| {
                                s.agents.retain(|a| *a != agent);
                                if on {
                                    s.agents.push(agent);
                                }
                            })
                        });
                    }))
                    .into_any_element()
            } else {
                div().text_size(px(12.)).text_color(theme.ink_3).child("Not installed").into_any_element()
            };
            row(agent.label().into(), Some(hint)).child(agent_icon(agent, 16., cx)).child(control)
        }));

        let scanned = if snapshot.scanned_ms > 0 { match ago(snapshot.scanned_ms).as_str() {
            "now" => "Last scanned just now. Codedeck also rescans when files change.".to_string(),
            age => format!("Last scanned {age} ago. Codedeck also rescans when files change."),
        } } else { "Codedeck also rescans when files change.".into() };
        let folders = group("Folders")
            .children(settings.roots.iter().enumerate().map(|(index, root)| {
                let remove = root.clone();
                let count = snapshot.projects.iter().filter(|p| p.root.starts_with(root)).count();
                row(tilde(root).into(), Some(format!("{count} {}", if count == 1 { "project" } else { "projects" })))
                    .child(GelButton::new(SharedString::from(format!("rm-root-{index}")), "Remove").paper().small().on_click(cx.listener(
                        move |this, _, _, cx| {
                            let remove = remove.clone();
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.roots.retain(|r| *r != remove)));
                        },
                    )))
            }))
            .child(
                row("Add a folder".into(), Some("Codedeck finds every project inside it.".into())).child(
                    GelButton::new("add-root", "Choose…").small().icon("icons/ui/folder.svg").on_click(cx.listener(|this, _, _, cx| {
                        this.add_folders(cx);
                    })),
                ),
            )
            .child(row("Skipped folders".into(), Some("Never read inside these, or any hidden folder.".into())).child(
                h_flex().flex_wrap().justify_end().max_w(px(320.)).gap(px(5.)).children(scan::SKIP.iter().map(|name| {
                    div().px(px(7.)).h(px(20.)).flex().items_center().rounded(px(6.)).border_1().border_color(theme.line_2).font_family(MONO).text_size(px(11.)).text_color(theme.ink_2).child(*name)
                })),
            ))
            .child(row("Depth".into(), Some("How deep to look for projects inside each folder.".into())).child(mono(settings.depth.to_string())))
            .child(row("Rescan".into(), Some(scanned)).child(GelButton::new("rescan", "Rescan now").paper().small().on_click(cx.listener(|this, _, _, cx| {
                this.state.update(cx, |state, cx| state.rescan(cx))
            }))));

        let promise = h_flex()
            .gap(px(14.))
            .p(px(16.))
            .mb(px(4.))
            .rounded(px(14.))
            .bg(theme.soft)
            .child(Avatar::mascot().size(34.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.))
                    .child(div().text_size(px(14.)).font_weight(FontWeight::MEDIUM).child("Codedeck only reads."))
                    .child(div().w_full().whitespace_normal().text_size(px(12.5)).text_color(theme.ink_3).child(
                        "It never moves, edits or uploads your code or your conversations. There is no account and no analytics. Only project names leave this Mac, as avatar seeds for DiceBear.",
                    )),
            );
        let privacy = group("Privacy").child(promise).child(
            row("Index".into(), Some(format!("{} · {}", tilde(&index::path()), size_label(index::size())))).child(
                h_flex()
                    .gap(px(6.))
                    .child(GelButton::new("reveal-index", "Reveal").paper().small().on_click(|_, _, _| {
                        if let Err(error) = launch::open_in("Finder", &support_dir()) {
                            log::warn!("{error:#}");
                        }
                    }))
                    .child(GelButton::new("clear-index", "Clear").paper().small().on_click(cx.listener(|_, _, _, cx| {
                        if let Err(error) = index::clear() {
                            log::warn!("{error:#}");
                        }
                        cx.notify();
                    }))),
            ),
        );

        v_flex()
            .w_full()
            .px(px(32.))
            .pt(px(52.))
            .pb(px(40.))
            .max_w(px(780.))
            .gap(px(30.))
            .child(div().text_size(px(30.)).child("Settings"))
            .child(appearance)
            .child(apps)
            .child(agents)
            .child(folders)
            .child(shortcuts)
            .child(privacy)
            .into_any_element()
    }
}

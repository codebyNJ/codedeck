use super::{installed_editors, tilde};
use crate::shell::Shell;
use codedeck_core::{Agent, launch};
use codedeck_ui::prelude::*;

const AGENTS: [Agent; 3] = [Agent::Claude, Agent::OpenCode, Agent::Codex];

impl Shell {
    pub fn render_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let settings = self.state.read(cx).settings.clone();
        let editors = installed_editors();
        let terminals: Vec<&str> = launch::TERMINALS.iter().copied().filter(|t| launch::installed(t) || *t == "Terminal").collect();

        let group = |title: &'static str| {
            v_flex()
                .gap(px(0.))
                .child(div().pb(px(8.)).font_family(MONO).text_size(px(10.5)).text_color(theme.ink_3).child(title.to_uppercase()))
        };
        let row = |label: &'static str, hint: Option<String>| {
            h_flex()
                .min_h(px(54.))
                .gap(px(16.))
                .border_b_1()
                .border_color(theme.line)
                .child(
                    v_flex()
                        .flex_1()
                        .child(div().text_size(px(14.)).child(label))
                        .when_some(hint, |col, hint| col.child(div().text_size(px(12.)).text_color(theme.ink_3).child(hint))),
                )
        };

        let appearance = group("Appearance").child(row("Theme", Some("Light is the default; dark is built in.".into())).child(
            Segmented::new("theme", ["Light", "Dark"]).selected(theme.dark as usize).on_select(cx.listener(|this, index, _, cx| {
                if (*index == 1) != cx.theme().dark {
                    this.toggle_theme(cx);
                }
            })),
        ));

        let apps = group("Apps")
            .child(
                row("Default editor", Some("Marked with the green dot on every card.".into())).child(
                    Segmented::new("editor", editors.iter().map(|(name, _)| *name))
                        .selected(editors.iter().position(|(n, _)| *n == super::default_editor(&settings.editor)).unwrap_or(0))
                        .on_select(cx.listener(move |this, index, _, cx| {
                            let Some((name, _)): Option<(&str, &str)> = installed_editors().get(*index).copied() else { return };
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.editor = name.into()));
                        })),
                ),
            )
            .child(
                row("Terminal", None).child(
                    Segmented::new("terminal", terminals.clone())
                        .selected(terminals.iter().position(|t| *t == settings.terminal).unwrap_or(0))
                        .on_select(cx.listener(move |this, index, _, cx| {
                            let Some(name): Option<&str> = terminals.get(*index).copied() else { return };
                            this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.terminal = name.into()));
                        })),
                ),
            );

        let agents = group("Agents").children(AGENTS.into_iter().map(|agent| {
            let on = settings.agents.contains(&agent);
            row(agent.label(), None).child(Toggle::new(SharedString::from(format!("agent-{agent:?}")), on).on_change(cx.listener(
                move |this, on, _, cx| {
                    let on = *on;
                    this.state.update(cx, |state, cx| {
                        state.update_settings(cx, |s| {
                            s.agents.retain(|a| *a != agent);
                            if on {
                                s.agents.push(agent);
                            }
                        })
                    });
                },
            )))
        }));

        let folders = group("Folders")
            .children(settings.roots.iter().map(|root| {
                row("Scanning", None).child(div().font_family(MONO).text_size(px(12.5)).text_color(theme.ink_2).child(tilde(root)))
            }))
            .child(row("Depth", Some("How deep to look for projects inside each folder.".into())).child(div().font_family(MONO).child(settings.depth.to_string())))
            .child(
                row("Rescan", Some("Codedeck also rescans when files change.".into()))
                    .child(GelButton::new("rescan", "Rescan now").paper().small().on_click(cx.listener(|this, _, _, cx| {
                        this.state.update(cx, |state, cx| state.rescan(cx))
                    }))),
            );

        let privacy = group("Privacy").child(row(
            "Stays on this Mac",
            Some("Codedeck reads agent history locally. Only project names leave, as avatar seeds for DiceBear.".into()),
        ));

        v_flex()
            .w_full()
            .px(px(32.))
            .pt(px(52.))
            .pb(px(40.))
            .max_w(px(760.))
            .gap(px(30.))
            .child(h_flex().gap(px(10.)).child(div().text_size(px(30.)).child("Settings")))
            .child(appearance)
            .child(apps)
            .child(agents)
            .child(folders)
            .child(privacy)
            .into_any_element()
    }
}

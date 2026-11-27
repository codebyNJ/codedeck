use super::{ago, tilde};
use crate::shell::{Screen, Shell};
use repobox_core::{Session, avatar};
use repobox_ui::prelude::*;
use std::path::PathBuf;

actions!(palette, [Close, Up, Down, Confirm]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        // The input binds escape to Clear; the deeper context wins.
        KeyBinding::new("escape", Close, Some("Palette > TextInput")),
        KeyBinding::new("up", Up, Some("Palette")),
        KeyBinding::new("down", Down, Some("Palette")),
        KeyBinding::new("enter", Confirm, Some("Palette")),
    ]);
}

#[derive(Clone)]
pub enum Hit {
    Project { root: PathBuf, name: String },
    Session { session: Session, project: String },
}

const PROJECTS: usize = 4;
const SESSIONS: usize = 8;

impl Shell {
    // ponytail: substring match, no fuzzy ranking; add nucleo when typos start missing.
    pub fn palette_hits(&self, cx: &App) -> Vec<Hit> {
        let query = self.palette_input.read(cx).text().trim().to_lowercase();
        let has = |s: &str| query.is_empty() || s.to_lowercase().contains(&query);
        let snapshot = &self.state.read(cx).snapshot;

        let projects = snapshot
            .projects
            .iter()
            .filter(|p| has(&p.name) || has(&p.root.to_string_lossy()))
            .take(PROJECTS)
            .map(|p| Hit::Project { root: p.root.clone(), name: p.name.clone() });

        let mut sessions: Vec<(&Session, String)> = snapshot
            .projects
            .iter()
            .flat_map(|p| p.sessions.iter().map(move |s| (s, p.name.clone())))
            .chain(snapshot.loose.iter().map(|s| (s, tilde(&s.cwd))))
            .filter(|(s, project)| has(&s.title) || has(project))
            .collect();
        sessions.sort_by_key(|(s, _)| std::cmp::Reverse(s.updated_ms));

        projects
            .chain(sessions.into_iter().take(SESSIONS).map(|(s, project)| Hit::Session { session: s.clone(), project }))
            .collect()
    }

    pub fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette {
            return self.close_palette(window, cx);
        }
        self.palette = true;
        self.palette_index = 0;
        self.palette_input.update(cx, |input, cx| input.reset(window, cx));
        let handle = self.palette_input.read(cx).focus_handle(cx);
        window.focus(&handle, cx);
        cx.notify();
    }

    fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette = false;
        self.palette_input.update(cx, |input, cx| input.reset(window, cx));
        window.blur(cx);
        cx.notify();
    }

    fn pick(&mut self, hit: Hit, window: &mut Window, cx: &mut Context<Self>) {
        self.close_palette(window, cx);
        match hit {
            Hit::Project { root, .. } => self.go(Screen::Project(root), cx),
            Hit::Session { session, .. } => self.resume(&session, cx),
        }
    }

    pub fn render_palette(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let hits = self.palette_hits(cx);
        let selected = self.palette_index.min(hits.len().saturating_sub(1));

        let rows = hits.iter().cloned().enumerate().map(|(i, hit)| {
            let row = h_flex()
                .id(("hit", i))
                .h(px(40.))
                .px(px(12.))
                .gap(px(10.))
                .rounded(px(8.))
                .when(i == selected, |r| r.bg(theme.hi));
            let row = match &hit {
                Hit::Project { root, name } => row
                    .child(Avatar::project(name.clone()).image(avatar::path(name)).size(22.))
                    .child(div().text_size(px(13.5)).child(name.clone()))
                    .child(div().flex_1().min_w_0().truncate().font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).child(tilde(root)))
                    .child(div().text_size(px(11.5)).text_color(theme.ink_3).child("Open")),
                Hit::Session { session, project } => row
                    .child(super::agent_icon(session.agent, 15., cx))
                    .child(div().flex_1().min_w_0().truncate().text_size(px(13.5)).child(session.title.clone()))
                    .child(div().max_w(px(150.)).truncate().text_size(px(11.5)).text_color(theme.ink_3).child(project.clone()))
                    .child(div().w(px(30.)).text_size(px(11.5)).text_color(theme.ink_3).child(ago(session.updated_ms))),
            };
            row.on_click(cx.listener(move |this, _, window, cx| this.pick(hit.clone(), window, cx)))
        });

        let card = v_flex()
            .w(px(640.))
            .mt(px(110.))
            .bg(theme.card)
            .border_1()
            .border_color(theme.line_2)
            .rounded(px(14.))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                h_flex()
                    .h(px(52.))
                    .px(px(16.))
                    .gap(px(10.))
                    .border_b_1()
                    .border_color(theme.line)
                    .child(svg().path("icons/ui/search.svg").size(px(15.)).text_color(theme.ink_3))
                    .child(div().flex_1().text_size(px(15.)).child(self.palette_input.clone())),
            )
            .child(if hits.is_empty() {
                div().p(px(20.)).text_color(theme.ink_3).child("No projects or sessions match.").into_any_element()
            } else {
                v_flex().p(px(6.)).children(rows).into_any_element()
            })
            .child(
                h_flex()
                    .px(px(16.))
                    .py(px(9.))
                    .gap(px(14.))
                    .border_t_1()
                    .border_color(theme.line)
                    .font_family(MONO)
                    .text_size(px(10.5))
                    .text_color(theme.ink_3)
                    .child("↑↓ move")
                    .child("↵ open project · resume session")
                    .child("esc close"),
            );

        let count = hits.len();
        div()
            .absolute()
            .inset_0()
            .occlude()
            .key_context("Palette")
            .flex()
            .justify_center()
            .items_start()
            .bg(hsla(0., 0., 0., 0.18))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| this.close_palette(window, cx)))
            .on_action(cx.listener(|this, _: &Close, window, cx| this.close_palette(window, cx)))
            .on_action(cx.listener(move |this, _: &Up, _, cx| {
                this.palette_index = this.palette_index.min(count.saturating_sub(1)).saturating_sub(1);
                cx.notify();
            }))
            .on_action(cx.listener(move |this, _: &Down, _, cx| {
                this.palette_index = (this.palette_index + 1).min(count.saturating_sub(1));
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Confirm, window, cx| {
                let hits = this.palette_hits(cx);
                if let Some(hit) = hits.get(this.palette_index.min(hits.len().saturating_sub(1))).cloned() {
                    this.pick(hit, window, cx);
                }
            }))
            .child(card)
            .into_any_element()
    }
}

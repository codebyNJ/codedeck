use super::{AGENTS, by_day, group_label, tilde};
use crate::shell::{Scope, Shell};
use codedeck_core::{Session, now_ms};
use codedeck_ui::prelude::*;
use std::path::PathBuf;

impl Shell {
    /// Every session, project or not, newest first: (session, project name, project root).
    pub fn all_sessions(&self, cx: &App) -> Vec<(Session, String, Option<PathBuf>)> {
        let snapshot = &self.state.read(cx).snapshot;
        let mut all: Vec<_> = snapshot
            .projects
            .iter()
            .flat_map(|p| p.sessions.iter().map(|s| (s.clone(), p.name.clone(), Some(p.root.clone()))))
            .chain(snapshot.loose.iter().map(|s| (s.clone(), tilde(&s.cwd), None)))
            .collect();
        all.sort_by_key(|(s, _, _)| std::cmp::Reverse(s.updated_ms));
        all
    }

    pub fn render_sessions(&mut self, scope: Scope, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let now = now_ms();
        let in_scope: Vec<_> = self
            .all_sessions(cx)
            .into_iter()
            .filter(|(s, _, root)| match scope {
                Scope::All => true,
                Scope::Running => s.is_live(now),
                Scope::Loose => root.is_none(),
                Scope::Agent(agent) => s.agent == agent,
            })
            .collect();

        // Agent filter: only agents that actually have sessions here.
        let agents: Vec<_> = AGENTS.into_iter().filter(|a| in_scope.iter().any(|(s, _, _)| s.agent == *a)).collect();
        let pick = self.agent_filter.min(agents.len());
        let shown: Vec<_> = in_scope.iter().filter(|(s, _, _)| pick == 0 || s.agent == agents[pick - 1]).collect();
        let projects = {
            let mut roots: Vec<_> = in_scope.iter().filter_map(|(_, _, r)| r.as_ref()).collect();
            roots.sort();
            roots.dedup();
            roots.len()
        };

        let title = match scope {
            Scope::All => "Sessions",
            Scope::Running => "Running now",
            Scope::Loose => "Loose sessions",
            Scope::Agent(agent) => agent.label(),
        };
        let meta = match scope {
            Scope::Loose => format!("{} outside any project", in_scope.len()),
            _ => format!("{} across {projects} {}", in_scope.len(), if projects == 1 { "project" } else { "projects" }),
        };

        let header = h_flex()
            .gap(px(10.))
            .child(div().text_size(px(30.)).child(title))
            .child(div().pt(px(8.)).text_size(px(13.)).text_color(theme.ink_3).child(meta))
            .child(div().flex_1())
            .when(agents.len() > 1, |row| {
                row.child(
                    Segmented::new("sess-agent", std::iter::once("All").chain(agents.iter().map(|a| a.label())))
                        .small()
                        .selected(pick)
                        .on_select(cx.listener(|this, index, _, cx| {
                            this.agent_filter = *index;
                            cx.notify();
                        })),
                )
            });

        let body = if shown.is_empty() {
            v_flex()
                .py(px(56.))
                .items_center()
                .gap(px(12.))
                .child(Avatar::mascot().size(44.))
                .child(div().text_color(theme.ink_3).child(match scope {
                    Scope::Running => "Nothing is running right now.",
                    _ => "No agent sessions here yet.",
                }))
                .into_any_element()
        } else {
            let groups = by_day(shown, |(s, _, _)| s.updated_ms);
            v_flex()
                .children(groups.into_iter().map(|(group, list)| {
                    v_flex().child(group_label(group.to_uppercase(), cx)).children(
                        list.into_iter().map(|(s, name, root)| self.session_row(s, Some((name.clone(), root.clone())), now, cx)),
                    )
                }))
                .into_any_element()
        };

        v_flex().w_full().px(px(32.)).pt(px(52.)).pb(px(40.)).gap(px(8.)).child(header).child(body).into_any_element()
    }
}

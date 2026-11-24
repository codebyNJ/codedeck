use super::{AGENTS, ago, agent_icon, by_day, group_label, stack_label, tilde};
use crate::shell::{Screen, Shell};
use codedeck_core::{avatar, git, launch, now_ms};
use codedeck_ui::prelude::*;
use std::path::Path;

impl Shell {
    pub fn render_project(&mut self, root: &Path, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let state = self.state.read(cx);
        let Some(project) = state.snapshot.projects.iter().find(|p| p.root == root).cloned() else {
            return v_flex()
                .p(px(40.))
                .pt(px(60.))
                .gap(px(12.))
                .child("This project isn't in the latest scan.")
                .child(GelButton::new("back", "Back to Home").paper().on_click(cx.listener(|this, _, _, cx| this.go(Screen::Home(None), cx))))
                .into_any_element();
        };
        let terminal = state.settings.terminal.clone();
        let now = now_ms();

        let header = h_flex()
            .gap(px(16.))
            .child(Avatar::project(project.name.clone()).image(avatar::path(&project.name)).size(64.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(4.))
                    .child(
                        h_flex()
                            .gap(px(10.))
                            .child(div().text_size(px(30.)).child(project.name.clone()))
                            .when(project.is_live(now), |row| row.child(Badge::running())),
                    )
                    .child(div().font_family(MONO).text_size(px(12.)).text_color(theme.ink_3).child(tilde(&project.root))),
            )
            .child(self.editor_buttons(&project.root, 36., cx))
            .child(div().w(px(1.)).h(px(22.)).mx(px(4.)).bg(theme.line_2))
            .child({
                let root = project.root.clone();
                GelButton::new("terminal", terminal.clone()).paper().icon("icons/ui/terminal.svg").on_click(move |_, _, _| {
                    if let Err(error) = launch::open_in(&terminal, &root) {
                        log::warn!("{error:#}");
                    }
                })
            })
            .child({
                let root = project.root.clone();
                GelButton::new("finder", "").paper().icon("icons/ui/folder.svg").on_click(move |_, _, _| {
                    if let Err(error) = launch::open_in("Finder", &root) {
                        log::warn!("{error:#}");
                    }
                })
            });

        let history = self.history.as_ref().filter(|(r, _)| r == root).and_then(|(_, h)| h.as_ref());
        let recent_commits: u32 = history.map_or(0, |h| h.per_day[git::HISTORY_DAYS - 14..].iter().sum());
        let facts = h_flex()
            .gap(px(6.))
            .when_some(project.branch.clone(), |row, branch| row.child(Pill::new(branch).icon("icons/ui/git-branch.svg")))
            .child(if project.dirty > 0 { Pill::new("uncommitted").bold(project.dirty.to_string()).warn() } else { Pill::new("clean") })
            .when(history.is_some(), |row| row.child(Pill::new("commits · 2 weeks").bold(recent_commits.to_string())));

        // Agent filter, only when more than one agent worked here.
        let agents: Vec<_> = AGENTS.into_iter().filter(|a| project.sessions.iter().any(|s| s.agent == *a)).collect();
        let pick = self.agent_filter.min(agents.len());
        let shown: Vec<_> = project.sessions.iter().filter(|s| pick == 0 || s.agent == agents[pick - 1]).collect();
        let live = project.is_live(now);

        let heading = h_flex()
            .gap(px(10.))
            .child(div().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child("Sessions"))
            .child(div().text_size(px(13.)).text_color(theme.ink_3).child(format!("{} here", project.sessions.len())))
            .child(div().flex_1())
            .when(agents.len() > 1, |row| {
                row.child(
                    Segmented::new("proj-agent", std::iter::once("All").chain(agents.iter().map(|a| a.label())))
                        .small()
                        .selected(pick)
                        .on_select(cx.listener(|this, index, _, cx| {
                            this.agent_filter = *index;
                            cx.notify();
                        })),
                )
            });

        let sessions = v_flex().children(by_day(shown, |s| s.updated_ms).into_iter().enumerate().map(|(i, (group, list))| {
            let label = if i == 0 && group == "Today" && live { "RUNNING NOW · TODAY".to_string() } else { group.to_uppercase() };
            v_flex().child(group_label(label, cx)).children(list.into_iter().map(|s| self.session_row(s, None, now, cx)))
        }));


        let block = |label: &'static str| v_flex().gap(px(8.)).child(div().font_family(MONO).text_size(px(10.5)).text_color(theme.ink_3).child(label.to_uppercase()));
        let agent_pills = h_flex().flex_wrap().gap(px(6.)).children(agents.iter().map(|agent| {
            let list: Vec<_> = project.sessions.iter().filter(|s| s.agent == *agent).collect();
            let prompts: u32 = list.iter().map(|s| s.prompts).sum();
            h_flex()
                .h(px(24.))
                .px(px(8.))
                .gap(px(6.))
                .rounded(px(7.))
                .bg(theme.soft)
                .text_size(px(12.))
                .text_color(theme.ink_2)
                .child(agent_icon(*agent, 12., cx))
                .child(format!("{} · {} · {prompts} prompts", agent.label(), list.len()))
        }));
        let stack = h_flex().flex_wrap().gap(px(6.)).children(project.stacks.iter().map(|slug| {
            h_flex()
                .h(px(24.))
                .px(px(8.))
                .gap(px(6.))
                .rounded(px(7.))
                .border_1()
                .border_color(theme.line_2)
                .text_size(px(12.))
                .text_color(theme.ink_2)
                .child(StackIcon::new(slug.clone()).size(12.))
                .child(stack_label(slug).to_string())
        }));
        let git_block = history.map(|h| {
            let peak = h.per_day.iter().copied().max().unwrap_or(0).max(1) as f32;
            let grid = div().flex().flex_wrap().gap(px(3.)).w(px(7. * 14. + 6. * 3.)).children(h.per_day.iter().map(|n| {
                let cell = div().size(px(14.)).rounded(px(3.));
                // Flat ink at four strengths, empty days on the soft fill.
                if *n == 0 { cell.bg(theme.soft) } else { cell.bg(theme.ink.opacity(0.25 + 0.65 * (*n as f32 / peak))) }
            }));
            block("Git")
                .child(
                    div().text_size(px(12.5)).text_color(theme.ink_2).child(format!(
                        "Last commit “{}”, {} ago{}.",
                        h.last_subject,
                        ago(h.last_ms),
                        project.branch.as_ref().map(|b| format!(" on {b}")).unwrap_or_default()
                    )),
                )
                .child(grid)
                .child(div().text_size(px(11.)).text_color(theme.ink_3).child(format!("Commits per day, last {} days", git::HISTORY_DAYS)))
        });
        let used = v_flex()
            .w(px(280.))
            .flex_shrink_0()
            .gap(px(22.))
            .child(div().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child("Used here"))
            .when(!agents.is_empty(), |col| col.child(block("Agents").child(agent_pills)))
            .when(!project.stacks.is_empty(), |col| col.child(block("Stack").child(stack)))
            .children(git_block);

        v_flex()
            .w_full()
            .px(px(32.))
            .pt(px(52.))
            .pb(px(40.))
            .gap(px(22.))
            .child(header)
            .child(facts)
            .child(h_flex().items_start().gap(px(36.)).child(div().flex_1().min_w_0().child(if project.sessions.is_empty() {
                v_flex()
                    .py(px(46.))
                    .items_center()
                    .gap(px(12.))
                    .child(Avatar::mascot().size(44.))
                    .child(div().text_color(theme.ink_3).child("No agent sessions in this project yet."))
                    .into_any_element()
            } else {
                v_flex().child(heading).child(sessions).into_any_element()
            })).child(used))
            .into_any_element()
    }
}

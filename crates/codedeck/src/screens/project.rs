use super::{ago, stack_label, tilde};
use crate::shell::{Screen, Shell};
use codedeck_core::{Session, avatar, launch, now_ms};
use codedeck_ui::prelude::*;
use std::path::Path;

/// Today / Yesterday / This week / Earlier, by local calendar day.
fn day_group(updated_ms: i64) -> &'static str {
    let today = jiff::ZonedDateTime::now().date();
    let Ok(stamp) = jiff::Timestamp::from_millisecond(updated_ms) else { return "Earlier" };
    let days = (today - stamp.to_zoned_datetime(jiff::tz::TimeZone::system()).date()).get_days();
    match days {
        0 => "Today",
        1 => "Yesterday",
        2..=6 => "This week",
        _ => "Earlier",
    }
}

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
                .child(GelButton::new("back", "Back to Home").paper().on_click(cx.listener(|this, _, _, cx| this.go(Screen::Home, cx))))
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
            });

        let facts = h_flex()
            .gap(px(6.))
            .when_some(project.branch.clone(), |row, branch| row.child(Pill::new(branch).icon("icons/ui/git-branch.svg")))
            .child(if project.dirty > 0 { Pill::new("uncommitted").bold(project.dirty.to_string()).warn() } else { Pill::new("clean") })
            .children(project.stacks.iter().map(|slug| {
                h_flex().gap(px(5.)).child(StackIcon::new(slug.clone()).size(13.)).child(Pill::new(stack_label(slug).to_string()))
            }));

        let mut groups: Vec<(&'static str, Vec<&Session>)> = Vec::new();
        for session in &project.sessions {
            let group = day_group(session.updated_ms);
            match groups.last_mut() {
                Some((name, list)) if *name == group => list.push(session),
                _ => groups.push((group, vec![session])),
            }
        }

        let sessions = v_flex().gap(px(22.)).children(groups.into_iter().map(|(group, list)| {
            v_flex()
                .gap(px(2.))
                .child(div().pb(px(6.)).font_family(MONO).text_size(px(10.5)).text_color(theme.ink_3).child(group.to_uppercase()))
                .children(list.into_iter().map(|session| {
                    h_flex()
                        .gap(px(12.))
                        .py(px(10.))
                        .border_b_1()
                        .border_color(theme.line)
                        .child(Pill::new(session.agent.label()))
                        .child(div().flex_1().min_w_0().text_size(px(13.5)).truncate().child(session.title.clone()))
                        .when(session.is_live(now), |row| row.child(Badge::running()))
                        .child(div().text_size(px(12.)).text_color(theme.ink_3).child(format!("{} prompts", session.prompts)))
                        .child(div().w(px(36.)).text_size(px(12.)).text_color(theme.ink_3).child(ago(session.updated_ms)))
                }))
        }));

        v_flex()
            .w_full()
            .px(px(32.))
            .pt(px(52.))
            .pb(px(40.))
            .gap(px(22.))
            .child(header)
            .child(facts)
            .child(
                div()
                    .font_family(SERIF)
                    .italic()
                    .text_size(px(14.))
                    .text_color(theme.ink_3)
                    .child("Open the project in your editor or terminal to continue any of these sessions."),
            )
            .child(if project.sessions.is_empty() {
                div().text_color(theme.ink_3).child("No agent sessions here yet.").into_any_element()
            } else {
                sessions.into_any_element()
            })
            .into_any_element()
    }
}

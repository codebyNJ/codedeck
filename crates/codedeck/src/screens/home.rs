use super::{ago, stack_label, tilde};
use crate::shell::{Screen, Shell};
use codedeck_core::{Agent, Project, avatar, now_ms};
use codedeck_ui::prelude::*;

/// Painting by time of day: (asset, credit).
fn painting(hour: i8) -> (&'static str, &'static str) {
    match hour {
        5..=10 => ("art/wheat-day.jpg", "Claude Monet, Stacks of Wheat (End of Day, Autumn), 1890–91"),
        11..=15 => ("art/jatte.jpg", "Georges Seurat, A Sunday on La Grande Jatte, 1884–86"),
        16..=19 => ("art/wheat-summer.jpg", "Claude Monet, Stacks of Wheat (End of Summer), 1890–91"),
        _ => ("art/farm.jpg", "George Inness, The Lonely Farm, Nantucket, 1892"),
    }
}

fn greeting(hour: i8) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=16 => "Good afternoon",
        _ => "Good evening",
    }
}

const SORTS: [&str; 3] = ["Recent", "A–Z", "Most chats"];

impl Shell {
    pub fn render_home(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let snapshot = self.state.read(cx).snapshot.clone();
        let now = now_ms();
        let hour = jiff::ZonedDateTime::now().hour();
        let (art, credit) = painting(hour);
        let running = snapshot.projects.iter().filter(|p| p.is_live(now)).count();
        let dirty: u32 = snapshot.projects.iter().map(|p| p.dirty).sum();

        // Filters: All plus the five most common stacks.
        let mut counts: Vec<(String, usize)> = Vec::new();
        for slug in snapshot.projects.iter().flat_map(|p| p.stacks.iter()) {
            match counts.iter_mut().find(|(s, _)| s == slug) {
                Some((_, n)) => *n += 1,
                None => counts.push((slug.clone(), 1)),
            }
        }
        counts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        counts.truncate(5);
        let mut filters = vec![("All".to_string(), None)];
        filters.extend(counts.iter().map(|(slug, _)| (stack_label(slug).to_string(), Some(slug.clone()))));
        self.filter = self.filter.min(filters.len() - 1);

        let mut projects: Vec<&Project> = snapshot
            .projects
            .iter()
            .filter(|p| filters[self.filter].1.as_ref().is_none_or(|slug| p.stacks.contains(slug)))
            .collect();
        match self.sort {
            1 => projects.sort_by_key(|p| p.name.to_lowercase()),
            2 => projects.sort_by_key(|p| std::cmp::Reverse(p.sessions.len())),
            _ => {}
        }

        let summary = match (running, dirty) {
            (0, 0) => "Everything is committed and quiet.".to_string(),
            (0, d) => format!("{d} files are waiting to be committed across your projects."),
            (r, d) => format!(
                "{} running right now. {d} files are waiting to be committed.",
                if r == 1 { "One agent is".to_string() } else { format!("{r} agents are") }
            ),
        };

        let banner = div()
            .relative()
            .h(px(230.))
            .rounded(px(20.))
            .overflow_hidden()
            .child(img(art).absolute().inset_0().size_full().object_fit(ObjectFit::Cover))
            .child(
                div().absolute().inset_0().bg(linear_gradient(
                    0.,
                    linear_color_stop(hsla(0., 0., 0., 0.55), 0.),
                    linear_color_stop(hsla(0., 0., 0., 0.0), 0.75),
                )),
            )
            .child(
                v_flex()
                    .absolute()
                    .left(px(28.))
                    .bottom(px(24.))
                    .gap(px(8.))
                    .text_color(white())
                    .child(div().font_family(SERIF).italic().text_size(px(40.)).line_height(px(44.)).child(format!("{}.", greeting(hour))))
                    .child(div().text_size(px(14.)).opacity(0.92).child(summary))
                    .child(
                        h_flex()
                            .mt(px(6.))
                            .h(px(30.))
                            .px(px(12.))
                            .gap(px(12.))
                            .rounded(px(10.))
                            .bg(hsla(0., 0., 1., 0.16))
                            .border_1()
                            .border_color(hsla(0., 0., 1., 0.28))
                            .text_size(px(12.5))
                            .child(format!("{} projects", snapshot.projects.len()))
                            .child(div().w(px(1.)).h(px(12.)).bg(hsla(0., 0., 1., 0.35)))
                            .child(format!("{} conversations", snapshot.session_count()))
                            .when(running > 0, |row| {
                                row.child(div().w(px(1.)).h(px(12.)).bg(hsla(0., 0., 1., 0.35))).child(
                                    h_flex().gap(px(6.)).child(div().size(px(6.)).rounded_full().bg(theme.live)).child(format!("{running} running")),
                                )
                            }),
                    ),
            )
            .child(div().absolute().right(px(16.)).bottom(px(12.)).text_size(px(10.5)).text_color(hsla(0., 0., 1., 0.7)).child(credit));

        let continue_tiles = self.render_continue(&snapshot.projects, cx);

        let header = h_flex()
            .gap(px(12.))
            .child(div().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child("Projects"))
            .child(
                Segmented::new("filter", filters.iter().map(|(label, _)| SharedString::from(label.clone())))
                    .selected(self.filter)
                    .on_select(cx.listener(|this, index, _, cx| {
                        this.filter = *index;
                        cx.notify();
                    })),
            )
            .child(div().flex_1())
            .child(Segmented::new("sort", SORTS).small().selected(self.sort).on_select(cx.listener(|this, index, _, cx| {
                this.sort = *index;
                cx.notify();
            })));

        let mut rows = Vec::new();
        for pair in projects.chunks(2) {
            let cards: Vec<AnyElement> = pair.iter().map(|project| self.project_card(project, now, cx)).collect();
            rows.push(
                h_flex()
                    .items_start()
                    .gap(px(12.))
                    .children(cards.into_iter().map(|card| div().flex_1().min_w_0().child(card)))
                    .when(pair.len() == 1, |row| row.child(div().flex_1())),
            );
        }

        v_flex()
            .w_full()
            .px(px(28.))
            .pt(px(44.))
            .pb(px(40.))
            .gap(px(22.))
            .child(banner)
            .child(continue_tiles)
            .child(header)
            .child(v_flex().gap(px(16.)).children(rows))
            .into_any_element()
    }

    /// The three most recent sessions, one per project.
    fn render_continue(&self, projects: &[Project], cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let mut recent: Vec<(&Project, &codedeck_core::Session)> =
            projects.iter().filter_map(|p| p.sessions.first().map(|s| (p, s))).collect();
        recent.sort_by_key(|(_, s)| std::cmp::Reverse(s.updated_ms));
        recent.truncate(3);
        let now = now_ms();
        h_flex().gap(px(12.)).children(recent.into_iter().map(|(project, session)| {
            let root = project.root.clone();
            let (letter, color) = match session.agent {
                Agent::Claude => ("C", rgb(0xd97757)),
                Agent::OpenCode => ("O", rgb(0x111112)),
                Agent::Codex => ("X", rgb(0x10a37f)),
            };
            h_flex()
                .id(SharedString::from(format!("cont-{}", session.id)))
                .flex_1()
                .min_w_0()
                .gap(px(11.))
                .p(px(12.))
                .rounded(px(16.))
                .border_1()
                .border_color(theme.line_2)
                .hover(|s| s.bg(theme.soft))
                .child(
                    div()
                        .relative()
                        .size(px(30.))
                        .flex_shrink_0()
                        .rounded(px(9.))
                        .bg(color)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(white())
                        .text_size(px(13.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(letter)
                        .when(session.is_live(now), |d| {
                            d.child(div().absolute().right(px(-2.)).top(px(-2.)).size(px(9.)).rounded_full().bg(theme.live).border_2().border_color(theme.bg))
                        }),
                )
                .child(
                    v_flex()
                        .min_w_0()
                        .child(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).truncate().child(session.title.clone()))
                        .child(div().text_size(px(11.5)).text_color(theme.ink_3).truncate().child(format!("{} · {}", project.name, ago(session.updated_ms)))),
                )
                .on_click(cx.listener(move |this, _, _, cx| this.go(Screen::Project(root.clone()), cx)))
        }))
        .into_any_element()
    }

    /// The v22 card.
    fn project_card(&self, project: &Project, now: i64, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let live = project.is_live(now);
        let root = project.root.clone();
        let sessions = project.sessions.len();
        let note = match sessions {
            0 => "No sessions yet".to_string(),
            1 => "1 session · open to continue".to_string(),
            n => format!("{n} sessions · open to continue"),
        };
        let top = h_flex()
            .items_start()
            .gap(px(12.))
            .child(Avatar::project(project.name.clone()).image(avatar::path(&project.name)).size(44.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .pt(px(2.))
                    .child(div().text_size(px(17.)).line_height(px(21.)).truncate().child(project.name.clone()))
                    .child(div().mt(px(3.)).font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).truncate().child(tilde(&project.root))),
            )
            .child(
                div()
                    .pt(px(4.))
                    .text_size(px(12.))
                    .text_color(theme.ink_3)
                    .child(if live { "active now".to_string() } else { format!("{} ago", ago(project.updated_ms)) }),
            );
        let pills = h_flex()
            .mt(px(12.))
            .pl(px(56.))
            .gap(px(4.))
            .when_some(project.branch.clone(), |row, branch| row.child(Pill::new(branch).icon("icons/ui/git-branch.svg")))
            .child(if project.dirty > 0 { Pill::new("uncommitted").bold(project.dirty.to_string()).warn() } else { Pill::new("clean") })
            .child(div().flex_1())
            .children(project.stacks.iter().take(3).map(|slug| StackIcon::new(slug.clone())));
        let footer = h_flex()
            .mt(px(14.))
            .mx(px(-16.))
            .mb(px(-16.))
            .px(px(16.))
            .py(px(10.))
            .border_t_1()
            .border_color(theme.line)
            .child(self.editor_buttons(&project.root, 28., cx))
            .child(div().flex_1())
            .child(div().font_family(SERIF).italic().text_size(px(12.5)).text_color(theme.ink_3).child(note));

        div()
            .id(SharedString::from(format!("card-{}", root.display())))
            .child(Card::new(SharedString::from(format!("c-{}", root.display()))).live(live).child(top).child(pills).child(footer))
            .on_click(cx.listener(move |this, _, _, cx| this.go(Screen::Project(root.clone()), cx)))
            .into_any_element()
    }
}

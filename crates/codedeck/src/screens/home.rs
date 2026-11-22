use super::{ago, stack_label, tilde};
use crate::shell::{Screen, Shell};
use codedeck_core::{Agent, Project, avatar, now_ms, settings::Layout};
use codedeck_ui::prelude::*;

/// Painting by time of day: (asset, credit).
fn painting(hour: i8) -> (&'static str, &'static str) {
    match hour {
        5..=11 => ("art/wheat-day.jpg", "Claude Monet, Stacks of Wheat (End of Day, Autumn), 1890–91"),
        12..=16 => ("art/jatte.jpg", "Georges Seurat, A Sunday on La Grande Jatte, 1884–86"),
        17..=20 => ("art/wheat-summer.jpg", "Claude Monet, Stacks of Wheat (End of Summer), 1890–91"),
        _ => ("art/farm.jpg", "George Inness, The Lonely Farm, Nantucket, 1892"),
    }
}

fn greeting(hour: i8) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=16 => "Good afternoon",
        17..=20 => "Good evening",
        _ => "Good evening",
    }
}

const SORTS: [&str; 3] = ["Recent", "A–Z", "Most chats"];
const LAYOUTS: [(Layout, &str, &str); 3] = [
    (Layout::Cards, "Cards", "icons/ui/columns-2.svg"),
    (Layout::Compact, "Compact", "icons/ui/layout-grid.svg"),
    (Layout::List, "List", "icons/ui/list.svg"),
];

/// Case-insensitive match on name, path and branch.
fn matches(project: &Project, query: &str) -> bool {
    query.is_empty()
        || project.name.to_lowercase().contains(query)
        || project.root.to_string_lossy().to_lowercase().contains(query)
        || project.branch.as_ref().is_some_and(|b| b.to_lowercase().contains(query))
}

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

        let query = self.search.read(cx).text().trim().to_lowercase();
        let layout = self.state.read(cx).settings.layout;
        let mut projects: Vec<&Project> = snapshot
            .projects
            .iter()
            .filter(|p| filters[self.filter].1.as_ref().is_none_or(|slug| p.stacks.contains(slug)))
            .filter(|p| matches(p, &query))
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

        let has_query = !query.is_empty();
        let search = h_flex()
            .id("search")
            .w(px(260.))
            .h(px(34.))
            .px(px(11.))
            .gap(px(8.))
            .rounded(px(10.))
            .border_1()
            .border_color(theme.line_2)
            .bg(theme.card)
            .text_size(px(13.5))
            .line_height(px(20.))
            .child(svg().path("icons/ui/search.svg").size(px(14.)).text_color(theme.ink_3))
            .child(div().flex_1().min_w_0().overflow_hidden().child(self.search.clone()))
            .child(if has_query {
                div()
                    .id("search-clear")
                    .size(px(18.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|d| d.bg(theme.soft))
                    .child(svg().path("icons/ui/x.svg").size(px(12.)).text_color(theme.ink_3))
                    .on_click(cx.listener(|this, _, window, cx| this.search.update(cx, |input, cx| input.reset(window, cx))))
                    .into_any_element()
            } else {
                div().font_family(MONO).text_size(px(11.)).text_color(theme.ink_4).child("⌘F").into_any_element()
            });

        let layout_index = LAYOUTS.iter().position(|(l, _, _)| *l == layout).unwrap_or(0);
        let header = v_flex()
            .gap(px(12.))
            .child(
                h_flex()
                    .gap(px(12.))
                    .child(div().text_size(px(19.)).font_weight(FontWeight::MEDIUM).child("Projects"))
                    .when(has_query, |row| {
                        row.child(div().text_size(px(13.)).text_color(theme.ink_3).child(match projects.len() {
                            1 => "1 match".to_string(),
                            n => format!("{n} matches"),
                        }))
                    })
                    .child(div().flex_1())
                    .child(search)
                    .child(
                        Segmented::new("layout", LAYOUTS.iter().map(|(_, label, _)| *label))
                            .icons(LAYOUTS.iter().map(|(_, _, icon)| *icon))
                            .selected(layout_index)
                            .on_select(cx.listener(|this, index: &usize, _, cx| {
                                let layout = LAYOUTS[*index].0;
                                this.state.update(cx, |state, cx| state.update_settings(cx, |s| s.layout = layout));
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(12.))
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
                    }))),
            );

        let list = if projects.is_empty() {
            v_flex()
                .py(px(48.))
                .items_center()
                .gap(px(6.))
                .child(div().font_family(SERIF).italic().text_size(px(20.)).child(if has_query {
                    format!("Nothing matches “{}”.", self.search.read(cx).text().trim())
                } else {
                    "No projects here yet.".to_string()
                }))
                .child(div().text_size(px(13.)).text_color(theme.ink_3).child(if has_query {
                    "Search looks at project names, paths and branches. Esc clears it."
                } else {
                    "Add a folder in Settings and Codedeck will find the repos inside."
                }))
                .into_any_element()
        } else {
            match layout {
                Layout::Cards => self.grid(&projects, 2, now, cx, Self::project_card),
                Layout::Compact => self.grid(&projects, 3, now, cx, Self::project_tile),
                Layout::List => self.project_list(&projects, now, cx),
            }
        };

        v_flex()
            .w_full()
            .px(px(28.))
            .pt(px(44.))
            .pb(px(40.))
            .gap(px(22.))
            .child(banner)
            .child(continue_tiles)
            .child(header)
            .child(list)
            .into_any_element()
    }

    /// Projects in rows of `columns`; a short last row keeps its cells the same width.
    fn grid(
        &self,
        projects: &[&Project],
        columns: usize,
        now: i64,
        cx: &mut Context<Self>,
        cell: fn(&Self, &Project, i64, &mut Context<Self>) -> AnyElement,
    ) -> AnyElement {
        let gap = if columns > 2 { 10. } else { 12. };
        v_flex()
            .gap(px(if columns > 2 { 10. } else { 16. }))
            .children(projects.chunks(columns).map(|chunk| {
                let cells: Vec<AnyElement> = chunk.iter().map(|project| cell(self, project, now, cx)).collect();
                h_flex()
                    .items_start()
                    .gap(px(gap))
                    .children(cells.into_iter().map(|c| div().flex_1().min_w_0().child(c)))
                    .children((chunk.len()..columns).map(|_| div().flex_1()))
            }))
            .into_any_element()
    }

    /// Compact layout: avatar, name, branch and changes, no footer.
    fn project_tile(&self, project: &Project, now: i64, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let live = project.is_live(now);
        let root = project.root.clone();
        let sessions = project.sessions.len();
        div()
            .id(SharedString::from(format!("tile-{}", root.display())))
            .child(
                Card::new(SharedString::from(format!("t-{}", root.display())))
                    .live(live)
                    .child(
                        h_flex()
                            .gap(px(10.))
                            .child(Avatar::project(project.name.clone()).image(avatar::path(&project.name)).size(34.))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(div().text_size(px(15.)).truncate().child(project.name.clone()))
                                    .child(div().text_size(px(11.5)).text_color(theme.ink_3).child(if live {
                                        "active now".to_string()
                                    } else {
                                        format!("{} ago · {}", ago(project.updated_ms), match sessions {
                                            0 => "no sessions".to_string(),
                                            1 => "1 session".to_string(),
                                            n => format!("{n} sessions"),
                                        })
                                    })),
                            ),
                    )
                    .child(
                        h_flex()
                            .mt(px(10.))
                            .gap(px(4.))
                            .overflow_hidden()
                            .when_some(project.branch.clone(), |row, branch| row.child(Pill::new(branch).icon("icons/ui/git-branch.svg")))
                            .when(project.dirty > 0, |row| row.child(Pill::new("").bold(project.dirty.to_string()).warn())),
                    ),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.go(Screen::Project(root.clone()), cx)))
            .into_any_element()
    }

    /// List layout: one hairline row per project.
    fn project_list(&self, projects: &[&Project], now: i64, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let last = projects.len().saturating_sub(1);
        let rows: Vec<AnyElement> = projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let live = project.is_live(now);
                let root = project.root.clone();
                let sessions = project.sessions.len();
                h_flex()
                    .id(SharedString::from(format!("row-{}", root.display())))
                    .h(px(56.))
                    .px(px(14.))
                    .gap(px(12.))
                    .when(index != last, |r| r.border_b_1().border_color(theme.line))
                    .hover(|r| r.bg(theme.soft))
                    .child(Avatar::project(project.name.clone()).image(avatar::path(&project.name)).size(30.))
                    .child(
                        v_flex()
                            .w(px(220.))
                            .flex_shrink_0()
                            .child(div().text_size(px(14.)).truncate().child(project.name.clone()))
                            .child(div().font_family(MONO).text_size(px(11.)).text_color(theme.ink_3).truncate().child(tilde(&project.root))),
                    )
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(4.))
                            .overflow_hidden()
                            .when_some(project.branch.clone(), |row, branch| row.child(Pill::new(branch).icon("icons/ui/git-branch.svg")))
                            .child(if project.dirty > 0 { Pill::new("uncommitted").bold(project.dirty.to_string()).warn() } else { Pill::new("clean") }),
                    )
                    .child(h_flex().gap(px(6.)).children(project.stacks.iter().take(3).map(|slug| StackIcon::new(slug.clone()))))
                    .child(div().w(px(84.)).text_right().text_size(px(12.)).text_color(theme.ink_3).child(match sessions {
                        0 => "—".to_string(),
                        1 => "1 session".to_string(),
                        n => format!("{n} sessions"),
                    }))
                    .child(
                        h_flex()
                            .w(px(84.))
                            .justify_end()
                            .gap(px(6.))
                            .text_size(px(12.))
                            .text_color(if live { theme.live } else { theme.ink_3 })
                            .when(live, |d| d.child(div().size(px(6.)).rounded_full().bg(theme.live)))
                            .child(if live { "active now".to_string() } else { format!("{} ago", ago(project.updated_ms)) }),
                    )
                    .child(self.editor_buttons(&project.root, 24., cx))
                    .on_click({
                        let root = root.clone();
                        cx.listener(move |this, _, _, cx| this.go(Screen::Project(root.clone()), cx))
                    })
                    .into_any_element()
            })
            .collect();
        v_flex().rounded(px(14.)).border_1().border_color(theme.line_2).overflow_hidden().children(rows).into_any_element()
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

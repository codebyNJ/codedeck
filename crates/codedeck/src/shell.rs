use crate::app_state::AppState;
use crate::screens::{AGENTS, agent_icon, tilde};
use codedeck_core::{Agent, now_ms};
use codedeck_ui::{motion, prelude::*};
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum Screen {
    /// `Some(root)`: only projects inside that scan folder.
    Home(Option<PathBuf>),
    Project(PathBuf),
    Sessions(Scope),
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Scope {
    All,
    Running,
    Loose,
    Agent(Agent),
}

/// One sidebar entry, before layout.
enum Item {
    Header(&'static str),
    Row { icon: Icon, label: SharedString, count: Option<usize>, screen: Screen },
}

enum Icon {
    Path(&'static str),
    Agent(Agent),
    Live,
}

/// The window's one view: sidebar plus the current screen. Screens are
/// `impl Shell` blocks in screens/*, so navigation is just setting `screen`.
pub struct Shell {
    pub state: Entity<AppState>,
    pub screen: Screen,
    pub filter: usize,
    pub sort: usize,
    pub search: Entity<TextInput>,
    pub palette: bool,
    pub palette_input: Entity<TextInput>,
    pub palette_index: usize,
    /// Agent filter on the project and Sessions screens; 0 is All.
    pub agent_filter: usize,
    /// Git history for the open project page, read when the page opens.
    pub history: Option<(PathBuf, Option<codedeck_core::git::History>)>,
    _subscriptions: Vec<Subscription>,
}

const ROW: f32 = 30.;
const STEP: f32 = 32.;
/// Height of a section label ("FOLDERS", "AGENTS").
const LABEL: f32 = 34.;

impl Shell {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&state, |_, _, cx| cx.notify());
        let search = cx.new(|cx| TextInput::new("Search projects", cx));
        let on_search = cx.subscribe(&search, |_, _, _: &InputChanged, cx| cx.notify());
        let palette_input = cx.new(|cx| TextInput::new("Jump to a project or session", cx));
        let on_palette = cx.subscribe(&palette_input, |this, _, _: &InputChanged, cx| {
            this.palette_index = 0;
            cx.notify();
        });
        Self {
            state,
            screen: Screen::Home(None),
            filter: 0,
            sort: 0,
            search,
            palette: false,
            palette_input,
            palette_index: 0,
            agent_filter: 0,
            history: None,
            _subscriptions: vec![observe, on_search, on_palette],
        }
    }

    pub fn go(&mut self, screen: Screen, cx: &mut Context<Self>) {
        if screen != self.screen {
            self.agent_filter = 0;
        }
        if let Screen::Project(root) = &screen
            && self.history.as_ref().is_none_or(|(r, _)| r != root)
        {
            self.history = Some((root.clone(), codedeck_core::git::history(root)));
        }
        self.screen = screen;
        cx.notify();
    }

    fn sidebar_items(&self, cx: &App) -> (Vec<Item>, Vec<Item>) {
        let state = self.state.read(cx);
        let snapshot = &state.snapshot;
        let now = now_ms();
        let all = snapshot.session_count() + snapshot.loose.len();
        let running = snapshot.projects.iter().flat_map(|p| &p.sessions).chain(&snapshot.loose).filter(|s| s.is_live(now)).count();

        let mut top = vec![
            Item::Row { icon: Icon::Path("icons/ui/house.svg"), label: "Home".into(), count: Some(snapshot.projects.len()), screen: Screen::Home(None) },
            Item::Row { icon: Icon::Path("icons/ui/layers.svg"), label: "Sessions".into(), count: Some(all), screen: Screen::Sessions(Scope::All) },
        ];
        if running > 0 {
            top.push(Item::Row { icon: Icon::Live, label: "Running now".into(), count: Some(running), screen: Screen::Sessions(Scope::Running) });
        }
        if !snapshot.loose.is_empty() {
            top.push(Item::Row {
                icon: Icon::Path("icons/ui/circle-dot.svg"),
                label: "Loose sessions".into(),
                count: Some(snapshot.loose.len()),
                screen: Screen::Sessions(Scope::Loose),
            });
        }
        if state.settings.roots.len() > 1 {
            top.push(Item::Header("FOLDERS"));
            for root in &state.settings.roots {
                let count = snapshot.projects.iter().filter(|p| p.root.starts_with(root)).count();
                top.push(Item::Row { icon: Icon::Path("icons/ui/folder.svg"), label: tilde(root).into(), count: Some(count), screen: Screen::Home(Some(root.clone())) });
            }
        }
        let agents: Vec<(Agent, usize)> = AGENTS
            .into_iter()
            .map(|a| (a, snapshot.projects.iter().flat_map(|p| &p.sessions).chain(&snapshot.loose).filter(|s| s.agent == a).count()))
            .filter(|(_, n)| *n > 0)
            .collect();
        if !agents.is_empty() {
            top.push(Item::Header("AGENTS"));
            for (agent, count) in agents {
                top.push(Item::Row { icon: Icon::Agent(agent), label: agent.label().into(), count: Some(count), screen: Screen::Sessions(Scope::Agent(agent)) });
            }
        }
        let bottom = vec![Item::Row { icon: Icon::Path("icons/ui/settings.svg"), label: "Settings".into(), count: None, screen: Screen::Settings }];
        (top, bottom)
    }

    /// A list of rows with one springy highlight behind the selected row.
    fn nav_list(&self, key: &'static str, items: Vec<Item>, cx: &mut Context<Self>) -> Div {
        let theme = cx.theme().clone();
        // A project page keeps its Home row lit.
        let current = match &self.screen {
            Screen::Project(_) => Screen::Home(None),
            other => other.clone(),
        };
        let mut y = 0.;
        let mut selected_top = None;
        let mut children = Vec::new();
        for (i, item) in items.into_iter().enumerate() {
            match item {
                Item::Header(label) => {
                    children.push(
                        div().h(px(LABEL)).pt(px(14.)).px(px(10.)).font_family(MONO).text_size(px(10.5)).text_color(theme.ink_3).child(label).into_any_element(),
                    );
                    y += LABEL;
                }
                Item::Row { icon, label, count, screen } => {
                    let on = screen == current;
                    if on {
                        selected_top = Some(y);
                    }
                    let tint = if on { theme.ink } else { theme.ink_3 };
                    let icon = match icon {
                        Icon::Path(path) => svg().path(path).size(px(15.)).flex_shrink_0().text_color(tint).into_any_element(),
                        Icon::Agent(agent) => agent_icon(agent, 14., cx).into_any_element(),
                        Icon::Live => div().w(px(15.)).flex().justify_center().child(div().size(px(7.)).rounded_full().bg(theme.live)).into_any_element(),
                    };
                    children.push(
                        h_flex()
                            .id(SharedString::from(format!("{key}-{i}")))
                            .h(px(ROW))
                            .mb(px(STEP - ROW))
                            .px(px(10.))
                            .gap(px(9.))
                            .rounded(px(8.))
                            .text_size(px(13.5))
                            .text_color(if on { theme.ink } else { theme.ink_2 })
                            .when(on, |r| r.font_weight(FontWeight::MEDIUM))
                            .child(icon)
                            .child(div().flex_1().min_w_0().truncate().child(label))
                            .when_some(count, |r, n| r.child(div().font_family(MONO).text_size(px(11.)).text_color(theme.ink_3).child(n.to_string())))
                            .on_click(cx.listener(move |this, _, _, cx| this.go(screen.clone(), cx)))
                            .into_any_element(),
                    );
                    y += STEP;
                }
            }
        }
        let height = y;
        let highlight = selected_top.map(|top| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .rounded(px(8.))
                .bg(theme.hi)
                .with_spring(SharedString::from(format!("{key}-top")), SpringAnimation::new(motion::lead()).to(top), |d, t| d.top(px(t)))
                .with_spring(SharedString::from(format!("{key}-bottom")), SpringAnimation::new(motion::trail()).to(height - top - ROW), |d, b| {
                    d.map_element(|d| d.bottom(px(b)))
                })
        });
        div().relative().h(px(height)).children(highlight).children(children)
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (top, bottom) = self.sidebar_items(cx);

        let find = h_flex()
            .id("side-find")
            .h(px(32.))
            .mb(px(14.))
            .px(px(10.))
            .gap(px(8.))
            .rounded(px(9.))
            .border_1()
            .border_color(theme.line_2)
            .bg(theme.card)
            .text_size(px(13.))
            .text_color(theme.ink_3)
            .child(svg().path("icons/ui/search.svg").size(px(14.)).text_color(theme.ink_3))
            .child(div().flex_1().child("Search"))
            .child(div().font_family(MONO).text_size(px(11.)).text_color(theme.ink_4).child("⌘K"))
            .on_click(cx.listener(|this, _, window, cx| this.open_palette(window, cx)));

        let state = self.state.read(cx);
        let status: SharedString = if state.scanning {
            "Scanning…".into()
        } else {
            format!("{} projects · {} sessions", state.snapshot.projects.len(), state.snapshot.session_count() + state.snapshot.loose.len()).into()
        };
        let scanning = state.scanning;

        v_flex()
            .w(px(216.))
            .h_full()
            .flex_shrink_0()
            .bg(theme.side)
            .border_r_1()
            .border_color(theme.line)
            .px(px(10.))
            .pt(px(54.))
            .pb(px(14.))
            .child(
                h_flex()
                    .px(px(8.))
                    .pb(px(16.))
                    .gap(px(9.))
                    .child(Avatar::mascot().size(24.))
                    .child(div().font_family(SERIF).italic().text_size(px(21.)).child("repobox")),
            )
            .child(find)
            .child(div().id("side-scroll").flex_1().min_h_0().overflow_y_scroll().child(self.nav_list("side", top, cx)))
            .child(div().pt(px(8.)).child(self.nav_list("side-b", bottom, cx)))
            .child(
                h_flex()
                    .pt(px(6.))
                    .px(px(10.))
                    .gap(px(7.))
                    .text_size(px(11.5))
                    .text_color(theme.ink_3)
                    .child(div().size(px(6.)).rounded_full().bg(if scanning { theme.amber } else { theme.live }))
                    .child(status),
            )
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let content = match self.screen.clone() {
            Screen::Home(folder) => self.render_home(folder.as_deref(), window, cx),
            Screen::Project(root) => self.render_project(&root, window, cx),
            Screen::Sessions(scope) => self.render_sessions(scope, window, cx),
            Screen::Settings => self.render_settings(window, cx),
        };
        let palette = self.palette.then(|| self.render_palette(cx));
        h_flex()
            .relative()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.ink)
            .font_family(SANS)
            .child(self.render_sidebar(cx))
            .child(div().id("content").flex_1().min_w_0().h_full().overflow_y_scroll().child(content))
            .children(palette)
    }
}

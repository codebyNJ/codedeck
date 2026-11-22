use crate::app_state::AppState;
use codedeck_ui::{motion, prelude::*};
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum Screen {
    Home,
    Project(PathBuf),
    Settings,
}

/// The window's one view: sidebar plus the current screen. Screens are
/// `impl Shell` blocks in screens/*, so navigation is just setting `screen`.
pub struct Shell {
    pub state: Entity<AppState>,
    pub screen: Screen,
    pub filter: usize,
    pub sort: usize,
    pub search: Entity<TextInput>,
    _subscriptions: Vec<Subscription>,
}

const ROW: f32 = 30.;
const STEP: f32 = 32.;
/// Height of the "Recent" label between the nav and the project list.
const LABEL: f32 = 34.;
const RECENT: usize = 6;

impl Shell {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&state, |_, _, cx| cx.notify());
        let search = cx.new(|cx| TextInput::new("Search projects", cx));
        let on_search = cx.subscribe(&search, |_, _, _: &InputChanged, cx| cx.notify());
        Self { state, screen: Screen::Home, filter: 0, sort: 0, search, _subscriptions: vec![observe, on_search] }
    }

    pub fn go(&mut self, screen: Screen, cx: &mut Context<Self>) {
        self.screen = screen;
        cx.notify();
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let recent: Vec<(PathBuf, SharedString)> = self
            .state
            .read(cx)
            .snapshot
            .projects
            .iter()
            .take(RECENT)
            .map(|p| (p.root.clone(), p.name.clone().into()))
            .collect();

        // Row tops are fixed, so the highlight needs no measuring.
        let selected_top = match &self.screen {
            Screen::Home => Some(0.),
            Screen::Settings => Some(STEP),
            Screen::Project(root) => recent.iter().position(|(r, _)| r == root).map(|i| 2. * STEP + LABEL + i as f32 * STEP),
        };
        let height = 2. * STEP + LABEL + recent.len() as f32 * STEP;
        let highlight = selected_top.map(|top| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .rounded(px(8.))
                .bg(theme.hi)
                .with_spring("side-top", SpringAnimation::new(motion::lead()).to(top), |d, t| d.top(px(t)))
                .with_spring("side-bottom", SpringAnimation::new(motion::trail()).to(height - top - ROW), |d, b| {
                    d.map_element(|d| d.bottom(px(b)))
                })
        });

        let row = |id: SharedString, icon: Option<&'static str>, label: SharedString, on: bool| {
            h_flex()
                .id(id)
                .h(px(ROW))
                .mb(px(STEP - ROW))
                .px(px(10.))
                .gap(px(9.))
                .rounded(px(8.))
                .text_size(px(13.5))
                .text_color(if on { theme.ink } else { theme.ink_2 })
                .when(on, |r| r.font_weight(FontWeight::MEDIUM))
                .when_some(icon, |r, icon| r.child(svg().path(icon).size(px(15.)).text_color(if on { theme.ink } else { theme.ink_3 })))
                .child(div().truncate().child(label))
        };

        let nav = div()
            .relative()
            .h(px(height))
            .children(highlight)
            .child(
                row("nav-home".into(), Some("icons/ui/house.svg"), "Home".into(), self.screen == Screen::Home)
                    .on_click(cx.listener(|this, _, _, cx| this.go(Screen::Home, cx))),
            )
            .child(
                row("nav-settings".into(), Some("icons/ui/settings.svg"), "Settings".into(), self.screen == Screen::Settings)
                    .on_click(cx.listener(|this, _, _, cx| this.go(Screen::Settings, cx))),
            )
            .child(
                div()
                    .h(px(LABEL))
                    .pt(px(12.))
                    .px(px(10.))
                    .font_family(MONO)
                    .text_size(px(10.5))
                    .text_color(theme.ink_3)
                    .child("RECENT"),
            )
            .children(recent.into_iter().map(|(root, name)| {
                let on = self.screen == Screen::Project(root.clone());
                row(format!("nav-{}", root.display()).into(), None, name, on)
                    .on_click(cx.listener(move |this, _, _, cx| this.go(Screen::Project(root.clone()), cx)))
            }));

        let state = self.state.read(cx);
        let status: SharedString = if state.scanning {
            "Scanning…".into()
        } else {
            format!("{} projects · {} sessions", state.snapshot.projects.len(), state.snapshot.session_count()).into()
        };

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
                    .pb(px(18.))
                    .gap(px(9.))
                    .child(Avatar::mascot().size(24.))
                    .child(div().text_size(px(15.)).font_weight(FontWeight::SEMIBOLD).child("Codedeck")),
            )
            .child(nav)
            .child(div().flex_1())
            .child(
                h_flex()
                    .px(px(10.))
                    .gap(px(7.))
                    .text_size(px(11.5))
                    .text_color(theme.ink_3)
                    .child(div().size(px(6.)).rounded_full().bg(if self.state.read(cx).scanning { theme.amber } else { theme.live }))
                    .child(status),
            )
    }
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let content = match self.screen.clone() {
            Screen::Home => self.render_home(window, cx),
            Screen::Project(root) => self.render_project(&root, window, cx),
            Screen::Settings => self.render_settings(window, cx),
        };
        h_flex()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.ink)
            .font_family(SANS)
            .child(self.render_sidebar(cx))
            .child(div().id("content").flex_1().min_w_0().h_full().overflow_y_scroll().child(content))
    }
}

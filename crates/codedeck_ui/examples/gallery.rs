//! cargo run -p codedeck_ui --example gallery — every piece, light and dark.

use codedeck_ui::prelude::*;

struct Gallery {
    filter: usize,
    sort: usize,
    toggle: bool,
    clicks: usize,
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let label = |text: &'static str| {
            div().font_family(MONO).text_size(px(11.)).text_color(theme.ink_3).child(text.to_uppercase())
        };
        v_flex()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.ink)
            .font_family(SANS)
            .p(px(40.))
            .pt(px(56.))
            .gap(px(26.))
            .child(
                h_flex()
                    .gap(px(10.))
                    .child(div().text_size(px(30.)).child("Codedeck"))
                    .child(div().font_family(SERIF).italic().text_size(px(30.)).text_color(theme.ink_3).child("design system"))
                    .child(div().flex_1())
                    .child(GelButton::new("theme", if theme.dark { "Light" } else { "Dark" }).paper().on_click(|_, _, cx| Theme::toggle(cx))),
            )
            .child(label("gel buttons"))
            .child(
                h_flex()
                    .gap(px(10.))
                    .child(GelButton::new("primary", format!("Clicked {}", self.clicks)).on_click(cx.listener(|this, _, _, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })))
                    .child(GelButton::new("paper", "Open in terminal").paper().icon("icons/ui/terminal.svg"))
                    .child(GelButton::new("small", "Rescan").small().paper()),
            )
            .child(label("segmented · squish knob"))
            .child(
                h_flex()
                    .gap(px(12.))
                    .child(
                        Segmented::new("filter", ["All", "Rust", "TypeScript", "Go", "Python", "Swift"])
                            .selected(self.filter)
                            .on_select(cx.listener(|this, index, _, cx| {
                                this.filter = *index;
                                cx.notify();
                            })),
                    )
                    .child(Segmented::new("sort", ["Recent", "A–Z", "Most chats"]).small().selected(self.sort).on_select(
                        cx.listener(|this, index, _, cx| {
                            this.sort = *index;
                            cx.notify();
                        }),
                    ))
                    .child(Toggle::new("toggle", self.toggle).on_change(cx.listener(|this, on, _, cx| {
                        this.toggle = *on;
                        cx.notify();
                    }))),
            )
            .child(label("pills · badge · stack · editors · avatars"))
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(Pill::new("main").icon("icons/ui/git-branch.svg"))
                    .child(Pill::new("uncommitted").bold("3").warn())
                    .child(Pill::new("clean"))
                    .child(Badge::running())
                    .child(StackIcon::new("rust"))
                    .child(StackIcon::new("typescript"))
                    .child(StackIcon::new("nextdotjs"))
                    .child(EditorButton::new("cursor", "cursor").default_editor(true))
                    .child(EditorButton::new("vscode", "vscode"))
                    .child(EditorButton::new("zed", "zed"))
                    .child(Avatar::mascot().size(36.))
                    .child(Avatar::project("deadend").size(36.))
                    .child(Avatar::project("codedeck").size(36.)),
            )
            .child(label("card"))
            .child(
                h_flex().gap(px(12.)).child(div().w(px(420.)).child(
                    Card::new("card").live(true).child(
                        h_flex()
                            .gap(px(12.))
                            .child(Avatar::project("codedeck"))
                            .child(
                                v_flex()
                                    .child(div().text_size(px(17.)).child("codedeck"))
                                    .child(div().font_family(MONO).text_size(px(11.5)).text_color(theme.ink_3).child("~/codes/codedeck")),
                            ),
                    ),
                )),
            )
    }
}

fn main() {
    env_logger::init();
    gpui_kit::application().with_assets(codedeck_ui::Assets).run(|cx| {
        gpui_kit::init(cx);
        codedeck_ui::init(false, cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                appears_transparent: true,
                traffic_light_position: Some(point(px(16.), px(16.))),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1080.), px(640.)), cx))),
            ..Default::default()
        };
        let opened = gpui_kit::open_window(options, cx, |_, cx| cx.new(|_| Gallery { filter: 0, sort: 0, toggle: true, clicks: 0 }));
        if let Err(error) = opened {
            log::error!("{error:#}");
        }
        cx.activate(true);
    });
}

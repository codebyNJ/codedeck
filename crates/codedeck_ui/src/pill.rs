use crate::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Soft,
    Warn,
}

/// Tight 20px filled pill: branch, "3 uncommitted", "clean".
#[derive(IntoElement)]
pub struct Pill {
    label: SharedString,
    bold: Option<SharedString>,
    icon: Option<SharedString>,
    tone: Tone,
}

impl Pill {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into(), bold: None, icon: None, tone: Tone::Soft }
    }
    pub fn warn(mut self) -> Self {
        self.tone = Tone::Warn;
        self
    }
    /// A bold lead, e.g. the "3" in "3 uncommitted".
    pub fn bold(mut self, text: impl Into<SharedString>) -> Self {
        self.bold = Some(text.into());
        self
    }
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }
}

impl RenderOnce for Pill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let (fill, ink) = match self.tone {
            Tone::Soft => (theme.soft, theme.ink_2),
            Tone::Warn => (theme.warn_bg, theme.amber),
        };
        h_flex()
            .h(px(20.))
            .px(px(7.))
            .gap(px(4.))
            .rounded(px(6.))
            .bg(fill)
            .text_color(ink)
            .text_size(px(11.5))
            .whitespace_nowrap()
            .when_some(self.icon, |pill, icon| pill.child(svg().path(icon).size(px(10.)).text_color(ink)))
            .when_some(self.bold, |pill, bold| pill.child(div().font_weight(FontWeight::SEMIBOLD).child(bold)))
            .child(self.label)
    }
}

/// Solid status badge that sits on a card's outline ("Running").
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
}

impl Badge {
    pub fn running() -> Self {
        Self { label: "Running".into() }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        h_flex()
            .h(px(20.))
            .pl(px(7.))
            .pr(px(8.))
            .gap(px(5.))
            .rounded(px(6.))
            .bg(theme.live)
            // The prototype's 3px page-coloured ring that cuts the outline.
            .border_3()
            .border_color(theme.bg)
            .text_color(gpui_kit::white())
            .text_size(px(11.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(
                div().size(px(6.)).rounded_full().bg(gpui_kit::white()).with_animation(
                    "pulse",
                    Animation::new(std::time::Duration::from_millis(1600)).repeat().with_easing(pulsating_between(0.3, 1.0)),
                    |dot, t| dot.opacity(t),
                ),
            )
            .child(self.label)
    }
}

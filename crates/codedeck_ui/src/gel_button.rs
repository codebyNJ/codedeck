use crate::{motion, prelude::*};
use std::rc::Rc;

type OnClick = crate::Handler<ClickEvent>;

/// Hover and press, kept per element so RenderOnce pieces can spring.
#[derive(Default, Clone, Copy)]
pub(crate) struct Touch {
    pub hover: bool,
    pub press: bool,
}

impl Touch {
    /// Spring target: pressed squashes in, hover swells out, release overshoots (the jelly).
    pub fn target(self) -> f32 {
        if self.press { 1.0 } else if self.hover { -0.45 } else { 0.0 }
    }
}

/// Wires hover/press onto `element` and returns the current touch state.
pub(crate) fn touchable(
    id: &ElementId,
    element: Stateful<Div>,
    window: &mut Window,
    cx: &mut App,
) -> (Stateful<Div>, Touch) {
    let state = window.use_keyed_state(id.clone(), cx, |_, _| Touch::default());
    let touch = *state.read(cx);
    let (hover, down, up, out) = (state.clone(), state.clone(), state.clone(), state);
    let element = element
        .on_hover(move |hovered, _, cx| hover.update(cx, |t, cx| { t.hover = *hovered; t.press &= *hovered; cx.notify() }))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| down.update(cx, |t, cx| { t.press = true; cx.notify() }))
        .on_mouse_up(MouseButton::Left, move |_, _, cx| up.update(cx, |t, cx| { t.press = false; cx.notify() }))
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| out.update(cx, |t, cx| { t.press = false; cx.notify() }));
    (element, touch)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tone {
    Ink,
    Paper,
}

/// The prototype's gel button: swells on hover, squashes on press, jellies on release.
/// GPUI can't scale a div, so the fill is its own layer whose insets spring.
#[derive(IntoElement)]
pub struct GelButton {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    tone: Tone,
    small: bool,
    on_click: Option<OnClick>,
}

impl GelButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), icon: None, tone: Tone::Ink, small: false, on_click: None }
    }
    pub fn paper(mut self) -> Self {
        self.tone = Tone::Paper;
        self
    }
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
    /// Asset path, e.g. "icons/ui/terminal.svg".
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for GelButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (fill, ink, edge) = match self.tone {
            Tone::Ink => (theme.ink, theme.bg, None),
            Tone::Paper => (theme.card, theme.ink, Some(theme.line_2)),
        };
        let height = if self.small { 28. } else { 34. };
        let base = div().id(self.id.clone());
        let (button, touch) = touchable(&self.id, base, window, cx);
        let layer = div()
            .absolute()
            .rounded(px(10.))
            .bg(fill)
            .when_some(edge, |layer, edge| layer.border_1().border_color(edge))
            .with_spring(
                crate::child_id(&self.id, "gel"),
                SpringAnimation::new(motion::jelly()).to(touch.target()),
                // Squash more vertically than horizontally, like scale(.94, .88).
                |layer, v| layer.left(px(3.2 * v)).right(px(3.2 * v)).top(px(2.6 * v)).bottom(px(2.6 * v)),
            );
        let on_click = self.on_click;
        button
            .relative()
            .h(px(height))
            .px(px(if self.small { 11. } else { 14. }))
            .flex()
            .items_center()
            .gap(px(7.))
            .text_size(px(13.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(ink)
            .child(layer)
            .when_some(self.icon, |button, icon| {
                button.child(svg().path(icon).size(px(14.)).text_color(ink).relative())
            })
            .child(div().relative().child(self.label))
            .when_some(on_click, |button, handler| button.on_click(move |event, window, cx| handler(event, window, cx)))
    }
}

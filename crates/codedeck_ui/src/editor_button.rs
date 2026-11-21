use crate::{gel_button::touchable, motion, prelude::*};
use std::rc::Rc;

/// Round "Open with" button: the editor's colour logo, a green dot when it's
/// the default, jelly on press.
#[derive(IntoElement)]
pub struct EditorButton {
    id: ElementId,
    icon: SharedString,
    is_default: bool,
    size: f32,
    on_click: Option<crate::Handler<ClickEvent>>,
}

impl EditorButton {
    /// `icon` is the file stem under icons/ide, e.g. "cursor".
    pub fn new(id: impl Into<ElementId>, icon: &str) -> Self {
        Self { id: id.into(), icon: format!("icons/ide/{icon}.svg").into(), is_default: false, size: 28., on_click: None }
    }
    pub fn default_editor(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for EditorButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let size = self.size;
        let (button, touch) = touchable(&self.id, div().id(self.id.clone()), window, cx);
        let ring = div()
            .absolute()
            .rounded_full()
            .bg(theme.card)
            .border_1()
            .border_color(if touch.hover { theme.ink_4 } else { theme.line_2 })
            .with_spring(
                crate::child_id(&self.id, "ring"),
                SpringAnimation::new(motion::jelly()).to(touch.target()),
                |ring, v| ring.left(px(2.4 * v)).right(px(2.4 * v)).top(px(2.4 * v)).bottom(px(2.4 * v)),
            );
        let on_click = self.on_click;
        button
            .relative()
            .size(px(size))
            .flex()
            .items_center()
            .justify_center()
            .child(ring)
            .child(img(self.icon).size(px(size * 0.5)).relative())
            .when(self.is_default, |button| {
                button.child(
                    div()
                        .absolute()
                        .right(px(-1.))
                        .bottom(px(-1.))
                        .size(px(8.))
                        .rounded_full()
                        .bg(theme.live)
                        .border_2()
                        .border_color(theme.card),
                )
            })
            .when_some(on_click, |button, handler| button.on_click(move |event, window, cx| handler(event, window, cx)))
    }
}

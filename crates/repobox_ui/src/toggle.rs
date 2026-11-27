use crate::{motion, prelude::*};
use std::rc::Rc;

/// Springy switch. The caller owns the value.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    on: bool,
    on_change: Option<crate::Handler<bool>>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
        Self { id: id.into(), on, on_change: None }
    }
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let on = self.on;
        let knob = div()
            .absolute()
            .top(px(2.))
            .size(px(18.))
            .rounded_full()
            .bg(gpui_kit::white())
            .with_spring(crate::child_id(&self.id, "knob"), SpringAnimation::new(motion::jelly()).to(if on { 16. } else { 2. }), |k, x| {
                k.left(px(x))
            });
        let on_change = self.on_change;
        div()
            .id(self.id)
            .relative()
            .w(px(36.))
            .h(px(22.))
            .rounded_full()
            .bg(if on { theme.live } else { theme.ink_4 })
            .child(knob)
            .when_some(on_change, |toggle, handler| toggle.on_click(move |_, window, cx| handler(&!on, window, cx)))
    }
}

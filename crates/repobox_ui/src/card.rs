use crate::prelude::*;

/// Card container: hairline, 20px radius, 16px padding. A running card gets a
/// green-tinted outline and a badge riding on its top edge.
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    live: bool,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), live: false, children: Vec::new() }
    }
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let edge = if self.live { theme.live.blend(theme.line_2.opacity(0.55)) } else { theme.line_2 };
        // GPUI paints a border above its own children, so the badge rides on a
        // sibling layer to sit over the outline instead of under it.
        div()
            .id(self.id)
            .relative()
            .child(
                v_flex()
                    .p(px(16.))
                    .rounded(px(20.))
                    .bg(theme.card)
                    .border_1()
                    .border_color(edge)
                    .children(self.children),
            )
            .when(self.live, |card| card.child(div().absolute().top(px(-11.)).right(px(16.)).child(Badge::running())))
    }
}

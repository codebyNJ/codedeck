use crate::{motion, prelude::*};
use std::rc::Rc;

#[derive(Default)]
struct Measured {
    /// Each option's (left, right), relative to the control's padding box.
    edges: Vec<(Pixels, Pixels)>,
    width: Pixels,
    last: usize,
    moving_right: bool,
}

const PAD: f32 = 3.;

/// Segmented control with the squish knob: the leading edge springs fast and
/// the trailing edge slow, so the knob stretches toward its target and settles.
#[derive(IntoElement)]
pub struct Segmented {
    id: ElementId,
    options: Vec<SharedString>,
    selected: usize,
    small: bool,
    on_select: Option<crate::Handler<usize>>,
}

impl Segmented {
    pub fn new(id: impl Into<ElementId>, options: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self { id: id.into(), options: options.into_iter().map(Into::into).collect(), selected: 0, small: false, on_select: None }
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Segmented {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| Measured::default());
        let selected = self.selected;
        state.update(cx, |m, _| {
            if m.last != selected {
                m.moving_right = selected > m.last;
                m.last = selected;
            }
        });
        let measured = state.read(cx);
        let knob = measured.edges.get(selected).copied().map(|(left, right)| {
            let (left_cfg, right_cfg) =
                if measured.moving_right { (motion::trail(), motion::lead()) } else { (motion::lead(), motion::trail()) };
            let right_inset = measured.width - right;
            div()
                .absolute()
                .top(px(PAD))
                .bottom(px(PAD))
                .rounded(px(8.))
                .bg(theme.card)
                .border_1()
                .border_color(theme.line_2)
                // Overshoot may squeeze the knob but never push it out of the track.
                .with_spring(crate::child_id(&self.id, "l"), SpringAnimation::new(left_cfg).to(left + px(PAD)), |k, l| {
                    k.left(l.max(px(PAD)))
                })
                .with_spring(crate::child_id(&self.id, "r"), SpringAnimation::new(right_cfg).to(right_inset + px(PAD)), |k, r| {
                    k.map_element(|k| k.right(r.max(px(PAD))))
                })
        });

        let items = self.options.iter().enumerate().map(|(index, label)| {
            let on_select = self.on_select.clone();
            div()
                .id(crate::child_id(&self.id, index.to_string()))
                .px(px(if self.small { 10. } else { 12. }))
                .h(px(if self.small { 24. } else { 28. }))
                .flex()
                .items_center()
                .text_size(px(if self.small { 12. } else { 13. }))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if index == selected { theme.ink } else { theme.ink_3 })
                .child(label.clone())
                .when_some(on_select, |item, handler| item.on_click(move |_, window, cx| handler(&index, window, cx)))
        });

        let measure = state.clone();
        let row = h_flex().relative().children(items).on_children_prepainted(move |bounds, window, cx| {
            let Some(first) = bounds.first() else { return };
            let edges: Vec<_> = bounds
                .iter()
                .map(|b| (b.origin.x - first.origin.x, b.origin.x - first.origin.x + b.size.width))
                .collect();
            let width = edges.last().map_or(px(0.), |e| e.1);
            if measure.read(cx).edges != edges {
                measure.update(cx, |m, _| {
                    m.edges = edges;
                    m.width = width;
                });
                window.refresh();
            }
        });

        div()
            .relative()
            .flex()
            .p(px(PAD))
            .rounded(px(11.))
            .bg(theme.segbg)
            .children(knob)
            .child(row)
    }
}

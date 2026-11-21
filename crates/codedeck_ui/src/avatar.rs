use crate::prelude::*;

/// Body colours for project bots; picked by a hash of the name so a project
/// keeps its colour forever.
const BODIES: &[u32] = &[0xe45e4d, 0x4e6a88, 0x2ea56a, 0xd9a03a, 0x8a63c9, 0x3a9fb4, 0xd2668f, 0x6b7d4a];

/// The robot face from the mascot: body, dark visor with a lit slit, grill.
/// `name = None` draws the red Codedeck mascot itself.
#[derive(IntoElement)]
pub struct Avatar {
    name: Option<SharedString>,
    image: Option<std::path::PathBuf>,
    size: f32,
}

impl Avatar {
    pub fn project(name: impl Into<SharedString>) -> Self {
        Self { name: Some(name.into()), image: None, size: 44. }
    }
    pub fn mascot() -> Self {
        Self { name: None, image: None, size: 44. }
    }
    /// A cached DiceBear SVG; used when the file exists, else the drawn bot.
    pub fn image(mut self, path: std::path::PathBuf) -> Self {
        self.image = path.exists().then_some(path);
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

fn hash(text: &str) -> usize {
    text.bytes().fold(5381usize, |h, b| h.wrapping_mul(33) ^ b as usize)
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let s = self.size;
        if let Some(path) = self.image {
            return img(path).size(px(s)).flex_shrink_0().rounded(px(s * 0.29)).into_any_element();
        }
        if self.name.is_none() {
            return img("art/codedeck-mascot.svg").size(px(s)).flex_shrink_0().rounded(px(s * 0.225)).into_any_element();
        }
        let body: Hsla = rgb(match &self.name {
            Some(name) => BODIES[hash(name) % BODIES.len()],
            None => 0xe45e4d,
        })
        .into();
        let visor: Hsla = rgb(0x17171a).into();
        let bar = || div().w(px(s * 0.06)).h(px(s * 0.16)).rounded_full().bg(visor);
        v_flex()
            .size(px(s))
            .flex_shrink_0()
            .rounded(px(s * 0.29))
            .bg(body)
            .items_center()
            .pt(px(s * 0.27))
            .gap(px(s * 0.12))
            .child(
                div()
                    .w(px(s * 0.72))
                    .h(px(s * 0.24))
                    .rounded_full()
                    .bg(visor)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(div().w(px(s * 0.42)).h(px(s * 0.055)).rounded_full().bg(body)),
            )
            .child(h_flex().gap(px(s * 0.07)).children((0..4).map(|_| bar())))
            .into_any_element()
    }
}

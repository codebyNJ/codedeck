use crate::prelude::*;

/// Brand colour per simple-icons slug; unknown slugs use ink.
fn brand(slug: &str) -> Option<u32> {
    Some(match slug {
        "rust" => 0xce422b,
        "go" => 0x00add8,
        "python" => 0x3776ab,
        "ruby" => 0xcc342d,
        "flutter" => 0x02569b,
        "swift" => 0xf05138,
        "kotlin" => 0x7f52ff,
        "php" => 0x777bb4,
        "elixir" => 0x4b275f,
        "javascript" => 0xd9b400,
        "typescript" => 0x3178c6,
        "vite" => 0x646cff,
        "svelte" => 0xff3e00,
        "astro" => 0xbc52ee,
        "tailwindcss" => 0x06b6d4,
        _ => return None, // nextdotjs, deno, openjdk, gradle: monochrome marks
    })
}

#[derive(IntoElement)]
pub struct StackIcon {
    slug: SharedString,
    size: f32,
}

impl StackIcon {
    pub fn new(slug: impl Into<SharedString>) -> Self {
        Self { slug: slug.into(), size: 14. }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for StackIcon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color: Hsla = brand(&self.slug).map_or(cx.theme().ink, |hex| rgb(hex).into());
        svg().path(format!("icons/stack/{}.svg", self.slug)).size(px(self.size)).flex_shrink_0().text_color(color)
    }
}

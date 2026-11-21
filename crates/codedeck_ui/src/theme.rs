use gpui_kit::{App, Global, Hsla, rgb, rgba};

/// The prototype's CSS tokens (29-prototype-v22.html), light and dark.
#[derive(Clone, Debug)]
pub struct Theme {
    pub dark: bool,
    pub bg: Hsla,
    pub card: Hsla,
    pub side: Hsla,
    pub soft: Hsla,
    pub soft_2: Hsla,
    pub hi: Hsla,
    pub segbg: Hsla,
    pub ink: Hsla,
    pub ink_2: Hsla,
    pub ink_3: Hsla,
    pub ink_4: Hsla,
    pub line: Hsla,
    pub line_2: Hsla,
    pub live: Hsla,
    pub live_bg: Hsla,
    pub live_fg: Hsla,
    pub amber: Hsla,
    pub warn_bg: Hsla,
    pub red: Hsla,
}

impl Global for Theme {}

impl Theme {
    pub fn light() -> Self {
        Self {
            dark: false,
            bg: rgb(0xffffff).into(),
            card: rgb(0xffffff).into(),
            side: rgb(0xfafaf9).into(),
            soft: rgb(0xf5f5f3).into(),
            soft_2: rgb(0xededea).into(),
            hi: rgb(0xececea).into(),
            segbg: rgba(0x0e0e0f0e).into(),
            ink: rgb(0x111112).into(),
            ink_2: rgb(0x4f4f55).into(),
            ink_3: rgb(0x8a8a90).into(),
            ink_4: rgb(0xc4c4c7).into(),
            line: rgb(0xebebe8).into(),
            line_2: rgb(0xdfdfdb).into(),
            live: rgb(0x2ea56a).into(),
            live_bg: rgb(0xe7f5ed).into(),
            live_fg: rgb(0x1f744a).into(),
            amber: rgb(0x9a5c17).into(),
            warn_bg: rgb(0xfbf0e1).into(),
            red: rgb(0xe45e4d).into(),
        }
    }

    pub fn dark() -> Self {
        Self {
            dark: true,
            bg: rgb(0x121214).into(),
            card: rgb(0x19191c).into(),
            side: rgb(0x151517).into(),
            soft: rgb(0x202024).into(),
            soft_2: rgb(0x2a2a2f).into(),
            hi: rgb(0x232327).into(),
            segbg: rgba(0xffffff0f).into(),
            ink: rgb(0xf2f2f3).into(),
            ink_2: rgb(0xb5b5bb).into(),
            ink_3: rgb(0x7f7f87).into(),
            ink_4: rgb(0x4b4b52).into(),
            line: rgb(0x24242a).into(),
            line_2: rgb(0x313138).into(),
            live: rgb(0x2ea56a).into(),
            live_bg: rgb(0x15301f).into(),
            live_fg: rgb(0x6ad59a).into(),
            amber: rgb(0xe3a55c).into(),
            warn_bg: rgb(0x33261a).into(),
            red: rgb(0xe45e4d).into(),
        }
    }

    pub fn toggle(cx: &mut App) {
        let next = if cx.global::<Theme>().dark { Self::light() } else { Self::dark() };
        cx.set_global(next);
        cx.refresh_windows();
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

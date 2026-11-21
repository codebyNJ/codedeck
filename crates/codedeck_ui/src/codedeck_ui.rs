//! Codedeck's design system: tokens, fonts, motion and the pieces screens are built from.

pub mod avatar;
pub mod card;
pub mod editor_button;
pub mod gel_button;
pub mod motion;
pub mod pill;
pub mod segmented;
pub mod stack_icon;
pub mod theme;
pub mod toggle;
pub mod typography;

pub mod prelude {
    pub use crate::{
        avatar::Avatar,
        card::Card,
        editor_button::EditorButton,
        gel_button::GelButton,
        pill::{Badge, Pill},
        segmented::Segmented,
        stack_icon::StackIcon,
        theme::{ActiveTheme, Theme},
        toggle::Toggle,
        typography::{MONO, SANS, SERIF},
    };
    pub use gpui_kit::base::{h_flex, v_flex};
    pub use gpui_kit::prelude::*;
    pub use gpui_kit::*;
}

/// A stable child id under `parent`, for springs and keyed state.
pub fn child_id(parent: &gpui_kit::ElementId, name: impl Into<gpui_kit::SharedString>) -> gpui_kit::ElementId {
    gpui_kit::ElementId::NamedChild(std::sync::Arc::new(parent.clone()), name.into())
}

use gpui_kit::{App, AssetSource, Result, SharedString};
use std::borrow::Cow;

/// Everything under /assets, embedded in the binary.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter().filter(|p| p.starts_with(path)).map(SharedString::from).collect())
    }
}

/// Fonts and the theme global. Call once, after `gpui_kit::init`.
pub fn init(dark: bool, cx: &mut App) {
    typography::load_fonts(cx);
    cx.set_global(if dark { theme::Theme::dark() } else { theme::Theme::light() });
}

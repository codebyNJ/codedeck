use crate::Assets;
use gpui_kit::App;

pub const SANS: &str = "Instrument Sans";
/// Accent words only, always italic.
pub const SERIF: &str = "Fraunces";
pub const MONO: &str = "Geist Mono";

pub(crate) fn load_fonts(cx: &mut App) {
    let fonts = Assets::iter()
        .filter(|path| path.starts_with("fonts/"))
        .filter_map(|path| Assets::get(&path).map(|file| file.data))
        .collect();
    if let Err(error) = cx.text_system().add_fonts(fonts) {
        log::error!("fonts: {error:#}");
    }
}

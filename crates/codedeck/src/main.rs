mod actions;
mod app_state;
mod screens;
mod shell;

use app_state::AppState;
use codedeck_ui::prelude::*;
use shell::Shell;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    gpui_kit::application().with_assets(codedeck_ui::Assets).run(|cx| {
        gpui_kit::init(cx);
        let settings = codedeck_core::Settings::load();
        codedeck_ui::init(settings.theme == codedeck_core::settings::Theme::Dark, cx);
        actions::install(cx);

        let state = cx.new(|cx| AppState::new(settings, cx));
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                appears_transparent: true,
                traffic_light_position: Some(point(px(18.), px(18.))),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1180.), px(760.)), cx))),
            window_min_size: Some(size(px(880.), px(560.))),
            ..Default::default()
        };
        if let Err(error) = gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Shell::new(state, window, cx))) {
            log::error!("couldn't open the window: {error:#}");
            cx.quit();
        }
        cx.activate(true);
    });
}

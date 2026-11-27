mod actions;
mod app_state;
mod screens;
mod shell;

use app_state::AppState;
use repobox_ui::prelude::*;
use shell::Shell;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    gpui_kit::application().with_assets(repobox_ui::Assets).run(|cx| {
        gpui_kit::init(cx);
        let settings = repobox_core::Settings::load();
        repobox_ui::init(settings.theme == repobox_core::settings::Theme::Dark, cx);
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
        match gpui_kit::open_window(options, cx, |_, cx| cx.new(|cx| Shell::new(state, cx))) {
            Ok((_, shell)) => actions::route_to(shell.downgrade(), cx),
            Err(error) => {
                log::error!("couldn't open the window: {error:#}");
                cx.quit();
            }
        }
        cx.activate(true);
    });
}

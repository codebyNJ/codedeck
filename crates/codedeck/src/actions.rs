use crate::shell::{Screen, Shell};
use codedeck_ui::prelude::*;

actions!(codedeck, [Quit, Rescan, GoHome, OpenSettings, ToggleTheme, FocusSearch, OpenPalette]);

/// Keybindings and the native menu bar. Shell handles the view actions.
pub fn install(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    codedeck_ui::text_input::bind_keys(cx);
    crate::screens::palette::bind_keys(cx);
    cx.bind_keys([
        KeyBinding::new("cmd-f", FocusSearch, None),
        KeyBinding::new("cmd-k", OpenPalette, None),
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-r", Rescan, None),
        KeyBinding::new("cmd-1", GoHome, None),
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-shift-l", ToggleTheme, None),
    ]);
    cx.set_menus([
        Menu::new("Codedeck").items([
            MenuItem::action("Settings…", OpenSettings),
            MenuItem::separator(),
            MenuItem::action("Quit Codedeck", Quit),
        ]),
        Menu::new("View").items([
            MenuItem::action("Home", GoHome),
            MenuItem::action("Search Projects", FocusSearch),
            MenuItem::action("Jump To…", OpenPalette),
            MenuItem::action("Rescan", Rescan),
            MenuItem::action("Toggle Dark Mode", ToggleTheme),
        ]),
    ]);
    // Closing the only window ends the app, like most single-window Mac utilities.
    cx.on_window_closed(|cx, _| cx.quit()).detach();
}

/// App-wide handlers, so shortcuts and menus work whatever has focus
/// (clicking a plain element clears focus, which would strand view-level handlers).
pub fn route_to(shell: WeakEntity<Shell>, cx: &mut App) {
    let go = |screen: Screen| {
        let shell = shell.clone();
        move |cx: &mut App| {
            shell.update(cx, |shell, cx| shell.go(screen.clone(), cx)).ok();
        }
    };
    let home = go(Screen::Home);
    let settings = go(Screen::Settings);
    cx.on_action(move |_: &GoHome, cx| home(cx));
    cx.on_action(move |_: &OpenSettings, cx| settings(cx));
    let search = shell.clone();
    cx.on_action(move |_: &FocusSearch, cx| {
        let Some(input) = search.update(cx, |shell, cx| {
            shell.go(Screen::Home, cx);
            shell.search.clone()
        }).ok() else { return };
        let handle = input.read(cx).focus_handle(cx);
        let Some(window) = cx.active_window().or_else(|| cx.windows().first().copied()) else { return };
        // Mid-dispatch the window is busy; focus once it's free.
        cx.defer(move |cx| {
            window.update(cx, |_, window, cx| window.focus(&handle, cx)).ok();
        });
    });
    let palette = shell.clone();
    cx.on_action(move |_: &OpenPalette, cx| {
        let Some(window) = cx.active_window().or_else(|| cx.windows().first().copied()) else { return };
        // A key binding fires while the window is mid-dispatch; open once it's free.
        let palette = palette.clone();
        cx.defer(move |cx| {
            if let Err(error) = window.update(cx, |_, window, cx| palette.update(cx, |shell, cx| shell.open_palette(window, cx))) {
                log::warn!("palette: {error:#}");
            }
        });
    });
    let rescan = shell.clone();
    cx.on_action(move |_: &Rescan, cx| {
        rescan.update(cx, |shell, cx| shell.state.update(cx, |state, cx| state.rescan(cx))).ok();
    });
    let theme = shell;
    cx.on_action(move |_: &ToggleTheme, cx| {
        theme.update(cx, |shell, cx| shell.toggle_theme(cx)).ok();
    });
}

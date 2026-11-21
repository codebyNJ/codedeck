use crate::shell::{Screen, Shell};
use codedeck_ui::prelude::*;

actions!(codedeck, [Quit, Rescan, GoHome, OpenSettings, ToggleTheme]);

/// Keybindings and the native menu bar. Shell handles the view actions.
pub fn install(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.bind_keys([
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
    let rescan = shell.clone();
    cx.on_action(move |_: &Rescan, cx| {
        rescan.update(cx, |shell, cx| shell.state.update(cx, |state, cx| state.rescan(cx))).ok();
    });
    let theme = shell;
    cx.on_action(move |_: &ToggleTheme, cx| {
        theme.update(cx, |shell, cx| shell.toggle_theme(cx)).ok();
    });
}

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

# Codedeck

A macOS app that maps every codebase on your Mac and the Claude Code, OpenCode and Codex sessions that belong to each one. Built in Rust on GPUI (via `gpui-kit`).

```sh
cargo run -p codedeck        # the app
cargo test -p codedeck_core  # logic tests
```

- `crates/codedeck`: the app (window, menus, actions, screens)
- `crates/codedeck_ui`: the design system (theme, springs, gel buttons, squish knob, cards)
- `crates/codedeck_core`: plain Rust, no UI (scan, agent session readers, matching, cache, watcher, launch)
- `design/`: HTML prototypes; `29-prototype-v22.html` is the reference
- `docs/gpui-architecture.md`: architecture and conventions

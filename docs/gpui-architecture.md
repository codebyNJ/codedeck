# Codedeck on GPUI: architecture and folder structure (proposal v1)

Status: **proposal for review, nothing built yet.** It replaces the Tauri plan in `architecture.md`. The domain sections of that doc still hold (scanner, session sources, matching, launcher, licensing). This doc changes the UI layer and the code layout.

Design source of truth: `design/29-prototype-v22.html`. White first, dark mode second, flat (no drop shadows), jelly motion, the original red robot mascot, and the pixel film.

---

## 1. What the research says (Nov 2026)

| Option | State today | Verdict |
|---|---|---|
| **`gpui-kit` 0.10 (Longbridge)** | One crate that pins a matching GPUI snapshot (`gpui-pre =0.3.12`) and adds `gpui-base`, which is unstyled behavior: focus, inputs, virtual list, scrollbars, popovers, motion helpers. The styled `gpui-component` layer is an optional feature. It ships Longbridge Pro, a real trading desktop app, and released 2026-11-16. | **Pick.** Use it with `default-features = false`, so we get GPUI plus behavior and own every pixel of the presentation. That matches their rule: "Behavior belongs to the foundation. Presentation belongs to the application." |
| `gpui` 0.2.2 (Zed, crates.io) | Upstream, pre-1.0, developed for Zed's needs, with frequent breaking changes. | Fallback. It's what `gpui-kit` wraps anyway. |
| `gpui-ce` 0.3 | Community fork with its own release cadence, about 28k downloads. | Watch it. Too young to bet on. |

Things the research flagged that shape the plan:

1. **No CSS-style `transform: scale()` on `div`.** In GPUI, transforms exist only on `svg()` elements (`with_transformation`). The jelly and wobble on gel buttons therefore has to be built either by animating size and position, or by a small custom `Element` that paints a scaled background quad. **This is spike #1**, because the design depends on it.
2. **Easing is just `Fn(f32) -> f32`.** There's no built-in spring, so we write `spring()` ourselves in about 5 lines. Overshoot past 1.0 works fine with linear interpolation.
3. **macOS window chrome is supported.** `TitlebarOptions { appears_transparent, traffic_light_position }` and `WindowBackgroundAppearance::Blurred` (NSVisualEffectView) both exist.
4. **Zed's own conventions** (from its `AGENTS.md`) are the house style for GPUI code, so we adopt them: no `mod.rs`, a named crate root, no `unwrap()`, full-word names, `cx` last, `window` before `cx`.

Sources: [gpui-kit](https://github.com/longbridge/gpui-kit) · [gpui-kit ARCHITECTURE.md](https://raw.githubusercontent.com/longbridge/gpui-kit/main/docs/ARCHITECTURE.md) · [Zed GPUI README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md) · [Zed AGENTS.md](https://github.com/zed-industries/zed/blob/main/AGENTS.md) · [GPUI-CE](https://gpui-ce.github.io/) · [gpui animation.rs](https://docs.rs/crate/gpui/0.2.2/source/src/elements/animation.rs) · [Transparency & blur PR #9610](https://git.secluded.site/zed/commit/49144d94bf0f34c95d18d89c5e9bb18ac90bf08c) · [Loungy (archived GPUI launcher)](https://github.com/MatthiasGrandl/Loungy)

---

## 2. Shape: three crates, one direction

```
codedeck (app binary)  ──►  codedeck_ui (design system)  ──►  gpui-kit (GPUI + gpui-base)
        │
        └──────────────►  codedeck_core (no GPUI: scan, sessions, index, launch, license)
```

- **`codedeck_core` has no UI dependency.** It's plain Rust you can unit test, run from a CLI, and learn Rust on. It's the part that will change most often (agent formats drift).
- **`codedeck_ui` owns the look:** tokens, fonts, spring and squish motion, gel buttons, seg, pills, toggles, cards, editor circles. It has a gallery example so motion can be tuned without clicking through the app.
- **`codedeck` is the app:** window, menus, actions, app state, and the screens. Screens compose `codedeck_ui` pieces and read `codedeck_core` data.

Dependencies only point down. `codedeck_ui` never imports `codedeck_core`, and `codedeck_core` never imports GPUI.

Why three crates and not one: core gets fast tests and compiles without GPUI, and the gallery needs its own crate. Why not more: anything finer is ceremony at this size. Zed has about 200 crates, but it's a different product.

---

## 3. Folder structure

```
codedeck/
├── Cargo.toml                    # workspace: members, shared deps, lints, profiles
├── Cargo.lock
├── rust-toolchain.toml           # pins stable Rust (GPUI needs latest stable)
├── rustfmt.toml                  # max_width = 100, imports_granularity = "Crate"
├── .gitignore
├── README.md
│
├── crates/
│   ├── codedeck/                 # ── the app (binary) ─────────────────────────
│   │   ├── Cargo.toml            # [[bin]] name = "Codedeck", path = "src/main.rs"
│   │   ├── build.rs              # macOS: embeds Info.plist bits, rerun on assets
│   │   └── src/
│   │       ├── main.rs           # Application::run → fonts, theme, menus, open window
│   │       ├── app_state.rs      # Entity<AppState>: projects, sessions, scan status; talks to core
│   │       ├── actions.rs        # actions!(codedeck, [OpenPalette, Rescan, OpenSettings, …]) + keymap
│   │       ├── menus.rs          # native macOS menu bar
│   │       ├── assets.rs         # AssetSource impl over the embedded /assets folder
│   │       ├── window.rs         # WindowOptions: size, transparent titlebar, traffic lights
│   │       ├── shell.rs          # root view: sidebar + routed screen + transitions
│   │       ├── sidebar.rs        # nav with squish highlight, theme seg
│   │       ├── screens.rs        # pub enum Screen + mod declarations
│   │       └── screens/
│   │           ├── onboarding.rs         # sequencer: film → scan → setup → home
│   │           ├── onboarding/
│   │           │   ├── film.rs           # pixel meadow + bot + lights (canvas paint)
│   │           │   ├── scan.rs           # kshvbgde pixel bar, legend, path scramble
│   │           │   └── setup.rs          # folders / agents / editors sentences
│   │           ├── home.rs               # painting banner, continue tiles, project grid
│   │           ├── project_card.rs       # the v22 card (running badge, filled pills, editors)
│   │           ├── project.rs            # single project page: sessions by day, "used here"
│   │           ├── sessions.rs           # all sessions board
│   │           ├── palette.rs            # ⌘K: input + nucleo results
│   │           ├── settings.rs
│   │           └── license.rs
│   │
│   ├── codedeck_ui/              # ── design system (lib, depends on gpui-kit) ──
│   │   ├── Cargo.toml            # [lib] path = "src/codedeck_ui.rs"
│   │   ├── src/
│   │   │   ├── codedeck_ui.rs    # pub mod …; pub use prelude
│   │   │   ├── theme.rs          # Theme { light, dark } tokens: bg, card, ink, line, live, warn…
│   │   │   ├── typography.rs     # Instrument Sans / Fraunces italic / Geist Mono, text styles
│   │   │   ├── motion.rs         # spring(), squish(), jelly(), durations; reduce-motion switch
│   │   │   ├── gel_button.rs     # GelButton: wob on hover, jelly on release, pointer light
│   │   │   ├── segmented.rs      # Segmented: squish knob
│   │   │   ├── toggle.rs         # springy Toggle
│   │   │   ├── pill.rs           # Pill { soft, warn } 20px filled + Badge (running)
│   │   │   ├── card.rs           # Card container: hairline, radius, live tint
│   │   │   ├── editor_button.rs  # round "Open with" circles + default dot + tooltip
│   │   │   ├── stack_icon.rs     # language / framework icons
│   │   │   ├── mascot.rs         # the red robot as SVG + moods (scan/talk/hop)
│   │   │   └── avatar.rs         # per-project bot avatar (pre-rendered SVGs)
│   │   └── examples/
│   │       └── gallery.rs        # cargo run -p codedeck_ui --example gallery
│   │
│   └── codedeck_core/            # ── domain (lib, no GPUI) ─────────────────────
│       ├── Cargo.toml            # [lib] path = "src/codedeck_core.rs"
│       ├── src/
│       │   ├── codedeck_core.rs  # pub mod …; re-exports Project, Session, Agent
│       │   ├── model.rs          # Project, Session, Agent, Stack, GitInfo (serde)
│       │   ├── scan.rs           # walk roots with `ignore`, find project roots
│       │   ├── stack.rs          # marker-file table → Vec<Stack>
│       │   ├── git.rs            # read .git/HEAD, dirty count
│       │   ├── sources.rs        # trait-free: fn read_all() -> Vec<Session>
│       │   ├── sources/
│       │   │   ├── claude.rs     # ~/.claude/projects/**/*.jsonl, streamed
│       │   │   ├── opencode.rs   # ~/.local/share/opencode/opencode.db (read-only)
│       │   │   └── codex.rs      # ~/.codex/sessions/**/rollout-*.jsonl
│       │   ├── matcher.rs        # session.cwd → longest project root prefix
│       │   ├── index.rs          # SQLite cache (rusqlite): instant startup
│       │   ├── watch.rs          # FSEvents via `notify` → change events
│       │   ├── launch.rs         # open in Cursor / VS Code / Zed, terminal resume
│       │   ├── settings.rs       # Settings struct ↔ settings.json
│       │   └── license.rs        # activate once, verify signed token offline
│       └── tests/
│           └── fixtures/         # sample jsonl / sqlite copied from real agents
│
├── assets/                       # embedded into the binary (rust-embed)
│   ├── fonts/                    # InstrumentSans, Fraunces (static italic cuts), GeistMono
│   ├── icons/
│   │   ├── ide/                  # cursor.svg, vscode.svg, zed.svg (from design/assets/ide)
│   │   ├── stack/                # simple-icons svgs
│   │   └── ui/                   # branch, terminal, search…
│   ├── art/                      # public-domain paintings + CREDITS.md
│   └── mascot/                   # mascot.svg, avatars/
│
├── bundle/                       # macOS packaging inputs
│   ├── Info.plist
│   ├── Codedeck.entitlements
│   └── AppIcon.icns
├── script/
│   ├── bundle                    # cargo bundle --release → .app
│   ├── notarize                  # codesign + notarytool + staple
│   └── clippy                    # cargo clippy --workspace -- -D warnings
│
├── design/                       # HTML prototypes (unchanged, history)
├── docs/                         # architecture.md (Tauri, superseded), this file
└── spike/                        # old Tauri spike: delete once the GPUI window works
```

Fixed ideas:

- `crates/<name>/` matches the crate name, and the crate root is `src/<name>.rs` (Zed style), so the editor tab says `codedeck_core.rs`, not one of 3 identical `lib.rs`.
- **No `mod.rs`.** A folder module is `screens.rs` + `screens/`.
- One logical component per file. Don't split into many tiny files. A view's helpers stay in the view's file until it passes about 600 lines.
- Tests for a pure function live at the bottom of its file in `#[cfg(test)] mod tests`. Parser tests use fixtures from `tests/fixtures/`.

---

## 4. File formatting and code patterns

### 4.1 Workspace `Cargo.toml`
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
edition = "2024"
publish = false

[workspace.dependencies]
gpui-kit = { version = "=0.10.0", default-features = false }   # exact pin: GPUI breaks often
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
ignore = "0.4"
notify = "6"
nucleo = "0.5"
dirs = "5"
jiff = "0.1"
rust-embed = "8"
log = "0.4"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
dbg_macro = "deny"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```
Each crate uses `edition.workspace = true`, `[lints] workspace = true`, and `dep.workspace = true`. Versions live only in the root file. The exact versions are checked with `cargo add` in step 1.

### 4.2 A view file (`screens/home.rs`), in a fixed order
```rust
//! Home: painting banner, continue tiles, project grid.

use crate::{app_state::AppState, screens::project_card::ProjectCard};
use codedeck_ui::prelude::*;

pub struct Home {
    state: Entity<AppState>,
    filter: Filter,
    _subscriptions: Vec<Subscription>,   // keeps observers alive (Zed rule)
}

impl Home {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let observe = cx.observe(&state, |_, _, cx| cx.notify());
        Self { state, filter: Filter::All, _subscriptions: vec![observe] }
    }

    fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        self.filter = filter;
        cx.notify();
    }
}

impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .size_full()
            .bg(theme.bg)
            .child(self.render_banner(cx))
            .child(self.render_grid(cx))
    }
}

// private render_* helpers below, then #[cfg(test)] mod tests
```
The order is always: imports, then the struct, then `new` and the state methods, then `impl Render`, then private `render_*` helpers, then tests.

### 4.3 A design-system piece (`codedeck_ui/src/pill.rs`): `RenderOnce`, builder style
```rust
#[derive(IntoElement)]
pub struct Pill { label: SharedString, tone: Tone, icon: Option<Icon> }

impl Pill {
    pub fn new(label: impl Into<SharedString>) -> Self { … }
    pub fn warn(mut self) -> Self { self.tone = Tone::Warn; self }
    pub fn icon(mut self, icon: Icon) -> Self { self.icon = Some(icon); self }
}

impl RenderOnce for Pill {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        h_flex().h(px(20.)).px(px(7.)).gap(px(4.)).rounded(px(6.))
            .bg(self.tone.fill(theme)).text_color(self.tone.ink(theme))
            .text_size(px(11.5))
            .when_some(self.icon, |pill, icon| pill.child(icon.size(px(10.))))
            .child(self.label)
    }
}
```
Call sites read like the prototype: `Pill::new("3 uncommitted").warn()`.

### 4.4 Motion (`codedeck_ui/src/motion.rs`)
```rust
/// Under-damped spring as an easing curve; overshoots past 1.0, settles at 1.0.
pub fn spring(damping: f32, frequency: f32) -> impl Fn(f32) -> f32 {
    move |t| 1.0 - (-damping * t).exp() * (frequency * t).cos()
}

pub const SQUISH: Duration = Duration::from_millis(420);
pub const JELLY: Duration  = Duration::from_millis(600);
```
The segmented knob animates `left` and `width` with `with_animation(id, Animation::new(SQUISH).with_easing(spring(6., 14.)), …)`. Stretching along the travel axis and then overshooting is exactly the v11 squish. Gel buttons depend on spike #1. If `div` can't scale, `GelButton` paints its own background quad at a scaled size inside a small custom `Element`.

### 4.5 Core: plain functions, data out
```rust
// codedeck_core/src/sources/claude.rs
pub fn read(root: &Path) -> anyhow::Result<Vec<Session>> { … }   // never panics on bad lines
```
- Every agent reader has the same signature. `sources::read_all()` calls each one and collects their errors into "couldn't read N sessions" instead of failing.
- There's no `trait Source` until a fourth agent proves the shape.

### 4.6 Threads and state flow
```
main thread (GPUI)                         background executor
──────────────────                         ───────────────────
AppState::rescan(cx)
  └─ cx.spawn(async move |this, cx| {
        let result = cx.background_spawn(async { core::scan_all(&roots) }).await;
        this.update(cx, |state, cx| { state.apply(result); cx.notify(); })
     }).detach_and_log_err(cx)
```
Views `observe` AppState and re-render. The watcher pushes change events through the same path.

### 4.7 Rules (Zed's house style, adopted)
- No `unwrap()`: use `?`, `.log_err()`, or a `match`. Don't discard errors with `let _ =`.
- Names use full words (`project`, not `p`). The context argument is always `cx`, `window` comes before `cx`, and callbacks come after.
- Comments explain *why* something is done, not what the code does.
- `Render` is for stateful views and `RenderOnce` is for design-system pieces. Use `.when()` and `.when_some()` instead of `if` branches that rebuild a whole element.
- Store a `Subscription` in `_subscriptions`, and keep `Task`s in fields or `.detach_and_log_err(cx)` them.
- Run `cargo fmt` and `script/clippy` with zero warnings before every commit.

### 4.8 Files on the user's Mac
| What | Where | Format |
|---|---|---|
| Index cache | `~/Library/Application Support/Codedeck/index.db` | SQLite |
| Settings | `~/Library/Application Support/Codedeck/settings.json` | JSON (serde, `#[serde(default)]` on every field so old files keep working) |
| License token | `~/Library/Application Support/Codedeck/license` | signed token |
| Logs | `~/Library/Logs/Codedeck/codedeck.log` | text |

---

## 5. Prototype → GPUI mapping

| Prototype (v22) | GPUI piece |
|---|---|
| `#win` 1180×740 window | `window.rs`: `WindowOptions`, transparent titlebar, traffic lights inset |
| `show(id)` + SCREENS registry | `shell.rs` holds `enum Screen` and the current `Entity` for each screen, and crossfades with `with_animation` on opacity |
| `.gel`, `squish()`, `.tog`, sidebar highlight | `gel_button.rs`, `segmented.rs`, `toggle.rs`, `sidebar.rs` |
| CSS tokens, `[data-theme="dark"]` | `theme.rs`, a `Theme` global with `light()` and `dark()`. It follows the OS appearance or the setting. |
| Pixel film canvas (236×148, K=5) | `film.rs`: paint the low-res frame into a pixel buffer and draw it as one image per frame, scaled with nearest-neighbour. **Spike #2** checks whether that, or about 35k `paint_quad`s, holds 120 fps. |
| Scanner canvas bar + glow | `scan.rs`: a `canvas()` element with `paint_quad` per segment |
| Painting banner by time of day | `img()` from `assets/art`, with a greeting overlay |
| Project card v22 | `project_card.rs` built from `Card`, `Badge`, `Pill`, `StackIcon`, `EditorButton` |
| ⌘K | `palette.rs`: the input from gpui-base, nucleo ranking, `uniform_list` |

---

## 6. Build order (each step ends in something you can run)

0. **Spikes (throwaway, in `spike-gpui/`):** a window with the transparent titlebar, one gel button with jelly, one squish seg, and the film at 120 fps. It answers the three risks before any real code. If the gel button can't be done well, we decide here rather than at step 5.
1. **Workspace skeleton:** the three crates compile, an empty window opens, fonts and theme load, and the gallery example runs.
2. **Core without UI:** `scan` + `sources::{claude, opencode}` + `matcher`, printed as JSON by `cargo run -p codedeck_core --example dump`, and checked against your real `~/codes`.
3. **Design system:** every piece in the gallery, light and dark, with motion tuned.
4. **Home + project card + project page** on real data, with Open with working.
5. **Onboarding:** film, scan, setup.
6. **⌘K, sessions, settings, license.**
7. **Index + watcher** (instant start, live updates), then bundling, signing, notarization, and the `.dmg`.

---

## 7. Decisions I need from you

1. **Library:** use `gpui-kit` without styled components (my pick), bare `gpui`, or `gpui-ce`?
2. **Three crates** (`codedeck`, `codedeck_ui`, `codedeck_core`), or one crate with folders?
3. **Crate root naming:** Zed style (`src/codedeck_ui.rs`), or the default `lib.rs`?
4. **Settings file:** JSON, or TOML?
5. **The Tauri `spike/`:** delete it now, or keep it until the GPUI window works?
6. **Scope:** macOS only for v1? I'm assuming yes; GPUI could do Windows and Linux later.

---

## 8. As built (v0, 2026-11-21)

Decisions taken: gpui-kit (styled components off) · three crates · `src/<crate>.rs` roots · JSON settings · delete the Tauri spike later · macOS only.

Where the build simplified the proposal:
- **Motion uses GPUI's own springs.** `with_spring(id, SpringAnimation::new(config).to(target), …)` is retargetable and keeps velocity. The squish knob is two springs: the leading edge is stiff, the trailing edge soft. Gel press springs to 1, and the release overshoot is the jelly. GPUI can't scale a `div`, so the fill sits on its own layer with springing insets (spike #1 settled).
- **Cache is a JSON file** (`cache.json`), not SQLite. rusqlite is only used to read OpenCode.
- **Folder walk uses `std::fs`**, not the `ignore` crate: we stop at the first project anyway.
- **One view.** `Shell` holds the screen and the UI state; each screen is an `impl Shell` block in `screens/*.rs`. Menus and keys live in `actions.rs`; window options in `main.rs`.
- **Assets are embedded by `codedeck_ui`** (rust-embed over `/assets`), so the gallery and the app share them.
- **Avatars** come from DiceBear bottts-neutral, fetched once with `curl` into `~/Library/Application Support/Codedeck/avatars/` after a scan. The UI draws a fallback bot until they land.
- **Sessions count prompts, not lines.** Claude logs tool results and streamed chunks as lines too.
- **Loose sessions** (run from ~, a scan root, /tmp, ~/Library) are kept in `Snapshot.loose` instead of becoming projects.

Not built yet: onboarding (film, scan, setup), ⌘K palette, all-sessions screen, license screen, folder picker, incremental scans, bundling, signing.

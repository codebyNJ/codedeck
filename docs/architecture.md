# Codedeck: research and architecture (v0)

Working name: **Codedeck**. A macOS app with a Rust core and a Tauri/React UI, sold as a one-time ~$15 purchase. It finds every local codebase, detects its stack, and lists the coding-agent sessions (Claude Code, OpenCode, Codex, …) that belong to it. From there you open the project in VS Code, a terminal, or resume a chosen session. It uses no AI and makes no network calls except license activation.

UI quality comes first. Everything below is ordered by that.

---

## 1. UI framework decision (revised 2026-11-16: Tauri)

**Tauri 2: a Rust backend plus a React + TypeScript UI in the system WebView (WKWebView).** We switched from GPUI after reading recent engineering write-ups:

- **Raycast 2.0** (Aug 2026) rebuilt its premium launcher as React inside WKWebView with a native shell and a Rust core ([deep dive](https://www.raycast.com/blog/a-technical-deep-dive-into-the-new-raycast)). The web UI is now an accepted route to a premium feel.
- **Tolaria** (Tauri v2 + React) got praise from a strict Mac reviewer for being fast and close to native ([shapeof.com](https://shapeof.com/archives/2026/4/tolaria_ai_and_rust.html)).
- **GPUI** got Zed to 1.0, but it's narrowing to Zed's own needs; the community fork `gpui-ce` exists for everything else. That's too risky for our only UI layer.

| Option | Verdict |
|---|---|
| **Tauri + React + Motion** | **Pick.** Best animation tools, hot reload, 4 MB app. Rust still does all the scanning. |
| Custom Swift shell + WebView (Raycast-style) | Only if Tauri's window handling ever blocks us. |
| GPUI | Pure Rust, but its future as a standalone framework is uncertain. |
| SwiftUI | Fully native, but no Rust and none of the web animation tools. |

### UI toolkit
React + TypeScript + Vite · **Motion** (springs, layout, shared-element transitions) · **cmdk** (⌘K) · plain CSS with variables (add Tailwind only if the CSS grows) · `@tanstack/react-virtual` when the lists get long.

### Proven by the spike (`spike/`)
- ✅ Native sidebar vibrancy works in a **release** `.app` (`windowEffects: sidebar`, `transparent`, `macOSPrivateApi`).
- ✅ Overlay title bar with repositioned traffic lights; release bundle is **4 MB**.
- ⚠️ **Hidden or inactive windows don't render.** WebKit pauses `requestAnimationFrame` and painting, so:
  - Never wait for a `requestAnimationFrame` to call `show()`. Show from a React effect, then call `setFocus()`.
  - Start the UI at opacity 0 so nothing flashes.
- ⚠️ **Measured 59 fps.** If your Mac has a 120 Hz screen, that's the WKWebView cap. The fix is `tauri-plugin-macos-fps`, which uses a private API; that's fine because we ship a `.dmg`, not through the App Store.

### Rules so it never feels like a website
- Animate only `transform` and `opacity`.
- No `cursor: pointer`, no text selection, no right-click Reload/Inspect in production, no rubber-band bounce on the whole window.
- System fonts: `-apple-system` for text, SF Mono for paths.
- Animations around 150–300 ms, springs without bounce, and respect "Reduce motion" (`MotionConfig reducedMotion="user"`).
- macOS draws the blur, not CSS. Settings later go in a separate native window, with native menus.

## 2. UI design

### Principles
- **Instant.** The window shows cached data in under 200 ms. Scanning runs behind it and animates new items in. No spinner screens.
- **Keyboard first.** `⌘K` opens a palette that fuzzy-searches across every project and session, and `↵` resumes. This is the feature people will pay for.
- **Native, not themed.** Vibrancy sidebar, system accent color, SF Pro / SF Mono, light and dark mode following the OS.
- **Motion with a purpose.** 150–250 ms ease-out on everything. Nothing bounces, and nothing animates while you type.

### Screens
1. **First launch.** The window fades in and scales from 0.96. One line asks "Where do you keep code?" with suggested folders already detected (`~/codes`, `~/Developer`, `~/Projects`). Pressing Scan starts the scan. Project cards stream into a grid as they are found, staggered 20 ms apart, each stack icon popping in as it is detected. This is the "wow" moment, so it gets the most design time.
2. **Main window.** Three panes:
   - Sidebar: Recent, All projects, grouped by stack, plus pinned items.
   - Project list: name, path, stack badges, git branch, last session time, and a small activity sparkline.
   - Detail: sessions for the selected project, newest first. Each row shows the agent icon (Claude / OpenCode / Codex), the title (first prompt or summary), relative time, and message count. Actions: **Open in VS Code**, **Open in Terminal**, **Resume session**.
3. **⌘K palette.** One input, mixed results (projects + sessions), with a preview on the right.
4. **Settings.** Scan roots, ignore list, preferred editor, preferred terminal, and license.

Before code, mock these in Figma or as HTML so motion and layout get settled cheaply.

---

## 3. Backend architecture

```
┌──────────────── WebView: React UI (TypeScript) ───────────────────────┐
│  state: projects + sessions (fetched via invoke, live via events)      │
│  Views: Onboarding · ProjectList · ProjectDetail · Palette · Settings   │
└───────────▲───────────────────────────────────────────────┬────────────┘
            │ Tauri events (scan progress, new session)     │ invoke(commands)
┌───────────┴────────── Rust (Tauri core, threads) ───┐  ┌──┴───────────┐
│ Scanner  → finds project roots, detects stack       │  │ Launcher     │
│ Sources  → ClaudeCode · OpenCode · Codex readers    │  │ open/osascript│
│ Matcher  → session.cwd → longest project root prefix│  └──────────────┘
│ Watcher  → FSEvents (notify) on roots + agent dirs  │
└───────────┬─────────────────────────────────────────┘
            │
      SQLite index (rusqlite)  ~/Library/Application Support/Codedeck/index.db
```

### 3.1 Scanner
- Walk the configured roots with the `ignore` crate, which skips `node_modules`, `target`, and anything gitignored for free. Default depth is 4.
- A folder is a project if it contains `.git` or a manifest file. Stop descending once one is found (no nested projects in v1).
- **Stack detection** is a table of marker files, no heuristics engine:
  `Cargo.toml→Rust`, `package.json→JS/TS` (refined by `next.config.*`, `vite.config.*`, `tsconfig.json`), `go.mod→Go`, `pyproject.toml|requirements.txt→Python`, `Gemfile→Ruby`, `pubspec.yaml→Flutter`, `*.xcodeproj|Package.swift→Swift`, `pom.xml|build.gradle*→JVM`, `composer.json→PHP`, `mix.exs→Elixir`. Multiple hits become multiple badges.
- Git info: read `.git/HEAD` directly for the branch. No libgit2 dependency is needed for v1.

### 3.2 Session sources (verified on this machine where noted)
Each source is a small module with the same shape: `fn read() -> Vec<Session>`, where `Session { agent, id, cwd, title, started, updated, message_count }`.

| Agent | Where | How |
|---|---|---|
| **Claude Code** ✅ | `~/.claude/projects/<encoded-cwd>/<session-id>.jsonl` | Read the `cwd` field inside the JSONL, not the folder name: the encoding (`/` and `.` → `-`) is lossy. Title comes from the `summary` line or the first user message. Stream lines with `serde_json`; never load whole files. |
| **OpenCode** ✅ | `~/.local/share/opencode/opencode.db` (SQLite) | `SELECT id, directory, title, time_created, time_updated FROM session WHERE parent_id IS NULL`. Open read-only, because OpenCode may be writing. |
| **Codex CLI** ⚠️ not installed here | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` | The first line, `session_meta`, has `cwd`. Verify against a real install. |
| Gemini CLI, Cursor | later | Formats unverified, so v1.1 at the earliest. |

These are **undocumented internal formats** and will change. Parse defensively: skip unknown lines, never crash, and show "couldn't read N sessions." This is the biggest maintenance cost of the product.

### 3.3 Matching sessions to projects
Find the longest project root that is a prefix of `session.cwd`. A session whose cwd is not under any project root becomes a project of its own (it is still a codebase you worked in).

### 3.4 Launcher (the three actions)
- **VS Code:** `open -a "Visual Studio Code" <path>`. Also offer Cursor and Zed when installed (check `/Applications`).
- **Terminal:** the user picks Terminal, iTerm2, Ghostty, or Warp. Use `osascript` for Terminal/iTerm and each app's CLI or URL scheme for the others.
- **Resume session:** open the chosen terminal with `cd <cwd> && claude --resume <id>` or `opencode --session <id>`. Check the exact flags against each CLI's `--help` during the build.

### 3.5 Persistence and freshness
- SQLite index holds projects, sessions, and file mtimes. On launch, render from the index immediately, then rescan only files whose mtime changed.
- The `notify` crate (FSEvents) watches scan roots and agent folders, so a new Claude session appears live.

### 3.6 macOS specifics
- **Distribution:** a direct `.dmg`, signed with Developer ID and notarized ($99/yr Apple Developer account). Not the Mac App Store: its sandbox blocks reading `~/.claude` and spawning terminals.
- **Privacy prompts:** scanning `~/Desktop` or `~/Documents` triggers macOS permission prompts. Default roots should avoid them, and onboarding should explain them.
- **Bundling:** `cargo-bundle` or `cargo-packager` builds the `.app`. Updates can use Sparkle via the appcast; v1 can just link to a download.

### 3.7 Licensing (one-time purchase)
Use Polar or Lemon Squeezy license keys. Activate once online, then store a signed token locally and validate it offline. Never phone home again. This is the only network code in the app.

---

## 4. Crates (minimal set)

| Need | Crate |
|---|---|
| App shell | `tauri` 2 (feature `macos-private-api`), `tauri-plugin-opener` |
| Walk folders | `ignore` |
| JSON lines | `serde`, `serde_json` |
| SQLite (index + OpenCode) | `rusqlite` (bundled) |
| File watching | `notify` |
| Fuzzy search | `nucleo` (the fuzzy matcher used by Helix) |
| Paths | `dirs` |
| Time | `jiff` or `chrono` |

Nothing else until a real need shows up.

---

## 5. Project layout

```
codedeck/
  spike/                 throwaway look-and-feel spike (fake data)
    src/                 App.tsx, data.ts, styles.css, main.tsx
    src-tauri/           Rust: lib.rs, tauri.conf.json, capabilities/
  docs/architecture.md
```
The real app takes the same shape. The Rust modules go under `src-tauri/src/`: `scan.rs`, `sources/{claude,opencode,codex}.rs`, `index.rs`, `launch.rs`, and `commands.rs` exposing them to the UI.

## 6. Build order

1. ✅ **Spike** (`spike/`): window, vibrancy, streaming grid, card→detail, ⌘K, FPS meter. Run it with `cd spike && npm run tauri dev`.
2. Backend without UI: `scan` + `sources::claude` + `sources::opencode`, printing JSON. Check it against your real `~/codes`.
3. Tauri commands and events. Swap `data.ts` for real data, and make the launch actions work.
4. SQLite index + file watcher (instant startup, live updates).
5. Onboarding flow and first-launch animation. 120 fps plugin if needed.
6. Settings, licensing, signing, notarization, `.dmg`.

## 7. Open questions for later
- Final name and icon.
- Whether to show token cost or usage per session. The data exists (OpenCode stores it; Claude JSONL has usage), but it widens scope.
- Whether there's a free tier (for example, limited to 5 projects) or a trial.

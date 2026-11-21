# Codedeck: how it's built, and trial run 1 (2026-11-21)

Run from scratch: no mock data. The spike, the examples and the test fixture were removed, `~/Library/Application Support/Codedeck` was wiped, and it ran as a release build against this Mac's real `~/codes`, `~/.claude` and OpenCode database.

## How a launch flows

```
main.rs ─ loads settings.json ─ codedeck_ui::init (fonts, theme) ─ actions::install (keys, menu bar)
   │
   ├─ AppState::new                      crates/codedeck/src/app_state.rs
   │    ├─ index::load()  → cache.json   (instant first paint on 2nd+ launch)
   │    ├─ watch::watch() → FSEvents on scan roots + agent folders
   │    ├─ poll task: every 3s, rescan if something changed (≥10s apart)
   │    └─ rescan(): background thread → codedeck_core::scan_all
   │            scan::find_projects (walk roots, stop at first project)
   │            sources::read_all   (claude jsonl, opencode sqlite, codex jsonl)
   │            matcher::assign     (session.cwd → longest project root)
   │            index::save         (cache.json)
   │         then UI thread: replace snapshot, notify → screens re-render
   │         then background: avatar::fetch_missing (DiceBear → avatars/*.svg)
   │
   └─ open_window → Shell (shell.rs): sidebar + current Screen
          actions::route_to: ⌘1 ⌘, ⌘R ⌘⇧L work whatever has focus
```

## Pages

| Page | File | Reads | Actions |
|---|---|---|---|
| Sidebar | `shell.rs` | 6 most recent projects, scan status | Home, Settings, recent project; springy highlight |
| Home | `screens/home.rs` | snapshot: projects, sessions, running, dirty | filter by stack, sort, continue tile → project, card → project, Open with → editor |
| Project | `screens/project.rs` | one project + its sessions grouped by day | Open with → editor, Terminal → new shell in the folder |
| Settings | `screens/settings.rs` | settings.json | theme, default editor, terminal, agent toggles, add/remove folders, rescan |

## Trial-run checklist

| # | Flow | Result |
|---|---|---|
| 1 | First launch, no state | ✅ window opens, scan ~3s, 37 projects · 62 sessions; ⚠ blank "0 projects" for those 3s (no onboarding yet) |
| 2 | Home banner, stats, continue tiles | ✅ real counts, running agent detected |
| 3 | Filter (TypeScript), sort (A–Z, Most chats) | ✅ knob squishes, grid re-sorts |
| 4 | Scroll to the bottom | ✅ |
| 5 | Continue tile → project | ✅ |
| 6 | Sidebar recent → project | ✅ highlight springs to the row |
| 7 | Card → project | ✅ |
| 8 | Open with VS Code | ✅ VS Code opened the project |
| 9 | Terminal | ✅ new Terminal shell in `~/codes/codedeck` |
| 10 | Settings: agent toggle off/on | ✅ 62 → 11 → 62 sessions, saved to settings.json |
| 11 | Settings: theme | ✅ dark everywhere, saved |
| 12 | ⌘, ⌘1 ⌘⇧L ⌘Q | ✅ (after fix 5) |
| 13 | Relaunch | ✅ real data on screen in ~0.45s from the cache |
| 14 | Add a folder (native picker) | ⏳ built, not yet seen open (test by hand) |

## Bugs found and fixed in this run

1. **Avatar race.** The UI read an SVG while curl was still writing it, and GPUI cached it as broken. Now it downloads to `.part` and renames.
2. **Every project looked active.** Our own `git status` rewrote `.git/index`. Now it runs with `--no-optional-locks`, and activity comes from `.git/logs/HEAD`.
3. **"1 prompts"** is now singular.
4. **Greeting and painting disagreed at 11am.** They now share the same hour ranges.
5. **Shortcuts died after clicking a control.** Focus moved off the view that held the handlers. Actions are now app-wide.
6. **No way to choose folders.** Settings now has Add a folder (native picker) and Remove.

## Gaps (the next steps)

1. **First run / onboarding.** A new user lands on an empty Home while roots are guessed (`~/codes`, `~/Developer`…). Next: setup asks which folders, agents and editor; then the scan (kshvbgde scanner); then Home. The pixel film comes after.
2. **⌘K** search over projects and sessions.
3. **Sessions** screen (all sessions, Running / Today / Earlier).
4. **License** screen and store activation.
5. **Packaging**: `.app` bundle with an icon, signing, notarization, `.dmg`.
6. **Windows / Linux** (later): core is portable except `launch.rs` (`open -a`) and `git.rs`; the UI is GPUI on both.

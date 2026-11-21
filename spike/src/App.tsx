import { useEffect, useMemo, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { AnimatePresence, LayoutGroup, MotionConfig, motion, type Transition } from "motion/react";
import { Command } from "cmdk";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AGENT, PROJECTS, STACK_COLOR, ago, type Agent, type Project, type Session } from "./data";

// Two springs for the whole app: one snappy, one for big moves. No bounce while working.
const snappy: Transition = { type: "spring", bounce: 0, duration: 0.3 };
const hero: Transition = { type: "spring", bounce: 0.12, duration: 0.5 };

// The order the fake "scan" finds projects in (random), so cards slot into sorted positions as they stream in.
const DISCOVERY = PROJECTS.map((_, i) => PROJECTS[(i * 7) % PROJECTS.length]);

type Filter = { kind: "all" } | { kind: "stack"; value: string } | { kind: "agent"; value: Agent };

export default function App() {
  const [found, setFound] = useState(0);
  const [filter, setFilter] = useState<Filter>({ kind: "all" });
  const [openId, setOpenId] = useState<string | null>(null);
  const [palette, setPalette] = useState(false);
  const [toast, setToast] = useState<string | null>(null);

  // The window starts hidden ("visible": false) and is shown once React has mounted.
  // Not via requestAnimationFrame: WebKit pauses rAF in hidden windows, so it would never fire.
  // The .app fade-in below starts at opacity 0, so there's no flash.
  useEffect(() => {
    if ("__TAURI_INTERNALS__" in window) getCurrentWindow().show().then(() => getCurrentWindow().setFocus());
  }, []);

  // Fake scan: one project every ~45 ms after a short pause.
  useEffect(() => {
    if (found >= DISCOVERY.length) return;
    const t = setTimeout(() => setFound((n) => n + 1), found === 0 ? 450 : 45);
    return () => clearTimeout(t);
  }, [found]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === "k") {
        e.preventDefault();
        setPalette((p) => !p);
      } else if (e.key === "Escape") {
        if (palette) setPalette(false);
        else setOpenId(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [palette]);

  useEffect(() => {
    if (!toast) return;
    const t = setTimeout(() => setToast(null), 2600);
    return () => clearTimeout(t);
  }, [toast]);

  const scanned = useMemo(() => DISCOVERY.slice(0, found).sort((a, b) => a.ago - b.ago), [found]);
  const visible = scanned.filter((p) =>
    filter.kind === "all" ? true
    : filter.kind === "stack" ? p.stacks.includes(filter.value)
    : p.sessions.some((s) => s.agent === filter.value),
  );
  const open = PROJECTS.find((p) => p.id === openId) ?? null;

  const resume = (p: Project, s: Session) => setToast(`cd ${p.path} && ${AGENT[s.agent].cmd} ${s.id}`);
  const openIn = (p: Project, app: string) => setToast(`open -a "${app}" ${p.path}`);

  return (
    <MotionConfig reducedMotion="user" transition={snappy}>
      <motion.div
        className="app"
        initial={{ opacity: 0, scale: 0.985 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ type: "spring", bounce: 0, duration: 0.45 }}
      >
        <Sidebar scanned={scanned} filter={filter} setFilter={setFilter} />

        <main className="main">
          <header className="toolbar" data-tauri-drag-region>
            <div data-tauri-drag-region>
              <h1 data-tauri-drag-region>{filterTitle(filter)}</h1>
              <ScanStatus found={found} total={DISCOVERY.length} />
            </div>
            <button className="search" onClick={() => setPalette(true)}>
              <span>Search projects and sessions</span>
              <kbd>⌘K</kbd>
            </button>
          </header>

          <div className="scroll">
            <LayoutGroup>
              <motion.div className="grid" layout>
                <AnimatePresence mode="popLayout">
                  {visible.map((p) => (
                    <Card key={p.id} p={p} hidden={p.id === openId} onOpen={() => setOpenId(p.id)} />
                  ))}
                </AnimatePresence>
              </motion.div>

              <AnimatePresence>
                {open && (
                  <Detail key={open.id} p={open} onClose={() => setOpenId(null)} onResume={resume} onOpenIn={openIn} />
                )}
              </AnimatePresence>
            </LayoutGroup>
          </div>
        </main>

        <AnimatePresence>
          {palette && (
            <Palette
              projects={scanned}
              onClose={() => setPalette(false)}
              onProject={(p) => { setPalette(false); setOpenId(p.id); }}
              onSession={(p, s) => { setPalette(false); resume(p, s); }}
            />
          )}
        </AnimatePresence>

        <AnimatePresence>
          {toast && (
            <motion.div
              className="toast"
              initial={{ opacity: 0, y: 12, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 6 }}
            >
              <span className="toast-label">Would run</span>
              <code>{toast}</code>
            </motion.div>
          )}
        </AnimatePresence>

        <Fps />
      </motion.div>
    </MotionConfig>
  );
}

function filterTitle(f: Filter) {
  return f.kind === "all" ? "All projects" : f.kind === "stack" ? f.value : AGENT[f.value].label;
}

function ScanStatus({ found, total }: { found: number; total: number }) {
  const done = found >= total;
  return (
    <div className="scan">
      <motion.span
        className="scan-dot"
        animate={done ? { scale: 1, opacity: 0.5 } : { scale: [1, 1.6, 1], opacity: [1, 0.4, 1] }}
        transition={done ? snappy : { repeat: Infinity, duration: 1.1 }}
      />
      {done ? `${total} projects` : `Scanning ~/codes… ${found} found`}
    </div>
  );
}

function Sidebar({ scanned, filter, setFilter }: { scanned: Project[]; filter: Filter; setFilter: (f: Filter) => void }) {
  const stacks = Object.keys(STACK_COLOR)
    .map((s) => [s, scanned.filter((p) => p.stacks.includes(s)).length] as const)
    .filter(([, n]) => n > 0);
  const agents = (Object.keys(AGENT) as Agent[]).map(
    (a) => [a, scanned.filter((p) => p.sessions.some((s) => s.agent === a)).length] as const,
  );
  const is = (f: Filter) => JSON.stringify(f) === JSON.stringify(filter);

  const Item = ({ f, dot, label, n }: { f: Filter; dot: ReactNode; label: string; n: number }) => (
    <button className={`nav ${is(f) ? "on" : ""}`} onClick={() => setFilter(f)}>
      {is(f) && <motion.span layoutId="nav-pill" className="nav-pill" transition={snappy} />}
      <span className="nav-dot">{dot}</span>
      <span className="nav-label">{label}</span>
      <motion.span className="nav-n" key={n} initial={{ opacity: 0.3 }} animate={{ opacity: 1 }}>{n}</motion.span>
    </button>
  );

  return (
    <aside className="sidebar" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>Codedeck</div>
      <Item f={{ kind: "all" }} dot="◇" label="All projects" n={scanned.length} />
      <div className="section">Stacks</div>
      {stacks.map(([s, n]) => (
        <Item key={s} f={{ kind: "stack", value: s }} dot={<i style={{ background: STACK_COLOR[s] }} />} label={s} n={n} />
      ))}
      <div className="section">Agents</div>
      {agents.map(([a, n]) => (
        <Item key={a} f={{ kind: "agent", value: a }} dot={<b style={{ color: AGENT[a].color }}>{AGENT[a].glyph}</b>} label={AGENT[a].label} n={n} />
      ))}
    </aside>
  );
}

function StackIcon({ p, layoutId }: { p: Project; layoutId?: string }) {
  const c = STACK_COLOR[p.stacks[0]] ?? "#71717a";
  return (
    <motion.div layoutId={layoutId} className="stack-icon" style={{ background: `linear-gradient(140deg, ${c}, ${c}88)` }}>
      {p.name[0].toUpperCase()}
    </motion.div>
  );
}

function Card({ p, hidden, onOpen }: { p: Project; hidden: boolean; onOpen: () => void }) {
  const glow = useRef<HTMLDivElement>(null);
  // Cursor light: move a blurred blob with transform only (no repaint of the card).
  const onMove = (e: MouseEvent) => {
    const r = e.currentTarget.getBoundingClientRect();
    glow.current!.style.transform = `translate(${e.clientX - r.left - 90}px, ${e.clientY - r.top - 90}px)`;
  };
  const counts = (Object.keys(AGENT) as Agent[])
    .map((a) => [a, p.sessions.filter((s) => s.agent === a).length] as const)
    .filter(([, n]) => n);

  return (
    <motion.button
      layout
      layoutId={`card-${p.id}`}
      className="card"
      style={{ borderRadius: 14, visibility: hidden ? "hidden" : "visible" }}
      initial={{ opacity: 0, scale: 0.94, y: 10 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.94 }}
      whileTap={{ scale: 0.98 }}
      onMouseMove={onMove}
      onClick={onOpen}
    >
      <div ref={glow} className="glow" />
      <div className="card-top">
        <StackIcon p={p} layoutId={`icon-${p.id}`} />
        <div className="card-titles">
          <motion.h3 layoutId={`title-${p.id}`}>{p.name}</motion.h3>
          <div className="path">{p.path}</div>
        </div>
        <span className="when">{ago(p.ago)}</span>
      </div>
      <div className="badges">
        {p.stacks.map((s) => <Badge key={s} s={s} />)}
      </div>
      <div className="card-foot">
        <span className="branch">⎇ {p.branch}</span>
        <span className="agents">
          {counts.length === 0 && <span className="muted">no sessions</span>}
          {counts.map(([a, n]) => (
            <span key={a} style={{ color: AGENT[a].color }}>{AGENT[a].glyph}<em>{n}</em></span>
          ))}
        </span>
      </div>
    </motion.button>
  );
}

function Badge({ s }: { s: string }) {
  return (
    <span className="badge" style={{ color: STACK_COLOR[s], background: `${STACK_COLOR[s]}1f` }}>{s}</span>
  );
}

function Detail({ p, onClose, onResume, onOpenIn }: {
  p: Project;
  onClose: () => void;
  onResume: (p: Project, s: Session) => void;
  onOpenIn: (p: Project, app: string) => void;
}) {
  const sessions = [...p.sessions].sort((a, b) => a.ago - b.ago);
  return (
    <>
      <motion.div className="scrim" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onClick={onClose} />
      <motion.div layoutId={`card-${p.id}`} className="detail" style={{ borderRadius: 18 }} transition={hero}>
        <div className="detail-head">
          <StackIcon p={p} layoutId={`icon-${p.id}`} />
          <div className="card-titles">
            <motion.h3 layoutId={`title-${p.id}`} className="big">{p.name}</motion.h3>
            <motion.div className="path" initial={{ opacity: 0 }} animate={{ opacity: 1, transition: { delay: 0.15 } }}>
              {p.path} · ⎇ {p.branch}
            </motion.div>
          </div>
          <button className="close" onClick={onClose} aria-label="Close">✕</button>
        </div>

        <motion.div
          className="actions"
          initial={{ opacity: 0, y: 6 }}
          animate={{ opacity: 1, y: 0, transition: { delay: 0.12 } }}
          exit={{ opacity: 0, transition: { duration: 0.08 } }}
        >
          <button className="primary" onClick={() => onOpenIn(p, "Visual Studio Code")}>Open in VS Code</button>
          <button onClick={() => onOpenIn(p, "Ghostty")}>Open in Terminal</button>
          <div className="badges">{p.stacks.map((s) => <Badge key={s} s={s} />)}</div>
        </motion.div>

        <motion.ul
          className="sessions"
          initial="hide"
          animate="show"
          exit="hide"
          variants={{ show: { transition: { staggerChildren: 0.035, delayChildren: 0.16 } }, hide: {} }}
        >
          {sessions.length === 0 && <li className="empty">No agent sessions in this project yet.</li>}
          {sessions.map((s) => (
            <motion.li
              key={s.id}
              variants={{ hide: { opacity: 0, y: 8 }, show: { opacity: 1, y: 0 } }}
              onDoubleClick={() => onResume(p, s)}
            >
              <span className="agent" style={{ color: AGENT[s.agent].color }}>{AGENT[s.agent].glyph}</span>
              <span className="s-title">{s.title}</span>
              <span className="muted">{s.messages} msgs</span>
              <span className="muted">{ago(s.ago)} ago</span>
              <button className="resume" onClick={() => onResume(p, s)}>Resume ↵</button>
            </motion.li>
          ))}
        </motion.ul>
      </motion.div>
    </>
  );
}

function Palette({ projects, onClose, onProject, onSession }: {
  projects: Project[];
  onClose: () => void;
  onProject: (p: Project) => void;
  onSession: (p: Project, s: Session) => void;
}) {
  const sessions = projects
    .flatMap((p) => p.sessions.map((s) => [p, s] as const))
    .sort((a, b) => a[1].ago - b[1].ago)
    .slice(0, 40);

  return (
    <>
      <motion.div className="scrim top" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onClick={onClose} />
      <motion.div
        className="palette"
        initial={{ opacity: 0, scale: 0.96, y: -10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.98, y: -4, transition: { duration: 0.12 } }}
        transition={{ type: "spring", bounce: 0.15, duration: 0.35 }}
      >
        <Command loop>
          <Command.Input autoFocus placeholder="Jump to a project or resume a session…" />
          <Command.List>
            <Command.Empty>Nothing matches.</Command.Empty>
            <Command.Group heading="Projects">
              {projects.map((p) => (
                <Command.Item key={p.id} value={`${p.name} ${p.path} ${p.stacks.join(" ")}`} onSelect={() => onProject(p)}>
                  <StackIcon p={p} />
                  <span>{p.name}</span>
                  <span className="muted path">{p.path}</span>
                </Command.Item>
              ))}
            </Command.Group>
            <Command.Group heading="Recent sessions">
              {sessions.map(([p, s]) => (
                <Command.Item key={s.id} value={`${s.title} ${p.name} ${s.id}`} onSelect={() => onSession(p, s)}>
                  <span className="agent" style={{ color: AGENT[s.agent].color }}>{AGENT[s.agent].glyph}</span>
                  <span>{s.title}</span>
                  <span className="muted">{p.name} · {ago(s.ago)}</span>
                </Command.Item>
              ))}
            </Command.Group>
          </Command.List>
        </Command>
      </motion.div>
    </>
  );
}

// Spike-only: shows the real frame rate, to check whether WKWebView runs at 120 fps on ProMotion.
function Fps() {
  const [fps, setFps] = useState(0);
  useEffect(() => {
    let frames = 0, start = performance.now(), id = 0;
    const loop = (now: number) => {
      frames++;
      if (now - start >= 1000) {
        setFps(Math.round((frames * 1000) / (now - start)));
        frames = 0;
        start = now;
      }
      id = requestAnimationFrame(loop);
    };
    id = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(id);
  }, []);
  return <div className="fps">{fps} fps</div>;
}

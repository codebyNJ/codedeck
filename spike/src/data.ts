// Fake data for the look-and-feel spike. The real version comes from the Rust scanner.

export type Agent = "claude" | "opencode" | "codex";

export type Session = {
  id: string;
  agent: Agent;
  title: string;
  ago: number; // minutes since last activity
  messages: number;
};

export type Project = {
  id: string;
  name: string;
  path: string;
  stacks: string[];
  branch: string;
  ago: number; // minutes since last activity
  sessions: Session[];
};

export const STACK_COLOR: Record<string, string> = {
  Rust: "#e0814f",
  TypeScript: "#3b82f6",
  React: "#38bdf8",
  "Next.js": "#a3a3a3",
  Go: "#22b8cf",
  Python: "#eab308",
  Swift: "#f97316",
  Tauri: "#facc15",
  Postgres: "#6366f1",
  Ruby: "#ef4444",
};

export const AGENT: Record<Agent, { label: string; glyph: string; color: string; cmd: string }> = {
  claude: { label: "Claude Code", glyph: "✻", color: "#d97757", cmd: "claude --resume" },
  opencode: { label: "OpenCode", glyph: "◧", color: "#a1a1aa", cmd: "opencode --session" },
  codex: { label: "Codex", glyph: "◎", color: "#10b981", cmd: "codex resume" },
};

const RAW: [string, string[], string][] = [
  ["codedeck", ["Rust", "Tauri", "TypeScript"], "main"],
  ["ankor-platform", ["Go", "Postgres"], "feat/billing"],
  ["murai-site", ["Next.js", "TypeScript"], "main"],
  ["sheen", ["React", "TypeScript"], "redesign"],
  ["eval-pipeline", ["Python"], "main"],
  ["ledger-cli", ["Rust"], "main"],
  ["pocket-api", ["Go"], "dev"],
  ["notes-sync", ["Swift"], "main"],
  ["shader-lab", ["TypeScript"], "main"],
  ["invoice-gen", ["Python", "Postgres"], "main"],
  ["portfolio", ["Next.js", "TypeScript"], "main"],
  ["dotfiles", [], "master"],
  ["rss-reader", ["Rust", "Tauri"], "main"],
  ["kv-store", ["Go"], "wal"],
  ["scraper", ["Python"], "main"],
  ["menubar-timer", ["Swift"], "main"],
  ["design-tokens", ["TypeScript"], "v2"],
  ["blog-engine", ["Ruby"], "main"],
  ["chat-relay", ["Rust", "Postgres"], "main"],
  ["habit-app", ["React", "TypeScript"], "main"],
  ["log-tail", ["Rust"], "main"],
  ["ml-notebooks", ["Python"], "main"],
  ["link-shortener", ["Go", "Postgres"], "main"],
  ["landing-v3", ["Next.js", "TypeScript"], "main"],
];

const TITLES = [
  "Fix flaky auth test on CI",
  "Add dark mode toggle",
  "Refactor scanner into modules",
  "Why is the build 3x slower?",
  "Write migration for invoices table",
  "Port settings screen to new layout",
  "Debug websocket reconnect loop",
  "Add pagination to /projects",
  "Explain this lifetime error",
  "Set up release workflow",
  "Make cards animate on hover",
  "Clean up unused dependencies",
];

const AGENTS: Agent[] = ["claude", "claude", "opencode", "codex"];

// Deterministic pseudo-random so the spike looks the same every launch.
let seed = 7;
const rand = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);

export const PROJECTS: Project[] = RAW.map(([name, stacks, branch], i) => {
  const base = Math.floor(Math.pow(i + 1, 2.2) * 9);
  const sessions: Session[] = Array.from({ length: Math.floor(rand() * 7) }, (_, j) => ({
    id: Math.floor(rand() * 0xffffffff).toString(16).padStart(8, "0"),
    agent: AGENTS[Math.floor(rand() * AGENTS.length)],
    title: TITLES[Math.floor(rand() * TITLES.length)],
    ago: base + j * Math.floor(60 + rand() * 900),
    messages: 4 + Math.floor(rand() * 120),
  }));
  return { id: name, name, path: `~/codes/${name}`, stacks, branch, ago: base, sessions };
});

export function ago(min: number): string {
  if (min < 1) return "now";
  if (min < 60) return `${min}m`;
  if (min < 60 * 24) return `${Math.floor(min / 60)}h`;
  if (min < 60 * 24 * 30) return `${Math.floor(min / 1440)}d`;
  return `${Math.floor(min / 43200)}mo`;
}

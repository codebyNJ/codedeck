#!/usr/bin/env python3
"""Build a fictional home folder for marketing captures of the real app.

    python3 packaging/demo-home.py /tmp/repobox-demo
    HOME=/tmp/repobox-demo target/release/repobox

Everything here is made up: six repos with real git history and uncommitted
changes, plus Claude Code, Codex and OpenCode sessions in their real on-disk
formats. Nothing from the real home folder is read or touched.
"""
import json, os, shutil, sqlite3, subprocess, sys, time, uuid
from datetime import datetime, timedelta, timezone

home = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else "/tmp/repobox-demo")
if os.path.exists(home):
    shutil.rmtree(home)
now = datetime.now(timezone.utc)
iso = lambda d: d.strftime("%Y-%m-%dT%H:%M:%S.000Z")
ago = lambda **k: now - timedelta(**k)

# ---------- repos ----------
REPOS = {
    "tidepool":     {"stack": {"package.json": '{"name":"tidepool","dependencies":{"next":"15.0.0","react":"19.0.0"}}', "tsconfig.json": "{}"}, "branch": "main", "dirty": 4, "last": dict(minutes=12), "msg": "Move session check after setSession()"},
    "lumen-api":    {"stack": {"go.mod": "module lumen-api\n\ngo 1.23\n"}, "branch": "rate-limit", "dirty": 3, "last": dict(hours=1), "msg": "Token bucket per API key"},
    "harbor-sync":  {"stack": {"Cargo.toml": '[package]\nname = "harbor-sync"\nversion = "0.3.0"\nedition = "2021"\n'}, "branch": "main", "dirty": 0, "last": dict(days=1), "msg": "Write sync queue to IndexedDB first"},
    "moss-ui":      {"stack": {"package.json": '{"name":"moss-ui","devDependencies":{"vite":"6.0.0","typescript":"5.6.0"}}', "vite.config.ts": "export default {}"}, "branch": "main", "dirty": 0, "last": dict(days=3), "msg": "301 old /blog URLs to /notes"},
    "sparrow-mail": {"stack": {"package.json": '{"name":"sparrow-mail","dependencies":{"svelte":"5.0.0"}}'}, "branch": "inbox-v2", "dirty": 2, "last": dict(days=1), "msg": "j and k move between threads"},
    "quill-cli":    {"stack": {"Cargo.toml": '[package]\nname = "quill-cli"\nversion = "0.4.0"\nedition = "2021"\n'}, "branch": "main", "dirty": 0, "last": dict(days=5), "msg": "Release notes for 0.4"},
}
codes = os.path.join(home, "codes")
env = {**os.environ, "GIT_AUTHOR_NAME": "Demo", "GIT_AUTHOR_EMAIL": "demo@example.com", "GIT_COMMITTER_NAME": "Demo", "GIT_COMMITTER_EMAIL": "demo@example.com", "HOME": home}
git = lambda repo, *a, **kw: subprocess.run(["git", "-C", repo, *a], check=True, capture_output=True, env={**env, **kw})
for name, r in REPOS.items():
    path = os.path.join(codes, name)
    os.makedirs(os.path.join(path, "src"))
    for f, body in r["stack"].items():
        open(os.path.join(path, f), "w").write(body)
    open(os.path.join(path, "README.md"), "w").write(f"# {name}\n")
    subprocess.run(["git", "init", "-q", "-b", "main", path], check=True, env=env)
    for i in range(14, 0, -1):  # two weeks of commits, so activity bars have shape
        if (i * 7 + len(name)) % 3 == 0:
            continue
        open(os.path.join(path, "src", f"part{i}.txt"), "w").write(f"{i}\n")
        d = iso(ago(days=i) + timedelta(days=0) if i > 1 else ago(**r["last"]))
        git(path, "add", "-A")
        git(path, "commit", "-q", "-m", f"Work on part {i}", GIT_AUTHOR_DATE=d, GIT_COMMITTER_DATE=d)
    open(os.path.join(path, "src", "latest.txt"), "w").write("latest\n")
    d = iso(ago(**r["last"]))
    git(path, "add", "-A")
    git(path, "commit", "-q", "-m", r["msg"], GIT_AUTHOR_DATE=d, GIT_COMMITTER_DATE=d)
    if r["branch"] != "main":
        git(path, "checkout", "-q", "-b", r["branch"])
    for k in range(r["dirty"]):
        open(os.path.join(path, "src", f"wip{k}.txt"), "w").write("wip\n")
    # files keep the age of their last commit, so "Recent" ordering looks lived-in
    t = ago(**r["last"]).timestamp()
    for root, dirs, files in os.walk(path):
        for f in files + dirs:
            os.utime(os.path.join(root, f), (t, t), follow_symlinks=False)
    os.utime(path, (t, t))

# ---------- sessions: (agent, repo, title, prompts, last activity) ----------
SESSIONS = [
    ("claude", "tidepool", "Fix login redirect loop", ["the login redirect loops after sign in", "the cookie is set but /dashboard still bounces back"], dict(minutes=3)),
    ("codex", "lumen-api", "Plan the rate limiter", ["plan a rate limiter for the public API", "use a token bucket per API key, not per IP"], dict(hours=1)),
    ("claude", "quill-cli", "Release notes for 0.4", ["draft release notes for 0.4", "group the changelog by breaking changes vs fixes"], dict(hours=4)),
    ("opencode", "harbor-sync", "offline-cache-v1", ["design the offline cache", "write the sync queue to IndexedDB first"], dict(days=1)),
    ("claude", "sparrow-mail", "Inbox keyboard shortcuts", ["add keyboard shortcuts to the inbox", "j and k should move between threads"], dict(days=1, hours=2)),
    ("codex", "moss-ui", "Redirect old /blog URLs", ["the old blog links 404 now", "301 every /blog/:slug to /notes/:slug"], dict(days=1, hours=5)),
    ("opencode", "moss-ui", "Dark mode tokens", ["set up dark mode", "map every gray to a semantic token"], dict(days=6)),
    ("claude", "harbor-sync", "Flaky upload test", ["the upload test fails sometimes", "it only fails when two chunks finish together"], dict(days=12)),
    ("claude", "lumen-api", "Retry webhooks with backoff", ["retry failed webhooks", "cap it at five tries, then park it in a dead-letter table"], dict(days=20)),
    ("claude", "tidepool", "Onboarding checklist", ["build the onboarding checklist", "persist progress per workspace"], dict(days=2)),
    ("codex", "tidepool", "Audit the auth middleware", ["review the auth middleware for edge cases"], dict(days=4)),
    ("claude", None, "Scratch: regex for semver", ["regex that matches semver with pre-release tags"], dict(days=3)),
]
oc = None
for agent, repo, title, prompts, last in SESSIONS:
    cwd = os.path.join(codes, repo) if repo else os.path.join(home, "Desktop")
    os.makedirs(cwd, exist_ok=True)
    end = ago(**last)
    times = [end - timedelta(minutes=9 * (len(prompts) - i)) for i in range(len(prompts))] + [end]
    if agent == "claude":
        sid = str(uuid.uuid5(uuid.NAMESPACE_URL, title))
        folder = os.path.join(home, ".claude/projects", cwd.replace("/", "-").replace(".", "-"))
        os.makedirs(folder, exist_ok=True)
        lines = []
        for i, p in enumerate(prompts):
            lines.append({"type": "user", "cwd": cwd, "sessionId": sid, "message": {"role": "user", "content": p}, "timestamp": iso(times[i]), "uuid": str(uuid.uuid4())})
            lines.append({"type": "assistant", "cwd": cwd, "sessionId": sid, "message": {"role": "assistant", "content": [{"type": "text", "text": "On it."}]}, "timestamp": iso(times[i + 1]), "uuid": str(uuid.uuid4())})
        lines.append({"type": "ai-title", "aiTitle": title, "sessionId": sid})
        f = os.path.join(folder, sid + ".jsonl")
        open(f, "w").write("\n".join(json.dumps(l) for l in lines) + "\n")
        os.utime(f, (end.timestamp(), end.timestamp()))
    elif agent == "codex":
        sid = str(uuid.uuid5(uuid.NAMESPACE_URL, "codex" + title))
        folder = os.path.join(home, ".codex/sessions", end.strftime("%Y/%m/%d"))
        os.makedirs(folder, exist_ok=True)
        f = os.path.join(folder, f"rollout-{end.strftime('%Y-%m-%dT%H-%M-%S')}-{sid}.jsonl")
        lines = [{"timestamp": iso(times[0]), "type": "session_meta", "payload": {"id": sid, "cwd": cwd, "timestamp": iso(times[0])}}]
        lines += [{"timestamp": iso(times[i]), "type": "response_item", "payload": {"type": "message", "role": "user", "content": [{"type": "input_text", "text": p}]}} for i, p in enumerate(prompts)]
        open(f, "w").write("\n".join(json.dumps(l) for l in lines) + "\n")
        os.utime(f, (end.timestamp(), end.timestamp()))
    else:
        if oc is None:
            os.makedirs(os.path.join(home, ".local/share/opencode"), exist_ok=True)
            oc = sqlite3.connect(os.path.join(home, ".local/share/opencode/opencode.db"))
            oc.executescript("""CREATE TABLE session(id TEXT PRIMARY KEY, parent_id TEXT, directory TEXT, title TEXT, time_created INTEGER, time_updated INTEGER, time_archived INTEGER);
              CREATE TABLE message(id TEXT PRIMARY KEY, session_id TEXT, time_created INTEGER, data TEXT);
              CREATE TABLE part(id TEXT PRIMARY KEY, message_id TEXT, session_id TEXT, time_created INTEGER, data TEXT);""")
        sid = "ses_" + uuid.uuid5(uuid.NAMESPACE_URL, title).hex[:12]
        ms = lambda d: int(d.timestamp() * 1000)
        oc.execute("INSERT INTO session VALUES (?,?,?,?,?,?,?)", (sid, None, cwd, title, ms(times[0]), ms(end), None))
        for i, p in enumerate(prompts):
            mid = f"msg_{sid}_{i}"
            oc.execute("INSERT INTO message VALUES (?,?,?,?)", (mid, sid, ms(times[i]), json.dumps({"role": "user"})))
            oc.execute("INSERT INTO part VALUES (?,?,?,?,?)", (f"prt_{sid}_{i}", mid, sid, ms(times[i]), json.dumps({"type": "text", "text": p})))
if oc:
    oc.commit()

# ---------- settings: skip onboarding, scan ~/codes ----------
support = os.path.join(home, "Library/Application Support/Repobox")
os.makedirs(support)
json.dump({"roots": [codes], "depth": 4, "agents": ["Claude", "OpenCode", "Codex"], "editor": "Cursor", "terminal": "Terminal",
           "theme": "Light", "layout": "Cards", "painting": "Auto", "onboarded": True}, open(os.path.join(support, "settings.json"), "w"), indent=2)
print(home)

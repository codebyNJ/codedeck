//! OpenCode: ~/.local/share/opencode/opencode.db (SQLite).

use super::{Read, short_title};
use crate::{Agent, Session};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

pub fn read(home: &Path) -> anyhow::Result<Read> {
    let db = home.join(".local/share/opencode/opencode.db");
    // Read-only: OpenCode may be writing to it right now.
    let conn = Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut statement = conn.prepare(
        "SELECT s.id, s.directory, s.title, s.time_updated,
                (SELECT count(*) FROM message m WHERE m.session_id = s.id AND json_extract(m.data, '$.role') = 'user'),
                (SELECT json_extract(p.data, '$.text') FROM part p JOIN message m ON m.id = p.message_id
                  WHERE p.session_id = s.id AND json_extract(m.data, '$.role') = 'user' AND json_extract(p.data, '$.type') = 'text'
                  ORDER BY p.time_created DESC LIMIT 1)
         FROM session s WHERE s.parent_id IS NULL AND s.time_archived IS NULL",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(Session {
            agent: Agent::OpenCode,
            id: row.get(0)?,
            cwd: row.get::<_, String>(1)?.into(),
            title: short_title(&row.get::<_, String>(2)?),
            updated_ms: row.get(3)?,
            prompts: row.get(4)?,
            last_prompt: short_title(&row.get::<_, Option<String>>(5)?.unwrap_or_default()),
        })
    })?;
    let mut read = Read::default();
    for row in rows {
        match row {
            Ok(session) => read.sessions.push(session),
            Err(_) => read.skipped += 1,
        }
    }
    Ok(read)
}

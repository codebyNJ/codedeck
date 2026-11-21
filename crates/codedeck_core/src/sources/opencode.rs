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
                (SELECT count(*) FROM message m WHERE m.session_id = s.id AND json_extract(m.data, '$.role') = 'user')
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

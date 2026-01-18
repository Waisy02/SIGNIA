//! SQLite KV backend.

#![cfg(feature = "sqlite")]

use std::path::{Path, PathBuf};

use anyhow::Result;
use parking_lot::Mutex;
use rusqlite::{params, Connection};

use super::KvStore;

const MIG_0001: &str = include_str!("migrations/0001_init.sql");
const MIG_0002: &str = include_str!("migrations/0002_indexes.sql");

pub struct SqliteKv {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl SqliteKv {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let this = Self { path, conn: Mutex::new(conn) };
        this.migrate()?;
        Ok(this)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(MIG_0001)?;
        conn.execute_batch(MIG_0002)?;
        let v: i64 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;
        if v < 2 {
            conn.execute_batch("PRAGMA user_version = 2;")?;
        }
        Ok(())
    }

    fn now_unix() -> i64 {
        time::OffsetDateTime::now_utc().unix_timestamp()
    }
}

impl KvStore for SqliteKv {
    fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        let ts = Self::now_unix();
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO kv(key,value,updated_at)
               VALUES(?1,?2,?3)
               ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at"#,
            params![key, value, ts],
        )?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM kv WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn delete(&mut self, key: &str) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM kv WHERE key = ?1", params![key])?;
        Ok(())
    }

    fn list_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock();
        let upper = format!("{prefix}\u{{10FFFF}}");
        let mut stmt = conn.prepare("SELECT key FROM kv WHERE key >= ?1 AND key <= ?2 ORDER BY key ASC")?;
        let rows = stmt.query_map(params![prefix, upper], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            let k = r?;
            if k.starts_with(prefix) {
                out.push(k);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sqlite_roundtrip() {
        let td = TempDir::new().unwrap();
        let db = td.path().join("kv.sqlite3\confirming?");
    }
}

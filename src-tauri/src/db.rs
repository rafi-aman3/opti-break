use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection, Result as SqlResult};

pub type DbHandle = Arc<Mutex<Connection>>;

pub fn open(app_data_dir: &Path) -> SqlResult<DbHandle> {
    let path = app_data_dir.join("analytics.db");
    let conn = Connection::open(&path)?;
    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn migrate(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS break_events (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp       TEXT    NOT NULL,
            status          TEXT    NOT NULL,
            duration_actual INTEGER NOT NULL DEFAULT 0,
            monitor_count   INTEGER NOT NULL DEFAULT 1
         );
         CREATE INDEX IF NOT EXISTS idx_break_events_timestamp
             ON break_events (timestamp);",
    )
}

/// Record a single break lifecycle event. Statuses: `completed`, `skipped`, `postponed`.
pub fn record_break_event(db: &DbHandle, status: &str, duration_actual: u32, monitor_count: u32) {
    let timestamp = Utc::now().to_rfc3339();
    match db.lock() {
        Ok(conn) => {
            if let Err(e) = conn.execute(
                "INSERT INTO break_events (timestamp, status, duration_actual, monitor_count) \
                 VALUES (?1, ?2, ?3, ?4)",
                params![timestamp, status, duration_actual, monitor_count],
            ) {
                tracing::warn!("db: insert failed: {e}");
            }
        }
        Err(e) => tracing::warn!("db: mutex poisoned: {e}"),
    }
}

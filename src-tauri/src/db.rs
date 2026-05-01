use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection, Result as SqlResult};
use serde::Serialize;

pub type DbHandle = Arc<Mutex<Connection>>;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct BreakEvent {
    pub id: i64,
    pub timestamp: String,
    pub status: String,
    pub duration_actual: u32,
    pub monitor_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DayStats {
    pub date: String, // YYYY-MM-DD local
    pub completed: u32,
    pub skipped: u32,
    pub postponed: u32,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Aggregates {
    pub total_completed: u32,
    pub total_skipped: u32,
    pub total_postponed: u32,
    /// Sum of `duration_actual` for completed events (seconds).
    pub total_duration_secs: u64,
}

// ── Open / migrate ────────────────────────────────────────────────────────────

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

// ── Write ─────────────────────────────────────────────────────────────────────

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

// ── Read ──────────────────────────────────────────────────────────────────────

/// Events with UTC timestamp in [from, to) — both in RFC3339.
pub fn query_events_range(db: &DbHandle, from: &str, to: &str) -> SqlResult<Vec<BreakEvent>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, status, duration_actual, monitor_count \
         FROM break_events WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp",
    )?;
    let rows = stmt
        .query_map(params![from, to], |row| {
            Ok(BreakEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                status: row.get(2)?,
                duration_actual: row.get(3)?,
                monitor_count: row.get(4)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(rows)
}

/// Per-day breakdown grouped by local date for the last `days` days.
pub fn query_day_stats(db: &DbHandle, days: u32) -> SqlResult<Vec<DayStats>> {
    let cutoff = (Utc::now() - chrono::Duration::days(days as i64)).to_rfc3339();

    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT timestamp, status FROM break_events WHERE timestamp >= ?1 ORDER BY timestamp",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(Result::ok)
        .collect();

    // Group by local date (handles DST and timezone correctly).
    let mut map: BTreeMap<String, DayStats> = BTreeMap::new();
    for (ts, status) in rows {
        if let Ok(dt) = DateTime::parse_from_rfc3339(&ts) {
            let date = dt.with_timezone(&Local).format("%Y-%m-%d").to_string();
            let entry = map.entry(date.clone()).or_insert(DayStats {
                date,
                completed: 0,
                skipped: 0,
                postponed: 0,
            });
            match status.as_str() {
                "completed" => entry.completed += 1,
                "skipped" => entry.skipped += 1,
                "postponed" => entry.postponed += 1,
                _ => {}
            }
        }
    }

    Ok(map.into_values().collect())
}

/// All-time aggregate counts and durations.
pub fn query_aggregates(db: &DbHandle) -> SqlResult<Aggregates> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT \
            COALESCE(SUM(CASE WHEN status='completed'  THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status='skipped'    THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status='postponed'  THEN 1 ELSE 0 END), 0), \
            COALESCE(SUM(CASE WHEN status='completed'  THEN duration_actual ELSE 0 END), 0) \
         FROM break_events",
        [],
        |row| {
            Ok(Aggregates {
                total_completed: row.get(0)?,
                total_skipped: row.get(1)?,
                total_postponed: row.get(2)?,
                total_duration_secs: row.get(3)?,
            })
        },
    )
}

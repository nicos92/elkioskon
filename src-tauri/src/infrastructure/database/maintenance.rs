use rusqlite::Connection;

pub(crate) fn purge_old_audit_logs(conn: &Connection) -> Result<(), rusqlite::Error> {
    const AUDIT_LOG_RETENTION_DAYS: i64 = 90;
    conn.execute(
        "DELETE FROM audit_logs
         WHERE datetime(created_at) < datetime('now', ?1, 'localtime')",
        rusqlite::params![format!("-{} days", AUDIT_LOG_RETENTION_DAYS)],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purge_removes_only_logs_older_than_retention() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE audit_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                screen TEXT NOT NULL,
                action TEXT NOT NULL,
                detail TEXT,
                created_at TEXT NOT NULL
            );",
        )
        .unwrap();

        let now = chrono::Utc::now();
        let old = (now - chrono::Duration::days(120)).to_rfc3339();
        let recent = now.to_rfc3339();

        conn.execute(
            "INSERT INTO audit_logs (user_id, username, screen, action, detail, created_at)
             VALUES (1, 'admin', 'Test', 'old', NULL, ?1)",
            rusqlite::params![old],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO audit_logs (user_id, username, screen, action, detail, created_at)
             VALUES (1, 'admin', 'Test', 'recent', NULL, ?1)",
            rusqlite::params![recent],
        )
        .unwrap();

        purge_old_audit_logs(&conn).unwrap();

        let mut stmt = conn.prepare("SELECT action FROM audit_logs").unwrap();
        let remaining: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(remaining, vec!["recent".to_string()]);
    }
}

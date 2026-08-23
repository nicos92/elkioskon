use rusqlite::Connection;

use crate::infrastructure::database::config::BCRYPT_COST;

pub(crate) fn seed_admin_user(conn: &Connection) -> Result<(), rusqlite::Error> {
    let username = "admin";
    let now = chrono::Utc::now().to_rfc3339();

    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?1)",
        [username],
        |row| row.get(0),
    )?;

    if !exists {
        let bcrypt_hash = bcrypt::hash("admin123", BCRYPT_COST).expect("Failed to hash password");

        conn.execute(
            "INSERT INTO users (username, password, active, created_at, modified_at) VALUES (?1, ?2, 1, ?3, ?3)",
            rusqlite::params![username, bcrypt_hash, now],
        )?;
    }

    let admin_id: i64 = conn.query_row(
        "SELECT id FROM users WHERE username = ?1",
        [username],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare("SELECT id FROM permissions")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let perm_id: i64 = row.get(0)?;
        conn.execute(
            "INSERT OR IGNORE INTO user_permissions (user_id, permission_id, assigned_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![admin_id, perm_id, now],
        )?;
    }

    Ok(())
}

use rusqlite::params;

use crate::domain::entities::AuditLog;
use crate::domain::repositories::{AuditLogFilter, AuditLogRepository, Page};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteAuditLogRepository;

impl Default for SqliteAuditLogRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteAuditLogRepository {
    pub fn new() -> Self {
        Self
    }
}

impl AuditLogRepository for SqliteAuditLogRepository {
    fn get_username(&self, user_id: i64) -> Result<Option<String>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let username: Option<String> = conn
            .query_row(
                "SELECT username FROM users WHERE id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .ok();

        Ok(username)
    }

    fn create(&self, log: &AuditLog) -> Result<AuditLog, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO audit_logs (user_id, username, screen, action, detail, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                log.user_id,
                log.username,
                log.screen,
                log.action,
                log.detail,
                log.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(AuditLog { id, ..log.clone() })
    }

    fn find_with_filters(&self, filter: &AuditLogFilter) -> Result<Page<AuditLog>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut sql = String::from(
            "SELECT id, user_id, username, screen, action, detail, created_at
             FROM audit_logs WHERE 1 = 1",
        );
        let mut conditions: Vec<String> = Vec::new();

        if let Some(user_id) = filter.user_id {
            conditions.push(format!("user_id = {}", user_id));
        }
        if let Some(screen) = &filter.screen {
            conditions.push(format!("screen = '{}'", screen.replace('\'', "''")));
        }
        if let Some(action) = &filter.action {
            conditions.push(format!("action = '{}'", action.replace('\'', "''")));
        }
        if let Some(from) = &filter.from {
            conditions.push(format!(
                "datetime(created_at) >= datetime('{}')",
                from.replace('\'', "''")
            ));
        }
        if let Some(to) = &filter.to {
            conditions.push(format!(
                "datetime(created_at) <= datetime('{}')",
                to.replace('\'', "''")
            ));
        }

        if !conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&conditions.join(" AND "));
        }

        let limit = filter.limit.unwrap_or(20).max(1);
        let offset = filter.offset.unwrap_or(0).max(0);

        let total: i64 = {
            let count_sql = format!(
                "SELECT COUNT(*) FROM audit_logs WHERE 1 = 1 AND {}",
                if conditions.is_empty() {
                    "1 = 1".to_string()
                } else {
                    conditions.join(" AND ")
                }
            );
            conn.query_row(&count_sql, [], |row| row.get(0))?
        };

        sql.push_str(" ORDER BY created_at DESC, id DESC");
        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;

        let mut logs = Vec::new();
        while let Some(row) = rows.next()? {
            logs.push(row_to_audit_log(row)?);
        }

        Ok(Page {
            items: logs,
            total,
            limit,
            offset,
        })
    }
}

fn row_to_audit_log(row: &rusqlite::Row) -> Result<AuditLog, AppError> {
    let created_at_str: String = row.get(6)?;
    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(AuditLog {
        id: row.get(0)?,
        user_id: row.get(1)?,
        username: row.get(2)?,
        screen: row.get(3)?,
        action: row.get(4)?,
        detail: row.get(5)?,
        created_at,
    })
}

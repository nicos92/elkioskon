use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::domain::entities::{Permission, User, UserPermission};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteUserRepository;

impl Default for SqliteUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteUserRepository {
    pub fn new() -> Self {
        Self
    }
}

impl UserRepository for SqliteUserRepository {
    fn create(&self, user: &User) -> Result<User, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO users (username, password, active, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user.username,
                user.password,
                user.active as i32,
                user.created_at.to_rfc3339(),
                user.modified_at.to_rfc3339()
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(User { id, ..user.clone() })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, username, password, active, created_at, modified_at FROM users WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, username, password, active, created_at, modified_at FROM users WHERE username = ?1"
        )?;

        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<User>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, username, password, active, created_at, modified_at FROM users WHERE username != 'admin'")?;

        let mut users = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            users.push(self.row_to_user(row)?);
        }

        Ok(users)
    }

    fn update(&self, user: &User) -> Result<User, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE users SET username = ?1, password = ?2, active = ?3, modified_at = ?4 WHERE id = ?5",
            params![
                user.username,
                user.password,
                user.active as i32,
                Utc::now().to_rfc3339(),
                user.id
            ],
        )?;

        Ok(user.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute(
            "DELETE FROM user_permissions WHERE user_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn add_permission(&self, user_id: i64, permission_id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR IGNORE INTO user_permissions (user_id, permission_id, assigned_at) VALUES (?1, ?2, ?3)",
            params![user_id, permission_id, now],
        )?;

        Ok(())
    }

    fn remove_permission(&self, user_id: i64, permission_id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "DELETE FROM user_permissions WHERE user_id = ?1 AND permission_id = ?2",
            params![user_id, permission_id],
        )?;

        Ok(())
    }

    fn get_user_permissions(&self, user_id: i64) -> Result<Vec<UserPermission>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT p.id, p.permission, p.created, up.assigned_at
             FROM permissions p 
             INNER JOIN user_permissions up ON p.id = up.permission_id 
             WHERE up.user_id = ?1",
        )?;

        let mut permissions = Vec::new();
        let mut rows = stmt.query(params![user_id])?;

        while let Some(row) = rows.next()? {
            permissions.push(UserPermission {
                id: row.get(0)?,
                permission: row.get(1)?,
                created: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                assigned_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }

        Ok(permissions)
    }

    fn get_all_permissions(&self) -> Result<Vec<Permission>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT id, permission, created FROM permissions")?;

        let mut permissions = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            permissions.push(Permission {
                id: row.get(0)?,
                permission: row.get(1)?,
                created: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }

        Ok(permissions)
    }

    fn create_permission(&self, permission: &Permission) -> Result<Permission, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO permissions (permission, created) VALUES (?1, ?2)",
            params![permission.permission, permission.created.to_rfc3339()],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Permission {
            id,
            ..permission.clone()
        })
    }

    fn has_permission(&self, user_id: i64, permission_name: &str) -> Result<bool, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM user_permissions up
             INNER JOIN permissions p ON up.permission_id = p.id
             WHERE up.user_id = ?1 AND p.permission = ?2",
            params![user_id, permission_name],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    fn get_user_permissions_by_names(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT p.permission 
             FROM permissions p 
             INNER JOIN user_permissions up ON p.id = up.permission_id 
             WHERE up.user_id = ?1",
        )?;

        let mut permissions = Vec::new();
        let mut rows = stmt.query(params![user_id])?;

        while let Some(row) = rows.next()? {
            permissions.push(row.get(0)?);
        }

        Ok(permissions)
    }
}

impl SqliteUserRepository {
    fn row_to_user(&self, row: &rusqlite::Row) -> Result<User, AppError> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            active: row.get::<_, i32>(3)? != 0,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let created = repo
            .create(&User::new("testuser".to_string(), "secret".to_string()))
            .unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.username, "testuser");
        assert_eq!(found.password, "secret");
        assert!(found.active);
    }

    #[test]
    fn find_by_username_returns_none_when_missing() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        assert!(repo.find_by_username("testuser").unwrap().is_none());

        let created = repo
            .create(&User::new("testuser".to_string(), "secret".to_string()))
            .unwrap();
        let found = repo.find_by_username("testuser").unwrap().unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn find_all_excludes_admin() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        repo.create(&User::new("testuser".to_string(), "secret".to_string()))
            .unwrap();
        let users = repo.find_all().unwrap();
        assert!(users.iter().any(|u| u.username == "testuser"));
        assert!(!users.iter().any(|u| u.username == "admin"));
    }

    #[test]
    fn update_changes_username() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let mut created = repo
            .create(&User::new("before".to_string(), "secret".to_string()))
            .unwrap();
        created.username = "after".to_string();
        repo.update(&created).unwrap();

        assert!(repo.find_by_username("before").unwrap().is_none());
        let updated = repo.find_by_username("after").unwrap().unwrap();
        assert_eq!(updated.id, created.id);
    }

    #[test]
    fn delete_removes_user_and_permissions() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let created = repo
            .create(&User::new("todelete".to_string(), "secret".to_string()))
            .unwrap();
        let perms = repo.get_all_permissions().unwrap();
        repo.add_permission(created.id, perms[0].id).unwrap();

        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
        assert!(repo.get_user_permissions(created.id).unwrap().is_empty());
    }

    #[test]
    fn add_and_remove_permission() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let user = repo
            .create(&User::new("permuser".to_string(), "secret".to_string()))
            .unwrap();
        let perms = repo.get_all_permissions().unwrap();
        let perm = perms
            .iter()
            .find(|p| p.permission == "ver_usuarios")
            .unwrap();

        assert!(!repo.has_permission(user.id, "ver_usuarios").unwrap());
        repo.add_permission(user.id, perm.id).unwrap();
        assert!(repo.has_permission(user.id, "ver_usuarios").unwrap());
        let names = repo.get_user_permissions_by_names(user.id).unwrap();
        assert!(names.contains(&"ver_usuarios".to_string()));

        repo.remove_permission(user.id, perm.id).unwrap();
        assert!(!repo.has_permission(user.id, "ver_usuarios").unwrap());
        assert!(repo.get_user_permissions(user.id).unwrap().is_empty());
    }

    #[test]
    fn create_permission_duplicate_maps_duplicate_value() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let err = repo
            .create_permission(&Permission::new("ver_usuarios".to_string()))
            .unwrap_err();
        assert!(matches!(err, AppError::DuplicateValue), "{:?}", err);
    }

    #[test]
    fn get_all_permissions_returns_seeded_permissions() {
        let _guard = fresh_db();
        let repo = SqliteUserRepository::new();

        let perms = repo.get_all_permissions().unwrap();
        assert_eq!(perms.len(), 47);
        assert!(perms.iter().any(|p| p.permission == "ver_usuarios"));
        assert!(perms.iter().any(|p| p.permission == "ver_clientes"));
    }
}

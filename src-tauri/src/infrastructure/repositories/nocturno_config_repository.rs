use rusqlite::params;

use crate::domain::entities::NocturnoConfig;
use crate::domain::repositories::NocturnoConfigRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteNocturnoConfigRepository;

impl Default for SqliteNocturnoConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteNocturnoConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

impl NocturnoConfigRepository for SqliteNocturnoConfigRepository {
    fn get_config(&self) -> Result<NocturnoConfig, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let config = conn.query_row(
            "SELECT activo, hora_inicio, hora_fin
             FROM nocturno_config WHERE id = 1",
            [],
            |row| {
                Ok(NocturnoConfig {
                    activo: row.get::<_, i64>(0)? != 0,
                    hora_inicio: row.get(1)?,
                    hora_fin: row.get(2)?,
                })
            },
        )?;

        Ok(config)
    }

    fn save_config(&self, config: &NocturnoConfig) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO nocturno_config (id, activo, hora_inicio, hora_fin)
             VALUES (1, ?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET
                 activo = excluded.activo,
                 hora_inicio = excluded.hora_inicio,
                 hora_fin = excluded.hora_fin",
            params![config.activo as i64, config.hora_inicio, config.hora_fin],
        )?;

        Ok(())
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
    fn get_config_returns_defaults_when_seeded() {
        let _guard = fresh_db();
        let repo = SqliteNocturnoConfigRepository::new();

        let config = repo.get_config().unwrap();
        assert!(!config.activo);
        assert_eq!(config.hora_inicio, "22:00");
        assert_eq!(config.hora_fin, "06:00");
    }

    #[test]
    fn save_config_persists_and_get_returns_it() {
        let _guard = fresh_db();
        let repo = SqliteNocturnoConfigRepository::new();

        let config = NocturnoConfig {
            activo: true,
            hora_inicio: "23:00".to_string(),
            hora_fin: "07:00".to_string(),
        };
        repo.save_config(&config).unwrap();

        let loaded = repo.get_config().unwrap();
        assert!(loaded.activo);
        assert_eq!(loaded.hora_inicio, "23:00");
        assert_eq!(loaded.hora_fin, "07:00");

        let count: i64 = DB
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM nocturno_config", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}
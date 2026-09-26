use std::path::Path;

use rusqlite::params;

use crate::domain::entities::{RespaldoInfo, RespaldoResult};
use crate::domain::repositories::RespaldoRepository;
use crate::infrastructure::database::{get_db_path, DB};
use crate::infrastructure::error::AppError;

pub struct SqliteRespaldoRepository;

impl Default for SqliteRespaldoRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteRespaldoRepository {
    pub fn new() -> Self {
        Self
    }
}

impl RespaldoRepository for SqliteRespaldoRepository {
    fn crear_respaldo(&self, destino: &Path) -> Result<RespaldoResult, AppError> {
        let destino_str = destino.to_string_lossy().to_string();
        if destino_str.trim().is_empty() {
            return Err(AppError::RespaldoPathInvalido);
        }

        let padre = match destino.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => Path::new(".").to_path_buf(),
        };
        std::fs::create_dir_all(&padre)
            .map_err(|e| AppError::RespaldoError(format!("{} ({})", padre.display(), e)))?;
        if !padre.is_dir() {
            return Err(AppError::RespaldoPathInvalido);
        }

        if destino.exists() {
            std::fs::remove_file(destino)
                .map_err(|e| AppError::RespaldoError(format!("{} ({})", destino.display(), e)))?;
        }

        let tamano_bytes = {
            let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
            conn.execute("VACUUM INTO ?1", params![destino_str])?;
            tamano_de_archivo(destino)?
        };

        Ok(RespaldoResult {
            ruta: destino_str,
            tamano_bytes,
            generado_en: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn info(&self) -> Result<RespaldoInfo, AppError> {
        let ruta = get_db_path();
        Ok(RespaldoInfo {
            ruta_base_datos: ruta.to_string_lossy().to_string(),
            tamano_base_datos_bytes: tamano_de_archivo(&ruta).unwrap_or(0),
        })
    }
}

fn tamano_de_archivo(ruta: &Path) -> Result<u64, AppError> {
    std::fs::metadata(ruta)
        .map(|m| m.len())
        .map_err(|e| AppError::RespaldoError(format!("{} ({})", ruta.display(), e)))
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

    fn repo() -> SqliteRespaldoRepository {
        SqliteRespaldoRepository::new()
    }

    #[test]
    fn crear_respaldo_genera_un_sqlite_valido() {
        let _guard = fresh_db();
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");

        let resultado = repo().crear_respaldo(&destino).unwrap();

        assert_eq!(resultado.ruta, destino.to_string_lossy().to_string());
        assert!(destino.exists());
        assert!(resultado.tamano_bytes > 0);
        assert!(!resultado.generado_en.is_empty());
    }

    #[test]
    fn crear_respaldo_conserva_los_datos_de_la_base() {
        let _guard = fresh_db();
        {
            let conn = DB.lock().unwrap();
            conn.execute("INSERT INTO categorias (categoria) VALUES ('Bebidas')", [])
                .unwrap();
        }

        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");
        repo().crear_respaldo(&destino).unwrap();

        let copia = rusqlite::Connection::open(&destino).unwrap();
        let total: i64 = copia
            .query_row(
                "SELECT COUNT(*) FROM categorias WHERE categoria = 'Bebidas'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 1);
    }

    #[test]
    fn crear_respaldo_sobrescribe_un_archivo_previo() {
        let _guard = fresh_db();
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");
        std::fs::write(&destino, b"contenido viejo").unwrap();

        let resultado = repo().crear_respaldo(&destino).unwrap();

        assert!(resultado.tamano_bytes > 0);
        let copia = rusqlite::Connection::open(&destino).unwrap();
        let integridad: String = copia
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
    }

    #[test]
    fn crear_respaldo_rechaza_ruta_vacia() {
        let _guard = fresh_db();
        assert!(matches!(
            repo().crear_respaldo(Path::new("")),
            Err(AppError::RespaldoPathInvalido)
        ));
    }

    #[test]
    fn info_expone_la_ruta_de_la_base_datos() {
        let _guard = fresh_db();
        let info = repo().info().unwrap();
        assert_eq!(
            info.ruta_base_datos,
            get_db_path().to_string_lossy().to_string()
        );
    }
}

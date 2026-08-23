use rusqlite::params;

use crate::domain::entities::TipoVenta;
use crate::domain::repositories::TipoVentaRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteTipoVentaRepository;

impl Default for SqliteTipoVentaRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteTipoVentaRepository {
    pub fn new() -> Self {
        Self
    }
}

impl TipoVentaRepository for SqliteTipoVentaRepository {
    fn create(&self, tipo: &TipoVenta) -> Result<TipoVenta, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO tipos_venta (nombre, hacia_donde, created_at) VALUES (?1, ?2, ?3)",
            params![tipo.nombre, tipo.hacia_donde, tipo.created_at],
        )?;

        let id = conn.last_insert_rowid();
        Ok(TipoVenta { id, ..tipo.clone() })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<TipoVenta>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, nombre, hacia_donde, created_at FROM tipos_venta WHERE id = ?1")?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_tipo_venta(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_nombre(&self, nombre: &str) -> Result<Option<TipoVenta>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, hacia_donde, created_at FROM tipos_venta WHERE nombre = ?1",
        )?;

        let mut rows = stmt.query(params![nombre])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_tipo_venta(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<TipoVenta>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT id, nombre, hacia_donde, created_at FROM tipos_venta ORDER BY id")?;

        let mut tipos = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            tipos.push(self.row_to_tipo_venta(row)?);
        }

        Ok(tipos)
    }

    fn update(&self, tipo: &TipoVenta) -> Result<TipoVenta, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE tipos_venta SET nombre = ?1, hacia_donde = ?2 WHERE id = ?3",
            params![tipo.nombre, tipo.hacia_donde, tipo.id],
        )?;

        Ok(tipo.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("DELETE FROM tipos_venta WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn has_ventas(&self, id: i64) -> Result<bool, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM ventas WHERE id_tipo_venta = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }
}

impl SqliteTipoVentaRepository {
    fn row_to_tipo_venta(&self, row: &rusqlite::Row) -> Result<TipoVenta, AppError> {
        Ok(TipoVenta {
            id: row.get(0)?,
            nombre: row.get(1)?,
            hacia_donde: row.get(2)?,
            created_at: row.get(3)?,
        })
    }
}

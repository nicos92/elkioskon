use rusqlite::params;

use crate::domain::entities::{Cierre, CierreTipo, CierreWithTipos};
use crate::domain::repositories::{CierreRepository, Page};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteCierreRepository;

impl Default for SqliteCierreRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteCierreRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn insert(
        &self,
        tx: &rusqlite::Transaction,
        cierre: &Cierre,
        tipos: &[CierreTipo],
    ) -> Result<i64, AppError> {
        tx.execute(
            "INSERT INTO cierres (fecha, dia, mes, anio, total_costo, total_ganancia, total_venta, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cierre.fecha,
                cierre.dia,
                cierre.mes,
                cierre.anio,
                cierre.total_costo,
                cierre.total_ganancia,
                cierre.total_venta,
                cierre.created_at
            ],
        )?;
        let cierre_id = tx.last_insert_rowid();

        for tipo in tipos {
            tx.execute(
                "INSERT INTO cierre_tipos (id_cierre, id_tipo_venta, total) VALUES (?1, ?2, ?3)",
                params![cierre_id, tipo.id_tipo_venta, tipo.total],
            )?;
        }

        Ok(cierre_id)
    }

    pub fn load_by_id(
        &self,
        conn: &rusqlite::Connection,
        id: i64,
    ) -> Result<Option<CierreWithTipos>, AppError> {
        let mut result = self.load_cierre(conn, id)?;
        if let Some(cierre) = result.as_mut() {
            cierre.tipos = self.load_tipos(conn, id)?;
        }
        Ok(result)
    }
}

impl CierreRepository for SqliteCierreRepository {
    fn find_by_fecha(&self, fecha: &str) -> Result<Option<CierreWithTipos>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut result = None;
        {
            let mut stmt = conn.prepare(
                "SELECT id, fecha, dia, mes, anio, total_costo, total_ganancia, total_venta, created_at
                 FROM cierres WHERE fecha = ?1",
            )?;

            let mut rows = stmt.query(params![fecha])?;
            if let Some(row) = rows.next()? {
                result = Some(self.row_to_cierre(row)?);
            }
        }

        if let Some(mut cierre) = result {
            cierre.tipos = self.load_tipos(&conn, cierre.cierre.id)?;
            Ok(Some(cierre))
        } else {
            Ok(None)
        }
    }

    fn find_page(&self, limit: i64, offset: i64) -> Result<Page<CierreWithTipos>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let limit = limit.max(1);
        let offset = offset.max(0);

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM cierres", [], |row| row.get(0))?;

        let mut stmt = conn.prepare(
            "SELECT id, fecha, dia, mes, anio, total_costo, total_ganancia, total_venta, created_at
             FROM cierres ORDER BY fecha DESC, id DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let mut rows = stmt.query(params![limit, offset])?;
        let mut cierres = Vec::new();
        while let Some(row) = rows.next()? {
            let mut cierre = self.row_to_cierre(row)?;
            cierre.tipos = self.load_tipos(&conn, cierre.cierre.id)?;
            cierres.push(cierre);
        }

        Ok(Page {
            items: cierres,
            total,
            limit,
            offset,
        })
    }

    fn delete_by_fecha(&self, fecha: &str) -> Result<(), AppError> {
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;

        let deleted = tx.execute("DELETE FROM cierres WHERE fecha = ?1", params![fecha])?;
        if deleted == 0 {
            return Err(AppError::CierreNotFound);
        }

        tx.commit()?;
        Ok(())
    }
}

impl SqliteCierreRepository {
    fn row_to_cierre(&self, row: &rusqlite::Row) -> Result<CierreWithTipos, AppError> {
        Ok(CierreWithTipos {
            cierre: Cierre {
                id: row.get(0)?,
                fecha: row.get(1)?,
                dia: row.get(2)?,
                mes: row.get(3)?,
                anio: row.get(4)?,
                total_costo: row.get(5)?,
                total_ganancia: row.get(6)?,
                total_venta: row.get(7)?,
                created_at: row.get(8)?,
            },
            tipos: Vec::new(),
        })
    }

    fn load_cierre(
        &self,
        conn: &rusqlite::Connection,
        id: i64,
    ) -> Result<Option<CierreWithTipos>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT id, fecha, dia, mes, anio, total_costo, total_ganancia, total_venta, created_at
             FROM cierres WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_cierre(row)?))
        } else {
            Ok(None)
        }
    }

    fn load_tipos(
        &self,
        conn: &rusqlite::Connection,
        cierre_id: i64,
    ) -> Result<Vec<CierreTipo>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT ct.id_tipo_venta, COALESCE(t.nombre, ''), ct.total
             FROM cierre_tipos ct
             LEFT JOIN tipos_venta t ON t.id = ct.id_tipo_venta
             WHERE ct.id_cierre = ?1
             ORDER BY ct.id",
        )?;

        let mut rows = stmt.query(params![cierre_id])?;
        let mut tipos = Vec::new();
        while let Some(row) = rows.next()? {
            tipos.push(CierreTipo {
                id_tipo_venta: row.get(0)?,
                tipo_venta: row.get(1)?,
                total: row.get(2)?,
            });
        }

        Ok(tipos)
    }
}

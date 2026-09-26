use rusqlite::params;

use crate::domain::entities::Proveedor;
use crate::domain::repositories::ProveedorRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteProveedorRepository;

impl Default for SqliteProveedorRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteProveedorRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ProveedorRepository for SqliteProveedorRepository {
    fn create(&self, proveedor: &Proveedor) -> Result<Proveedor, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO proveedores (cuit, proveedor, nombre, tel, email, observacion) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                proveedor.cuit,
                proveedor.proveedor,
                proveedor.nombre,
                proveedor.tel,
                proveedor.email,
                proveedor.observacion
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Proveedor {
            id,
            ..proveedor.clone()
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Proveedor>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, cuit, proveedor, nombre, tel, email, observacion FROM proveedores WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_proveedor(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_cuit(&self, cuit: &str) -> Result<Option<Proveedor>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, cuit, proveedor, nombre, tel, email, observacion FROM proveedores WHERE cuit = ?1"
        )?;

        let mut rows = stmt.query(params![cuit])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_proveedor(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<Proveedor>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, cuit, proveedor, nombre, tel, email, observacion FROM proveedores ORDER BY proveedor"
        )?;

        let mut proveedores = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            proveedores.push(self.row_to_proveedor(row)?);
        }

        Ok(proveedores)
    }

    fn update(&self, proveedor: &Proveedor) -> Result<Proveedor, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE proveedores SET cuit = ?1, proveedor = ?2, nombre = ?3, tel = ?4, email = ?5, observacion = ?6 WHERE id = ?7",
            params![
                proveedor.cuit,
                proveedor.proveedor,
                proveedor.nombre,
                proveedor.tel,
                proveedor.email,
                proveedor.observacion,
                proveedor.id
            ],
        )?;

        Ok(proveedor.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("DELETE FROM proveedores WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn has_articulos(&self, id: i64) -> Result<bool, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM articulos WHERE id_proveedor = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }
}

impl SqliteProveedorRepository {
    fn row_to_proveedor(&self, row: &rusqlite::Row) -> Result<Proveedor, AppError> {
        Ok(Proveedor {
            id: row.get(0)?,
            cuit: row.get(1)?,
            proveedor: row.get(2)?,
            nombre: row.get(3)?,
            tel: row.get(4)?,
            email: row.get(5)?,
            observacion: row.get(6)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::fresh_test_db;

    fn sample_proveedor() -> Proveedor {
        Proveedor::new(
            "TESTPROV".to_string(),
            "Proveedor Test".to_string(),
            Some("30-00000000-1".to_string()),
            Some("1234".to_string()),
            Some("t@t.com".to_string()),
            Some("obs".to_string()),
        )
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        let created = repo.create(&sample_proveedor()).unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.proveedor, "TESTPROV");
        assert_eq!(found.nombre, "Proveedor Test");
        assert_eq!(found.cuit.as_deref(), Some("30-00000000-1"));
        assert_eq!(found.tel.as_deref(), Some("1234"));
        assert_eq!(found.email.as_deref(), Some("t@t.com"));
        assert_eq!(found.observacion.as_deref(), Some("obs"));
    }

    #[test]
    fn create_with_duplicate_cuit_maps_duplicate_value() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        repo.create(&sample_proveedor()).unwrap();
        let err = repo.create(&sample_proveedor()).unwrap_err();
        assert!(matches!(err, AppError::DuplicateValue), "{:?}", err);
    }

    #[test]
    fn find_by_cuit_returns_none_when_missing() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        assert!(repo.find_by_cuit("30-00000000-1").unwrap().is_none());

        let created = repo.create(&sample_proveedor()).unwrap();
        let found = repo.find_by_cuit("30-00000000-1").unwrap().unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn find_all_and_update() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        let mut created = repo.create(&sample_proveedor()).unwrap();
        assert!(repo.find_all().unwrap().iter().any(|p| p.id == created.id));

        created.tel = Some("9999".to_string());
        repo.update(&created).unwrap();
        let updated = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(updated.tel.as_deref(), Some("9999"));
    }

    #[test]
    fn delete_removes_proveedor() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        let created = repo.create(&sample_proveedor()).unwrap();
        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
    }

    #[test]
    fn has_articulos_returns_false_without_articulos() {
        let _guard = fresh_test_db();
        let repo = SqliteProveedorRepository::new();

        let created = repo.create(&sample_proveedor()).unwrap();
        assert!(!repo.has_articulos(created.id).unwrap());
    }
}

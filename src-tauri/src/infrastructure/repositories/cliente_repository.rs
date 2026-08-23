use rusqlite::params;

use crate::domain::entities::Cliente;
use crate::domain::repositories::ClienteRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteClienteRepository;

impl Default for SqliteClienteRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteClienteRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ClienteRepository for SqliteClienteRepository {
    fn create(&self, cliente: &Cliente) -> Result<Cliente, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO clientes (nombre, apellido, telefono, email, direccion, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                cliente.nombre,
                cliente.apellido,
                cliente.telefono,
                cliente.email,
                cliente.direccion,
                cliente.created_at,
                cliente.updated_at
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Cliente {
            id,
            ..cliente.clone()
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Cliente>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, apellido, telefono, email, direccion, created_at, updated_at
             FROM clientes WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_cliente(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_default(&self) -> Result<Option<Cliente>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, apellido, telefono, email, direccion, created_at, updated_at
             FROM clientes WHERE nombre = 'Consumidor' AND apellido = 'Final' LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_cliente(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<Cliente>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, apellido, telefono, email, direccion, created_at, updated_at
             FROM clientes ORDER BY nombre COLLATE NOCASE, apellido COLLATE NOCASE",
        )?;

        let mut clientes = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            clientes.push(self.row_to_cliente(row)?);
        }

        Ok(clientes)
    }

    fn update(&self, cliente: &Cliente) -> Result<Cliente, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE clientes
             SET nombre = ?1, apellido = ?2, telefono = ?3, email = ?4, direccion = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                cliente.nombre,
                cliente.apellido,
                cliente.telefono,
                cliente.email,
                cliente.direccion,
                cliente.updated_at,
                cliente.id
            ],
        )?;

        Ok(cliente.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("DELETE FROM clientes WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl SqliteClienteRepository {
    fn row_to_cliente(&self, row: &rusqlite::Row) -> Result<Cliente, AppError> {
        Ok(Cliente {
            id: row.get(0)?,
            nombre: row.get(1)?,
            apellido: row.get(2)?,
            telefono: row.get(3)?,
            email: row.get(4)?,
            direccion: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
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

    fn sample_cliente() -> Cliente {
        Cliente::new(
            Some("Juan".to_string()),
            Some("Pérez".to_string()),
            Some("555-1234".to_string()),
            Some("juan@mail.com".to_string()),
            None,
        )
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        let created = repo.create(&sample_cliente()).unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.nombre.as_deref(), Some("Juan"));
        assert_eq!(found.apellido.as_deref(), Some("Pérez"));
        assert_eq!(found.telefono.as_deref(), Some("555-1234"));
        assert_eq!(found.email.as_deref(), Some("juan@mail.com"));
        assert!(found.direccion.is_none());
        assert_eq!(found.created_at, found.updated_at);
    }

    #[test]
    fn find_default_returns_seeded_consumidor_final() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        let default = repo.find_default().unwrap().unwrap();
        assert_eq!(default.nombre.as_deref(), Some("Consumidor"));
        assert_eq!(default.apellido.as_deref(), Some("Final"));
    }

    #[test]
    fn find_default_is_unique_after_multiple_seeds() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        let all = repo.find_all().unwrap();
        let count = all.iter().filter(|c| c.is_default()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn find_all_and_update() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        let mut created = repo.create(&sample_cliente()).unwrap();
        assert!(repo.find_all().unwrap().iter().any(|c| c.id == created.id));

        created.telefono = Some("9999".to_string());
        created.updated_at = chrono::Utc::now().to_rfc3339();
        repo.update(&created).unwrap();
        let updated = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(updated.telefono.as_deref(), Some("9999"));
    }

    #[test]
    fn delete_removes_cliente() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        let created = repo.create(&sample_cliente()).unwrap();
        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
    }

    #[test]
    fn find_all_orders_by_nombre_then_apellido() {
        let _guard = fresh_db();
        let repo = SqliteClienteRepository::new();

        repo.create(&Cliente::new(
            Some("Zoe".to_string()),
            None,
            None,
            None,
            None,
        ))
        .unwrap();
        repo.create(&Cliente::new(
            Some("Ana".to_string()),
            Some("López".to_string()),
            None,
            None,
            None,
        ))
        .unwrap();

        let all = repo.find_all().unwrap();
        let names: Vec<Option<String>> = all.iter().map(|c| c.nombre.clone()).collect();
        let ana = names
            .iter()
            .position(|n| n.as_deref() == Some("Ana"))
            .unwrap();
        let zoe = names
            .iter()
            .position(|n| n.as_deref() == Some("Zoe"))
            .unwrap();
        assert!(ana < zoe);
    }
}

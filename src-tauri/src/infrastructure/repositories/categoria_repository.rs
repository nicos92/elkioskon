use rusqlite::params;

use crate::domain::entities::Categoria;
use crate::domain::repositories::CategoriaRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteCategoriaRepository;

impl Default for SqliteCategoriaRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteCategoriaRepository {
    pub fn new() -> Self {
        Self
    }
}

impl CategoriaRepository for SqliteCategoriaRepository {
    fn create(&self, categoria: &Categoria) -> Result<Categoria, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO categorias (categoria) VALUES (?1)",
            params![categoria.categoria],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Categoria {
            id,
            ..categoria.clone()
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Categoria>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT id, categoria FROM categorias WHERE id = ?1")?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_categoria(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_name(&self, categoria: &str) -> Result<Option<Categoria>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT id, categoria FROM categorias WHERE categoria = ?1")?;

        let mut rows = stmt.query(params![categoria])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_categoria(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<Categoria>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare("SELECT id, categoria FROM categorias ORDER BY categoria")?;

        let mut categorias = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            categorias.push(self.row_to_categoria(row)?);
        }

        Ok(categorias)
    }

    fn update(&self, categoria: &Categoria) -> Result<Categoria, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE categorias SET categoria = ?1 WHERE id = ?2",
            params![categoria.categoria, categoria.id],
        )?;

        Ok(categoria.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("DELETE FROM categorias WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn has_sub_categorias(&self, id: i64) -> Result<bool, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sub_categorias WHERE id_categoria = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }
}

impl SqliteCategoriaRepository {
    fn row_to_categoria(&self, row: &rusqlite::Row) -> Result<Categoria, AppError> {
        Ok(Categoria {
            id: row.get(0)?,
            categoria: row.get(1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::SubCategoria;
    use crate::domain::repositories::SubCategoriaRepository;
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use crate::infrastructure::repositories::SqliteSubCategoriaRepository;
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_db();
        let repo = SqliteCategoriaRepository::new();

        let created = repo
            .create(&Categoria::new("Test Cat".to_string()))
            .unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.categoria, "Test Cat");
    }

    #[test]
    fn find_by_name_and_duplicate_maps_duplicate_value() {
        let _guard = fresh_db();
        let repo = SqliteCategoriaRepository::new();

        repo.create(&Categoria::new("Bebidas Test".to_string()))
            .unwrap();
        let found = repo.find_by_name("Bebidas Test").unwrap().unwrap();
        assert_eq!(found.categoria, "Bebidas Test");
        assert!(repo.find_by_name("No existe").unwrap().is_none());

        let err = repo
            .create(&Categoria::new("Bebidas Test".to_string()))
            .unwrap_err();
        assert!(matches!(err, AppError::DuplicateValue), "{:?}", err);
    }

    #[test]
    fn find_all_and_update() {
        let _guard = fresh_db();
        let repo = SqliteCategoriaRepository::new();

        let mut created = repo.create(&Categoria::new("Cat A".to_string())).unwrap();
        assert!(repo.find_all().unwrap().iter().any(|c| c.id == created.id));

        created.categoria = "Cat B".to_string();
        repo.update(&created).unwrap();
        let updated = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(updated.categoria, "Cat B");
    }

    #[test]
    fn delete_removes_categoria() {
        let _guard = fresh_db();
        let repo = SqliteCategoriaRepository::new();

        let created = repo.create(&Categoria::new("Cat X".to_string())).unwrap();
        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
    }

    #[test]
    fn delete_categoria_with_sub_categorias_maps_foreign_key() {
        let _guard = fresh_db();
        let repo = SqliteCategoriaRepository::new();

        let created = repo.create(&Categoria::new("Cat Y".to_string())).unwrap();
        let sub_repo = SqliteSubCategoriaRepository::new();
        sub_repo
            .create(&SubCategoria::new("Sub Y".to_string(), created.id))
            .unwrap();

        let err = repo.delete(created.id).unwrap_err();
        assert!(matches!(err, AppError::ForeignKeyConstraint), "{:?}", err);
    }
}

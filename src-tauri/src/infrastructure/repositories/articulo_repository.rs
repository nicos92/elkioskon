use rusqlite::params;

use crate::domain::entities::Articulo;
use crate::domain::repositories::ArticuloRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteArticuloRepository;

impl Default for SqliteArticuloRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteArticuloRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ArticuloRepository for SqliteArticuloRepository {
    fn create(&self, articulo: &Articulo) -> Result<Articulo, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO articulos (articulo, cod_articulo, id_sub_categoria, id_proveedor) VALUES (?1, ?2, ?3, ?4)",
            params![
                articulo.articulo,
                articulo.cod_articulo,
                articulo.id_sub_categoria,
                articulo.id_proveedor
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Articulo {
            id,
            ..articulo.clone()
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Articulo>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, articulo, cod_articulo, id_sub_categoria, id_proveedor FROM articulos WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_articulo(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_codigo(&self, cod_articulo: &str) -> Result<Option<Articulo>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, articulo, cod_articulo, id_sub_categoria, id_proveedor FROM articulos WHERE cod_articulo = ?1"
        )?;

        let mut rows = stmt.query(params![cod_articulo])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_articulo(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<Articulo>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, articulo, cod_articulo, id_sub_categoria, id_proveedor FROM articulos ORDER BY articulo"
        )?;

        let mut articulos = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            articulos.push(self.row_to_articulo(row)?);
        }

        Ok(articulos)
    }

    fn update(&self, articulo: &Articulo) -> Result<Articulo, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "UPDATE articulos SET articulo = ?1, cod_articulo = ?2, id_sub_categoria = ?3, id_proveedor = ?4 WHERE id = ?5",
            params![
                articulo.articulo,
                articulo.cod_articulo,
                articulo.id_sub_categoria,
                articulo.id_proveedor,
                articulo.id
            ],
        )?;

        Ok(articulo.clone())
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute("DELETE FROM articulos WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl SqliteArticuloRepository {
    fn row_to_articulo(&self, row: &rusqlite::Row) -> Result<Articulo, AppError> {
        Ok(Articulo {
            id: row.get(0)?,
            articulo: row.get(1)?,
            cod_articulo: row.get(2)?,
            id_sub_categoria: row.get(3)?,
            id_proveedor: row.get(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Categoria, Proveedor, SubCategoria};
    use crate::domain::repositories::{
        CategoriaRepository, ProveedorRepository, SubCategoriaRepository,
    };
    use crate::infrastructure::database::fresh_test_db;
    use crate::infrastructure::repositories::{
        SqliteCategoriaRepository, SqliteProveedorRepository, SqliteSubCategoriaRepository,
    };

    struct Fixture {
        id_sub_categoria: i64,
        id_proveedor: i64,
    }

    fn create_fixture() -> Fixture {
        let cat_repo = SqliteCategoriaRepository::new();
        let cat = cat_repo
            .create(&Categoria::new("Cat ART".to_string()))
            .unwrap();
        let sub_repo = SqliteSubCategoriaRepository::new();
        let sub = sub_repo
            .create(&SubCategoria::new("Sub ART".to_string(), cat.id))
            .unwrap();
        let prov_repo = SqliteProveedorRepository::new();
        let prov = prov_repo
            .create(&Proveedor::new(
                "PROV ART".to_string(),
                "Prov Art".to_string(),
                None,
                None,
                None,
                None,
            ))
            .unwrap();
        Fixture {
            id_sub_categoria: sub.id,
            id_proveedor: prov.id,
        }
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_test_db();
        let fx = create_fixture();
        let repo = SqliteArticuloRepository::new();

        let created = repo
            .create(&Articulo::new(
                "Articulo Test".to_string(),
                "ART-001".to_string(),
                fx.id_sub_categoria,
                fx.id_proveedor,
            ))
            .unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.articulo, "Articulo Test");
        assert_eq!(found.cod_articulo, "ART-001");
        assert_eq!(found.id_sub_categoria, fx.id_sub_categoria);
        assert_eq!(found.id_proveedor, fx.id_proveedor);
    }

    #[test]
    fn find_by_codigo_and_duplicate_codigo_error() {
        let _guard = fresh_test_db();
        let fx = create_fixture();
        let repo = SqliteArticuloRepository::new();

        let created = repo
            .create(&Articulo::new(
                "A1".to_string(),
                "ART-002".to_string(),
                fx.id_sub_categoria,
                fx.id_proveedor,
            ))
            .unwrap();
        let found = repo.find_by_codigo("ART-002").unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert!(repo.find_by_codigo("NO-EXIST").unwrap().is_none());

        let err = repo
            .create(&Articulo::new(
                "A2".to_string(),
                "ART-002".to_string(),
                fx.id_sub_categoria,
                fx.id_proveedor,
            ))
            .unwrap_err();
        assert!(matches!(err, AppError::DuplicateValue), "{:?}", err);
    }

    #[test]
    fn find_all_and_update() {
        let _guard = fresh_test_db();
        let fx = create_fixture();
        let repo = SqliteArticuloRepository::new();

        let mut created = repo
            .create(&Articulo::new(
                "A1".to_string(),
                "ART-003".to_string(),
                fx.id_sub_categoria,
                fx.id_proveedor,
            ))
            .unwrap();
        assert!(repo.find_all().unwrap().iter().any(|a| a.id == created.id));

        created.articulo = "A1 Renombrado".to_string();
        repo.update(&created).unwrap();
        assert_eq!(
            repo.find_by_id(created.id).unwrap().unwrap().articulo,
            "A1 Renombrado"
        );
    }

    #[test]
    fn delete_removes_articulo() {
        let _guard = fresh_test_db();
        let fx = create_fixture();
        let repo = SqliteArticuloRepository::new();

        let created = repo
            .create(&Articulo::new(
                "A1".to_string(),
                "ART-004".to_string(),
                fx.id_sub_categoria,
                fx.id_proveedor,
            ))
            .unwrap();
        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
    }
}

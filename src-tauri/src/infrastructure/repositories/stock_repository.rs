use chrono::Utc;
use rusqlite::params;

use crate::domain::entities::{Stock, StockPreview};
use crate::domain::repositories::StockRepository;
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteStockRepository;

impl Default for SqliteStockRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteStockRepository {
    pub fn new() -> Self {
        Self
    }
}

impl StockRepository for SqliteStockRepository {
    fn create(&self, stock: &Stock) -> Result<Stock, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO stock (id_articulo, cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                stock.id_articulo,
                stock.cantidad,
                stock.costo,
                stock.ganancia,
                stock.ganancia_diurna,
                stock.ganancia_nocturna,
                now
            ],
        )?;

        let id = conn.last_insert_rowid();
        Ok(Stock {
            id,
            updated_at: Some(now),
            ..stock.clone()
        })
    }

    fn find_by_id(&self, id: i64) -> Result<Option<Stock>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, id_articulo, cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna, updated_at FROM stock WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_stock(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_articulo(&self, id_articulo: i64) -> Result<Option<Stock>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, id_articulo, cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna, updated_at FROM stock WHERE id_articulo = ?1",
        )?;

        let mut rows = stmt.query(params![id_articulo])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_stock(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_all(&self) -> Result<Vec<Stock>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, id_articulo, cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna, updated_at FROM stock ORDER BY id",
        )?;

        let mut stocks = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            stocks.push(self.row_to_stock(row)?);
        }

        Ok(stocks)
    }

    fn update(&self, stock: &Stock) -> Result<Stock, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE stock SET cantidad = ?1, costo = ?2, ganancia = ?3, ganancia_diurna = ?4, ganancia_nocturna = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                stock.cantidad,
                stock.costo,
                stock.ganancia,
                stock.ganancia_diurna,
                stock.ganancia_nocturna,
                now,
                stock.id
            ],
        )?;

        Ok(Stock {
            updated_at: Some(now),
            ..stock.clone()
        })
    }

    fn delete(&self, id: i64) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.execute("DELETE FROM stock WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn has_ventas(&self, id_articulo: i64) -> Result<bool, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM venta_detalle WHERE id_articulo = ?1",
            params![id_articulo],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    fn find_filtered_with_preview(
        &self,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<Vec<StockPreview>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT s.id, s.id_articulo, a.cod_articulo, a.articulo,
                    c.categoria, sc.sub_categoria, p.proveedor,
                    s.costo, s.ganancia, s.ganancia_diurna, s.ganancia_nocturna,
                    ROUND(s.costo * (1.0 + ?1 / 100.0), 2) AS costo_nuevo,
                    s.cantidad
             FROM stock s
             INNER JOIN articulos a ON s.id_articulo = a.id
             INNER JOIN sub_categorias sc ON a.id_sub_categoria = sc.id
             INNER JOIN categorias c ON sc.id_categoria = c.id
             INNER JOIN proveedores p ON a.id_proveedor = p.id
             WHERE (?2 IS NULL OR c.id = ?2)
               AND (?3 IS NULL OR sc.id = ?3)
               AND (?4 IS NULL OR p.id = ?4)
             ORDER BY a.articulo",
        )?;

        let mut previews = Vec::new();
        let mut rows = stmt.query(params![
            porcentaje,
            id_categoria,
            id_sub_categoria,
            id_proveedor
        ])?;

        while let Some(row) = rows.next()? {
            previews.push(StockPreview {
                id_stock: row.get(0)?,
                id_articulo: row.get(1)?,
                cod_articulo: row.get(2)?,
                articulo: row.get(3)?,
                categoria: row.get(4)?,
                sub_categoria: row.get(5)?,
                proveedor: row.get(6)?,
                costo_actual: row.get(7)?,
                ganancia: row.get(8)?,
                ganancia_diurna: row.get(9)?,
                ganancia_nocturna: row.get(10)?,
                costo_nuevo: row.get(11)?,
                cantidad: row.get(12)?,
            });
        }

        Ok(previews)
    }

    fn apply_costo_percentage(
        &self,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<i64, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        let affected = conn.execute(
            "UPDATE stock
             SET costo = ROUND(costo * (1.0 + ?1 / 100.0), 2),
                 updated_at = ?5
             WHERE id IN (
                 SELECT s.id
                 FROM stock s
                 INNER JOIN articulos a ON s.id_articulo = a.id
                 INNER JOIN sub_categorias sc ON a.id_sub_categoria = sc.id
                 INNER JOIN categorias c ON sc.id_categoria = c.id
                 INNER JOIN proveedores p ON a.id_proveedor = p.id
                 WHERE (?2 IS NULL OR c.id = ?2)
                   AND (?3 IS NULL OR sc.id = ?3)
                   AND (?4 IS NULL OR p.id = ?4)
             )",
            params![
                porcentaje,
                id_categoria,
                id_sub_categoria,
                id_proveedor,
                now
            ],
        )?;

        Ok(affected as i64)
    }
}

impl SqliteStockRepository {
    fn row_to_stock(&self, row: &rusqlite::Row) -> Result<Stock, AppError> {
        Ok(Stock {
            id: row.get(0)?,
            id_articulo: row.get(1)?,
            cantidad: row.get(2)?,
            costo: row.get(3)?,
            ganancia: row.get(4)?,
            ganancia_diurna: row.get(5)?,
            ganancia_nocturna: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Articulo, Categoria, Proveedor, SubCategoria};
    use crate::domain::repositories::{
        ArticuloRepository, CategoriaRepository, ProveedorRepository, SubCategoriaRepository,
    };
    use crate::infrastructure::database::{reset_test_db, DB, TEST_LOCK};
    use crate::infrastructure::repositories::{
        SqliteArticuloRepository, SqliteCategoriaRepository, SqliteProveedorRepository,
        SqliteSubCategoriaRepository,
    };
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
    }

    fn create_articulo() -> Articulo {
        let cat_repo = SqliteCategoriaRepository::new();
        let cat = cat_repo
            .create(&Categoria::new("Cat STK".to_string()))
            .unwrap();
        let sub_repo = SqliteSubCategoriaRepository::new();
        let sub = sub_repo
            .create(&SubCategoria::new("Sub STK".to_string(), cat.id))
            .unwrap();
        let prov_repo = SqliteProveedorRepository::new();
        let prov = prov_repo
            .create(&Proveedor::new(
                "PROV STK".to_string(),
                "Prov Stk".to_string(),
                None,
                None,
                None,
                None,
            ))
            .unwrap();
        SqliteArticuloRepository::new()
            .create(&Articulo::new(
                "Art STK".to_string(),
                "STK-001".to_string(),
                sub.id,
                prov.id,
            ))
            .unwrap()
    }

    #[test]
    fn create_assigns_id_and_find_by_id_round_trip() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();

        let mut stock = Stock::new(articulo.id, 10.5, 100.0, 25.0);
        stock.ganancia_diurna = 10.0;
        stock.ganancia_nocturna = 40.0;
        let created = repo.create(&stock).unwrap();
        assert!(created.id > 0);

        let found = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(found.id_articulo, articulo.id);
        assert_eq!(found.cantidad, 10.5);
        assert_eq!(found.costo, 100.0);
        assert_eq!(found.ganancia, 25.0);
        assert_eq!(found.ganancia_diurna, 10.0);
        assert_eq!(found.ganancia_nocturna, 40.0);
    }

    #[test]
    fn find_by_articulo_returns_stock() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();

        let created = repo
            .create(&Stock::new(articulo.id, 5.0, 50.0, 10.0))
            .unwrap();
        let found = repo.find_by_articulo(articulo.id).unwrap().unwrap();
        assert_eq!(found.id, created.id);
        assert!(repo.find_by_articulo(99999).unwrap().is_none());
    }

    #[test]
    fn update_changes_cantidad() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();

        let mut created = repo
            .create(&Stock::new(articulo.id, 5.0, 50.0, 10.0))
            .unwrap();
        created.cantidad = 20.0;
        created.ganancia_diurna = 15.0;
        created.ganancia_nocturna = 35.0;
        repo.update(&created).unwrap();

        let updated = repo.find_by_id(created.id).unwrap().unwrap();
        assert_eq!(updated.cantidad, 20.0);
        assert_eq!(updated.ganancia_diurna, 15.0);
        assert_eq!(updated.ganancia_nocturna, 35.0);
    }

    #[test]
    fn delete_removes_stock() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();

        let created = repo
            .create(&Stock::new(articulo.id, 5.0, 50.0, 10.0))
            .unwrap();
        repo.delete(created.id).unwrap();
        assert!(repo.find_by_id(created.id).unwrap().is_none());
        assert!(repo.find_all().unwrap().iter().all(|s| s.id != created.id));
    }

    fn insert_venta_for_articulo(id_articulo: i64) {
        let conn = DB.lock().unwrap();
        let user_id: i64 = conn
            .query_row("SELECT id FROM users WHERE username = 'admin'", [], |row| {
                row.get(0)
            })
            .unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO ventas (user_id, fecha, total, descuento, anulada, observacion, id_tipo_venta, created_at, cliente_id)
             VALUES (?1, ?2, 150.0, 0, 0, NULL, NULL, ?3, (SELECT id FROM clientes WHERE nombre = 'Consumidor' AND apellido = 'Final' LIMIT 1))",
            params![user_id, now, now],
        )
        .unwrap();
        let venta_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO venta_detalle (id_venta, id_articulo, cantidad, costo_unitario, precio_unitario, subtotal) VALUES (?1, ?2, 1.0, 100.0, 150.0, 150.0)",
            params![venta_id, id_articulo],
        )
        .unwrap();
    }

    #[test]
    fn has_ventas_returns_true_when_article_has_sales() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();
        repo.create(&Stock::new(articulo.id, 10.0, 100.0, 25.0))
            .unwrap();

        insert_venta_for_articulo(articulo.id);

        assert!(repo.has_ventas(articulo.id).unwrap());
    }

    #[test]
    fn has_ventas_returns_false_when_article_has_no_sales() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let repo = SqliteStockRepository::new();
        repo.create(&Stock::new(articulo.id, 10.0, 100.0, 25.0))
            .unwrap();

        assert!(!repo.has_ventas(articulo.id).unwrap());
        assert!(!repo.has_ventas(99999).unwrap());
    }

    fn create_articulo_with_names(cat_name: &str, sub_name: &str, prov_code: &str) -> Articulo {
        let cat_repo = SqliteCategoriaRepository::new();
        let cat = cat_repo
            .create(&Categoria::new(cat_name.to_string()))
            .unwrap();
        let sub_repo = SqliteSubCategoriaRepository::new();
        let sub = sub_repo
            .create(&SubCategoria::new(sub_name.to_string(), cat.id))
            .unwrap();
        let prov_repo = SqliteProveedorRepository::new();
        let prov = prov_repo
            .create(&Proveedor::new(
                prov_code.to_string(),
                prov_code.to_string(),
                None,
                None,
                None,
                None,
            ))
            .unwrap();
        SqliteArticuloRepository::new()
            .create(&Articulo::new(
                format!("Art {}", prov_code),
                format!("COD-{}", prov_code),
                sub.id,
                prov.id,
            ))
            .unwrap()
    }

    #[test]
    fn find_filtered_with_preview_no_filter_returns_all() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat A", "Sub A", "P1");
        let art2 = create_articulo_with_names("Cat B", "Sub B", "P2");
        let repo = SqliteStockRepository::new();

        repo.create(&Stock::new(art1.id, 10.0, 100.0, 25.0))
            .unwrap();
        repo.create(&Stock::new(art2.id, 5.0, 200.0, 30.0)).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat_a = cats.iter().find(|c| c.categoria == "Cat A").unwrap();
        let cat_b = cats.iter().find(|c| c.categoria == "Cat B").unwrap();

        let preview_a = repo
            .find_filtered_with_preview(20.0, Some(cat_a.id), None, None)
            .unwrap();
        assert_eq!(preview_a.len(), 1);
        assert!((preview_a[0].costo_nuevo - 120.0).abs() < 0.01);

        let preview_b = repo
            .find_filtered_with_preview(20.0, Some(cat_b.id), None, None)
            .unwrap();
        assert_eq!(preview_b.len(), 1);
        assert!((preview_b[0].costo_nuevo - 240.0).abs() < 0.01);
    }

    #[test]
    fn find_filtered_with_preview_matches_category_filter() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat Cable", "Sub Cable", "P1");
        let art2 = create_articulo_with_names("Cat Otro", "Sub Otro", "P2");
        let repo = SqliteStockRepository::new();

        repo.create(&Stock::new(art1.id, 10.0, 1000.0, 20.0))
            .unwrap();
        repo.create(&Stock::new(art2.id, 5.0, 500.0, 30.0)).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat_cable = cats.iter().find(|c| c.categoria == "Cat Cable").unwrap();

        let previews = repo
            .find_filtered_with_preview(10.0, Some(cat_cable.id), None, None)
            .unwrap();
        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].articulo, "Art P1");
        assert!((previews[0].costo_nuevo - 1100.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_increases_all() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat X", "Sub X", "PX1");
        let art2 = create_articulo_with_names("Cat Y", "Sub Y", "PX2");
        let repo = SqliteStockRepository::new();

        repo.create(&Stock::new(art1.id, 10.0, 1000.0, 20.0))
            .unwrap();
        repo.create(&Stock::new(art2.id, 5.0, 2000.0, 30.0))
            .unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat_x = cats.iter().find(|c| c.categoria == "Cat X").unwrap();
        let cat_y = cats.iter().find(|c| c.categoria == "Cat Y").unwrap();

        let affected_x = repo
            .apply_costo_percentage(20.0, Some(cat_x.id), None, None)
            .unwrap();
        assert_eq!(affected_x, 1);

        let affected_y = repo
            .apply_costo_percentage(20.0, Some(cat_y.id), None, None)
            .unwrap();
        assert_eq!(affected_y, 1);

        let s1 = repo.find_by_articulo(art1.id).unwrap().unwrap();
        let s2 = repo.find_by_articulo(art2.id).unwrap().unwrap();
        assert!((s1.costo - 1200.0).abs() < 0.01);
        assert!((s2.costo - 2400.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_with_category_filter() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat Filter", "Sub Filter", "PF1");
        let art2 = create_articulo_with_names("Cat Other", "Sub Other", "PF2");
        let repo = SqliteStockRepository::new();

        repo.create(&Stock::new(art1.id, 10.0, 1000.0, 20.0))
            .unwrap();
        repo.create(&Stock::new(art2.id, 5.0, 2000.0, 30.0))
            .unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat_filter = cats.iter().find(|c| c.categoria == "Cat Filter").unwrap();

        let affected = repo
            .apply_costo_percentage(10.0, Some(cat_filter.id), None, None)
            .unwrap();
        assert_eq!(affected, 1);

        let s1 = repo.find_by_articulo(art1.id).unwrap().unwrap();
        let s2 = repo.find_by_articulo(art2.id).unwrap().unwrap();
        assert!((s1.costo - 1100.0).abs() < 0.01);
        assert!((s2.costo - 2000.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_no_filter_returns_zero_when_no_stock() {
        let _guard = fresh_db();
        let _art = create_articulo_with_names("Cat Empty", "Sub Empty", "EMPTY1");
        let repo = SqliteStockRepository::new();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat Empty").unwrap();

        let affected = repo
            .apply_costo_percentage(20.0, Some(cat.id), None, None)
            .unwrap();
        assert_eq!(affected, 0);
    }
}

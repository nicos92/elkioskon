use chrono::Utc;
use rusqlite::params;

use crate::domain::entities::{CostUpdateEstado, CostUpdateItem, CostUpdateOperation};
use crate::domain::repositories::{
    CostUpdateApplyResult, CostUpdateRepository, CostUpdateUndoResult,
};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteCostUpdateRepository;

impl Default for SqliteCostUpdateRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteCostUpdateRepository {
    pub fn new() -> Self {
        Self
    }
}

impl CostUpdateRepository for SqliteCostUpdateRepository {
    fn apply_with_history(
        &self,
        user_id: i64,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<CostUpdateApplyResult, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result: Result<CostUpdateApplyResult, AppError> = (|| {
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.costo
                     FROM stock s
                     INNER JOIN articulos a ON s.id_articulo = a.id
                     INNER JOIN sub_categorias sc ON a.id_sub_categoria = sc.id
                     INNER JOIN categorias c ON sc.id_categoria = c.id
                     INNER JOIN proveedores p ON a.id_proveedor = p.id
                     WHERE (?1 IS NULL OR c.id = ?1)
                       AND (?2 IS NULL OR sc.id = ?2)
                       AND (?3 IS NULL OR p.id = ?3)",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            let mut snapshots: Vec<(i64, f64)> = Vec::new();
            let mut rows = stmt
                .query(params![id_categoria, id_sub_categoria, id_proveedor])
                .map_err(|e| AppError::Database(e.to_string()))?;

            while let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
                let id: i64 = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
                let costo: f64 = row.get(1).map_err(|e| AppError::Database(e.to_string()))?;
                snapshots.push((id, costo));
            }

            if snapshots.is_empty() {
                return Err(AppError::BulkUpdateNoMatches);
            }

            let now = Utc::now().to_rfc3339();

            let operation_id: i64 = {
                conn.execute(
                    "INSERT INTO cost_update_operations
                     (user_id, porcentaje, filtro_categoria, filtro_sub_categoria, filtro_proveedor, affected_count, estado, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, 'aplicada', ?6)",
                    params![user_id, porcentaje, id_categoria, id_sub_categoria, id_proveedor, now],
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
                conn.last_insert_rowid()
            };

            for (stock_id, costo_actual) in &snapshots {
                let costo_nuevo: f64 = conn
                    .query_row(
                        "SELECT ROUND(?1 * (1.0 + ?2 / 100.0), 2)",
                        params![costo_actual, porcentaje],
                        |row| row.get(0),
                    )
                    .map_err(|e| AppError::Database(e.to_string()))?;
                conn.execute(
                    "INSERT INTO cost_update_items (operation_id, id_stock, costo_anterior, costo_nuevo)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![operation_id, stock_id, costo_actual, costo_nuevo],
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
            }

            let affected = snapshots.len() as i64;

            let stock_ids: Vec<i64> = snapshots.iter().map(|(id, _)| *id).collect();
            let placeholders: Vec<String> = stock_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 1))
                .collect();
            let sql = format!(
                "UPDATE stock SET costo = ROUND(costo * (1.0 + ?{} / 100.0), 2),
                       updated_at = ?{}
                 WHERE id IN ({})",
                stock_ids.len() + 1,
                stock_ids.len() + 2,
                placeholders.join(", ")
            );
            let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = stock_ids
                .iter()
                .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
                .collect();
            params_vec.push(Box::new(porcentaje));
            params_vec.push(Box::new(now.clone()));

            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&sql, param_refs.as_slice())
                .map_err(|e| AppError::Database(e.to_string()))?;

            conn.execute(
                "UPDATE cost_update_operations SET affected_count = ?1 WHERE id = ?2",
                params![affected, operation_id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(CostUpdateApplyResult {
                operation_id,
                affected_count: affected,
            })
        })();

        match result {
            Ok(r) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(r)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn find_last_undoable(&self) -> Result<Option<CostUpdateOperation>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, porcentaje, filtro_categoria, filtro_sub_categoria,
                        filtro_proveedor, affected_count, estado, created_at, undone_at
                 FROM cost_update_operations
                 WHERE estado = 'aplicada'
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(row_to_operation(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_items_by_operation(&self, operation_id: i64) -> Result<Vec<CostUpdateItem>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, operation_id, id_stock, costo_anterior, costo_nuevo
                 FROM cost_update_items
                 WHERE operation_id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut items = Vec::new();
        let mut rows = stmt
            .query(params![operation_id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        while let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            items.push(CostUpdateItem {
                id: row.get(0).map_err(|e| AppError::Database(e.to_string()))?,
                operation_id: row.get(1).map_err(|e| AppError::Database(e.to_string()))?,
                id_stock: row.get(2).map_err(|e| AppError::Database(e.to_string()))?,
                costo_anterior: row.get(3).map_err(|e| AppError::Database(e.to_string()))?,
                costo_nuevo: row.get(4).map_err(|e| AppError::Database(e.to_string()))?,
            });
        }

        Ok(items)
    }

    fn undo_operation(&self, operation_id: i64) -> Result<CostUpdateUndoResult, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result: Result<CostUpdateUndoResult, AppError> = (|| {
            let estado: String = conn
                .query_row(
                    "SELECT estado FROM cost_update_operations WHERE id = ?1",
                    params![operation_id],
                    |row| row.get(0),
                )
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => AppError::CostUpdateNotFound,
                    other => AppError::Database(other.to_string()),
                })?;

            if estado != "aplicada" {
                return Err(AppError::CostUpdateAlreadyUndone);
            }

            let modified_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM cost_update_items item
                     INNER JOIN stock ON item.id_stock = stock.id
                     INNER JOIN cost_update_operations op ON op.id = item.operation_id
                     WHERE item.operation_id = ?1
                       AND stock.updated_at > op.created_at
                       AND ABS(stock.costo - item.costo_nuevo) > 0.001",
                    params![operation_id],
                    |row| row.get(0),
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            if modified_count > 0 {
                return Err(AppError::CostUpdateModifiedAfter(modified_count));
            }

            let now_restore = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE stock SET costo = (
                    SELECT item.costo_anterior
                    FROM cost_update_items item
                    WHERE item.id_stock = stock.id AND item.operation_id = ?1
                ),
                updated_at = ?2
                WHERE id IN (
                    SELECT item.id_stock
                    FROM cost_update_items item
                    WHERE item.operation_id = ?1
                )",
                params![operation_id, now_restore],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

            let restored_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM cost_update_items WHERE operation_id = ?1",
                    params![operation_id],
                    |row| row.get(0),
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE cost_update_operations SET estado = 'deshecha', undone_at = ?1 WHERE id = ?2",
                params![now, operation_id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(CostUpdateUndoResult { restored_count })
        })();

        match result {
            Ok(r) => {
                conn.execute("COMMIT", [])
                    .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(r)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    fn cleanup_old_operations(&self) -> Result<i64, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let _ = conn.execute("DELETE FROM cost_update_items", []);

        let deleted_ops: i64 = conn
            .execute("DELETE FROM cost_update_operations", [])
            .map(|n| n as i64)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(deleted_ops)
    }
}

fn row_to_operation(row: &rusqlite::Row) -> Result<CostUpdateOperation, AppError> {
    let estado_str: String = row.get(7).map_err(|e| AppError::Database(e.to_string()))?;
    let estado = CostUpdateEstado::parse(&estado_str)
        .ok_or_else(|| AppError::Database(format!("Invalid estado: {}", estado_str)))?;

    Ok(CostUpdateOperation {
        id: row.get(0).map_err(|e| AppError::Database(e.to_string()))?,
        user_id: row.get(1).map_err(|e| AppError::Database(e.to_string()))?,
        porcentaje: row.get(2).map_err(|e| AppError::Database(e.to_string()))?,
        filtro_categoria: row.get(3).map_err(|e| AppError::Database(e.to_string()))?,
        filtro_sub_categoria: row.get(4).map_err(|e| AppError::Database(e.to_string()))?,
        filtro_proveedor: row.get(5).map_err(|e| AppError::Database(e.to_string()))?,
        affected_count: row.get(6).map_err(|e| AppError::Database(e.to_string()))?,
        estado,
        created_at: row.get(8).map_err(|e| AppError::Database(e.to_string()))?,
        undone_at: row.get(9).map_err(|e| AppError::Database(e.to_string()))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Articulo, Categoria, Proveedor, Stock, SubCategoria};
    use crate::domain::repositories::{
        ArticuloRepository, CategoriaRepository, ProveedorRepository, StockRepository,
        SubCategoriaRepository,
    };
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use crate::infrastructure::repositories::{
        SqliteArticuloRepository, SqliteCategoriaRepository, SqliteProveedorRepository,
        SqliteStockRepository, SqliteSubCategoriaRepository,
    };
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
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

    fn get_cat_id(cat_name: &str) -> i64 {
        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        cats.iter().find(|c| c.categoria == cat_name).unwrap().id
    }

    fn get_admin_user_id() -> i64 {
        let conn = DB.lock().unwrap();
        conn.query_row("SELECT id FROM users WHERE username = 'admin'", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    #[test]
    fn apply_with_history_creates_operation_and_items() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat CU1", "Sub CU1", "CU1");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat CU1");

        let result = repo
            .apply_with_history(user_id, 20.0, Some(cat_id), None, None)
            .unwrap();

        assert_eq!(result.affected_count, 1);
        assert!(result.operation_id > 0);

        let operation = repo.find_last_undoable().unwrap().unwrap();
        assert_eq!(operation.id, result.operation_id);
        assert_eq!(operation.porcentaje, 20.0);
        assert_eq!(operation.affected_count, 1);
        assert_eq!(operation.estado, CostUpdateEstado::Aplicada);

        let items = repo.find_items_by_operation(result.operation_id).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id_stock, stock.id);
        assert!((items[0].costo_anterior - 1000.0).abs() < 0.01);
        assert!((items[0].costo_nuevo - 1200.0).abs() < 0.01);

        let updated = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((updated.costo - 1200.0).abs() < 0.01);
    }

    #[test]
    fn apply_with_history_with_category_filter() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat CU3", "Sub CU3", "CU3");
        let art2 = create_articulo_with_names("Cat Other", "Sub Other", "CU3o");
        let stock_repo = SqliteStockRepository::new();
        let s1 = stock_repo
            .create(&Stock::new(art1.id, 10.0, 1000.0, 20.0))
            .unwrap();
        let s2 = stock_repo
            .create(&Stock::new(art2.id, 5.0, 2000.0, 30.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat CU3");

        let result = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();

        assert_eq!(result.affected_count, 1);

        let updated1 = stock_repo.find_by_id(s1.id).unwrap().unwrap();
        let updated2 = stock_repo.find_by_id(s2.id).unwrap().unwrap();
        assert!((updated1.costo - 1100.0).abs() < 0.01);
        assert!((updated2.costo - 2000.0).abs() < 0.01);
    }

    #[test]
    fn apply_with_history_returns_no_matches_when_empty() {
        let _guard = fresh_db();
        let _art = create_articulo_with_names("Cat Empty", "Sub Empty", "EMPTY");
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat Empty");

        let err = repo
            .apply_with_history(user_id, 20.0, Some(cat_id), None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateNoMatches));
    }

    #[test]
    fn apply_with_history_negative_porcentaje() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat CU4", "Sub CU4", "CU4");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat CU4");

        let result = repo
            .apply_with_history(user_id, -50.0, Some(cat_id), None, None)
            .unwrap();

        assert_eq!(result.affected_count, 1);
        let updated = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((updated.costo - 500.0).abs() < 0.01);

        let items = repo.find_items_by_operation(result.operation_id).unwrap();
        assert!((items[0].costo_anterior - 1000.0).abs() < 0.01);
        assert!((items[0].costo_nuevo - 500.0).abs() < 0.01);
    }

    #[test]
    fn find_last_undoable_returns_none_when_empty() {
        let _guard = fresh_db();
        let repo = SqliteCostUpdateRepository::new();
        assert!(repo.find_last_undoable().unwrap().is_none());
    }

    #[test]
    fn find_last_undoable_returns_most_recent() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat LR1", "Sub LR1", "LR1");
        let art2 = create_articulo_with_names("Cat LR2", "Sub LR2", "LR2");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art1.id, 5.0, 100.0, 10.0))
            .unwrap();
        stock_repo
            .create(&Stock::new(art2.id, 5.0, 200.0, 10.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat1 = get_cat_id("Cat LR1");
        let cat2 = get_cat_id("Cat LR2");

        let r1 = repo
            .apply_with_history(user_id, 10.0, Some(cat1), None, None)
            .unwrap();
        let r2 = repo
            .apply_with_history(user_id, 20.0, Some(cat2), None, None)
            .unwrap();

        let last = repo.find_last_undoable().unwrap().unwrap();
        assert_eq!(last.id, r2.operation_id);
        assert_eq!(last.porcentaje, 20.0);

        repo.undo_operation(r2.operation_id).unwrap();
        let last = repo.find_last_undoable().unwrap().unwrap();
        assert_eq!(last.id, r1.operation_id);
    }

    #[test]
    fn find_last_undoable_returns_none_when_all_undone() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat NU", "Sub NU", "NU");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 5.0, 100.0, 10.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat NU");

        let r = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();
        repo.undo_operation(r.operation_id).unwrap();

        assert!(repo.find_last_undoable().unwrap().is_none());
    }

    #[test]
    fn undo_operation_restores_costo() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat UN1", "Sub UN1", "UN1");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat UN1");

        let result = repo
            .apply_with_history(user_id, 20.0, Some(cat_id), None, None)
            .unwrap();

        let updated = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((updated.costo - 1200.0).abs() < 0.01);

        let undo = repo.undo_operation(result.operation_id).unwrap();
        assert_eq!(undo.restored_count, 1);

        let restored = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((restored.costo - 1000.0).abs() < 0.01);
    }

    #[test]
    fn undo_operation_marks_as_undone() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat UN2", "Sub UN2", "UN2");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 5.0, 100.0, 10.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat UN2");

        let result = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();

        let op_before = repo.find_last_undoable().unwrap().unwrap();
        assert_eq!(op_before.estado, CostUpdateEstado::Aplicada);

        repo.undo_operation(result.operation_id).unwrap();

        let op_after = repo.find_last_undoable().unwrap();
        assert!(op_after.is_none());

        let conn = DB.lock().unwrap();
        let estado: String = conn
            .query_row(
                "SELECT estado FROM cost_update_operations WHERE id = ?1",
                params![result.operation_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(estado, "deshecha");
    }

    #[test]
    fn undo_operation_rejects_already_undone() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat UN3", "Sub UN3", "UN3");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 5.0, 100.0, 10.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat UN3");

        let result = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();
        repo.undo_operation(result.operation_id).unwrap();

        let err = repo.undo_operation(result.operation_id).unwrap_err();
        assert!(matches!(err, AppError::CostUpdateAlreadyUndone));
    }

    #[test]
    fn undo_operation_rejects_when_stock_modified_after() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat UN4", "Sub UN4", "UN4");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat UN4");

        let result = repo
            .apply_with_history(user_id, 20.0, Some(cat_id), None, None)
            .unwrap();

        let mut updated = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        updated.costo = 9999.0;
        stock_repo.update(&updated).unwrap();

        let err = repo.undo_operation(result.operation_id).unwrap_err();
        assert!(matches!(err, AppError::CostUpdateModifiedAfter(1)));
    }

    #[test]
    fn undo_operation_rejects_nonexistent() {
        let _guard = fresh_db();
        let repo = SqliteCostUpdateRepository::new();

        let err = repo.undo_operation(99999).unwrap_err();
        assert!(matches!(err, AppError::CostUpdateNotFound));
    }

    #[test]
    fn apply_then_undo_roundtrip() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat RT", "Sub RT", "RT");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat RT");

        let r1 = repo
            .apply_with_history(user_id, 20.0, Some(cat_id), None, None)
            .unwrap();
        let after_first = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((after_first.costo - 1200.0).abs() < 0.01);

        let r2 = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();
        let after_second = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((after_second.costo - 1320.0).abs() < 0.01);

        let undo2 = repo.undo_operation(r2.operation_id).unwrap();
        assert_eq!(undo2.restored_count, 1);
        let after_undo2 = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((after_undo2.costo - 1200.0).abs() < 0.01);

        let undo1 = repo.undo_operation(r1.operation_id).unwrap();
        assert_eq!(undo1.restored_count, 1);
        let after_undo1 = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((after_undo1.costo - 1000.0).abs() < 0.01);
    }

    #[test]
    fn apply_stores_filters_in_operation() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat FL", "Sub FL", "FL");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 5.0, 100.0, 10.0))
            .unwrap();
        let repo = SqliteCostUpdateRepository::new();
        let user_id = get_admin_user_id();
        let cat_id = get_cat_id("Cat FL");

        let result = repo
            .apply_with_history(user_id, 10.0, Some(cat_id), None, None)
            .unwrap();

        let op = repo.find_last_undoable().unwrap().unwrap();
        assert_eq!(op.filtro_categoria, Some(cat_id));
        assert!(op.filtro_sub_categoria.is_none());
        assert!(op.filtro_proveedor.is_none());
        assert_eq!(op.id, result.operation_id);
    }
}

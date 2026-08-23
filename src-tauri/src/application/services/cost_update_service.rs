use std::sync::Arc;

use crate::domain::entities::{CostUpdateOperation, StockPreview};
use crate::domain::repositories::{
    CostUpdateApplyResult, CostUpdateRepository, CostUpdateUndoResult, StockRepository,
};
use crate::infrastructure::error::AppError;

pub struct CostUpdateService {
    cost_update_repository: Arc<dyn CostUpdateRepository>,
    stock_repository: Arc<dyn StockRepository>,
}

impl CostUpdateService {
    pub fn new(
        cost_update_repository: Arc<dyn CostUpdateRepository>,
        stock_repository: Arc<dyn StockRepository>,
    ) -> Self {
        Self {
            cost_update_repository,
            stock_repository,
        }
    }

    pub fn apply_costo_percentage(
        &self,
        user_id: i64,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<CostUpdateApplyResult, AppError> {
        if !porcentaje.is_finite() || porcentaje == 0.0 || porcentaje < -100.0 {
            return Err(AppError::BulkUpdateInvalidPorcentaje);
        }
        self.cost_update_repository.apply_with_history(
            user_id,
            porcentaje,
            id_categoria,
            id_sub_categoria,
            id_proveedor,
        )
    }

    pub fn get_preview(
        &self,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<Vec<StockPreview>, AppError> {
        if !porcentaje.is_finite() || porcentaje == 0.0 || porcentaje < -100.0 {
            return Err(AppError::BulkUpdateInvalidPorcentaje);
        }
        self.stock_repository.find_filtered_with_preview(
            porcentaje,
            id_categoria,
            id_sub_categoria,
            id_proveedor,
        )
    }

    pub fn get_last_undoable(&self) -> Result<Option<CostUpdateOperation>, AppError> {
        self.cost_update_repository.find_last_undoable()
    }

    pub fn undo_operation(&self, operation_id: i64) -> Result<CostUpdateUndoResult, AppError> {
        self.cost_update_repository.undo_operation(operation_id)
    }

    pub fn cleanup_old_operations(&self) -> Result<i64, AppError> {
        self.cost_update_repository.cleanup_old_operations()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Articulo, Categoria, Proveedor, Stock, SubCategoria};
    use crate::domain::repositories::{
        ArticuloRepository, CategoriaRepository, ProveedorRepository, SubCategoriaRepository,
    };
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use crate::infrastructure::repositories::{
        SqliteArticuloRepository, SqliteCategoriaRepository, SqliteCostUpdateRepository,
        SqliteProveedorRepository, SqliteStockRepository, SqliteSubCategoriaRepository,
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

    fn create_service() -> CostUpdateService {
        CostUpdateService::new(
            Arc::new(SqliteCostUpdateRepository::new()),
            Arc::new(SqliteStockRepository::new()),
        )
    }

    fn get_admin_user_id() -> i64 {
        let conn = crate::infrastructure::database::DB.lock().unwrap();
        conn.query_row("SELECT id FROM users WHERE username = 'admin'", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    #[test]
    fn apply_rejects_zero_porcentaje() {
        let _guard = fresh_db();
        let service = create_service();
        let user_id = get_admin_user_id();
        let err = service
            .apply_costo_percentage(user_id, 0.0, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_rejects_nan() {
        let _guard = fresh_db();
        let service = create_service();
        let user_id = get_admin_user_id();
        let err = service
            .apply_costo_percentage(user_id, f64::NAN, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_rejects_below_minus_100() {
        let _guard = fresh_db();
        let service = create_service();
        let user_id = get_admin_user_id();
        let err = service
            .apply_costo_percentage(user_id, -101.0, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_creates_operation_with_correct_data() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat SV1", "Sub SV1", "SV1");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let service = create_service();
        let user_id = get_admin_user_id();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat SV1").unwrap();

        let result = service
            .apply_costo_percentage(user_id, 20.0, Some(cat.id), None, None)
            .unwrap();

        assert_eq!(result.affected_count, 1);

        let op = service.get_last_undoable().unwrap().unwrap();
        assert_eq!(op.porcentaje, 20.0);
        assert_eq!(op.affected_count, 1);
    }

    #[test]
    fn get_preview_returns_preview() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat SV2", "Sub SV2", "SV2");
        let stock_repo = SqliteStockRepository::new();
        stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let service = create_service();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat SV2").unwrap();

        let previews = service.get_preview(20.0, Some(cat.id), None, None).unwrap();
        assert_eq!(previews.len(), 1);
        assert!((previews[0].costo_actual - 1000.0).abs() < 0.01);
        assert!((previews[0].costo_nuevo - 1200.0).abs() < 0.01);
    }

    #[test]
    fn get_preview_rejects_invalid_porcentaje() {
        let _guard = fresh_db();
        let service = create_service();
        assert!(matches!(
            service.get_preview(0.0, None, None, None),
            Err(AppError::BulkUpdateInvalidPorcentaje)
        ));
    }

    #[test]
    fn undo_restores_and_is_reflected() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat SV3", "Sub SV3", "SV3");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let service = create_service();
        let user_id = get_admin_user_id();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat SV3").unwrap();

        let result = service
            .apply_costo_percentage(user_id, 20.0, Some(cat.id), None, None)
            .unwrap();

        let updated = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((updated.costo - 1200.0).abs() < 0.01);

        let undo = service.undo_operation(result.operation_id).unwrap();
        assert_eq!(undo.restored_count, 1);

        let restored = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        assert!((restored.costo - 1000.0).abs() < 0.01);
    }

    #[test]
    fn undo_rejects_modified_after() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat SV4", "Sub SV4", "SV4");
        let stock_repo = SqliteStockRepository::new();
        let stock = stock_repo
            .create(&Stock::new(art.id, 10.0, 1000.0, 25.0))
            .unwrap();
        let service = create_service();
        let user_id = get_admin_user_id();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat SV4").unwrap();

        let result = service
            .apply_costo_percentage(user_id, 20.0, Some(cat.id), None, None)
            .unwrap();

        let mut modified = stock_repo.find_by_id(stock.id).unwrap().unwrap();
        modified.costo = 9999.0;
        stock_repo.update(&modified).unwrap();

        let err = service.undo_operation(result.operation_id).unwrap_err();
        assert!(matches!(err, AppError::CostUpdateModifiedAfter(1)));
    }
}

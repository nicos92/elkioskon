use std::sync::Arc;

use crate::domain::entities::{Stock, StockPreview};
use crate::domain::repositories::StockRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteStockRepository;

pub struct StockService {
    repository: Arc<dyn StockRepository>,
}

impl Default for StockService {
    fn default() -> Self {
        Self::new()
    }
}

impl StockService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteStockRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn StockRepository>) -> Self {
        Self { repository }
    }

    pub fn create(
        &self,
        id_articulo: i64,
        cantidad: f64,
        costo: f64,
        ganancia: f64,
    ) -> Result<Stock, AppError> {
        let existing = self.repository.find_by_articulo(id_articulo)?;
        if existing.is_some() {
            return Err(AppError::StockExistsForArticulo);
        }

        let new_stock = Stock::new(id_articulo, cantidad, costo, ganancia);
        self.repository.create(&new_stock)
    }

    pub fn get_all(&self) -> Result<Vec<Stock>, AppError> {
        self.repository.find_all()
    }

    pub fn get_by_id(&self, id: i64) -> Result<Stock, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::StockNotFound)
    }

    pub fn get_by_articulo(&self, id_articulo: i64) -> Result<Option<Stock>, AppError> {
        self.repository.find_by_articulo(id_articulo)
    }

    pub fn update(
        &self,
        id: i64,
        cantidad: f64,
        costo: f64,
        ganancia: f64,
    ) -> Result<Stock, AppError> {
        let mut existing = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::StockNotFound)?;

        existing.cantidad = cantidad;
        existing.costo = costo;
        existing.ganancia = ganancia;

        self.repository.update(&existing)
    }

    pub fn delete(&self, id: i64) -> Result<(), AppError> {
        let existing = self
            .repository
            .find_by_id(id)?
            .ok_or(AppError::StockNotFound)?;

        let has_ventas = self.repository.has_ventas(existing.id_articulo)?;
        if has_ventas {
            return Err(AppError::StockHasVentas);
        }

        self.repository.delete(id)
    }

    pub fn get_precio_venta(&self, id: i64) -> Result<f64, AppError> {
        let stock = self.get_by_id(id)?;
        let precio_venta = stock.costo * (1.0 + stock.ganancia / 100.0);
        Ok(precio_venta)
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
        self.repository.find_filtered_with_preview(
            porcentaje,
            id_categoria,
            id_sub_categoria,
            id_proveedor,
        )
    }

    pub fn apply_costo_percentage(
        &self,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<i64, AppError> {
        if !porcentaje.is_finite() || porcentaje == 0.0 || porcentaje < -100.0 {
            return Err(AppError::BulkUpdateInvalidPorcentaje);
        }
        let affected = self.repository.apply_costo_percentage(
            porcentaje,
            id_categoria,
            id_sub_categoria,
            id_proveedor,
        )?;
        if affected == 0 {
            return Err(AppError::BulkUpdateNoMatches);
        }
        Ok(affected)
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
            .create(&Categoria::new("Cat STK SV".to_string()))
            .unwrap();
        let sub_repo = SqliteSubCategoriaRepository::new();
        let sub = sub_repo
            .create(&SubCategoria::new("Sub STK SV".to_string(), cat.id))
            .unwrap();
        let prov_repo = SqliteProveedorRepository::new();
        let prov = prov_repo
            .create(&Proveedor::new(
                "PROV STK SV".to_string(),
                "Prov Stk Sv".to_string(),
                None,
                None,
                None,
                None,
            ))
            .unwrap();
        SqliteArticuloRepository::new()
            .create(&Articulo::new(
                "Art STK SV".to_string(),
                "STK-SV-001".to_string(),
                sub.id,
                prov.id,
            ))
            .unwrap()
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
            rusqlite::params![user_id, now, now],
        )
        .unwrap();
        let venta_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO venta_detalle (id_venta, id_articulo, cantidad, costo_unitario, precio_unitario, subtotal) VALUES (?1, ?2, 1.0, 100.0, 150.0, 150.0)",
            rusqlite::params![venta_id, id_articulo],
        )
        .unwrap();
    }

    #[test]
    fn delete_rejects_stock_with_sales() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let service = StockService::new();
        let stock = service.create(articulo.id, 10.0, 100.0, 25.0).unwrap();

        insert_venta_for_articulo(articulo.id);

        let err = service.delete(stock.id).unwrap_err();
        assert!(matches!(err, AppError::StockHasVentas));
        assert!(service.get_by_id(stock.id).is_ok());
    }

    #[test]
    fn delete_allows_stock_without_sales() {
        let _guard = fresh_db();
        let articulo = create_articulo();
        let service = StockService::new();
        let stock = service.create(articulo.id, 10.0, 100.0, 25.0).unwrap();

        service.delete(stock.id).unwrap();
        assert!(matches!(
            service.get_by_id(stock.id),
            Err(AppError::StockNotFound)
        ));
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
    fn apply_costo_percentage_rejects_zero() {
        let _guard = fresh_db();
        let service = StockService::new();
        let err = service
            .apply_costo_percentage(0.0, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_costo_percentage_rejects_below_minus_100() {
        let _guard = fresh_db();
        let service = StockService::new();
        let err = service
            .apply_costo_percentage(-101.0, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_costo_percentage_rejects_nan() {
        let _guard = fresh_db();
        let service = StockService::new();
        let err = service
            .apply_costo_percentage(f64::NAN, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_costo_percentage_rejects_infinity() {
        let _guard = fresh_db();
        let service = StockService::new();
        let err = service
            .apply_costo_percentage(f64::INFINITY, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateInvalidPorcentaje));
    }

    #[test]
    fn apply_costo_percentage_increases_cost() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat Bulk", "Sub Bulk", "BULK1");
        let service = StockService::new();
        let stock = service.create(art.id, 10.0, 1000.0, 25.0).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat Bulk").unwrap();

        let count = service
            .apply_costo_percentage(20.0, Some(cat.id), None, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = service.get_by_id(stock.id).unwrap();
        assert!((updated.costo - 1200.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_no_filter_updates_all() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat All", "Sub All", "BALL1");
        let art2 = create_articulo_with_names("Cat All2", "Sub All2", "BALL2");
        let service = StockService::new();
        let s1 = service.create(art1.id, 5.0, 500.0, 10.0).unwrap();
        let s2 = service.create(art2.id, 3.0, 300.0, 20.0).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat1 = cats.iter().find(|c| c.categoria == "Cat All").unwrap();
        let cat2 = cats.iter().find(|c| c.categoria == "Cat All2").unwrap();

        let count1 = service
            .apply_costo_percentage(-10.0, Some(cat1.id), None, None)
            .unwrap();
        let count2 = service
            .apply_costo_percentage(-10.0, Some(cat2.id), None, None)
            .unwrap();
        assert_eq!(count1, 1);
        assert_eq!(count2, 1);

        let updated1 = service.get_by_id(s1.id).unwrap();
        let updated2 = service.get_by_id(s2.id).unwrap();
        assert!((updated1.costo - 450.0).abs() < 0.01);
        assert!((updated2.costo - 270.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_rejects_when_no_matches() {
        let _guard = fresh_db();
        let service = StockService::new();
        let err = service
            .apply_costo_percentage(20.0, Some(99999), None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::BulkUpdateNoMatches));
    }

    #[test]
    fn get_preview_rejects_invalid_porcentaje() {
        let _guard = fresh_db();
        let service = StockService::new();
        assert!(matches!(
            service.get_preview(0.0, None, None, None),
            Err(AppError::BulkUpdateInvalidPorcentaje)
        ));
    }

    #[test]
    fn get_preview_returns_preview() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat Prev", "Sub Prev", "PREV1");
        let service = StockService::new();
        service.create(art.id, 10.0, 1000.0, 25.0).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat Prev").unwrap();

        let previews = service.get_preview(20.0, Some(cat.id), None, None).unwrap();
        assert_eq!(previews.len(), 1);
        assert!((previews[0].costo_actual - 1000.0).abs() < 0.01);
        assert!((previews[0].costo_nuevo - 1200.0).abs() < 0.01);
    }

    #[test]
    fn apply_costo_percentage_with_category_filter() {
        let _guard = fresh_db();
        let art1 = create_articulo_with_names("Cat Filt", "Sub Filt", "FILT1");
        let art2 = create_articulo_with_names("Cat NoFilt", "Sub NoFilt", "FILT2");
        let service = StockService::new();
        service.create(art1.id, 10.0, 1000.0, 20.0).unwrap();
        service.create(art2.id, 5.0, 2000.0, 30.0).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat Filt").unwrap();

        let count = service
            .apply_costo_percentage(10.0, Some(cat.id), None, None)
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn apply_costo_percentage_negative_reduces_cost() {
        let _guard = fresh_db();
        let art = create_articulo_with_names("Cat Neg", "Sub Neg", "NEG1");
        let service = StockService::new();
        let stock = service.create(art.id, 10.0, 1000.0, 25.0).unwrap();

        let cat_repo = SqliteCategoriaRepository::new();
        let cats = cat_repo.find_all().unwrap();
        let cat = cats.iter().find(|c| c.categoria == "Cat Neg").unwrap();

        let count = service
            .apply_costo_percentage(-50.0, Some(cat.id), None, None)
            .unwrap();
        assert_eq!(count, 1);

        let updated = service.get_by_id(stock.id).unwrap();
        assert!((updated.costo - 500.0).abs() < 0.01);
    }
}

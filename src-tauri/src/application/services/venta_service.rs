use std::sync::Arc;

use crate::domain::entities::{Venta, VentaDetalle, VentaWithDetalle};
use crate::domain::repositories::{ClienteRepository, Page, VentaRepository};
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::{SqliteClienteRepository, SqliteVentaRepository};

pub struct VentaService {
    repository: Arc<dyn VentaRepository>,
    cliente_repository: Arc<dyn ClienteRepository>,
}

impl Default for VentaService {
    fn default() -> Self {
        Self::new()
    }
}

impl VentaService {
    pub fn new() -> Self {
        Self::with_repositories(
            Arc::new(SqliteVentaRepository::new()),
            Arc::new(SqliteClienteRepository::new()),
        )
    }

    pub fn with_repository(repository: Arc<dyn VentaRepository>) -> Self {
        Self::with_repositories(repository, Arc::new(SqliteClienteRepository::new()))
    }

    pub fn with_repositories(
        repository: Arc<dyn VentaRepository>,
        cliente_repository: Arc<dyn ClienteRepository>,
    ) -> Self {
        Self {
            repository,
            cliente_repository,
        }
    }

    fn resolve_cliente_id(&self, cliente_id: Option<i64>) -> Result<i64, AppError> {
        match cliente_id {
            Some(id) => {
                self.cliente_repository
                    .find_by_id(id)?
                    .ok_or(AppError::ClienteNotFound)?;
                Ok(id)
            }
            None => self
                .cliente_repository
                .find_default()?
                .map(|c| c.id)
                .ok_or(AppError::ClienteDefectoNotFound),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        user_id: i64,
        detalles: Vec<VentaDetalle>,
        descuento: f64,
        observacion: Option<String>,
        id_tipo_venta: Option<i64>,
        cliente_id: Option<i64>,
        allow_negative_stock: bool,
    ) -> Result<VentaWithDetalle, AppError> {
        if !(0.0..=100.0).contains(&descuento) {
            return Err(AppError::DescuentoInvalido);
        }

        let cliente_id = self.resolve_cliente_id(cliente_id)?;

        let mut venta = Venta::new(
            user_id,
            chrono::Utc::now().to_rfc3339(),
            descuento,
            observacion,
        );
        venta.id_tipo_venta = id_tipo_venta;
        venta.cliente_id = cliente_id;
        self.repository
            .create(&venta, &detalles, allow_negative_stock)
    }

    pub fn get_all(&self) -> Result<Vec<VentaWithDetalle>, AppError> {
        self.repository.find_all()
    }

    pub fn get_page(&self, limit: i64, offset: i64) -> Result<Page<VentaWithDetalle>, AppError> {
        self.repository.find_page(limit, offset)
    }

    pub fn get_by_id(&self, id: i64) -> Result<VentaWithDetalle, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::VentaNotFound)
    }

    pub fn get_ventas_por_cliente(
        &self,
        cliente_id: i64,
    ) -> Result<Vec<VentaWithDetalle>, AppError> {
        self.cliente_repository
            .find_by_id(cliente_id)?
            .ok_or(AppError::ClienteNotFound)?;
        self.repository.find_by_cliente(cliente_id)
    }

    pub fn anular(&self, id: i64) -> Result<(), AppError> {
        self.repository.anular(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Cliente;
    use crate::domain::repositories::cliente_repository::MockClienteRepository;
    use crate::domain::repositories::venta_repository::MockVentaRepository;
    use mockall::predicate::*;

    fn cliente_with_id(id: i64, nombre: &str, apellido: &str) -> Cliente {
        let mut c = Cliente::new(
            Some(nombre.to_string()),
            Some(apellido.to_string()),
            None,
            None,
            None,
        );
        c.id = id;
        c
    }

    fn venta_result(id: i64, cliente_id: i64, descuento: f64) -> VentaWithDetalle {
        VentaWithDetalle {
            id,
            user_id: 1,
            username: "admin".to_string(),
            fecha: "2026-01-01T00:00:00Z".to_string(),
            subtotal: 100.0,
            descuento,
            total: 100.0,
            anulada: false,
            observacion: None,
            tipo_venta: None,
            cliente_id,
            cliente_nombre: Some("Consumidor".to_string()),
            cliente_apellido: Some("Final".to_string()),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            items: vec![],
        }
    }

    #[test]
    fn create_without_cliente_resolves_default() {
        let mut venta_repo = MockVentaRepository::new();
        venta_repo
            .expect_create()
            .returning(|v, _d, _n| Ok(venta_result(1, v.cliente_id, v.descuento)));

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_default()
            .return_once(|| Ok(Some(cliente_with_id(7, "Consumidor", "Final"))));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let result = service
            .create(1, vec![], 0.0, None, None, None, true)
            .unwrap();
        assert_eq!(result.cliente_id, 7);
    }

    #[test]
    fn create_with_explicit_cliente_id_uses_it() {
        let mut venta_repo = MockVentaRepository::new();
        venta_repo
            .expect_create()
            .returning(|v, _d, _n| Ok(venta_result(1, v.cliente_id, v.descuento)));

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(5))
            .return_once(|_| Ok(Some(cliente_with_id(5, "Juan", "Pérez"))));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let result = service
            .create(1, vec![], 10.0, None, None, Some(5), true)
            .unwrap();
        assert_eq!(result.cliente_id, 5);
        assert_eq!(result.descuento, 10.0);
    }

    #[test]
    fn create_with_nonexistent_cliente_rejects() {
        let venta_repo = MockVentaRepository::new();

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(999))
            .return_once(|_| Ok(None));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let err = service
            .create(1, vec![], 0.0, None, None, Some(999), true)
            .unwrap_err();
        assert!(matches!(err, AppError::ClienteNotFound));
    }

    #[test]
    fn create_rejects_when_no_default_client_exists() {
        let venta_repo = MockVentaRepository::new();

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo.expect_find_default().return_once(|| Ok(None));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let err = service
            .create(1, vec![], 0.0, None, None, None, true)
            .unwrap_err();
        assert!(matches!(err, AppError::ClienteDefectoNotFound));
    }

    #[test]
    fn get_ventas_por_cliente_returns_sales() {
        let mut venta_repo = MockVentaRepository::new();
        venta_repo
            .expect_find_by_cliente()
            .with(eq(5))
            .return_once(|_| Ok(vec![venta_result(2, 5, 0.0)]));

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(5))
            .return_once(|_| Ok(Some(cliente_with_id(5, "Juan", "Pérez"))));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let ventas = service.get_ventas_por_cliente(5).unwrap();
        assert_eq!(ventas.len(), 1);
        assert_eq!(ventas[0].cliente_id, 5);
    }

    #[test]
    fn get_ventas_por_cliente_rejects_missing_cliente() {
        let venta_repo = MockVentaRepository::new();

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(42))
            .return_once(|_| Ok(None));

        let service = VentaService::with_repositories(Arc::new(venta_repo), Arc::new(cliente_repo));
        let err = service.get_ventas_por_cliente(42).unwrap_err();
        assert!(matches!(err, AppError::ClienteNotFound));
    }
}

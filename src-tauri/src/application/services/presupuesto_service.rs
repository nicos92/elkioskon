use std::sync::Arc;

use crate::domain::entities::{
    Presupuesto, PresupuestoDetalle, PresupuestoEstado, PresupuestoWithDetalle,
};
use crate::domain::repositories::{
    ClienteRepository, Page, PresupuestoFilter, PresupuestoRepository,
};
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::{SqliteClienteRepository, SqlitePresupuestoRepository};

pub struct PresupuestoService {
    repository: Arc<dyn PresupuestoRepository>,
    cliente_repository: Arc<dyn ClienteRepository>,
}

impl Default for PresupuestoService {
    fn default() -> Self {
        Self::new()
    }
}

impl PresupuestoService {
    pub fn new() -> Self {
        Self::with_repositories(
            Arc::new(SqlitePresupuestoRepository::new()),
            Arc::new(SqliteClienteRepository::new()),
        )
    }

    pub fn with_repository(repository: Arc<dyn PresupuestoRepository>) -> Self {
        Self::with_repositories(repository, Arc::new(SqliteClienteRepository::new()))
    }

    pub fn with_repositories(
        repository: Arc<dyn PresupuestoRepository>,
        cliente_repository: Arc<dyn ClienteRepository>,
    ) -> Self {
        Self {
            repository,
            cliente_repository,
        }
    }

    fn resolve_cliente_id(&self, cliente_id: Option<i64>) -> Result<Option<i64>, AppError> {
        match cliente_id {
            Some(id) => {
                self.cliente_repository
                    .find_by_id(id)?
                    .ok_or(AppError::ClienteNotFound)?;
                Ok(Some(id))
            }
            None => Ok(None),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        user_id: i64,
        detalles: Vec<PresupuestoDetalle>,
        descuento: f64,
        observacion: Option<String>,
        fecha_vencimiento: Option<String>,
        cliente_id: Option<i64>,
    ) -> Result<PresupuestoWithDetalle, AppError> {
        if !(0.0..=100.0).contains(&descuento) {
            return Err(AppError::DescuentoInvalido);
        }

        let cliente_id = self.resolve_cliente_id(cliente_id)?;

        let presupuesto = Presupuesto::new(
            user_id,
            chrono::Utc::now().to_rfc3339(),
            descuento,
            observacion,
            fecha_vencimiento,
            cliente_id,
        );
        self.repository.create(&presupuesto, &detalles)
    }

    pub fn get_by_id(&self, id: i64) -> Result<PresupuestoWithDetalle, AppError> {
        self.repository
            .find_by_id(id)?
            .ok_or(AppError::PresupuestoNotFound)
    }

    pub fn get_page(
        &self,
        filter: &PresupuestoFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Page<PresupuestoWithDetalle>, AppError> {
        self.repository.find_page(filter, limit, offset)
    }

    pub fn cambiar_estado(&self, id: i64, estado: PresupuestoEstado) -> Result<(), AppError> {
        self.repository.update_estado(id, estado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Cliente;
    use crate::domain::repositories::cliente_repository::MockClienteRepository;
    use crate::domain::repositories::presupuesto_repository::MockPresupuestoRepository;
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

    fn presupuesto_result(
        id: i64,
        descuento: f64,
        cliente_id: Option<i64>,
    ) -> PresupuestoWithDetalle {
        PresupuestoWithDetalle {
            id,
            user_id: 1,
            username: "admin".to_string(),
            fecha: "2026-01-01T00:00:00Z".to_string(),
            subtotal: 100.0,
            descuento,
            total: 100.0,
            estado: "pendiente".to_string(),
            fecha_vencimiento: None,
            observacion: None,
            cliente_id,
            cliente_nombre: None,
            cliente_apellido: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            items: vec![],
        }
    }

    #[test]
    fn create_with_optional_cliente_persists_null() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo.expect_create().returning(|p, _d| {
            assert_eq!(p.cliente_id, None);
            Ok(presupuesto_result(1, p.descuento, None))
        });

        let cliente_repo = MockClienteRepository::new();

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let result = service.create(1, vec![], 0.0, None, None, None).unwrap();
        assert_eq!(result.cliente_id, None);
    }

    #[test]
    fn create_with_explicit_cliente_validates_it() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo.expect_create().returning(|p, _d| {
            assert_eq!(p.cliente_id, Some(5));
            Ok(presupuesto_result(1, p.descuento, p.cliente_id))
        });

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(5))
            .return_once(|_| Ok(Some(cliente_with_id(5, "Juan", "Pérez"))));

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let result = service
            .create(1, vec![], 10.0, None, None, Some(5))
            .unwrap();
        assert_eq!(result.cliente_id, Some(5));
        assert_eq!(result.descuento, 10.0);
    }

    #[test]
    fn create_with_nonexistent_cliente_rejects() {
        let presupuesto_repo = MockPresupuestoRepository::new();

        let mut cliente_repo = MockClienteRepository::new();
        cliente_repo
            .expect_find_by_id()
            .with(eq(999))
            .return_once(|_| Ok(None));

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let err = service
            .create(1, vec![], 0.0, None, None, Some(999))
            .unwrap_err();
        assert!(matches!(err, AppError::ClienteNotFound));
    }

    #[test]
    fn create_rejects_invalid_descuento() {
        let presupuesto_repo = MockPresupuestoRepository::new();
        let cliente_repo = MockClienteRepository::new();

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let err = service
            .create(1, vec![], 150.0, None, None, None)
            .unwrap_err();
        assert!(matches!(err, AppError::DescuentoInvalido));
    }

    #[test]
    fn get_by_id_returns_presupuesto() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo
            .expect_find_by_id()
            .with(eq(3))
            .return_once(|_| Ok(Some(presupuesto_result(3, 0.0, None))));

        let cliente_repo = MockClienteRepository::new();

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let found = service.get_by_id(3).unwrap();
        assert_eq!(found.id, 3);
    }

    #[test]
    fn get_by_id_rejects_missing_presupuesto() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo
            .expect_find_by_id()
            .with(eq(99))
            .return_once(|_| Ok(None));

        let cliente_repo = MockClienteRepository::new();

        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let err = service.get_by_id(99).unwrap_err();
        assert!(matches!(err, AppError::PresupuestoNotFound));
    }

    #[test]
    fn get_page_delegates_filter_and_pagination() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo
            .expect_find_page()
            .with(eq(PresupuestoFilter::default()), eq(25), eq(0))
            .return_once(|_, _, _| {
                Ok(crate::domain::repositories::Page {
                    items: vec![presupuesto_result(1, 0.0, None)],
                    total: 1,
                    limit: 25,
                    offset: 0,
                })
            });

        let cliente_repo = MockClienteRepository::new();
        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let page = service
            .get_page(&PresupuestoFilter::default(), 25, 0)
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items.len(), 1);
    }

    #[test]
    fn cambiar_estado_delegates_to_repository() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo
            .expect_update_estado()
            .with(eq(7), eq(PresupuestoEstado::Anulado))
            .return_once(|_, _| Ok(()));

        let cliente_repo = MockClienteRepository::new();
        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        service
            .cambiar_estado(7, PresupuestoEstado::Anulado)
            .unwrap();
    }

    #[test]
    fn cambiar_estado_propagates_repository_error() {
        let mut presupuesto_repo = MockPresupuestoRepository::new();
        presupuesto_repo
            .expect_update_estado()
            .with(eq(9), eq(PresupuestoEstado::Anulado))
            .return_once(|_, _| Err(AppError::PresupuestoEstadoInvalido));

        let cliente_repo = MockClienteRepository::new();
        let service = PresupuestoService::with_repositories(
            Arc::new(presupuesto_repo),
            Arc::new(cliente_repo),
        );
        let err = service
            .cambiar_estado(9, PresupuestoEstado::Anulado)
            .unwrap_err();
        assert!(matches!(err, AppError::PresupuestoEstadoInvalido));
    }
}

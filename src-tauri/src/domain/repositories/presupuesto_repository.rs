use crate::domain::entities::{
    Presupuesto, PresupuestoDetalle, PresupuestoEstado, PresupuestoWithDetalle,
};
use crate::domain::repositories::Page;
use crate::infrastructure::error::AppError;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PresupuestoFilter {
    pub estado: Option<PresupuestoEstado>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    pub query: Option<String>,
}

#[cfg_attr(test, mockall::automock)]
pub trait PresupuestoRepository: Send + Sync {
    fn create(
        &self,
        presupuesto: &Presupuesto,
        detalles: &[PresupuestoDetalle],
    ) -> Result<PresupuestoWithDetalle, AppError>;
    fn find_by_id(&self, id: i64) -> Result<Option<PresupuestoWithDetalle>, AppError>;
    fn find_page(
        &self,
        filter: &PresupuestoFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Page<PresupuestoWithDetalle>, AppError>;
    fn update_estado(&self, id: i64, estado: PresupuestoEstado) -> Result<(), AppError>;
}

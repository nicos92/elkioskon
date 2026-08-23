use crate::domain::entities::{CostUpdateItem, CostUpdateOperation};
use crate::infrastructure::error::AppError;

#[derive(Debug, Clone)]
pub struct CostUpdateApplyResult {
    pub operation_id: i64,
    pub affected_count: i64,
}

#[derive(Debug, Clone)]
pub struct CostUpdateUndoResult {
    pub restored_count: i64,
}

#[cfg_attr(test, mockall::automock)]
pub trait CostUpdateRepository: Send + Sync {
    fn apply_with_history(
        &self,
        user_id: i64,
        porcentaje: f64,
        id_categoria: Option<i64>,
        id_sub_categoria: Option<i64>,
        id_proveedor: Option<i64>,
    ) -> Result<CostUpdateApplyResult, AppError>;

    fn find_last_undoable(&self) -> Result<Option<CostUpdateOperation>, AppError>;

    fn find_items_by_operation(&self, operation_id: i64) -> Result<Vec<CostUpdateItem>, AppError>;

    fn undo_operation(&self, operation_id: i64) -> Result<CostUpdateUndoResult, AppError>;

    fn cleanup_old_operations(&self) -> Result<i64, AppError>;
}

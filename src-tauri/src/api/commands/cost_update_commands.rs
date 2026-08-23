use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, CostUpdateService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, StockPreview};
use crate::infrastructure::error::AppError;

pub struct CostUpdateAppState {
    pub cost_update_service: Mutex<CostUpdateService>,
}

impl Default for CostUpdateAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl CostUpdateAppState {
    pub fn new() -> Self {
        use crate::infrastructure::repositories::{
            SqliteCostUpdateRepository, SqliteStockRepository,
        };
        use std::sync::Arc;

        Self {
            cost_update_service: Mutex::new(CostUpdateService::new(
                Arc::new(SqliteCostUpdateRepository::new()),
                Arc::new(SqliteStockRepository::new()),
            )),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ApplyCostoPercentageRequest {
    pub porcentaje: f64,
    pub id_categoria: Option<i64>,
    pub id_sub_categoria: Option<i64>,
    pub id_proveedor: Option<i64>,
}

#[derive(serde::Serialize)]
pub struct ApplyCostoPercentageResult {
    pub updated_count: i64,
}

#[derive(serde::Serialize)]
pub struct CostUpdateOperationResponse {
    pub id: i64,
    pub porcentaje: f64,
    pub affected_count: i64,
    pub categoria_nombre: Option<String>,
    pub sub_categoria_nombre: Option<String>,
    pub proveedor_nombre: Option<String>,
    pub created_at: String,
}

#[derive(serde::Serialize)]
pub struct UndoOperationResult {
    pub restored_count: i64,
}

#[tauri::command(async)]
pub fn get_stock_preview_costo(
    user_id: i64,
    porcentaje: f64,
    id_categoria: Option<i64>,
    id_sub_categoria: Option<i64>,
    id_proveedor: Option<i64>,
    state: State<CostUpdateAppState>,
) -> Result<Vec<StockPreview>, AppError> {
    let service = state
        .cost_update_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewStock)?;
    service.get_preview(porcentaje, id_categoria, id_sub_categoria, id_proveedor)
}

#[tauri::command(async)]
pub fn apply_costo_percentage_stock(
    user_id: i64,
    request: ApplyCostoPercentageRequest,
    state: State<CostUpdateAppState>,
) -> Result<ApplyCostoPercentageResult, AppError> {
    let service = state
        .cost_update_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateStock)?;
    let result = service.apply_costo_percentage(
        user_id,
        request.porcentaje,
        request.id_categoria,
        request.id_sub_categoria,
        request.id_proveedor,
    )?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Update,
        Some(format!(
            "Actualización masiva de costos: {}% ({} artículos)",
            request.porcentaje, result.affected_count
        )),
    )?;
    Ok(ApplyCostoPercentageResult {
        updated_count: result.affected_count,
    })
}

#[tauri::command(async)]
pub fn get_last_undoable_cost_update(
    user_id: i64,
    state: State<CostUpdateAppState>,
) -> Result<Option<CostUpdateOperationResponse>, AppError> {
    let service = state
        .cost_update_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateStock)?;
    let op = service.get_last_undoable()?;

    match op {
        Some(operation) => {
            let (cat_name, sub_name, prov_name) = resolve_filter_names(
                operation.filtro_categoria,
                operation.filtro_sub_categoria,
                operation.filtro_proveedor,
            )?;
            Ok(Some(CostUpdateOperationResponse {
                id: operation.id,
                porcentaje: operation.porcentaje,
                affected_count: operation.affected_count,
                categoria_nombre: cat_name,
                sub_categoria_nombre: sub_name,
                proveedor_nombre: prov_name,
                created_at: operation.created_at,
            }))
        }
        None => Ok(None),
    }
}

#[tauri::command(async)]
pub fn undo_cost_update(
    user_id: i64,
    operation_id: i64,
    state: State<CostUpdateAppState>,
) -> Result<UndoOperationResult, AppError> {
    let service = state
        .cost_update_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateStock)?;
    let result = service.undo_operation(operation_id)?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Update,
        Some(format!(
            "Deshacer actualización de costos (operación #{}): {} artículos restaurados",
            operation_id, result.restored_count
        )),
    )?;
    Ok(UndoOperationResult {
        restored_count: result.restored_count,
    })
}

#[derive(serde::Serialize)]
pub struct CleanupResult {
    pub deleted_count: i64,
}

#[tauri::command(async)]
pub fn cleanup_cost_update_operations(
    user_id: i64,
    state: State<CostUpdateAppState>,
) -> Result<CleanupResult, AppError> {
    let service = state
        .cost_update_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateStock)?;
    let count = service.cleanup_old_operations()?;
    Ok(CleanupResult {
        deleted_count: count,
    })
}

type FilterNames = (Option<String>, Option<String>, Option<String>);

fn resolve_filter_names(
    cat_id: Option<i64>,
    sub_id: Option<i64>,
    prov_id: Option<i64>,
) -> Result<FilterNames, AppError> {
    use crate::infrastructure::database::DB;
    use rusqlite::params;

    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let cat_name = match cat_id {
        Some(id) => {
            let name: Result<String, _> = conn.query_row(
                "SELECT categoria FROM categorias WHERE id = ?1",
                params![id],
                |row| row.get(0),
            );
            Some(name.map_err(|e| AppError::Database(e.to_string()))?)
        }
        None => None,
    };

    let sub_name = match sub_id {
        Some(id) => {
            let name: Result<String, _> = conn.query_row(
                "SELECT sub_categoria FROM sub_categorias WHERE id = ?1",
                params![id],
                |row| row.get(0),
            );
            Some(name.map_err(|e| AppError::Database(e.to_string()))?)
        }
        None => None,
    };

    let prov_name = match prov_id {
        Some(id) => {
            let name: Result<String, _> = conn.query_row(
                "SELECT proveedor FROM proveedores WHERE id = ?1",
                params![id],
                |row| row.get(0),
            );
            Some(name.map_err(|e| AppError::Database(e.to_string()))?)
        }
        None => None,
    };

    Ok((cat_name, sub_name, prov_name))
}

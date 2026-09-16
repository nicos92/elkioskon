use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, opt_str, AuditDetail, TipoVentaService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, TipoVenta};
use crate::infrastructure::error::AppError;

pub struct TipoVentaAppState {
    pub tipo_venta_service: Mutex<TipoVentaService>,
}

impl Default for TipoVentaAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl TipoVentaAppState {
    pub fn new() -> Self {
        Self {
            tipo_venta_service: Mutex::new(TipoVentaService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct TipoVentaRequest {
    pub nombre: String,
    pub hacia_donde: Option<String>,
}

#[tauri::command(async)]
pub fn get_all_tipos_venta(
    user_id: i64,
    state: State<TipoVentaAppState>,
) -> Result<Vec<TipoVenta>, AppError> {
    let service = state
        .tipo_venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewTiposVenta)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn create_tipo_venta(
    user_id: i64,
    request: TipoVentaRequest,
    state: State<TipoVentaAppState>,
) -> Result<TipoVenta, AppError> {
    let service = state
        .tipo_venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateTipoVenta)?;
    let result = service.create(request.nombre, request.hacia_donde)?;
    log_audit(
        user_id,
        AuditScreen::TiposVenta,
        AuditAction::Create,
        Some(format!(
            "Tipo de venta creado: {} (id {})",
            result.nombre, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_tipo_venta(
    user_id: i64,
    id: i64,
    request: TipoVentaRequest,
    state: State<TipoVentaAppState>,
) -> Result<TipoVenta, AppError> {
    let service = state
        .tipo_venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateTipoVenta)?;
    let antes = service.get_by_id(id)?;
    let result = service.update(id, request.nombre, request.hacia_donde)?;
    let detail = AuditDetail::new("tipo_venta", format!("{} (id {})", result.nombre, result.id))
        .cambio("nombre", &antes.nombre, &result.nombre)
        .cambio("hacia_donde", opt_str(&antes.hacia_donde), opt_str(&result.hacia_donde))
        .to_json();
    log_audit(
        user_id,
        AuditScreen::TiposVenta,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_tipo_venta(
    user_id: i64,
    id: i64,
    state: State<TipoVentaAppState>,
) -> Result<(), AppError> {
    let service = state
        .tipo_venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteTipoVenta)?;
    let antes = service.get_by_id(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::TiposVenta,
        AuditAction::Delete,
        Some(format!(
            "Tipo de venta eliminado: {} (id {})",
            antes.nombre, id
        )),
    )?;
    Ok(())
}

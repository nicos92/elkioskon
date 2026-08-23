use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, CierreService};
use crate::domain::entities::{AuditAction, AuditScreen, CierreWithTipos, PermissionCode};
use crate::domain::repositories::Page;
use crate::infrastructure::error::AppError;

pub struct CierreAppState {
    pub cierre_service: Mutex<CierreService>,
}

impl Default for CierreAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl CierreAppState {
    pub fn new() -> Self {
        Self {
            cierre_service: Mutex::new(CierreService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CrearCierreRequest {
    pub fecha: String,
}

#[derive(serde::Deserialize)]
pub struct GetCierresRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command(async)]
pub fn crear_cierre(
    user_id: i64,
    request: CrearCierreRequest,
    state: State<CierreAppState>,
) -> Result<CierreWithTipos, AppError> {
    let service = state
        .cierre_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateCierre)?;
    let cierre = service.crear_cierre(&request.fecha)?;
    log_audit(
        user_id,
        AuditScreen::Cierres,
        AuditAction::Create,
        Some(format!(
            "Cierre del día: {} (id {})",
            cierre.cierre.fecha, cierre.cierre.id
        )),
    )?;
    Ok(cierre)
}

#[tauri::command(async)]
pub fn reabrir_cierre(
    user_id: i64,
    request: CrearCierreRequest,
    state: State<CierreAppState>,
) -> Result<(), AppError> {
    let service = state
        .cierre_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ReopenCierre)?;
    service.reabrir_cierre(&request.fecha)?;
    log_audit(
        user_id,
        AuditScreen::Cierres,
        AuditAction::Delete,
        Some(format!("Cierre del día {} reabierto", request.fecha)),
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn is_dia_cerrado(user_id: i64, state: State<CierreAppState>) -> Result<bool, AppError> {
    let service = state
        .cierre_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateVenta)?;
    service.is_dia_cerrado()
}

#[tauri::command(async)]
pub fn get_all_cierres(
    user_id: i64,
    request: GetCierresRequest,
    state: State<CierreAppState>,
) -> Result<Page<CierreWithTipos>, AppError> {
    let service = state
        .cierre_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewCierres)?;
    let limit = request.limit.unwrap_or(10).max(1);
    let offset = request.offset.unwrap_or(0).max(0);
    service.get_page(limit, offset)
}

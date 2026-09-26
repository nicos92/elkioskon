use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, AuditDetail, RespaldoService};
use crate::domain::entities::{
    AuditAction, AuditScreen, PermissionCode, RespaldoInfo, RespaldoResult,
};
use crate::infrastructure::error::AppError;

pub struct RespaldoAppState {
    pub respaldo_service: Mutex<RespaldoService>,
}

impl Default for RespaldoAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl RespaldoAppState {
    pub fn new() -> Self {
        Self {
            respaldo_service: Mutex::new(RespaldoService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CrearRespaldoRequest {
    pub destino: String,
}

#[tauri::command(async)]
pub fn crear_respaldo(
    user_id: i64,
    request: CrearRespaldoRequest,
    state: State<RespaldoAppState>,
) -> Result<RespaldoResult, AppError> {
    let service = state
        .respaldo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GestionarRespaldos)?;

    let resultado = service.crear(&request.destino)?;
    let detail = AuditDetail::new(
        "respaldo",
        format!("Copia de seguridad: {}", resultado.ruta),
    )
    .cambio("ruta", "-", &resultado.ruta)
    .cambio("tamano_bytes", 0, resultado.tamano_bytes)
    .to_json();
    log_audit(
        user_id,
        AuditScreen::Configuracion,
        AuditAction::Create,
        Some(detail),
    )?;
    Ok(resultado)
}

#[tauri::command(async)]
pub fn get_respaldo_info(
    user_id: i64,
    state: State<RespaldoAppState>,
) -> Result<RespaldoInfo, AppError> {
    let service = state
        .respaldo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GestionarRespaldos)?;
    service.info()
}

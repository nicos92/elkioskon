use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, NocturnoConfigService};
use crate::domain::entities::{AuditAction, AuditScreen, NocturnoConfig, PermissionCode};
use crate::infrastructure::error::AppError;

pub struct NocturnoConfigAppState {
    pub nocturno_config_service: Mutex<NocturnoConfigService>,
}

impl Default for NocturnoConfigAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl NocturnoConfigAppState {
    pub fn new() -> Self {
        Self {
            nocturno_config_service: Mutex::new(NocturnoConfigService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SaveNocturnoConfigRequest {
    pub activo: bool,
    pub porcentaje: f64,
    pub hora_inicio: String,
    pub hora_fin: String,
}

#[tauri::command(async)]
pub fn get_nocturno_config(
    user_id: i64,
    state: State<NocturnoConfigAppState>,
) -> Result<NocturnoConfig, AppError> {
    let service = state
        .nocturno_config_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewVentas)?;
    service.get()
}

#[tauri::command(async)]
pub fn save_nocturno_config(
    user_id: i64,
    request: SaveNocturnoConfigRequest,
    state: State<NocturnoConfigAppState>,
) -> Result<(), AppError> {
    let service = state
        .nocturno_config_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ConfigurarRecargoNocturno)?;

    let config = NocturnoConfig {
        activo: request.activo,
        porcentaje: request.porcentaje,
        hora_inicio: request.hora_inicio,
        hora_fin: request.hora_fin,
    };
    service.save(&config)?;
    log_audit(
        user_id,
        AuditScreen::Configuracion,
        AuditAction::Update,
        Some(format!(
            "Recargo nocturno: activo={}, porcentaje={}%, {}–{}",
            config.activo, config.porcentaje, config.hora_inicio, config.hora_fin
        )),
    )?;
    Ok(())
}
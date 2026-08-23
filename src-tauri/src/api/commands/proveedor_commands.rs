use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, ProveedorService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, Proveedor};
use crate::infrastructure::error::AppError;

pub struct ProveedorAppState {
    pub proveedor_service: Mutex<ProveedorService>,
}

impl Default for ProveedorAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProveedorAppState {
    pub fn new() -> Self {
        Self {
            proveedor_service: Mutex::new(ProveedorService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateProveedorRequest {
    pub proveedor: String,
    pub nombre: String,
    pub cuit: Option<String>,
    pub tel: Option<String>,
    pub email: Option<String>,
    pub observacion: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateProveedorRequest {
    pub id: i64,
    pub proveedor: String,
    pub nombre: String,
    pub cuit: Option<String>,
    pub tel: Option<String>,
    pub email: Option<String>,
    pub observacion: Option<String>,
}

#[tauri::command(async)]
pub fn get_all_proveedores(
    user_id: i64,
    state: State<ProveedorAppState>,
) -> Result<Vec<Proveedor>, AppError> {
    let service = state
        .proveedor_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewProveedores)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn get_proveedor_by_id(
    user_id: i64,
    id: i64,
    state: State<ProveedorAppState>,
) -> Result<Proveedor, AppError> {
    let service = state
        .proveedor_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewProveedores)?;
    service.get_by_id(id)
}

#[tauri::command(async)]
pub fn create_proveedor(
    user_id: i64,
    request: CreateProveedorRequest,
    state: State<ProveedorAppState>,
) -> Result<Proveedor, AppError> {
    let service = state
        .proveedor_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateProveedor)?;
    let result = service.create(
        request.proveedor,
        request.nombre,
        request.cuit,
        request.tel,
        request.email,
        request.observacion,
    )?;
    log_audit(
        user_id,
        AuditScreen::Proveedores,
        AuditAction::Create,
        Some(format!(
            "Proveedor: {} ({}) (id {})",
            result.proveedor, result.nombre, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_proveedor(
    user_id: i64,
    request: UpdateProveedorRequest,
    state: State<ProveedorAppState>,
) -> Result<Proveedor, AppError> {
    let service = state
        .proveedor_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateProveedor)?;
    let proveedor = Proveedor {
        id: request.id,
        proveedor: request.proveedor,
        nombre: request.nombre,
        cuit: request.cuit,
        tel: request.tel,
        email: request.email,
        observacion: request.observacion,
    };
    let result = service.update(&proveedor)?;
    log_audit(
        user_id,
        AuditScreen::Proveedores,
        AuditAction::Update,
        Some(format!(
            "Proveedor: {} ({}) (id {})",
            result.proveedor, result.nombre, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_proveedor(
    user_id: i64,
    id: i64,
    state: State<ProveedorAppState>,
) -> Result<(), AppError> {
    let service = state
        .proveedor_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteProveedor)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Proveedores,
        AuditAction::Delete,
        Some(format!("Proveedor (id {})", id)),
    )?;
    Ok(())
}

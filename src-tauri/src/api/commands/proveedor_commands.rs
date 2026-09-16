use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, opt_str, AuditDetail, ProveedorService};
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
        Some(format!("Proveedor creado: {}", proveedor_label(&result))),
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
    let antes = service.get_by_id(request.id)?;
    let result = service.update(&proveedor)?;
    let detail = AuditDetail::new("proveedor", proveedor_label(&result))
        .cambio("proveedor", &antes.proveedor, &result.proveedor)
        .cambio("nombre", &antes.nombre, &result.nombre)
        .cambio("cuit", opt_str(&antes.cuit), opt_str(&result.cuit))
        .cambio("tel", opt_str(&antes.tel), opt_str(&result.tel))
        .cambio("email", opt_str(&antes.email), opt_str(&result.email))
        .cambio("observacion", opt_str(&antes.observacion), opt_str(&result.observacion))
        .to_json();
    log_audit(
        user_id,
        AuditScreen::Proveedores,
        AuditAction::Update,
        Some(detail),
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
    let antes = service.get_by_id(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Proveedores,
        AuditAction::Delete,
        Some(format!(
            "Proveedor eliminado: {}",
            proveedor_label(&antes)
        )),
    )?;
    Ok(())
}

fn proveedor_label(p: &Proveedor) -> String {
    let cuit = p.cuit.as_deref().unwrap_or("-");
    format!("{} ({}) cuit={} (id {})", p.proveedor, p.nombre, cuit, p.id)
}

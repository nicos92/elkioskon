use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, opt_str, AuditDetail, ClienteService};
use crate::domain::entities::{AuditAction, AuditScreen, Cliente, PermissionCode};
use crate::infrastructure::error::AppError;

pub struct ClienteAppState {
    pub cliente_service: Mutex<ClienteService>,
}

impl Default for ClienteAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClienteAppState {
    pub fn new() -> Self {
        Self {
            cliente_service: Mutex::new(ClienteService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateClienteRequest {
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub direccion: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateClienteRequest {
    pub id: i64,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub direccion: Option<String>,
}

#[tauri::command(async)]
pub fn get_all_clientes(
    user_id: i64,
    state: State<ClienteAppState>,
) -> Result<Vec<Cliente>, AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewClientes)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn get_cliente_by_id(
    user_id: i64,
    id: i64,
    state: State<ClienteAppState>,
) -> Result<Cliente, AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewClientes)?;
    service.get_by_id(id)
}

#[tauri::command(async)]
pub fn get_cliente_defecto(
    user_id: i64,
    state: State<ClienteAppState>,
) -> Result<Cliente, AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewClientes)?;
    service.get_default()
}

#[tauri::command(async)]
pub fn crear_cliente(
    user_id: i64,
    request: CreateClienteRequest,
    state: State<ClienteAppState>,
) -> Result<Cliente, AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateCliente)?;
    let result = service.create(
        request.nombre,
        request.apellido,
        request.telefono,
        request.email,
        request.direccion,
    )?;
    log_audit(
        user_id,
        AuditScreen::Clientes,
        AuditAction::Create,
        Some(format!(
            "Cliente creado: {} (id {})",
            cliente_label(&result),
            result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn actualizar_cliente(
    user_id: i64,
    request: UpdateClienteRequest,
    state: State<ClienteAppState>,
) -> Result<Cliente, AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateCliente)?;
    let cliente = Cliente {
        id: request.id,
        nombre: request.nombre,
        apellido: request.apellido,
        telefono: request.telefono,
        email: request.email,
        direccion: request.direccion,
        created_at: String::new(),
        updated_at: String::new(),
    };
    let antes = service.get_by_id(request.id)?;
    let result = service.update(&cliente)?;
    let detail = AuditDetail::new("cliente", format!("{} (id {})", cliente_label(&result), result.id))
        .cambio("nombre", opt_str(&antes.nombre), opt_str(&result.nombre))
        .cambio("apellido", opt_str(&antes.apellido), opt_str(&result.apellido))
        .cambio("telefono", opt_str(&antes.telefono), opt_str(&result.telefono))
        .cambio("email", opt_str(&antes.email), opt_str(&result.email))
        .cambio("direccion", opt_str(&antes.direccion), opt_str(&result.direccion))
        .to_json();
    log_audit(
        user_id,
        AuditScreen::Clientes,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn eliminar_cliente(
    user_id: i64,
    id: i64,
    state: State<ClienteAppState>,
) -> Result<(), AppError> {
    let service = state
        .cliente_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteCliente)?;
    let antes = service.get_by_id(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Clientes,
        AuditAction::Delete,
        Some(format!(
            "Cliente eliminado: {} (id {})",
            cliente_label(&antes),
            id
        )),
    )?;
    Ok(())
}

fn cliente_label(c: &Cliente) -> String {
    let nombre = c.nombre.as_deref().unwrap_or("");
    let apellido = c.apellido.as_deref().unwrap_or("");
    let label = format!("{} {}", nombre, apellido).trim().to_string();
    if label.is_empty() {
        "(sin nombre)".to_string()
    } else {
        label
    }
}

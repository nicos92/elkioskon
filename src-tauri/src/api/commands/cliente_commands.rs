use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, ClienteService};
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
            "Cliente: {} (id {})",
            result.nombre.as_deref().unwrap_or(""),
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
    let result = service.update(&cliente)?;
    log_audit(
        user_id,
        AuditScreen::Clientes,
        AuditAction::Update,
        Some(format!(
            "Cliente: {} (id {})",
            result.nombre.as_deref().unwrap_or(""),
            result.id
        )),
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
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Clientes,
        AuditAction::Delete,
        Some(format!("Cliente (id {})", id)),
    )?;
    Ok(())
}

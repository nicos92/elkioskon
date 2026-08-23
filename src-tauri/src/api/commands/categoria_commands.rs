use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, CategoriaService};
use crate::domain::entities::{AuditAction, AuditScreen, Categoria, PermissionCode};
use crate::infrastructure::error::AppError;

pub struct CategoriaAppState {
    pub categoria_service: Mutex<CategoriaService>,
}

impl Default for CategoriaAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoriaAppState {
    pub fn new() -> Self {
        Self {
            categoria_service: Mutex::new(CategoriaService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateCategoriaRequest {
    pub categoria: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateCategoriaRequest {
    pub id: i64,
    pub categoria: String,
}

#[tauri::command(async)]
pub fn get_all_categorias(
    user_id: i64,
    state: State<CategoriaAppState>,
) -> Result<Vec<Categoria>, AppError> {
    let service = state
        .categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewCategorias)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn create_categoria(
    user_id: i64,
    request: CreateCategoriaRequest,
    state: State<CategoriaAppState>,
) -> Result<Categoria, AppError> {
    let service = state
        .categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateCategoria)?;
    let result = service.create(request.categoria)?;
    log_audit(
        user_id,
        AuditScreen::Categorias,
        AuditAction::Create,
        Some(format!(
            "Categoría: {} (id {})",
            result.categoria, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_categoria(
    user_id: i64,
    request: UpdateCategoriaRequest,
    state: State<CategoriaAppState>,
) -> Result<Categoria, AppError> {
    let service = state
        .categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateCategoria)?;
    let result = service.update(request.id, request.categoria)?;
    log_audit(
        user_id,
        AuditScreen::Categorias,
        AuditAction::Update,
        Some(format!(
            "Categoría: {} (id {})",
            result.categoria, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_categoria(
    user_id: i64,
    id: i64,
    state: State<CategoriaAppState>,
) -> Result<(), AppError> {
    let service = state
        .categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteCategoria)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Categorias,
        AuditAction::Delete,
        Some(format!("Categoría (id {})", id)),
    )?;
    Ok(())
}

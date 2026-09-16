use std::sync::Mutex;
use tauri::State;

use rusqlite::params;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, AuditDetail, SubCategoriaService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, SubCategoria};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SubCategoriaAppState {
    pub sub_categoria_service: Mutex<SubCategoriaService>,
}

impl Default for SubCategoriaAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl SubCategoriaAppState {
    pub fn new() -> Self {
        Self {
            sub_categoria_service: Mutex::new(SubCategoriaService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateSubCategoriaRequest {
    pub sub_categoria: String,
    pub id_categoria: i64,
}

#[derive(serde::Deserialize)]
pub struct UpdateSubCategoriaRequest {
    pub id: i64,
    pub sub_categoria: String,
    pub id_categoria: i64,
}

#[tauri::command(async)]
pub fn get_all_sub_categorias(
    user_id: i64,
    state: State<SubCategoriaAppState>,
) -> Result<Vec<SubCategoria>, AppError> {
    let service = state
        .sub_categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewSubCategorias)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn get_sub_categorias_by_categoria(
    user_id: i64,
    id_categoria: i64,
    state: State<SubCategoriaAppState>,
) -> Result<Vec<SubCategoria>, AppError> {
    let service = state
        .sub_categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewSubCategorias)?;
    service.get_by_categoria(id_categoria)
}

#[tauri::command(async)]
pub fn create_sub_categoria(
    user_id: i64,
    request: CreateSubCategoriaRequest,
    state: State<SubCategoriaAppState>,
) -> Result<SubCategoria, AppError> {
    let service = state
        .sub_categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateSubCategoria)?;
    let result = service.create(request.sub_categoria, request.id_categoria)?;
    log_audit(
        user_id,
        AuditScreen::SubCategorias,
        AuditAction::Create,
        Some(format!(
            "Sub categoría creada: {} (id {}) de categoría {}",
            result.sub_categoria,
            result.id,
            categoria_name(result.id_categoria)?
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_sub_categoria(
    user_id: i64,
    request: UpdateSubCategoriaRequest,
    state: State<SubCategoriaAppState>,
) -> Result<SubCategoria, AppError> {
    let service = state
        .sub_categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateSubCategoria)?;
    let antes = service.get_by_id(request.id)?;
    let result = service.update(request.id, request.sub_categoria, request.id_categoria)?;
    let descripcion = format!(
        "{} (id {}) de categoría {}",
        result.sub_categoria,
        result.id,
        categoria_name(result.id_categoria)?
    );
    let detail = AuditDetail::new("sub_categoria", descripcion)
        .cambio(
            "sub_categoria",
            &antes.sub_categoria,
            &result.sub_categoria,
        )
        .cambio(
            "categoria",
            categoria_name(antes.id_categoria)?,
            categoria_name(result.id_categoria)?,
        )
        .to_json();
    log_audit(
        user_id,
        AuditScreen::SubCategorias,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_sub_categoria(
    user_id: i64,
    id: i64,
    state: State<SubCategoriaAppState>,
) -> Result<(), AppError> {
    let service = state
        .sub_categoria_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteSubCategoria)?;
    let antes = service.get_by_id(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::SubCategorias,
        AuditAction::Delete,
        Some(format!(
            "Sub categoría eliminada: {} (id {}) de categoría {}",
            antes.sub_categoria,
            id,
            categoria_name(antes.id_categoria)?
        )),
    )?;
    Ok(())
}

fn categoria_name(id: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let name: String = conn
        .query_row(
            "SELECT categoria FROM categorias WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(name)
}

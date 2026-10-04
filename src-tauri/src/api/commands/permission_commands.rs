use rusqlite::params;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::api::commands::AppState;
use crate::application::services::log_audit;
use crate::domain::entities::{
    AuditAction, AuditScreen, Permission, PermissionCode, UserPermission,
};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

#[derive(serde::Deserialize)]
pub struct AddPermissionRequest {
    pub user_id: i64,
    pub permission_id: i64,
}

fn permission_code_by_id(id: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let name: String = conn
        .query_row(
            "SELECT permission FROM permissions WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(name)
}

#[tauri::command(async)]
pub fn add_permission_to_user(
    user_id: i64,
    request: AddPermissionRequest,
    state: State<AppState>,
) -> Result<(), AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::AssignPermission)?;
    service.add_permission_to_user(request.user_id, request.permission_id)?;
    let perm = permission_code_by_id(request.permission_id)?;
    let target = service.get_user(request.user_id)?;
    log_audit(
        user_id,
        AuditScreen::Permisos,
        AuditAction::Update,
        Some(format!(
            "Permiso {} asignado al usuario {} (id {})",
            perm, target.username, request.user_id
        )),
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn remove_permission_from_user(
    user_id: i64,
    request: AddPermissionRequest,
    state: State<AppState>,
) -> Result<(), AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::RemovePermission)?;
    service.remove_permission_from_user(request.user_id, request.permission_id)?;
    let perm = permission_code_by_id(request.permission_id)?;
    let target = service.get_user(request.user_id)?;
    log_audit(
        user_id,
        AuditScreen::Permisos,
        AuditAction::Update,
        Some(format!(
            "Permiso {} quitado al usuario {} (id {})",
            perm, target.username, request.user_id
        )),
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_user_permissions(
    user_id: i64,
    target_user_id: i64,
    state: State<AppState>,
) -> Result<Vec<UserPermission>, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewPermissions)?;
    service.get_user_permissions(target_user_id)
}

#[tauri::command(async)]
pub fn get_all_permissions(
    user_id: i64,
    state: State<AppState>,
) -> Result<Vec<Permission>, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewPermissions)?;
    service.get_all_permissions()
}

#[tauri::command(async)]
pub fn create_permission(
    user_id: i64,
    name: String,
    state: State<AppState>,
) -> Result<Permission, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateCategoria)?;
    let permission = service.create_permission(name)?;
    log_audit(
        user_id,
        AuditScreen::Permisos,
        AuditAction::Create,
        Some(format!(
            "Permiso: {} (id {})",
            permission.permission, permission.id
        )),
    )?;
    Ok(permission)
}

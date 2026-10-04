use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::api::commands::AppState;
use crate::application::services::{log_audit, AuditDetail};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, User};
use crate::infrastructure::error::AppError;

#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateUserRequest {
    pub id: i64,
    pub username: String,
    pub active: bool,
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub target_user_id: i64,
    pub current_password: Option<String>,
    pub new_password: String,
}

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub active: bool,
    pub created_at: String,
    pub modified_at: String,
    pub permissions: Vec<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            active: user.active,
            created_at: user.created_at.to_rfc3339(),
            modified_at: user.modified_at.to_rfc3339(),
            permissions: Vec::new(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
    pub permissions: Vec<String>,
}

#[tauri::command(async)]
pub fn login(request: LoginRequest, state: State<AppState>) -> Result<LoginResponse, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user = service.login(request.username, request.password)?;
    let permissions = service
        .get_user_permissions_by_names(user.id)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(LoginResponse {
        user: user.into(),
        permissions,
    })
}

#[tauri::command(async)]
pub fn create_user(
    user_id: i64,
    request: CreateUserRequest,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateUser)?;
    let user = service.create_user(request.username, request.password)?;
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Create,
        Some(format!(
            "Usuario creado: {} (id {})",
            user.username, user.id
        )),
    )?;
    Ok(user.into())
}

#[tauri::command(async)]
pub fn get_all_users(user_id: i64, state: State<AppState>) -> Result<Vec<UserResponse>, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewUsers)?;
    let users = service.get_all_users()?;
    Ok(users.into_iter().map(|u| u.into()).collect())
}

#[tauri::command(async)]
pub fn update_user(
    user_id: i64,
    request: UpdateUserRequest,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateUser)?;
    let antes = service.get_user(request.id)?;
    let user = service.update_user(request.id, request.username, request.active)?;
    let detail = AuditDetail::new("usuario", format!("{} (id {})", user.username, user.id))
        .cambio("username", &antes.username, &user.username)
        .cambio("activo", antes.active, user.active)
        .to_json();
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(user.into())
}

#[tauri::command(async)]
pub fn change_password(
    user_id: i64,
    request: ChangePasswordRequest,
    state: State<AppState>,
) -> Result<UserResponse, AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if user_id != request.target_user_id {
        check_permission(user_id, PermissionCode::ChangeUserPassword)?;
    }

    let target = service.get_user(request.target_user_id)?;
    let user = service.change_password(
        user_id,
        request.target_user_id,
        request.current_password,
        request.new_password,
    )?;
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Update,
        Some(format!(
            "Contraseña actualizada para el usuario {} (id {})",
            target.username, request.target_user_id
        )),
    )?;
    Ok(user.into())
}

#[tauri::command(async)]
pub fn delete_user(user_id: i64, id: i64, state: State<AppState>) -> Result<(), AppError> {
    let service = state
        .user_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteUser)?;
    let antes = service.get_user(id)?;
    service.delete_user(user_id, id)?;
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Delete,
        Some(format!("Usuario eliminado: {} (id {})", antes.username, id)),
    )?;
    Ok(())
}

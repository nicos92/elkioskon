use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, UserService};
use crate::domain::entities::{
    AuditAction, AuditScreen, Permission, PermissionCode, User, UserPermission,
};
use crate::infrastructure::error::AppError;

pub mod articulo_commands;
pub mod audit_log_commands;
pub mod categoria_commands;
pub mod cierre_commands;
pub mod cliente_commands;
pub mod cost_update_commands;
pub mod dollar_commands;
pub mod home_commands;
pub mod nocturno_commands;
pub mod permissions;
pub mod presupuesto_commands;
pub mod proveedor_commands;
pub mod stock_commands;
pub mod sub_categoria_commands;
pub mod tipo_venta_commands;
pub mod venta_commands;

pub use articulo_commands::{
    create_articulo, delete_articulo, get_all_articulos, update_articulo, ArticuloAppState,
};
pub use audit_log_commands::{get_audit_logs, AuditLogAppState};
pub use categoria_commands::{
    create_categoria, delete_categoria, get_all_categorias, update_categoria, CategoriaAppState,
};
pub use cierre_commands::{
    crear_cierre, get_all_cierres, is_dia_cerrado, reabrir_cierre, CierreAppState,
};
pub use cliente_commands::{
    actualizar_cliente, crear_cliente, eliminar_cliente, get_all_clientes, get_cliente_by_id,
    get_cliente_defecto, ClienteAppState,
};
pub use cost_update_commands::{
    apply_costo_percentage_stock, cleanup_cost_update_operations, get_last_undoable_cost_update,
    get_stock_preview_costo, undo_cost_update, CostUpdateAppState,
};
pub use dollar_commands::{
    delete_dollar_quote, fetch_dollar_rates_manual, get_dollar_quotes, DollarAppState,
};
pub use home_commands::{get_home_stats, HomeStatsAppState};
pub use nocturno_commands::{
    get_nocturno_config, save_nocturno_config, NocturnoConfigAppState,
};
pub use presupuesto_commands::{
    cambiar_estado_presupuesto, crear_presupuesto, get_all_presupuestos, get_presupuesto_by_id,
    PresupuestoAppState,
};
pub use proveedor_commands::{
    create_proveedor, delete_proveedor, get_all_proveedores, get_proveedor_by_id, update_proveedor,
    ProveedorAppState,
};
pub use stock_commands::{
    create_stock, delete_stock, get_all_stock, get_precio_venta, get_stock_by_articulo,
    get_stock_by_id, update_stock, StockAppState,
};
pub use sub_categoria_commands::{
    create_sub_categoria, delete_sub_categoria, get_all_sub_categorias,
    get_sub_categorias_by_categoria, update_sub_categoria, SubCategoriaAppState,
};
pub use tipo_venta_commands::{
    create_tipo_venta, delete_tipo_venta, get_all_tipos_venta, update_tipo_venta, TipoVentaAppState,
};
pub use venta_commands::{
    anular_venta, create_venta, get_all_ventas, get_venta_by_id, get_ventas_por_cliente,
    VentaAppState,
};

pub struct AppState {
    pub user_service: Mutex<UserService>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user_service: Mutex::new(UserService::new()),
        }
    }
}

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
pub struct AddPermissionRequest {
    pub user_id: i64,
    pub permission_id: i64,
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

#[tauri::command]
pub async fn ensure_db_ready() -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(|| {
        let _ = &*crate::infrastructure::database::DB;
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
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
        Some(format!("Usuario: {} (id {})", user.username, user.id)),
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
    let user = service.update_user(request.id, request.username, request.active)?;
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Update,
        Some(format!("Usuario: {} (id {})", user.username, user.id)),
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
            "Contraseña actualizada (usuario id {})",
            request.target_user_id
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
    service.delete_user(user_id, id)?;
    log_audit(
        user_id,
        AuditScreen::Usuarios,
        AuditAction::Delete,
        Some(format!("Usuario (id {})", id)),
    )?;
    Ok(())
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
    log_audit(
        user_id,
        AuditScreen::Permisos,
        AuditAction::Update,
        Some(format!(
            "Permiso (id {}) asignado al usuario (id {})",
            request.permission_id, request.user_id
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
    log_audit(
        user_id,
        AuditScreen::Permisos,
        AuditAction::Update,
        Some(format!(
            "Permiso (id {}) quitado al usuario (id {})",
            request.permission_id, request.user_id
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

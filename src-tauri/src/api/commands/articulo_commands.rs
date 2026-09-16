use std::sync::Mutex;
use tauri::State;

use rusqlite::params;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, AuditDetail, ArticuloService};
use crate::domain::entities::{Articulo, AuditAction, AuditScreen, PermissionCode};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct ArticuloAppState {
    pub articulo_service: Mutex<ArticuloService>,
}

impl Default for ArticuloAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl ArticuloAppState {
    pub fn new() -> Self {
        Self {
            articulo_service: Mutex::new(ArticuloService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateArticuloRequest {
    pub articulo: String,
    pub cod_articulo: String,
    pub id_sub_categoria: i64,
    pub id_proveedor: i64,
}

#[derive(serde::Deserialize)]
pub struct UpdateArticuloRequest {
    pub id: i64,
    pub articulo: String,
    pub cod_articulo: String,
    pub id_sub_categoria: i64,
    pub id_proveedor: i64,
}

#[tauri::command(async)]
pub fn get_all_articulos(
    user_id: i64,
    state: State<ArticuloAppState>,
) -> Result<Vec<Articulo>, AppError> {
    let service = state
        .articulo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewArticulos)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn create_articulo(
    user_id: i64,
    request: CreateArticuloRequest,
    state: State<ArticuloAppState>,
) -> Result<Articulo, AppError> {
    let service = state
        .articulo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateArticulo)?;
    let result = service.create(
        request.articulo,
        request.cod_articulo,
        request.id_sub_categoria,
        request.id_proveedor,
    )?;
    log_audit(
        user_id,
        AuditScreen::Articulos,
        AuditAction::Create,
        Some(format!(
            "Artículo creado: {} (cód. {}) sub-categoría {}, proveedor {} (id {})",
            result.articulo,
            result.cod_articulo,
            sub_categoria_name(result.id_sub_categoria)?,
            proveedor_name(result.id_proveedor)?,
            result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_articulo(
    user_id: i64,
    request: UpdateArticuloRequest,
    state: State<ArticuloAppState>,
) -> Result<Articulo, AppError> {
    let service = state
        .articulo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateArticulo)?;
    let antes = service.get_by_id(request.id)?;
    let result = service.update(
        request.id,
        request.articulo,
        request.cod_articulo,
        request.id_sub_categoria,
        request.id_proveedor,
    )?;
    let descripcion = format!("{} (cód. {})", result.articulo, result.cod_articulo);
    let detail = AuditDetail::new("articulo", descripcion)
        .cambio("articulo", &antes.articulo, &result.articulo)
        .cambio("cod_articulo", &antes.cod_articulo, &result.cod_articulo)
        .cambio(
            "sub_categoria",
            sub_categoria_name(antes.id_sub_categoria)?,
            sub_categoria_name(result.id_sub_categoria)?,
        )
        .cambio(
            "proveedor",
            proveedor_name(antes.id_proveedor)?,
            proveedor_name(result.id_proveedor)?,
        )
        .to_json();
    log_audit(
        user_id,
        AuditScreen::Articulos,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_articulo(
    user_id: i64,
    id: i64,
    state: State<ArticuloAppState>,
) -> Result<(), AppError> {
    let service = state
        .articulo_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteArticulo)?;
    let antes = service.get_by_id(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Articulos,
        AuditAction::Delete,
        Some(format!(
            "Artículo eliminado: {} (cód. {}) (id {})",
            antes.articulo, antes.cod_articulo, id
        )),
    )?;
    Ok(())
}

fn sub_categoria_name(id: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let name: String = conn
        .query_row(
            "SELECT sub_categoria FROM sub_categorias WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(name)
}

fn proveedor_name(id: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let name: String = conn
        .query_row(
            "SELECT nombre FROM proveedores WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(name)
}

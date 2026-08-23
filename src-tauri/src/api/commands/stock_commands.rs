use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, StockService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, Stock};
use crate::infrastructure::error::AppError;

pub struct StockAppState {
    pub stock_service: Mutex<StockService>,
}

impl Default for StockAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl StockAppState {
    pub fn new() -> Self {
        Self {
            stock_service: Mutex::new(StockService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateStockRequest {
    pub id_articulo: i64,
    pub cantidad: f64,
    pub costo: f64,
    pub ganancia: f64,
}

#[derive(serde::Deserialize)]
pub struct UpdateStockRequest {
    pub id: i64,
    pub cantidad: f64,
    pub costo: f64,
    pub ganancia: f64,
}

#[tauri::command(async)]
pub fn get_all_stock(user_id: i64, state: State<StockAppState>) -> Result<Vec<Stock>, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewStock)?;
    service.get_all()
}

#[tauri::command(async)]
pub fn get_stock_by_id(
    user_id: i64,
    id: i64,
    state: State<StockAppState>,
) -> Result<Stock, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewStock)?;
    service.get_by_id(id)
}

#[tauri::command(async)]
pub fn get_stock_by_articulo(
    user_id: i64,
    id_articulo: i64,
    state: State<StockAppState>,
) -> Result<Option<Stock>, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewStock)?;
    service.get_by_articulo(id_articulo)
}

#[tauri::command(async)]
pub fn create_stock(
    user_id: i64,
    request: CreateStockRequest,
    state: State<StockAppState>,
) -> Result<Stock, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateStock)?;
    let result = service.create(
        request.id_articulo,
        request.cantidad,
        request.costo,
        request.ganancia,
    )?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Create,
        Some(format!(
            "Stock artículo {} (id {})",
            result.id_articulo, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn update_stock(
    user_id: i64,
    request: UpdateStockRequest,
    state: State<StockAppState>,
) -> Result<Stock, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::UpdateStock)?;
    let result = service.update(
        request.id,
        request.cantidad,
        request.costo,
        request.ganancia,
    )?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Update,
        Some(format!(
            "Stock artículo {} (id {})",
            result.id_articulo, result.id
        )),
    )?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_stock(user_id: i64, id: i64, state: State<StockAppState>) -> Result<(), AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteStock)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Delete,
        Some(format!("Stock (id {})", id)),
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_precio_venta(
    user_id: i64,
    id: i64,
    state: State<StockAppState>,
) -> Result<f64, AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewStock)?;
    service.get_precio_venta(id)
}

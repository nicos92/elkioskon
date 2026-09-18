use std::sync::Mutex;
use tauri::State;

use rusqlite::params;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, AuditDetail, StockService};
use crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, Stock};
use crate::infrastructure::database::DB;
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
    pub ganancia_diurna: f64,
    pub ganancia_nocturna: f64,
}

#[derive(serde::Deserialize)]
pub struct UpdateStockRequest {
    pub id: i64,
    pub cantidad: f64,
    pub costo: f64,
    pub ganancia: f64,
    pub ganancia_diurna: f64,
    pub ganancia_nocturna: f64,
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
        request.ganancia_diurna,
        request.ganancia_nocturna,
    )?;
    let label = articulo_label(result.id_articulo)?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Create,
        Some(format!(
            "Stock creado: {}, cantidad={}, costo={}, ganancia={}, ganancia_diurna={}, ganancia_nocturna={}",
            label, result.cantidad, result.costo, result.ganancia, result.ganancia_diurna, result.ganancia_nocturna
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
    let antes = service.get_by_id(request.id)?;
    let result = service.update(
        request.id,
        request.cantidad,
        request.costo,
        request.ganancia,
        request.ganancia_diurna,
        request.ganancia_nocturna,
    )?;
    let label = articulo_label(result.id_articulo)?;
    let detail = AuditDetail::new("stock", label)
        .cambio("cantidad", antes.cantidad, result.cantidad)
        .cambio("costo", antes.costo, result.costo)
        .cambio("ganancia", antes.ganancia, result.ganancia)
        .cambio("ganancia_diurna", antes.ganancia_diurna, result.ganancia_diurna)
        .cambio(
            "ganancia_nocturna",
            antes.ganancia_nocturna,
            result.ganancia_nocturna,
        )
        .to_json();
    log_audit(user_id, AuditScreen::Stock, AuditAction::Update, Some(detail))?;
    Ok(result)
}

#[tauri::command(async)]
pub fn delete_stock(user_id: i64, id: i64, state: State<StockAppState>) -> Result<(), AppError> {
    let service = state
        .stock_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::DeleteStock)?;
    let label = articulo_label_by_stock(id)?;
    service.delete(id)?;
    log_audit(
        user_id,
        AuditScreen::Stock,
        AuditAction::Delete,
        Some(format!("Stock eliminado: {}", label)),
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

fn articulo_label(id_articulo: i64) -> Result<String, AppError> {
    query_articulo_label("SELECT articulo, cod_articulo FROM articulos WHERE id = ?1", id_articulo)
}

fn articulo_label_by_stock(id_stock: i64) -> Result<String, AppError> {
    query_articulo_label(
        "SELECT a.articulo, a.cod_articulo
         FROM stock s JOIN articulos a ON a.id = s.id_articulo
         WHERE s.id = ?1",
        id_stock,
    )
}

fn query_articulo_label(sql: &str, param: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let (articulo, cod): (String, String) = conn
        .query_row(sql, params![param], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(format!("{} (cód. {})", articulo, cod))
}

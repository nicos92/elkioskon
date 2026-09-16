use std::sync::Mutex;

use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, DollarService};
use crate::domain::entities::{AuditAction, AuditScreen, DollarQuote, PermissionCode};
use crate::infrastructure::error::AppError;

pub struct DollarAppState {
    pub dollar_service: Mutex<DollarService>,
}

impl Default for DollarAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl DollarAppState {
    pub fn new() -> Self {
        Self {
            dollar_service: Mutex::new(DollarService::new()),
        }
    }
}

#[tauri::command(async)]
pub fn get_dollar_quotes(
    user_id: i64,
    state: State<DollarAppState>,
) -> Result<Vec<DollarQuote>, AppError> {
    check_permission(user_id, PermissionCode::ViewDolar)?;
    let service = state
        .dollar_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    service.get_history()
}

#[tauri::command(async)]
pub async fn fetch_dollar_rates_manual(
    user_id: i64,
    state: State<'_, DollarAppState>,
) -> Result<Vec<DollarQuote>, AppError> {
    check_permission(user_id, PermissionCode::ViewDolar)?;
    let service = state
        .dollar_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();
    service.fetch_and_persist().await?;
    let history = service.get_history()?;
    let detail = match history.first() {
        Some(quote) => format!(
            "Cotización del dólar actualizada manualmente: oficial {}/{} — blue {}/{}",
            quote.official_buy, quote.official_sell, quote.blue_buy, quote.blue_sell
        ),
        None => "Cotización del dólar actualizada manualmente (sin datos)".to_string(),
    };
    log_audit(
        user_id,
        AuditScreen::Dolar,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(history)
}

#[tauri::command(async)]
pub fn delete_dollar_quote(
    user_id: i64,
    id: i64,
    state: State<DollarAppState>,
) -> Result<Vec<DollarQuote>, AppError> {
    check_permission(user_id, PermissionCode::ViewDolar)?;
    let service = state
        .dollar_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let antes = service
        .get_history()?
        .into_iter()
        .find(|q| q.id == id)
        .map(|q| format!(
            "Cotización del dólar id={} eliminada (oficial {}/{}, blue {}/{})",
            id, q.official_buy, q.official_sell, q.blue_buy, q.blue_sell
        ))
        .unwrap_or_else(|| format!("Cotización del dólar id={} eliminada", id));
    service.delete(id)?;
    let history = service.get_history()?;
    log_audit(
        user_id,
        AuditScreen::Dolar,
        AuditAction::Delete,
        Some(antes),
    )?;
    Ok(history)
}

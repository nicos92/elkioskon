use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, VentaService};
use crate::domain::entities::{
    AuditAction, AuditScreen, PermissionCode, VentaDetalle, VentaWithDetalle,
};
use crate::domain::repositories::Page;
use crate::infrastructure::error::AppError;

pub struct VentaAppState {
    pub venta_service: Mutex<VentaService>,
}

impl Default for VentaAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl VentaAppState {
    pub fn new() -> Self {
        Self {
            venta_service: Mutex::new(VentaService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreateVentaDetalleRequest {
    pub id_articulo: i64,
    pub cantidad: f64,
    pub precio_unitario: Option<f64>,
}

#[derive(serde::Deserialize)]
pub struct CreateVentaRequest {
    pub items: Vec<CreateVentaDetalleRequest>,
    pub descuento: Option<f64>,
    pub observacion: Option<String>,
    pub id_tipo_venta: Option<i64>,
    pub cliente_id: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct GetVentasRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn can_sell_without_stock(user_id: i64) -> Result<bool, AppError> {
    match check_permission(user_id, PermissionCode::VenderSinStock) {
        Ok(()) => Ok(true),
        Err(AppError::PermissionDenied) => Ok(false),
        Err(e) => Err(e),
    }
}

#[tauri::command(async)]
pub fn create_venta(
    user_id: i64,
    request: CreateVentaRequest,
    state: State<VentaAppState>,
) -> Result<VentaWithDetalle, AppError> {
    let service = state
        .venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::CreateVenta)?;
    let allow_negative_stock = can_sell_without_stock(user_id)?;

    let detalles: Vec<VentaDetalle> = request
        .items
        .into_iter()
        .map(|item| {
            VentaDetalle::new(
                item.id_articulo,
                item.cantidad,
                0.0,
                item.precio_unitario.unwrap_or(0.0),
            )
        })
        .collect();

    let venta = service.create(
        user_id,
        detalles,
        request.descuento.unwrap_or(0.0),
        request.observacion,
        request.id_tipo_venta,
        request.cliente_id,
        allow_negative_stock,
    )?;
    log_audit(
        user_id,
        AuditScreen::Ventas,
        AuditAction::Create,
        Some(format!(
            "Venta creada: id {}, total=${}, {} ítems, cliente {}",
            venta.id,
            venta.total,
            venta.items.len(),
            venta_cliente_label(&venta.cliente_nombre, &venta.cliente_apellido)
        )),
    )?;
    Ok(venta)
}

#[tauri::command(async)]
pub fn get_all_ventas(
    user_id: i64,
    request: Option<GetVentasRequest>,
    state: State<VentaAppState>,
) -> Result<Page<VentaWithDetalle>, AppError> {
    let service = state
        .venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewVentas)?;
    let limit = request.as_ref().and_then(|r| r.limit).unwrap_or(50);
    let offset = request.as_ref().and_then(|r| r.offset).unwrap_or(0);
    service.get_page(limit, offset)
}

#[tauri::command(async)]
pub fn get_venta_by_id(
    user_id: i64,
    id: i64,
    state: State<VentaAppState>,
) -> Result<VentaWithDetalle, AppError> {
    let service = state
        .venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewVentas)?;
    service.get_by_id(id)
}

#[tauri::command(async)]
pub fn get_ventas_por_cliente(
    user_id: i64,
    cliente_id: i64,
    state: State<VentaAppState>,
) -> Result<Vec<VentaWithDetalle>, AppError> {
    let service = state
        .venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::ViewVentas)?;
    service.get_ventas_por_cliente(cliente_id)
}

#[tauri::command(async)]
pub fn anular_venta(user_id: i64, id: i64, state: State<VentaAppState>) -> Result<(), AppError> {
    let service = state
        .venta_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::AnularVenta)?;
    let antes = service.get_by_id(id)?;
    service.anular(id)?;
    log_audit(
        user_id,
        AuditScreen::Ventas,
        AuditAction::Update,
        Some(format!(
            "Venta {} anulada (total=${}, {} ítems, cliente {})",
            id,
            antes.total,
            antes.items.len(),
            venta_cliente_label(&antes.cliente_nombre, &antes.cliente_apellido)
        )),
    )?;
    Ok(())
}

fn venta_cliente_label(nombre: &Option<String>, apellido: &Option<String>) -> String {
    let n = nombre.as_deref().unwrap_or("");
    let a = apellido.as_deref().unwrap_or("");
    let label = format!("{} {}", n, a).trim().to_string();
    if label.is_empty() {
        "-".to_string()
    } else {
        label
    }
}

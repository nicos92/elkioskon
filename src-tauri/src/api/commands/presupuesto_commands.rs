use std::sync::Mutex;
use tauri::State;

use crate::api::commands::permissions::check_permission;
use crate::application::services::{log_audit, AuditDetail, PresupuestoService};
use crate::domain::entities::{
    AuditAction, AuditScreen, PermissionCode, PresupuestoDetalle, PresupuestoEstado,
    PresupuestoWithDetalle,
};
use crate::domain::repositories::{Page, PresupuestoFilter};
use crate::infrastructure::error::AppError;

pub struct PresupuestoAppState {
    pub presupuesto_service: Mutex<PresupuestoService>,
}

impl Default for PresupuestoAppState {
    fn default() -> Self {
        Self::new()
    }
}

impl PresupuestoAppState {
    pub fn new() -> Self {
        Self {
            presupuesto_service: Mutex::new(PresupuestoService::new()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct CreatePresupuestoDetalleRequest {
    pub id_articulo: i64,
    pub cantidad: f64,
    pub precio_unitario: Option<f64>,
}

#[derive(serde::Deserialize)]
pub struct CreatePresupuestoRequest {
    pub items: Vec<CreatePresupuestoDetalleRequest>,
    pub descuento: Option<f64>,
    pub observacion: Option<String>,
    pub fecha_vencimiento: Option<String>,
    pub cliente_id: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct GetPresupuestosRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub estado: Option<PresupuestoEstado>,
    pub fecha_desde: Option<String>,
    pub fecha_hasta: Option<String>,
    pub query: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CambiarEstadoPresupuestoRequest {
    pub id: i64,
    pub estado: PresupuestoEstado,
}

#[tauri::command(async)]
pub fn crear_presupuesto(
    user_id: i64,
    request: CreatePresupuestoRequest,
    state: State<PresupuestoAppState>,
) -> Result<PresupuestoWithDetalle, AppError> {
    let service = state
        .presupuesto_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GenerarPresupuesto)?;

    let detalles: Vec<PresupuestoDetalle> = request
        .items
        .into_iter()
        .map(|item| {
            PresupuestoDetalle::new(
                item.id_articulo,
                item.cantidad,
                0.0,
                item.precio_unitario.unwrap_or(0.0),
            )
        })
        .collect();

    let presupuesto = service.create(
        user_id,
        detalles,
        request.descuento.unwrap_or(0.0),
        request.observacion,
        request.fecha_vencimiento,
        request.cliente_id,
    )?;
    log_audit(
        user_id,
        AuditScreen::Presupuestos,
        AuditAction::Create,
        Some(format!(
            "Presupuesto creado: id {}, total=${}, {} ítems, cliente {}",
            presupuesto.id,
            presupuesto.total,
            presupuesto.items.len(),
            cliente_label(&presupuesto.cliente_nombre, &presupuesto.cliente_apellido)
        )),
    )?;
    Ok(presupuesto)
}

#[tauri::command(async)]
pub fn get_all_presupuestos(
    user_id: i64,
    request: Option<GetPresupuestosRequest>,
    state: State<PresupuestoAppState>,
) -> Result<Page<PresupuestoWithDetalle>, AppError> {
    let service = state
        .presupuesto_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GenerarPresupuesto)?;
    let limit = request.as_ref().and_then(|r| r.limit).unwrap_or(50);
    let offset = request.as_ref().and_then(|r| r.offset).unwrap_or(0);
    let filter = PresupuestoFilter {
        estado: request.as_ref().and_then(|r| r.estado),
        fecha_desde: request.as_ref().and_then(|r| r.fecha_desde.clone()),
        fecha_hasta: request.as_ref().and_then(|r| r.fecha_hasta.clone()),
        query: request.as_ref().and_then(|r| r.query.clone()),
    };
    service.get_page(&filter, limit, offset)
}

#[tauri::command(async)]
pub fn cambiar_estado_presupuesto(
    user_id: i64,
    request: CambiarEstadoPresupuestoRequest,
    state: State<PresupuestoAppState>,
) -> Result<(), AppError> {
    let service = state
        .presupuesto_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GenerarPresupuesto)?;
    let antes = service.get_by_id(request.id)?;
    service.cambiar_estado(request.id, request.estado)?;
    let descripcion = format!(
        "Presupuesto id {} (cliente {})",
        request.id,
        cliente_label(&antes.cliente_nombre, &antes.cliente_apellido)
    );
    let detail = AuditDetail::new("presupuesto", descripcion)
        .cambio("estado", &antes.estado, request.estado.as_str())
        .to_json();
    log_audit(
        user_id,
        AuditScreen::Presupuestos,
        AuditAction::Update,
        Some(detail),
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_presupuesto_by_id(
    user_id: i64,
    id: i64,
    state: State<PresupuestoAppState>,
) -> Result<PresupuestoWithDetalle, AppError> {
    let service = state
        .presupuesto_service
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    check_permission(user_id, PermissionCode::GenerarPresupuesto)?;
    service.get_by_id(id)
}

fn cliente_label(nombre: &Option<String>, apellido: &Option<String>) -> String {
    let n = nombre.as_deref().unwrap_or("");
    let a = apellido.as_deref().unwrap_or("");
    let label = format!("{} {}", n, a).trim().to_string();
    if label.is_empty() {
        "-".to_string()
    } else {
        label
    }
}

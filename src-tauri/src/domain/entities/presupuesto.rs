use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresupuestoEstado {
    Pendiente,
    Aprobado,
    Vencido,
    Convertido,
    Anulado,
}

impl PresupuestoEstado {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresupuestoEstado::Pendiente => "pendiente",
            PresupuestoEstado::Aprobado => "aprobado",
            PresupuestoEstado::Vencido => "vencido",
            PresupuestoEstado::Convertido => "convertido",
            PresupuestoEstado::Anulado => "anulado",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presupuesto {
    pub id: i64,
    pub user_id: i64,
    pub fecha: String,
    pub total: f64,
    pub descuento: f64,
    pub estado: PresupuestoEstado,
    pub fecha_vencimiento: Option<String>,
    pub observacion: Option<String>,
    pub cliente_id: Option<i64>,
    pub created_at: String,
}

impl Presupuesto {
    pub fn new(
        user_id: i64,
        fecha: String,
        descuento: f64,
        observacion: Option<String>,
        fecha_vencimiento: Option<String>,
        cliente_id: Option<i64>,
    ) -> Self {
        Self {
            id: 0,
            user_id,
            fecha,
            total: 0.0,
            descuento,
            estado: PresupuestoEstado::Pendiente,
            fecha_vencimiento,
            observacion,
            cliente_id,
            created_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresupuestoDetalle {
    pub id: i64,
    pub id_presupuesto: i64,
    pub id_articulo: i64,
    pub cantidad: f64,
    pub costo_unitario: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

impl PresupuestoDetalle {
    pub fn new(id_articulo: i64, cantidad: f64, costo_unitario: f64, precio_unitario: f64) -> Self {
        Self {
            id: 0,
            id_presupuesto: 0,
            id_articulo,
            cantidad,
            costo_unitario,
            precio_unitario,
            subtotal: cantidad * precio_unitario,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresupuestoDetalleConArticulo {
    pub id: i64,
    pub id_articulo: i64,
    pub cod_articulo: String,
    pub articulo: String,
    pub cantidad: f64,
    pub costo_unitario: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresupuestoWithDetalle {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub fecha: String,
    pub subtotal: f64,
    pub descuento: f64,
    pub total: f64,
    pub estado: String,
    pub fecha_vencimiento: Option<String>,
    pub observacion: Option<String>,
    pub cliente_id: Option<i64>,
    pub cliente_nombre: Option<String>,
    pub cliente_apellido: Option<String>,
    pub created_at: String,
    pub items: Vec<PresupuestoDetalleConArticulo>,
}

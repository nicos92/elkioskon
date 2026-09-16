use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venta {
    pub id: i64,
    pub user_id: i64,
    pub fecha: String,
    pub total: f64,
    pub descuento: f64,
    pub anulada: bool,
    pub observacion: Option<String>,
    pub id_tipo_venta: Option<i64>,
    pub cliente_id: i64,
    pub created_at: String,
    pub porcentaje_nocturno: f64,
}

impl Venta {
    pub fn new(user_id: i64, fecha: String, descuento: f64, observacion: Option<String>) -> Self {
        Self {
            id: 0,
            user_id,
            fecha,
            total: 0.0,
            descuento,
            anulada: false,
            observacion,
            id_tipo_venta: None,
            cliente_id: 0,
            created_at: String::new(),
            porcentaje_nocturno: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VentaDetalle {
    pub id: i64,
    pub id_venta: i64,
    pub id_articulo: i64,
    pub cantidad: f64,
    pub costo_unitario: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

impl VentaDetalle {
    pub fn new(id_articulo: i64, cantidad: f64, costo_unitario: f64, precio_unitario: f64) -> Self {
        Self {
            id: 0,
            id_venta: 0,
            id_articulo,
            cantidad,
            costo_unitario,
            precio_unitario,
            subtotal: cantidad * precio_unitario,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VentaDetalleConArticulo {
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
pub struct VentaWithDetalle {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub fecha: String,
    pub subtotal: f64,
    pub descuento: f64,
    pub total: f64,
    pub anulada: bool,
    pub observacion: Option<String>,
    pub tipo_venta: Option<String>,
    pub cliente_id: i64,
    pub cliente_nombre: Option<String>,
    pub cliente_apellido: Option<String>,
    pub created_at: String,
    pub porcentaje_nocturno: f64,
    pub items: Vec<VentaDetalleConArticulo>,
}

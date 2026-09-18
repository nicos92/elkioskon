use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockPreview {
    pub id_stock: i64,
    pub id_articulo: i64,
    pub cod_articulo: String,
    pub articulo: String,
    pub categoria: String,
    pub sub_categoria: String,
    pub proveedor: String,
    pub costo_actual: f64,
    pub ganancia: f64,
    pub ganancia_diurna: f64,
    pub ganancia_nocturna: f64,
    pub costo_nuevo: f64,
    pub cantidad: f64,
}

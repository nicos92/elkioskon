use chrono::TimeZone;

use crate::domain::entities::{CategoriaConSub, HomeStats, StockBajoItem, SubCategoriaInfo};
use crate::infrastructure::error::AppError;

pub struct HomeService;

impl Default for HomeService {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_stats(&self) -> Result<HomeStats, AppError> {
        let conn = crate::infrastructure::database::DB
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let total_articulos: i64 =
            conn.query_row("SELECT COUNT(*) FROM articulos", [], |row| row.get(0))?;

        let articulos_con_stock: i64 =
            conn.query_row("SELECT COUNT(DISTINCT id_articulo) FROM stock", [], |row| {
                row.get(0)
            })?;

        let (total_usuarios, usuarios_activos): (i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(active), 0) FROM users WHERE id != 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let total_proveedores: i64 =
            conn.query_row("SELECT COUNT(*) FROM proveedores", [], |row| row.get(0))?;

        let total_categorias: i64 =
            conn.query_row("SELECT COUNT(*) FROM categorias", [], |row| row.get(0))?;

        let total_sub_categorias: i64 =
            conn.query_row("SELECT COUNT(*) FROM sub_categorias", [], |row| row.get(0))?;

        let (inicio_hoy, inicio_manana) = hoy_utc_range();
        let (ventas_hoy, total_ventas_hoy): (i64, f64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(total), 0)
             FROM ventas
             WHERE anulada = 0 AND fecha >= ?1 AND fecha < ?2",
            rusqlite::params![inicio_hoy, inicio_manana],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let mut stock_bajo = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT s.id, s.id_articulo, a.cod_articulo, a.articulo, s.cantidad
                 FROM stock s
                 INNER JOIN articulos a ON a.id = s.id_articulo
                 WHERE s.cantidad < 10
                 ORDER BY s.cantidad ASC",
            )?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                stock_bajo.push(StockBajoItem {
                    id_stock: row.get(0)?,
                    id_articulo: row.get(1)?,
                    cod_articulo: row.get(2)?,
                    articulo: row.get(3)?,
                    cantidad: row.get(4)?,
                });
            }
        }

        let mut categorias: Vec<CategoriaConSub> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT c.id, c.categoria, sc.id, sc.sub_categoria
                 FROM categorias c
                 LEFT JOIN sub_categorias sc ON sc.id_categoria = c.id
                 ORDER BY c.categoria, sc.sub_categoria",
            )?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let cat_id: i64 = row.get(0)?;
                let categoria: String = row.get(1)?;
                let sub_id: Option<i64> = row.get(2)?;
                let sub_nombre: Option<String> = row.get(3)?;

                match categorias.iter_mut().find(|c| c.id == cat_id) {
                    Some(entry) => {
                        if let (Some(id), Some(sub_categoria)) = (sub_id, sub_nombre) {
                            entry
                                .sub_categorias
                                .push(SubCategoriaInfo { id, sub_categoria });
                        }
                    }
                    None => {
                        let mut entry = CategoriaConSub {
                            id: cat_id,
                            categoria,
                            sub_categorias: Vec::new(),
                        };
                        if let (Some(id), Some(sub_categoria)) = (sub_id, sub_nombre) {
                            entry
                                .sub_categorias
                                .push(SubCategoriaInfo { id, sub_categoria });
                        }
                        categorias.push(entry);
                    }
                }
            }
        }

        Ok(HomeStats {
            total_articulos,
            articulos_con_stock,
            total_usuarios,
            usuarios_activos,
            usuarios_inactivos: total_usuarios - usuarios_activos,
            total_proveedores,
            total_categorias,
            total_sub_categorias,
            ventas_hoy,
            total_ventas_hoy,
            stock_bajo,
            categorias,
        })
    }
}

fn hoy_utc_range() -> (String, String) {
    let hoy = chrono::Local::now().date_naive();
    let inicio = local_to_utc(&hoy.and_hms_opt(0, 0, 0).unwrap());
    let fin = local_to_utc(
        &(hoy + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    );
    (inicio, fin)
}

fn local_to_utc(dt: &chrono::NaiveDateTime) -> String {
    chrono::Local
        .from_local_datetime(dt)
        .earliest()
        .or_else(|| chrono::Local.from_local_datetime(dt).latest())
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339()
}

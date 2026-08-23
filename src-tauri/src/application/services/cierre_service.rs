use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate, TimeZone};

use crate::domain::entities::{Cierre, CierreTipo, CierreWithTipos};
use crate::domain::repositories::{CierreRepository, Page};
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteCierreRepository;

pub struct CierreService {
    repository: Arc<SqliteCierreRepository>,
}

impl Default for CierreService {
    fn default() -> Self {
        Self::new()
    }
}

impl CierreService {
    pub fn new() -> Self {
        Self {
            repository: Arc::new(SqliteCierreRepository::new()),
        }
    }

    pub fn crear_cierre(&self, fecha: &str) -> Result<CierreWithTipos, AppError> {
        let day = NaiveDate::parse_from_str(fecha, "%Y-%m-%d")
            .map_err(|e| AppError::Internal(format!("Fecha inválida: {}", e)))?;

        let today = Local::now().date_naive();
        if day > today {
            return Err(AppError::CierreFechaFutura);
        }

        let start = local_to_utc(&day.and_hms_opt(0, 0, 0).unwrap());
        let end = local_to_utc(
            &(day + chrono::Duration::days(1))
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );

        let mut conn = crate::infrastructure::database::DB
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;

        let exists: i64 = tx.query_row(
            "SELECT COUNT(*) FROM cierres WHERE fecha = ?1",
            rusqlite::params![fecha],
            |row| row.get(0),
        )?;
        if exists > 0 {
            return Err(AppError::CierreYaExiste);
        }

        let mut ventas_by_tipo: BTreeMap<i64, (String, f64)> = BTreeMap::new();
        {
            let mut stmt = tx.prepare(
                "SELECT v.id_tipo_venta, COALESCE(t.nombre, 'Efectivo'), v.total
                 FROM ventas v
                 LEFT JOIN tipos_venta t ON t.id = v.id_tipo_venta
                 WHERE v.anulada = 0 AND v.fecha >= ?1 AND v.fecha < ?2
                 ORDER BY v.id_tipo_venta",
            )?;
            let mut rows = stmt.query(rusqlite::params![start, end])?;
            while let Some(row) = rows.next()? {
                let id_tipo: i64 = row.get(0)?;
                let nombre: String = row.get(1)?;
                let total: f64 = row.get(2)?;
                let entry = ventas_by_tipo.entry(id_tipo).or_insert((nombre, 0.0));
                entry.1 += total;
            }
        }

        if ventas_by_tipo.is_empty() {
            return Err(AppError::CierreSinVentas);
        }

        let total_venta: f64 = ventas_by_tipo.values().map(|(_, total)| *total).sum();

        let mut total_costo = 0.0;
        {
            let mut stmt = tx.prepare(
                "SELECT d.costo_unitario * d.cantidad
                 FROM venta_detalle d
                 INNER JOIN ventas v ON v.id = d.id_venta
                 WHERE v.anulada = 0 AND v.fecha >= ?1 AND v.fecha < ?2",
            )?;
            let mut rows = stmt.query(rusqlite::params![start, end])?;
            while let Some(row) = rows.next()? {
                let costo: f64 = row.get(0)?;
                total_costo += costo;
            }
        }

        let total_costo = round2(total_costo);
        let total_venta = round2(total_venta);
        let total_ganancia = round2(total_venta - total_costo);

        let tipos: Vec<CierreTipo> = ventas_by_tipo
            .into_iter()
            .map(|(id_tipo, (tipo_venta, total))| CierreTipo {
                id_tipo_venta: id_tipo,
                tipo_venta,
                total: round2(total),
            })
            .collect();

        let cierre = Cierre {
            id: 0,
            fecha: fecha.to_string(),
            dia: day.day() as i64,
            mes: day.month() as i64,
            anio: day.year() as i64,
            total_costo,
            total_ganancia,
            total_venta,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let id = self.repository.insert(&tx, &cierre, &tipos)?;
        tx.commit()?;

        self.repository
            .load_by_id(&conn, id)?
            .ok_or(AppError::CierreNotFound)
    }

    pub fn reabrir_cierre(&self, fecha: &str) -> Result<(), AppError> {
        self.repository.delete_by_fecha(fecha)
    }

    pub fn is_dia_cerrado(&self) -> Result<bool, AppError> {
        let hoy = Local::now().format("%Y-%m-%d").to_string();
        Ok(self.repository.find_by_fecha(&hoy)?.is_some())
    }

    pub fn get_page(&self, limit: i64, offset: i64) -> Result<Page<CierreWithTipos>, AppError> {
        self.repository.find_page(limit, offset)
    }
}

fn local_to_utc(dt: &chrono::NaiveDateTime) -> String {
    Local
        .from_local_datetime(dt)
        .earliest()
        .or_else(|| Local.from_local_datetime(dt).latest())
        .unwrap()
        .with_timezone(&chrono::Utc)
        .to_rfc3339()
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

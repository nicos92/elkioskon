use rusqlite::params;

use crate::domain::entities::{
    es_nocturno_ahora, margen_efectivo, NocturnoConfig, Presupuesto, PresupuestoDetalle,
    PresupuestoDetalleConArticulo, PresupuestoEstado, PresupuestoWithDetalle,
};
use crate::domain::repositories::{Page, PresupuestoFilter, PresupuestoRepository};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqlitePresupuestoRepository;

impl Default for SqlitePresupuestoRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqlitePresupuestoRepository {
    pub fn new() -> Self {
        Self
    }
}

impl PresupuestoRepository for SqlitePresupuestoRepository {
    fn create(
        &self,
        presupuesto: &Presupuesto,
        detalles: &[PresupuestoDetalle],
    ) -> Result<PresupuestoWithDetalle, AppError> {
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;

        let mut items: Vec<PresupuestoDetalle> = Vec::new();
        let mut total = 0.0;

        let (config_activo, es_nocturno) = turno_actual(&tx)?;

        for detalle in detalles {
            let stock = tx.query_row(
                "SELECT costo, ganancia, ganancia_diurna, ganancia_nocturna FROM stock WHERE id_articulo = ?1",
                params![detalle.id_articulo],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                    ))
                },
            );

            let (costo, ganancia, ganancia_diurna, ganancia_nocturna) = match stock {
                Ok(values) => values,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Err(AppError::ArticuloWithoutStock);
                }
                Err(e) => return Err(e.into()),
            };

            let margen = margen_efectivo(
                ganancia,
                ganancia_diurna,
                ganancia_nocturna,
                config_activo,
                es_nocturno,
            );

            let precio_unitario = if config_activo {
                costo * (1.0 + margen / 100.0)
            } else if detalle.precio_unitario > 0.0 {
                detalle.precio_unitario
            } else {
                costo * (1.0 + ganancia / 100.0)
            };

            let mut item = detalle.clone();
            item.costo_unitario = costo;
            item.precio_unitario = precio_unitario;
            item.subtotal = item.cantidad * item.precio_unitario;
            total += item.subtotal;
            items.push(item);
        }

        let now = chrono::Utc::now().to_rfc3339();
        total = (total * (1.0 - presupuesto.descuento / 100.0) * 100.0).round() / 100.0;
        tx.execute(
            "INSERT INTO presupuestos (user_id, fecha, total, descuento, estado, fecha_vencimiento, observacion, cliente_id, created_at) VALUES (?1, ?2, ?3, ?4, 'pendiente', ?5, ?6, ?7, ?8)",
            params![
                presupuesto.user_id,
                now,
                total,
                presupuesto.descuento,
                &presupuesto.fecha_vencimiento,
                &presupuesto.observacion,
                presupuesto.cliente_id,
                now
            ],
        )?;
        let presupuesto_id = tx.last_insert_rowid();

        for item in &items {
            tx.execute(
                "INSERT INTO detalle_presupuestos (id_presupuesto, id_articulo, cantidad, costo_unitario, precio_unitario, subtotal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    presupuesto_id,
                    item.id_articulo,
                    item.cantidad,
                    item.costo_unitario,
                    item.precio_unitario,
                    item.subtotal
                ],
            )?;
        }

        tx.commit()?;

        let mut presupuesto = self
            .load_presupuesto(&conn, presupuesto_id)?
            .ok_or(AppError::PresupuestoNotFound)?;
        presupuesto.items = self.load_items(&conn, presupuesto_id)?;
        presupuesto.subtotal = presupuesto.items.iter().map(|i| i.subtotal).sum();
        Ok(presupuesto)
    }

    fn find_by_id(&self, id: i64) -> Result<Option<PresupuestoWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut presupuesto = self.load_presupuesto(&conn, id)?;
        if let Some(p) = presupuesto.as_mut() {
            p.items = self.load_items(&conn, id)?;
            p.subtotal = p.items.iter().map(|i| i.subtotal).sum();
        }
        Ok(presupuesto)
    }

    fn find_page(
        &self,
        filter: &PresupuestoFilter,
        limit: i64,
        offset: i64,
    ) -> Result<Page<PresupuestoWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let limit = limit.max(1);
        let offset = offset.max(0);

        let (where_clause, mut params) = build_filter_where(filter);

        let count_sql = format!(
            "SELECT COUNT(*) FROM presupuestos p
             LEFT JOIN clientes c ON c.id = p.cliente_id
             LEFT JOIN users u ON u.id = p.user_id
             {}",
            where_clause
        );
        let count_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let total: i64 = conn.query_row(
            &count_sql,
            rusqlite::params_from_iter(count_params),
            |row| row.get(0),
        )?;

        let sql = format!(
            "SELECT p.id, p.user_id, COALESCE(u.username, ''), p.fecha, p.total, p.descuento, p.estado, p.fecha_vencimiento, p.observacion, p.created_at, p.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM presupuestos p
             LEFT JOIN users u ON u.id = p.user_id
             LEFT JOIN clientes c ON c.id = p.cliente_id
             {}
             ORDER BY p.id DESC
             LIMIT ? OFFSET ?",
            where_clause
        );

        params.push(Box::new(limit));
        params.push(Box::new(offset));
        let list_params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(list_params))?;
        let mut presupuestos = Vec::new();
        let mut ids: Vec<i64> = Vec::new();

        while let Some(row) = rows.next()? {
            let presupuesto = self.row_to_presupuesto(row)?;
            ids.push(presupuesto.id);
            presupuestos.push(presupuesto);
        }

        if !ids.is_empty() {
            let items = self.load_items_bulk(&conn, &ids)?;
            for presupuesto in presupuestos.iter_mut() {
                presupuesto.items = items.get(&presupuesto.id).cloned().unwrap_or_default();
                presupuesto.subtotal = presupuesto.items.iter().map(|i| i.subtotal).sum();
            }
        }

        Ok(Page {
            items: presupuestos,
            total,
            limit,
            offset,
        })
    }

    fn update_estado(&self, id: i64, estado: PresupuestoEstado) -> Result<(), AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let current: String = conn
            .query_row(
                "SELECT estado FROM presupuestos WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::PresupuestoNotFound,
                other => other.into(),
            })?;

        if current == "convertido" || current == "anulado" {
            return Err(AppError::PresupuestoEstadoInvalido);
        }

        conn.execute(
            "UPDATE presupuestos SET estado = ?1 WHERE id = ?2",
            params![estado.as_str(), id],
        )?;
        Ok(())
    }
}

fn build_filter_where(filter: &PresupuestoFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(estado) = filter.estado {
        conditions.push("p.estado = ?".to_string());
        params.push(Box::new(estado.as_str().to_string()));
    }

    if let Some(fecha_desde) = &filter.fecha_desde {
        if !fecha_desde.is_empty() {
            conditions.push("p.fecha >= ?".to_string());
            params.push(Box::new(fecha_desde.clone()));
        }
    }

    if let Some(fecha_hasta) = &filter.fecha_hasta {
        if !fecha_hasta.is_empty() {
            conditions.push("p.fecha <= ?".to_string());
            params.push(Box::new(fecha_hasta.clone()));
        }
    }

    if let Some(query) = &filter.query {
        if !query.trim().is_empty() {
            let like = format!("%{}%", query.trim());
            conditions.push(
                "(CAST(p.id AS TEXT) LIKE ?
                 OR c.nombre LIKE ?
                 OR c.apellido LIKE ?
                 OR u.username LIKE ?
                 OR EXISTS (
                     SELECT 1 FROM detalle_presupuestos d
                     INNER JOIN articulos a ON a.id = d.id_articulo
                     WHERE d.id_presupuesto = p.id
                       AND (a.articulo LIKE ? OR a.cod_articulo LIKE ?)
                 ))"
                .to_string(),
            );
            for _ in 0..6 {
                params.push(Box::new(like.clone()));
            }
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    (where_clause, params)
}

impl SqlitePresupuestoRepository {
    fn row_to_presupuesto(&self, row: &rusqlite::Row) -> Result<PresupuestoWithDetalle, AppError> {
        Ok(PresupuestoWithDetalle {
            id: row.get(0)?,
            user_id: row.get(1)?,
            username: row.get(2)?,
            fecha: row.get(3)?,
            subtotal: 0.0,
            descuento: row.get(5)?,
            total: row.get(4)?,
            estado: row.get(6)?,
            fecha_vencimiento: row.get(7)?,
            observacion: row.get(8)?,
            created_at: row.get(9)?,
            cliente_id: row.get(10)?,
            cliente_nombre: row.get(11)?,
            cliente_apellido: row.get(12)?,
            items: Vec::new(),
        })
    }

    fn load_presupuesto(
        &self,
        conn: &rusqlite::Connection,
        id: i64,
    ) -> Result<Option<PresupuestoWithDetalle>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.user_id, COALESCE(u.username, ''), p.fecha, p.total, p.descuento, p.estado, p.fecha_vencimiento, p.observacion, p.created_at, p.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM presupuestos p
             LEFT JOIN users u ON u.id = p.user_id
             LEFT JOIN clientes c ON c.id = p.cliente_id
             WHERE p.id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_presupuesto(row)?))
        } else {
            Ok(None)
        }
    }

    fn load_items(
        &self,
        conn: &rusqlite::Connection,
        presupuesto_id: i64,
    ) -> Result<Vec<PresupuestoDetalleConArticulo>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT d.id, d.id_articulo, COALESCE(a.cod_articulo, ''), COALESCE(a.articulo, ''), d.cantidad, d.costo_unitario, d.precio_unitario, d.subtotal
             FROM detalle_presupuestos d
             LEFT JOIN articulos a ON a.id = d.id_articulo
             WHERE d.id_presupuesto = ?1
             ORDER BY d.id",
        )?;

        let mut rows = stmt.query(params![presupuesto_id])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(PresupuestoDetalleConArticulo {
                id: row.get(0)?,
                id_articulo: row.get(1)?,
                cod_articulo: row.get(2)?,
                articulo: row.get(3)?,
                cantidad: row.get(4)?,
                costo_unitario: row.get(5)?,
                precio_unitario: row.get(6)?,
                subtotal: row.get(7)?,
            });
        }

        Ok(items)
    }

    fn load_items_bulk(
        &self,
        conn: &rusqlite::Connection,
        presupuesto_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<PresupuestoDetalleConArticulo>>, AppError> {
        if presupuesto_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let placeholders: Vec<String> = presupuesto_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT d.id, d.id_presupuesto, d.id_articulo, COALESCE(a.cod_articulo, ''), COALESCE(a.articulo, ''), d.cantidad, d.costo_unitario, d.precio_unitario, d.subtotal
             FROM detalle_presupuestos d
             LEFT JOIN articulos a ON a.id = d.id_articulo
             WHERE d.id_presupuesto IN ({})
             ORDER BY d.id",
            placeholders.join(", ")
        );

        let params_vec: Vec<&dyn rusqlite::ToSql> = presupuesto_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params_vec))?;

        let mut map: std::collections::HashMap<i64, Vec<PresupuestoDetalleConArticulo>> =
            std::collections::HashMap::new();

        while let Some(row) = rows.next()? {
            let id_presupuesto: i64 = row.get(1)?;
            let item = PresupuestoDetalleConArticulo {
                id: row.get(0)?,
                id_articulo: row.get(2)?,
                cod_articulo: row.get(3)?,
                articulo: row.get(4)?,
                cantidad: row.get(5)?,
                costo_unitario: row.get(6)?,
                precio_unitario: row.get(7)?,
                subtotal: row.get(8)?,
            };
            map.entry(id_presupuesto).or_default().push(item);
        }

        Ok(map)
    }
}

fn turno_actual(conn: &rusqlite::Connection) -> Result<(bool, bool), AppError> {
    let (activo, hora_inicio, hora_fin): (i64, String, String) = conn.query_row(
        "SELECT activo, hora_inicio, hora_fin FROM nocturno_config WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let config = NocturnoConfig {
        activo: activo != 0,
        hora_inicio,
        hora_fin,
    };

    match es_nocturno_ahora(&config) {
        Some(es_nocturno) => Ok((config.activo, es_nocturno)),
        None => Ok((false, false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use std::sync::MutexGuard;

    fn fresh_db() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().unwrap();
        reset_test_db().unwrap();
        guard
    }

    fn admin_user_id() -> i64 {
        let conn = DB.lock().unwrap();
        conn.query_row("SELECT id FROM users WHERE username = 'admin'", [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    fn first_articulo_with_stock() -> (i64, f64, f64, f64) {
        let conn = DB.lock().unwrap();
        conn.query_row(
            "SELECT s.id_articulo, s.cantidad, s.costo, s.ganancia FROM stock s ORDER BY s.id LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                ))
            },
        )
        .unwrap()
    }

    fn presupuesto_con_detalle() -> PresupuestoWithDetalle {
        let (id_articulo, _, _, _) = first_articulo_with_stock();

        let presupuesto = Presupuesto::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            0.0,
            None,
            None,
            None,
        );
        let detalle = PresupuestoDetalle::new(id_articulo, 1.0, 0.0, 100.0);
        SqlitePresupuestoRepository::new()
            .create(&presupuesto, &[detalle])
            .unwrap()
    }

    #[test]
    fn create_persists_encabezado_y_detalles() {
        let _guard = fresh_db();
        let (id_articulo, _, _, _) = first_articulo_with_stock();

        let presupuesto = Presupuesto::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            10.0,
            Some("cotización".to_string()),
            Some("2026-02-01".to_string()),
            None,
        );
        let detalle = PresupuestoDetalle::new(id_articulo, 2.0, 0.0, 100.0);

        let created = SqlitePresupuestoRepository::new()
            .create(&presupuesto, &[detalle])
            .unwrap();

        assert_eq!(created.estado, "pendiente");
        assert_eq!(created.descuento, 10.0);
        assert_eq!(created.fecha_vencimiento.as_deref(), Some("2026-02-01"));
        assert_eq!(created.observacion.as_deref(), Some("cotización"));
        assert_eq!(created.cliente_id, None);
        assert_eq!(created.items.len(), 1);
        assert_eq!(created.items[0].id_articulo, id_articulo);
        assert_eq!(created.items[0].precio_unitario, 100.0);
        assert_eq!(created.total, 180.0);

        let found = SqlitePresupuestoRepository::new()
            .find_by_id(created.id)
            .unwrap()
            .unwrap();
        assert_eq!(found.items.len(), 1);
        assert_eq!(found.items[0].subtotal, 200.0);
    }

    #[test]
    fn create_no_decrements_stock() {
        let _guard = fresh_db();
        let (id_articulo, cantidad_antes, _, _) = first_articulo_with_stock();

        let presupuesto = Presupuesto::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            0.0,
            None,
            None,
            None,
        );
        let detalle = PresupuestoDetalle::new(id_articulo, 5.0, 0.0, 100.0);
        SqlitePresupuestoRepository::new()
            .create(&presupuesto, &[detalle])
            .unwrap();

        let conn = DB.lock().unwrap();
        let cantidad_despues: f64 = conn
            .query_row(
                "SELECT cantidad FROM stock WHERE id_articulo = ?1",
                params![id_articulo],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cantidad_despues, cantidad_antes);
    }

    #[test]
    fn create_with_optional_cliente() {
        let _guard = fresh_db();
        let created = presupuesto_con_detalle();
        assert_eq!(created.cliente_id, None);
        assert_eq!(created.cliente_nombre.as_deref(), Some(""));

        let found = SqlitePresupuestoRepository::new()
            .find_by_id(created.id)
            .unwrap()
            .unwrap();
        assert_eq!(found.cliente_id, None);
    }

    #[test]
    fn create_allowed_when_day_closed() {
        let _guard = fresh_db();
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        {
            let conn = DB.lock().unwrap();
            conn.execute(
                "INSERT INTO cierres (fecha, dia, mes, anio, total_costo, total_ganancia, total_venta, created_at) VALUES (?1, 1, 1, 2026, 0, 0, 0, '2026-01-01T00:00:00Z')",
                params![hoy],
            )
            .unwrap();
        }

        let created = presupuesto_con_detalle();
        assert!(created.id > 0);
    }

    #[test]
    fn create_rejects_articulo_sin_stock() {
        let _guard = fresh_db();
        let presupuesto = Presupuesto::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            0.0,
            None,
            None,
            None,
        );
        let detalle = PresupuestoDetalle::new(999999, 1.0, 0.0, 100.0);
        let err = SqlitePresupuestoRepository::new()
            .create(&presupuesto, &[detalle])
            .unwrap_err();
        assert!(matches!(err, AppError::ArticuloWithoutStock));
    }

    #[test]
    fn find_page_returns_items_bulk() {
        let _guard = fresh_db();
        let p1 = presupuesto_con_detalle();
        let p2 = presupuesto_con_detalle();

        let page = SqlitePresupuestoRepository::new()
            .find_page(&PresupuestoFilter::default(), 50, 0)
            .unwrap();
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 2);
        assert!(page.items.iter().any(|p| p.id == p1.id));
        assert!(page.items.iter().any(|p| p.id == p2.id));
        for p in &page.items {
            assert_eq!(p.items.len(), 1);
        }
    }

    #[test]
    fn find_page_filters_by_estado() {
        let _guard = fresh_db();
        presupuesto_con_detalle();
        let p2 = presupuesto_con_detalle();
        SqlitePresupuestoRepository::new()
            .update_estado(p2.id, PresupuestoEstado::Aprobado)
            .unwrap();

        let filter = PresupuestoFilter {
            estado: Some(PresupuestoEstado::Pendiente),
            ..PresupuestoFilter::default()
        };
        let page = SqlitePresupuestoRepository::new()
            .find_page(&filter, 50, 0)
            .unwrap();
        assert_eq!(page.total, 1);
        assert!(page.items.iter().all(|p| p.estado == "pendiente"));

        let filter = PresupuestoFilter {
            estado: Some(PresupuestoEstado::Aprobado),
            ..PresupuestoFilter::default()
        };
        let page = SqlitePresupuestoRepository::new()
            .find_page(&filter, 50, 0)
            .unwrap();
        assert_eq!(page.total, 1);
        assert!(page.items.iter().all(|p| p.estado == "aprobado"));
    }

    #[test]
    fn find_page_filters_by_date_range() {
        let _guard = fresh_db();
        let p1 = presupuesto_con_detalle();
        let p2 = presupuesto_con_detalle();
        {
            let conn = DB.lock().unwrap();
            conn.execute(
                "UPDATE presupuestos SET fecha = '2026-01-01T00:00:00Z' WHERE id = ?1",
                params![p1.id],
            )
            .unwrap();
            conn.execute(
                "UPDATE presupuestos SET fecha = '2026-03-15T00:00:00Z' WHERE id = ?1",
                params![p2.id],
            )
            .unwrap();
        }

        let filter = PresupuestoFilter {
            fecha_desde: Some("2026-02-01T00:00:00Z".to_string()),
            fecha_hasta: Some("2026-12-31T00:00:00Z".to_string()),
            ..PresupuestoFilter::default()
        };
        let page = SqlitePresupuestoRepository::new()
            .find_page(&filter, 50, 0)
            .unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0].id, p2.id);
    }

    #[test]
    fn find_page_filters_by_query_on_articulo() {
        let _guard = fresh_db();
        let (id_articulo, _, _, _) = first_articulo_with_stock();
        let articulo_nombre: String = {
            let conn = DB.lock().unwrap();
            conn.query_row(
                "SELECT articulo FROM articulos WHERE id = ?1",
                params![id_articulo],
                |row| row.get(0),
            )
            .unwrap()
        };

        let presupuesto = Presupuesto::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            0.0,
            None,
            None,
            None,
        );
        let detalle = PresupuestoDetalle::new(id_articulo, 1.0, 0.0, 100.0);
        SqlitePresupuestoRepository::new()
            .create(&presupuesto, &[detalle])
            .unwrap();

        let filter = PresupuestoFilter {
            query: Some(articulo_nombre),
            ..PresupuestoFilter::default()
        };
        let page = SqlitePresupuestoRepository::new()
            .find_page(&filter, 50, 0)
            .unwrap();
        assert_eq!(page.total, 1);
    }

    #[test]
    fn update_estado_allows_from_non_terminal_states() {
        let _guard = fresh_db();
        let p = presupuesto_con_detalle();
        let repo = SqlitePresupuestoRepository::new();

        repo.update_estado(p.id, PresupuestoEstado::Aprobado)
            .unwrap();
        repo.update_estado(p.id, PresupuestoEstado::Anulado)
            .unwrap();

        let estado: String = DB
            .lock()
            .unwrap()
            .query_row(
                "SELECT estado FROM presupuestos WHERE id = ?1",
                params![p.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(estado, "anulado");
    }

    #[test]
    fn update_estado_rejects_terminal_states() {
        let _guard = fresh_db();
        let p = presupuesto_con_detalle();
        let repo = SqlitePresupuestoRepository::new();

        repo.update_estado(p.id, PresupuestoEstado::Convertido)
            .unwrap();
        let err = repo
            .update_estado(p.id, PresupuestoEstado::Anulado)
            .unwrap_err();
        assert!(matches!(err, AppError::PresupuestoEstadoInvalido));

        let p2 = presupuesto_con_detalle();
        repo.update_estado(p2.id, PresupuestoEstado::Anulado)
            .unwrap();
        let err = repo
            .update_estado(p2.id, PresupuestoEstado::Pendiente)
            .unwrap_err();
        assert!(matches!(err, AppError::PresupuestoEstadoInvalido));
    }

    #[test]
    fn update_estado_rejects_missing_presupuesto() {
        let _guard = fresh_db();
        let err = SqlitePresupuestoRepository::new()
            .update_estado(999999, PresupuestoEstado::Anulado)
            .unwrap_err();
        assert!(matches!(err, AppError::PresupuestoNotFound));
    }
}

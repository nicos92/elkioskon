use rusqlite::params;

use crate::domain::entities::{
    es_nocturno_ahora, margen_efectivo, NocturnoConfig, Venta, VentaDetalle,
    VentaDetalleConArticulo, VentaWithDetalle,
};
use crate::domain::repositories::{Page, VentaRepository};
use crate::infrastructure::database::DB;
use crate::infrastructure::error::AppError;

pub struct SqliteVentaRepository;

impl Default for SqliteVentaRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteVentaRepository {
    pub fn new() -> Self {
        Self
    }
}

impl VentaRepository for SqliteVentaRepository {
    fn create(
        &self,
        venta: &Venta,
        detalles: &[VentaDetalle],
        allow_negative_stock: bool,
    ) -> Result<VentaWithDetalle, AppError> {
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;

        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        if Self::is_dia_cerrado(&tx, &hoy)? {
            return Err(AppError::DiaCerrado);
        }

        let mut items: Vec<VentaDetalle> = Vec::new();
        let mut total = 0.0;

        let (config_activo, es_nocturno) = turno_actual(&tx)?;
        let mut margen_registrado = 0.0;

        for detalle in detalles {
            let stock = tx.query_row(
                "SELECT cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna FROM stock WHERE id_articulo = ?1",
                params![detalle.id_articulo],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, f64>(4)?,
                    ))
                },
            );

            let (stock_cantidad, costo, ganancia, ganancia_diurna, ganancia_nocturna) =
                match stock {
                    Ok(values) => values,
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        return Err(AppError::ArticuloWithoutStock);
                    }
                    Err(e) => return Err(e.into()),
                };

            if !allow_negative_stock && detalle.cantidad > stock_cantidad {
                return Err(AppError::InsufficientStock);
            }

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

            if config_activo {
                margen_registrado = margen;
            }

            let mut item = detalle.clone();
            item.costo_unitario = costo;
            item.precio_unitario = precio_unitario;
            item.subtotal = item.cantidad * item.precio_unitario;
            total += item.subtotal;
            items.push(item);
        }

        let now = chrono::Utc::now().to_rfc3339();
        total = (total * (1.0 - venta.descuento / 100.0) * 100.0).round() / 100.0;

        tx.execute(
            "INSERT INTO ventas (user_id, fecha, total, descuento, anulada, observacion, id_tipo_venta, cliente_id, created_at, porcentaje_nocturno) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, ?9)",
            params![
                venta.user_id,
                now,
                total,
                venta.descuento,
                &venta.observacion,
                venta.id_tipo_venta,
                venta.cliente_id,
                now,
                margen_registrado
            ],
        )?;
        let venta_id = tx.last_insert_rowid();

        for item in &items {
            tx.execute(
                "INSERT INTO venta_detalle (id_venta, id_articulo, cantidad, costo_unitario, precio_unitario, subtotal) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    venta_id,
                    item.id_articulo,
                    item.cantidad,
                    item.costo_unitario,
                    item.precio_unitario,
                    item.subtotal
                ],
            )?;

            tx.execute(
                "UPDATE stock SET cantidad = cantidad - ?1, updated_at = ?3 WHERE id_articulo = ?2",
                params![item.cantidad, item.id_articulo, now],
            )?;
        }

        tx.commit()?;

        let mut venta = self
            .load_venta(&conn, venta_id)?
            .ok_or(AppError::VentaNotFound)?;
        venta.items = self.load_items(&conn, venta_id)?;
        venta.subtotal = venta.items.iter().map(|i| i.subtotal).sum();
        Ok(venta)
    }

    fn find_by_id(&self, id: i64) -> Result<Option<VentaWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut venta = self.load_venta(&conn, id)?;
        if let Some(v) = venta.as_mut() {
            v.items = self.load_items(&conn, id)?;
            v.subtotal = v.items.iter().map(|i| i.subtotal).sum();
        }
        Ok(venta)
    }

    fn find_by_cliente(&self, cliente_id: i64) -> Result<Vec<VentaWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT v.id, v.user_id, COALESCE(u.username, ''), v.fecha, v.total, v.descuento, v.anulada, v.observacion, COALESCE(t.nombre, 'Efectivo'), v.created_at, v.porcentaje_nocturno, v.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM ventas v
             LEFT JOIN users u ON u.id = v.user_id
             LEFT JOIN tipos_venta t ON t.id = v.id_tipo_venta
             LEFT JOIN clientes c ON c.id = v.cliente_id
             WHERE v.cliente_id = ?1
             ORDER BY v.id DESC",
        )?;

        let mut rows = stmt.query(params![cliente_id])?;
        let mut ventas = Vec::new();

        while let Some(row) = rows.next()? {
            let mut venta = self.row_to_venta(row)?;
            venta.items = self.load_items(&conn, venta.id)?;
            venta.subtotal = venta.items.iter().map(|i| i.subtotal).sum();
            ventas.push(venta);
        }

        Ok(ventas)
    }

    fn find_all(&self) -> Result<Vec<VentaWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT v.id, v.user_id, COALESCE(u.username, ''), v.fecha, v.total, v.descuento, v.anulada, v.observacion, COALESCE(t.nombre, 'Efectivo'), v.created_at, v.porcentaje_nocturno, v.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM ventas v
             LEFT JOIN users u ON u.id = v.user_id
             LEFT JOIN tipos_venta t ON t.id = v.id_tipo_venta
             LEFT JOIN clientes c ON c.id = v.cliente_id
             ORDER BY v.id DESC",
        )?;

        let mut rows = stmt.query([])?;
        let mut ventas = Vec::new();

        while let Some(row) = rows.next()? {
            let mut venta = self.row_to_venta(row)?;
            venta.items = self.load_items(&conn, venta.id)?;
            venta.subtotal = venta.items.iter().map(|i| i.subtotal).sum();
            ventas.push(venta);
        }

        Ok(ventas)
    }

    fn find_page(&self, limit: i64, offset: i64) -> Result<Page<VentaWithDetalle>, AppError> {
        let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        let limit = limit.max(1);
        let offset = offset.max(0);

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM ventas", [], |row| row.get(0))?;

        let mut stmt = conn.prepare(
            "SELECT v.id, v.user_id, COALESCE(u.username, ''), v.fecha, v.total, v.descuento, v.anulada, v.observacion, COALESCE(t.nombre, 'Efectivo'), v.created_at, v.porcentaje_nocturno, v.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM ventas v
             LEFT JOIN users u ON u.id = v.user_id
             LEFT JOIN tipos_venta t ON t.id = v.id_tipo_venta
             LEFT JOIN clientes c ON c.id = v.cliente_id
             ORDER BY v.id DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let mut rows = stmt.query(params![limit, offset])?;
        let mut ventas = Vec::new();
        let mut ids: Vec<i64> = Vec::new();

        while let Some(row) = rows.next()? {
            let venta = self.row_to_venta(row)?;
            ids.push(venta.id);
            ventas.push(venta);
        }

        if !ids.is_empty() {
            let items = self.load_items_bulk(&conn, &ids)?;
            for venta in ventas.iter_mut() {
                venta.items = items.get(&venta.id).cloned().unwrap_or_default();
                venta.subtotal = venta.items.iter().map(|i| i.subtotal).sum();
            }
        }

        Ok(Page {
            items: ventas,
            total,
            limit,
            offset,
        })
    }

    fn anular(&self, id: i64) -> Result<(), AppError> {
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        let tx = conn.transaction()?;
        let now = chrono::Utc::now().to_rfc3339();

        let (anulada, fecha): (bool, String) = tx
            .query_row(
                "SELECT anulada, fecha FROM ventas WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => AppError::VentaNotFound,
                other => other.into(),
            })?;

        if anulada {
            return Err(AppError::VentaAlreadyAnulada);
        }

        if Self::is_dia_cerrado(&tx, &Self::utc_to_local_date(&fecha)?)? {
            return Err(AppError::DiaCerradoAnulacion);
        }

        {
            let mut stmt =
                tx.prepare("SELECT id_articulo, cantidad FROM venta_detalle WHERE id_venta = ?1")?;
            let mut rows = stmt.query(params![id])?;

            while let Some(row) = rows.next()? {
                let id_articulo: i64 = row.get(0)?;
                let cantidad: f64 = row.get(1)?;

                let updated = tx.execute(
                    "UPDATE stock SET cantidad = cantidad + ?1, updated_at = ?3 WHERE id_articulo = ?2",
                    params![cantidad, id_articulo, now],
                )?;
                if updated == 0 {
                    return Err(AppError::ArticuloWithoutStock);
                }
            }
        }

        tx.execute("UPDATE ventas SET anulada = 1 WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }
}

impl SqliteVentaRepository {
    fn is_dia_cerrado(conn: &rusqlite::Connection, fecha_local: &str) -> Result<bool, AppError> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cierres WHERE fecha = ?1",
            params![fecha_local],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn utc_to_local_date(utc_rfc3339: &str) -> Result<String, AppError> {
        let dt = chrono::DateTime::parse_from_rfc3339(utc_rfc3339)
            .map_err(|e| AppError::Internal(format!("Fecha inválida: {}", e)))?;
        Ok(dt
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string())
    }

    fn row_to_venta(&self, row: &rusqlite::Row) -> Result<VentaWithDetalle, AppError> {
        Ok(VentaWithDetalle {
            id: row.get(0)?,
            user_id: row.get(1)?,
            username: row.get(2)?,
            fecha: row.get(3)?,
            subtotal: 0.0,
            descuento: row.get(5)?,
            total: row.get(4)?,
            anulada: row.get(6)?,
            observacion: row.get(7)?,
            tipo_venta: row.get(8)?,
            created_at: row.get(9)?,
            porcentaje_nocturno: row.get(10)?,
            cliente_id: row.get(11)?,
            cliente_nombre: row.get(12)?,
            cliente_apellido: row.get(13)?,
            items: Vec::new(),
        })
    }

    fn load_venta(
        &self,
        conn: &rusqlite::Connection,
        id: i64,
    ) -> Result<Option<VentaWithDetalle>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT v.id, v.user_id, COALESCE(u.username, ''), v.fecha, v.total, v.descuento, v.anulada, v.observacion, COALESCE(t.nombre, 'Efectivo'), v.created_at, v.porcentaje_nocturno, v.cliente_id, COALESCE(c.nombre, ''), COALESCE(c.apellido, '')
             FROM ventas v
             LEFT JOIN users u ON u.id = v.user_id
             LEFT JOIN tipos_venta t ON t.id = v.id_tipo_venta
             LEFT JOIN clientes c ON c.id = v.cliente_id
             WHERE v.id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_venta(row)?))
        } else {
            Ok(None)
        }
    }

    fn load_items(
        &self,
        conn: &rusqlite::Connection,
        venta_id: i64,
    ) -> Result<Vec<VentaDetalleConArticulo>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT d.id, d.id_articulo, COALESCE(a.cod_articulo, ''), COALESCE(a.articulo, ''), d.cantidad, d.costo_unitario, d.precio_unitario, d.subtotal
             FROM venta_detalle d
             LEFT JOIN articulos a ON a.id = d.id_articulo
             WHERE d.id_venta = ?1
             ORDER BY d.id",
        )?;

        let mut rows = stmt.query(params![venta_id])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(VentaDetalleConArticulo {
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
        venta_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, Vec<VentaDetalleConArticulo>>, AppError> {
        if venta_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let placeholders: Vec<String> = venta_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT d.id, d.id_venta, d.id_articulo, COALESCE(a.cod_articulo, ''), COALESCE(a.articulo, ''), d.cantidad, d.costo_unitario, d.precio_unitario, d.subtotal
             FROM venta_detalle d
             LEFT JOIN articulos a ON a.id = d.id_articulo
             WHERE d.id_venta IN ({})
             ORDER BY d.id",
            placeholders.join(", ")
        );

        let params_vec: Vec<&dyn rusqlite::ToSql> = venta_ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(params_vec))?;

        let mut map: std::collections::HashMap<i64, Vec<VentaDetalleConArticulo>> =
            std::collections::HashMap::new();

        while let Some(row) = rows.next()? {
            let id_venta: i64 = row.get(1)?;
            let item = VentaDetalleConArticulo {
                id: row.get(0)?,
                id_articulo: row.get(2)?,
                cod_articulo: row.get(3)?,
                articulo: row.get(4)?,
                cantidad: row.get(5)?,
                costo_unitario: row.get(6)?,
                precio_unitario: row.get(7)?,
                subtotal: row.get(8)?,
            };
            map.entry(id_venta).or_default().push(item);
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
    use crate::domain::entities::Cliente;
    use crate::domain::repositories::ClienteRepository;
    use crate::infrastructure::database::{reset_test_db, TEST_LOCK};
    use crate::infrastructure::repositories::SqliteClienteRepository;
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

    fn create_cliente(nombre: &str, apellido: &str) -> Cliente {
        SqliteClienteRepository::new()
            .create(&Cliente::new(
                Some(nombre.to_string()),
                Some(apellido.to_string()),
                None,
                None,
                None,
            ))
            .unwrap()
    }

    fn venta_con_detalle(cliente_id: i64) -> VentaWithDetalle {
        let id_articulo: i64 = {
            let conn = DB.lock().unwrap();
            conn.query_row("SELECT id_articulo FROM stock LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap()
        };

        let mut venta = Venta::new(
            admin_user_id(),
            "2026-01-01T00:00:00Z".to_string(),
            0.0,
            None,
        );
        venta.cliente_id = cliente_id;

        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);
        SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap()
    }

    #[test]
    fn create_persists_cliente_id_and_name() {
        let _guard = fresh_db();
        let cliente = create_cliente("Ana", "López");

        let created = venta_con_detalle(cliente.id);
        assert_eq!(created.cliente_id, cliente.id);
        assert_eq!(created.cliente_nombre.as_deref(), Some("Ana"));
        assert_eq!(created.cliente_apellido.as_deref(), Some("López"));

        let found = SqliteVentaRepository::new()
            .find_by_id(created.id)
            .unwrap()
            .unwrap();
        assert_eq!(found.cliente_id, cliente.id);
        assert_eq!(found.cliente_nombre.as_deref(), Some("Ana"));
    }

    #[test]
    fn create_with_default_client_returns_consumidor_final() {
        let _guard = fresh_db();
        let default_id: i64 = {
            let conn = DB.lock().unwrap();
            conn.query_row(
                "SELECT id FROM clientes WHERE nombre = 'Consumidor' AND apellido = 'Final'",
                [],
                |row| row.get(0),
            )
            .unwrap()
        };

        let created = venta_con_detalle(default_id);
        assert_eq!(created.cliente_nombre.as_deref(), Some("Consumidor"));
        assert_eq!(created.cliente_apellido.as_deref(), Some("Final"));
    }

    #[test]
    fn find_by_cliente_returns_only_that_clients_sales() {
        let _guard = fresh_db();
        let cliente_a = create_cliente("Ana", "López");
        let cliente_b = create_cliente("Bruno", "Gómez");

        let v1 = venta_con_detalle(cliente_a.id);
        let _v2 = venta_con_detalle(cliente_b.id);

        let repo = SqliteVentaRepository::new();
        let de_a = repo.find_by_cliente(cliente_a.id).unwrap();
        assert_eq!(de_a.len(), 1);
        assert_eq!(de_a[0].id, v1.id);
        assert_eq!(de_a[0].cliente_nombre.as_deref(), Some("Ana"));

        let de_b = repo.find_by_cliente(cliente_b.id).unwrap();
        assert_eq!(de_b.len(), 1);
        assert_eq!(de_b[0].cliente_nombre.as_deref(), Some("Bruno"));
    }

    fn configurar_nocturno(activo: bool, inicio: &str, fin: &str) {
        let conn = DB.lock().unwrap();
        conn.execute(
            "UPDATE nocturno_config SET activo = ?1, hora_inicio = ?2, hora_fin = ?3 WHERE id = 1",
            params![activo as i64, inicio, fin],
        )
        .unwrap();
    }

    fn preparar_stock(costo: f64, ganancia: f64, diurna: f64, nocturna: f64) -> i64 {
        let id_articulo = id_articulo_primero();
        let conn = DB.lock().unwrap();
        conn.execute(
            "UPDATE stock SET costo = ?1, ganancia = ?2, ganancia_diurna = ?3, ganancia_nocturna = ?4 WHERE id_articulo = ?5",
            params![costo, ganancia, diurna, nocturna, id_articulo],
        )
        .unwrap();
        id_articulo
    }

    #[test]
    fn create_aplica_margen_nocturno_cuando_activo_y_en_rango() {
        let _guard = fresh_db();
        configurar_nocturno(true, "00:00", "23:59");
        let id_articulo = preparar_stock(100.0, 20.0, 10.0, 30.0);
        let cliente = create_cliente("Ana", "López");

        let mut venta = Venta::new(admin_user_id(), "2026-01-01T00:00:00Z".to_string(), 0.0, None);
        venta.cliente_id = cliente.id;
        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);

        let created = SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap();

        assert_eq!(created.porcentaje_nocturno, 30.0);
        assert_eq!(created.total, 130.0);
    }

    #[test]
    fn create_aplica_margen_diurno_fuera_del_rango_nocturno() {
        let _guard = fresh_db();
        configurar_nocturno(true, "10:00", "10:00");
        let id_articulo = preparar_stock(100.0, 20.0, 10.0, 30.0);
        let cliente = create_cliente("Ana", "López");

        let mut venta = Venta::new(admin_user_id(), "2026-01-01T00:00:00Z".to_string(), 0.0, None);
        venta.cliente_id = cliente.id;
        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);

        let created = SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap();

        assert_eq!(created.porcentaje_nocturno, 10.0);
        assert_eq!(created.total, 110.0);
    }

    #[test]
    fn create_usa_precio_cliente_cuando_config_inactiva() {
        let _guard = fresh_db();
        configurar_nocturno(false, "00:00", "23:59");
        let id_articulo = preparar_stock(100.0, 20.0, 10.0, 30.0);
        let cliente = create_cliente("Ana", "López");

        let mut venta = Venta::new(admin_user_id(), "2026-01-01T00:00:00Z".to_string(), 0.0, None);
        venta.cliente_id = cliente.id;
        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);

        let created = SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap();

        assert_eq!(created.porcentaje_nocturno, 0.0);
        assert_eq!(created.total, 100.0);
    }

    #[test]
    fn create_aplica_diurno_cuando_rango_inicio_fin_iguales() {
        let _guard = fresh_db();
        configurar_nocturno(true, "10:00", "10:00");
        let id_articulo = preparar_stock(100.0, 20.0, 10.0, 0.0);
        let cliente = create_cliente("Ana", "López");

        let mut venta = Venta::new(admin_user_id(), "2026-01-01T00:00:00Z".to_string(), 0.0, None);
        venta.cliente_id = cliente.id;
        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);

        let created = SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap();

        assert_eq!(created.total, 110.0);
    }

    #[test]
    fn create_aplica_margen_sobre_costo_con_descuento() {
        let _guard = fresh_db();
        configurar_nocturno(true, "00:00", "23:59");
        let id_articulo = preparar_stock(100.0, 20.0, 10.0, 30.0);
        let cliente = create_cliente("Ana", "López");

        let mut venta = Venta::new(admin_user_id(), "2026-01-01T00:00:00Z".to_string(), 10.0, None);
        venta.cliente_id = cliente.id;
        let detalle = VentaDetalle::new(id_articulo, 1.0, 0.0, 100.0);

        let created = SqliteVentaRepository::new()
            .create(&venta, &[detalle], false)
            .unwrap();

        assert_eq!(created.porcentaje_nocturno, 30.0);
        assert_eq!(created.total, 117.0);
    }

    fn id_articulo_primero() -> i64 {
        let conn = DB.lock().unwrap();
        conn.query_row("SELECT id_articulo FROM stock LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap()
    }
}

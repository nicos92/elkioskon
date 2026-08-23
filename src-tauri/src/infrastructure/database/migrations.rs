use rusqlite::Connection;

pub(crate) fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    column_ddl: &str,
) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(());
        }
    }

    conn.execute_batch(&format!(
        "ALTER TABLE {} ADD COLUMN {} {};",
        table, column, column_ddl
    ))
}

pub(crate) fn backfill_stock_updated_at(conn: &Connection) -> Result<(), rusqlite::Error> {
    ensure_column(conn, "stock", "updated_at", "TEXT")?;
    let stock_count: i64 = conn.query_row("SELECT COUNT(*) FROM stock", [], |row| row.get(0))?;
    if stock_count > 0 {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE stock SET updated_at = ?1 WHERE updated_at IS NULL",
            rusqlite::params![now],
        )?;
    }

    Ok(())
}

pub(crate) fn run_column_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    ensure_column(conn, "ventas", "descuento", "REAL NOT NULL DEFAULT 0")?;
    ensure_column(
        conn,
        "ventas",
        "id_tipo_venta",
        "INTEGER REFERENCES tipos_venta(id)",
    )?;
    ensure_column(
        conn,
        "ventas",
        "cliente_id",
        "INTEGER REFERENCES clientes(id)",
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ventas_cliente_id ON ventas(cliente_id)",
        [],
    )?;

    Ok(())
}

pub(crate) fn backfill_ventas_tipo_venta(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE ventas SET id_tipo_venta = (SELECT id FROM tipos_venta WHERE nombre = 'Efectivo') WHERE id_tipo_venta IS NULL",
        [],
    )?;

    Ok(())
}

pub(crate) fn backfill_ventas_cliente(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE ventas
         SET cliente_id = (SELECT id FROM clientes WHERE nombre = 'Consumidor' AND apellido = 'Final' LIMIT 1)
         WHERE cliente_id IS NULL",
        [],
    )?;

    Ok(())
}

pub(crate) fn migrate_presupuestos_estado(conn: &Connection) -> Result<(), rusqlite::Error> {
    let table_sql: Option<String> = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'presupuestos'",
        [],
        |row| row.get(0),
    )?;

    let table_sql = match table_sql {
        Some(sql) if sql.contains("anulado") => return Ok(()),
        Some(sql) => sql,
        None => return Ok(()),
    };

    if !table_sql.to_lowercase().contains("check") {
        return Ok(());
    }

    conn.execute_batch(
        "PRAGMA foreign_keys = OFF;
         DROP TABLE IF EXISTS presupuestos_new;

         CREATE TABLE presupuestos_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            fecha TEXT NOT NULL,
            total REAL NOT NULL,
            descuento REAL NOT NULL DEFAULT 0,
            estado TEXT NOT NULL DEFAULT 'pendiente'
                CHECK (estado IN ('pendiente','aprobado','vencido','convertido','anulado')),
            fecha_vencimiento TEXT,
            observacion TEXT,
            cliente_id INTEGER REFERENCES clientes(id),
            created_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
         );

         INSERT INTO presupuestos_new (id, user_id, fecha, total, descuento, estado, fecha_vencimiento, observacion, cliente_id, created_at)
            SELECT id, user_id, fecha, total, descuento, estado, fecha_vencimiento, observacion, cliente_id, created_at FROM presupuestos;

         DROP TABLE presupuestos;
         ALTER TABLE presupuestos_new RENAME TO presupuestos;

         CREATE INDEX IF NOT EXISTS idx_detalle_presupuestos_id_presupuesto ON detalle_presupuestos(id_presupuesto);
         CREATE INDEX IF NOT EXISTS idx_presupuestos_estado ON presupuestos(estado);

         PRAGMA foreign_keys = ON;",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::DB;

    #[test]
    fn migrate_presupuestos_estado_adds_anulado_and_preserves_data() {
        let _guard = crate::infrastructure::database::TEST_LOCK.lock().unwrap();
        crate::infrastructure::database::reset_test_db().unwrap();
        let conn = DB.lock().unwrap();

        conn.execute_batch(
            "PRAGMA foreign_keys = OFF;
             DROP TABLE IF EXISTS detalle_presupuestos;
             DROP TABLE IF EXISTS presupuestos;

             CREATE TABLE presupuestos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                fecha TEXT NOT NULL,
                total REAL NOT NULL,
                descuento REAL NOT NULL DEFAULT 0,
                estado TEXT NOT NULL DEFAULT 'pendiente'
                    CHECK (estado IN ('pendiente','aprobado','vencido','convertido')),
                fecha_vencimiento TEXT,
                observacion TEXT,
                cliente_id INTEGER REFERENCES clientes(id),
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
             );

             CREATE TABLE detalle_presupuestos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                id_presupuesto INTEGER NOT NULL,
                id_articulo INTEGER NOT NULL,
                cantidad REAL NOT NULL,
                costo_unitario REAL NOT NULL,
                precio_unitario REAL NOT NULL,
                subtotal REAL NOT NULL,
                FOREIGN KEY (id_presupuesto) REFERENCES presupuestos(id) ON DELETE CASCADE,
                FOREIGN KEY (id_articulo) REFERENCES articulos(id)
             );

             PRAGMA foreign_keys = ON;",
        )
        .unwrap();

        conn.execute(
            "INSERT INTO presupuestos (user_id, fecha, total, descuento, estado, created_at)
             VALUES (1, '2026-01-01T00:00:00Z', 100.0, 0.0, 'pendiente', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let presupuesto_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO detalle_presupuestos (id_presupuesto, id_articulo, cantidad, costo_unitario, precio_unitario, subtotal)
             VALUES (?1, (SELECT id FROM articulos LIMIT 1), 1, 0, 100, 100)",
            rusqlite::params![presupuesto_id],
        )
        .unwrap();

        migrate_presupuestos_estado(&conn).unwrap();

        let table_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'presupuestos'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_sql.contains("anulado"));

        let estado: String = conn
            .query_row(
                "SELECT estado FROM presupuestos WHERE id = ?1",
                rusqlite::params![presupuesto_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(estado, "pendiente");

        let detalle_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM detalle_presupuestos WHERE id_presupuesto = ?1",
                rusqlite::params![presupuesto_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(detalle_count, 1);

        let check_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM presupuestos p
                 INNER JOIN detalle_presupuestos d ON d.id_presupuesto = p.id
                 WHERE p.id = ?1",
                rusqlite::params![presupuesto_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(check_count, 1);
    }

    #[test]
    fn migrate_presupuestos_estado_is_idempotent() {
        let _guard = crate::infrastructure::database::TEST_LOCK.lock().unwrap();
        crate::infrastructure::database::reset_test_db().unwrap();
        let conn = DB.lock().unwrap();

        let count_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM presupuestos", [], |row| row.get(0))
            .unwrap();
        migrate_presupuestos_estado(&conn).unwrap();
        let count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM presupuestos", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count_before, count_after);
    }
}

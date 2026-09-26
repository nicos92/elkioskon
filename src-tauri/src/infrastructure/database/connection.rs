use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::sync::Mutex;
#[cfg(test)]
use std::sync::{MutexGuard, PoisonError};

use super::config::get_db_path;
use super::maintenance;
use super::migrations;
use super::schema;
use super::seeds;

pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = init_database().expect("Failed to initialize database");
    Mutex::new(conn)
});

pub fn init_database() -> Result<Connection, rusqlite::Error> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;
    initialize(&conn)?;
    Ok(conn)
}

fn initialize(conn: &Connection) -> Result<(), rusqlite::Error> {
    schema::apply_schema(conn)?;

    migrations::backfill_stock_updated_at(conn)?;
    migrations::run_column_migrations(conn)?;

    seeds::seed_tipos_venta(conn)?;
    migrations::backfill_ventas_tipo_venta(conn)?;

    seeds::seed_permissions(conn)?;
    seeds::seed_admin_user(conn)?;
    seeds::seed_cliente_defecto(conn)?;
    seeds::seed_proveedor_defecto(conn)?;
    seeds::seed_nocturno_config(conn)?;
    migrations::backfill_ventas_cliente(conn)?;

    seeds::seed_demo_data(conn)?;

    maintenance::purge_old_audit_logs(conn)?;

    migrations::migrate_presupuestos_estado(conn)?;

    Ok(())
}

/// Serializes the tests that share the in-memory database.
///
/// Poisoning is deliberately ignored: this lock only guards test isolation,
/// not an invariant, and `fresh_test_db` rebuilds the schema right after
/// acquiring it. Recovering keeps a single failing test from cascading into
/// every other DB test with a `PoisonError` that hides the real failure.
#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Serializes a DB test and rebuilds the schema before it runs.
///
/// Hold the returned guard for the whole test to keep the database private.
#[cfg(test)]
pub fn fresh_test_db() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    reset_test_db().expect("failed to rebuild the test database");
    guard
}

#[cfg(test)]
pub fn reset_test_db() -> Result<(), rusqlite::Error> {
    let conn = DB.lock().unwrap_or_else(PoisonError::into_inner);
    drop_all_tables(&conn)?;
    initialize(&conn)?;
    Ok(())
}

#[cfg(test)]
fn drop_all_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    let mut sql = String::from("PRAGMA foreign_keys = OFF;\n");
    for table in schema::TABLES {
        sql.push_str(&format!("DROP TABLE IF EXISTS {};\n", table));
    }
    conn.execute_batch(&sql)
}

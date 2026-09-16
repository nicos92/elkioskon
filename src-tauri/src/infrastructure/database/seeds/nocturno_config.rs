use rusqlite::Connection;

pub(crate) fn seed_nocturno_config(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO nocturno_config (id, activo, porcentaje, hora_inicio, hora_fin)
         VALUES (1, 0, 0, '22:00', '06:00')",
        [],
    )?;

    Ok(())
}
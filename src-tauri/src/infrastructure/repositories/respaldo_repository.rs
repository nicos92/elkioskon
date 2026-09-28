use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OpenFlags, TransactionBehavior};

use crate::domain::entities::{RespaldoInfo, RespaldoResult, RestauracionResult};
use crate::domain::repositories::RespaldoRepository;
use crate::infrastructure::database::{get_db_path, initialize, DB, TABLES};
use crate::infrastructure::error::AppError;

pub struct SqliteRespaldoRepository;

impl Default for SqliteRespaldoRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteRespaldoRepository {
    pub fn new() -> Self {
        Self
    }
}

impl RespaldoRepository for SqliteRespaldoRepository {
    fn crear_respaldo(&self, destino: &Path) -> Result<RespaldoResult, AppError> {
        let destino_str = destino.to_string_lossy().to_string();
        if destino_str.trim().is_empty() {
            return Err(AppError::RespaldoPathInvalido);
        }

        let padre = match destino.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => Path::new(".").to_path_buf(),
        };
        std::fs::create_dir_all(&padre)
            .map_err(|e| AppError::RespaldoError(format!("{} ({})", padre.display(), e)))?;
        if !padre.is_dir() {
            return Err(AppError::RespaldoPathInvalido);
        }

        if destino.exists() {
            std::fs::remove_file(destino)
                .map_err(|e| AppError::RespaldoError(format!("{} ({})", destino.display(), e)))?;
        }

        let tamano_bytes = {
            let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;
            conn.execute("VACUUM INTO ?1", params![destino_str])?;
            tamano_de_archivo(destino)?
        };

        Ok(RespaldoResult {
            ruta: destino_str,
            tamano_bytes,
            generado_en: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn info(&self) -> Result<RespaldoInfo, AppError> {
        let ruta = get_db_path();
        Ok(RespaldoInfo {
            ruta_base_datos: ruta.to_string_lossy().to_string(),
            tamano_base_datos_bytes: tamano_de_archivo(&ruta).unwrap_or(0),
        })
    }

    /// Replaces the live data with the contents of a backup.
    ///
    /// The data travels through the already-open connection instead of the
    /// file: `DB` holds the only handle on the database, so replacing the file
    /// on disk would either be a no-op (the handle keeps writing to the old
    /// inode on Unix) or be rejected outright (SQLite holds the file on
    /// Windows). The backup is attached, its schema is recreated in `main`
    /// and its rows are copied over, all inside one transaction.
    fn restaurar(&self, origen: &Path) -> Result<RestauracionResult, AppError> {
        let esquema = leer_esquema_del_origen(origen)?;
        let mut conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

        // The safety copy is taken while the old data is still in place, so a
        // restore of the wrong file can be undone by hand. It goes next to the
        // backup that was chosen, which is where the user is looking.
        let ruta_respaldo_previo =
            copiar_datos_actuales(&conn, &get_db_path(), &carpeta_de(origen))?;

        let (tablas, registros) = intercambiar_datos(&mut conn, origen, &esquema)?;
        // Bring a backup taken by an older build up to the current schema.
        initialize(&conn).map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

        Ok(RestauracionResult {
            ruta_origen: origen.to_string_lossy().to_string(),
            tablas_restauradas: tablas,
            registros_restaurados: registros,
            ruta_respaldo_previo,
            restaurado_en: chrono::Utc::now().to_rfc3339(),
        })
    }
}

/// The schema of a candidate backup: which tables it has, and the DDL needed
/// to rebuild them.
struct EsquemaOrigen {
    tablas: Vec<String>,
    ddl: Vec<String>,
}

/// Opens the backup read-only and refuses anything that is not a complete,
/// uncorrupted copy of the app database.
///
/// Runs before the live data is touched so a bad selection leaves the current
/// data exactly as it was.
fn leer_esquema_del_origen(origen: &Path) -> Result<EsquemaOrigen, AppError> {
    if !origen.is_file() {
        return Err(AppError::RespaldoInvalido(format!(
            "no se encontró el archivo {}",
            origen.display()
        )));
    }

    let conn =
        Connection::open_with_flags(origen, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
            AppError::RespaldoInvalido(format!(
                "{} no es una base de datos ({})",
                origen.display(),
                e
            ))
        })?;

    let integridad: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| {
            AppError::RespaldoInvalido(format!("{} no se pudo leer ({})", origen.display(), e))
        })?;
    if integridad != "ok" {
        return Err(AppError::RespaldoInvalido(format!(
            "el archivo está dañado ({})",
            integridad
        )));
    }

    // Objects are ordered so that every table exists before the indexes,
    // views and triggers that point at it.
    let mut stmt = conn
        .prepare(
            "SELECT type, sql FROM sqlite_master
             WHERE type IN ('table', 'index', 'view', 'trigger')
               AND sql IS NOT NULL
               AND name NOT LIKE 'sqlite_%'
             ORDER BY CASE type
                        WHEN 'table' THEN 0
                        WHEN 'index' THEN 1
                        WHEN 'view' THEN 2
                        ELSE 3
                      END,
                      name",
        )
        .map_err(|e| {
            AppError::RespaldoInvalido(format!("{} no se pudo leer ({})", origen.display(), e))
        })?;

    let filas = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| {
            AppError::RespaldoInvalido(format!("{} no se pudo leer ({})", origen.display(), e))
        })?;

    let mut tablas = Vec::new();
    let mut ddl = Vec::new();
    for fila in filas {
        let (tipo, sql) = fila.map_err(|e| {
            AppError::RespaldoInvalido(format!("{} no se pudo leer ({})", origen.display(), e))
        })?;
        if tipo == "table" {
            tablas.push(objeto_nombre(&sql)?);
        }
        ddl.push(sql);
    }

    let faltantes: Vec<&str> = TABLES
        .iter()
        .copied()
        .filter(|esperada| !tablas.iter().any(|nombre| nombre == esperada))
        .collect();
    if !faltantes.is_empty() {
        return Err(AppError::RespaldoInvalido(format!(
            "faltan las tablas {}",
            faltantes.join(", ")
        )));
    }

    Ok(EsquemaOrigen { tablas, ddl })
}

/// Recovers the object name from its `CREATE TABLE` statement, so the name is
/// available from the DDL that has to be replayed anyway.
///
/// The name is interpolated into `DROP`/`INSERT` statements, so anything that
/// is not a plain identifier is refused instead of quoted into something
/// unexpected. `IF NOT EXISTS` is skipped because `SCHEMA_SQL` uses it.
fn objeto_nombre(ddl: &str) -> Result<String, AppError> {
    let palabras: Vec<&str> = ddl.split_whitespace().collect();
    let indice = palabras
        .iter()
        .position(|palabra| palabra.eq_ignore_ascii_case("table"))
        .ok_or_else(|| {
            AppError::RespaldoInvalido(format!("no se pudo leer la definición: {}", ddl))
        })?;

    let nombre = palabras
        .iter()
        .skip(indice + 1)
        .find(|palabra| {
            !["if", "not", "exists"]
                .iter()
                .any(|reservado| palabra.eq_ignore_ascii_case(reservado))
        })
        .map(|palabra| palabra.trim_matches(|c: char| c == '"' || c == '`' || c == '[' || c == ']'))
        // A schema qualifier is not part of the name; the table always lives
        // in `main` once it is attached.
        .map(|palabra| match palabra.rsplit_once('.') {
            Some((_, nombre)) => nombre,
            None => palabra,
        })
        .filter(|nombre| {
            !nombre.is_empty()
                && nombre
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .ok_or_else(|| {
            AppError::RespaldoInvalido(format!("nombre de tabla inesperado: {}", ddl))
        })?;

    Ok(nombre.to_string())
}

/// Copies the data that is about to be replaced, so the restore can be undone
/// by hand.
///
/// The copy lands in `carpeta`, the folder the chosen backup lives in, because
/// that is where the person doing the restore is looking. That folder is picked
/// by the user, though, so it may not be writable (a read-only stick, a DVD, a
/// network mount, no space left). When the copy cannot go there it falls back to
/// the live database's own folder, which the app can always write to, and a
/// restore is never blocked by a folder it did not create.
///
/// Returns `None` when the live database is not file backed, in which case
/// there is nothing to copy and no file to leave behind.
fn copiar_datos_actuales(
    conn: &Connection,
    ruta_actual: &Path,
    carpeta: &Path,
) -> Result<Option<String>, AppError> {
    if ruta_actual.as_os_str() == ":memory:" {
        return Ok(None);
    }

    let carpeta_base = carpeta_de(ruta_actual);
    if carpeta == carpeta_base {
        return intentar_copiar(conn, ruta_actual, &carpeta_base);
    }

    // The first error is dropped on purpose: once the copy lands in the live
    // folder the reason the chosen one failed no longer matters.
    match intentar_copiar(conn, ruta_actual, carpeta) {
        Ok(ruta) => Ok(ruta),
        Err(_) => intentar_copiar(conn, ruta_actual, &carpeta_base),
    }
}

fn intentar_copiar(
    conn: &Connection,
    ruta_actual: &Path,
    carpeta: &Path,
) -> Result<Option<String>, AppError> {
    let destino = ruta_del_respaldo_previo(ruta_actual, carpeta);
    if destino.exists() {
        std::fs::remove_file(&destino).map_err(|e| {
            AppError::RestaurarRespaldoError(format!("{} ({})", destino.display(), e))
        })?;
    }
    conn.execute("VACUUM INTO ?1", params![destino.to_string_lossy()])
        .map_err(|e| AppError::RestaurarRespaldoError(format!("{} ({})", destino.display(), e)))?;

    Ok(Some(destino.to_string_lossy().to_string()))
}

fn carpeta_de(ruta: &Path) -> PathBuf {
    match ruta.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// The name comes from the live database, so the copy reads as "a copy of the
/// app data" and does not get mixed up with the dated backups sitting next to
/// it.
fn ruta_del_respaldo_previo(ruta_actual: &Path, carpeta: &Path) -> PathBuf {
    let sello = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let nombre = ruta_actual
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("respaldo");
    let extension = ruta_actual
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("db");

    carpeta.join(format!(
        "{}.pre-restauracion-{}.{}",
        nombre, sello, extension
    ))
}

/// Rebuilds `main` from the attached backup. Returns the number of tables
/// copied and the total number of rows.
fn intercambiar_datos(
    conn: &mut Connection,
    origen: &Path,
    esquema: &EsquemaOrigen,
) -> Result<(usize, u64), AppError> {
    // Foreign keys are off for the duration: the backup rows are inserted in
    // whatever order the copy produced, and the constraint check would reject
    // the intermediate states. `PRAGMA foreign_keys` is a no-op inside a
    // transaction, so it is toggled around it.
    let claves_previas: bool = conn
        .pragma_query_value(None, "foreign_keys", |row| row.get(0))
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;
    conn.pragma_update(None, "foreign_keys", "OFF")
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

    let resultado = copiar_tablas(conn, origen, esquema);

    conn.pragma_update(
        None,
        "foreign_keys",
        if claves_previas { "ON" } else { "OFF" },
    )
    .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

    resultado
}

fn copiar_tablas(
    conn: &mut Connection,
    origen: &Path,
    esquema: &EsquemaOrigen,
) -> Result<(usize, u64), AppError> {
    conn.execute(
        "ATTACH DATABASE ?1 AS orig",
        params![origen.to_string_lossy()],
    )
    .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

    let copia = copiar_en_transaccion(conn, esquema);

    // `DETACH` cannot run while a transaction is open, and the transaction is
    // already resolved by now, so this always gets a chance to clean up. The
    // swap error wins: a failed `DETACH` leaves the connection dirty but says
    // nothing about whether the data made it in.
    let detach = conn
        .execute("DETACH DATABASE orig", [])
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()));

    match (copia, detach) {
        (Ok(copiado), _) => Ok(copiado),
        (Err(error), _) => Err(error),
    }
}

fn copiar_en_transaccion(
    conn: &mut Connection,
    esquema: &EsquemaOrigen,
) -> Result<(usize, u64), AppError> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

    // Every main table goes, not just the ones the backup has, so a table
    // dropped from the schema in a newer build does not survive the restore.
    let mut stmt = tx
        .prepare(
            "SELECT name FROM main.sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;
    let tablas_viejas: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?
        .collect::<Result<_, _>>()
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;
    drop(stmt);

    for tabla in &tablas_viejas {
        tx.execute(&format!("DROP TABLE IF EXISTS main.\"{}\"", tabla), [])
            .map_err(|e| {
                AppError::RestaurarRespaldoError(format!("al borrar {} ({})", tabla, e))
            })?;
    }

    for sentencia in &esquema.ddl {
        tx.execute_batch(sentencia)
            .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;
    }

    let mut registros = 0u64;
    for tabla in &esquema.tablas {
        let insertados = tx
            .execute(
                &format!("INSERT INTO main.\"{0}\" SELECT * FROM orig.\"{0}\"", tabla),
                [],
            )
            .map_err(|e| {
                AppError::RestaurarRespaldoError(format!("al copiar {} ({})", tabla, e))
            })?;
        registros += insertados as u64;
    }

    tx.commit()
        .map_err(|e| AppError::RestaurarRespaldoError(e.to_string()))?;

    Ok((esquema.tablas.len(), registros))
}

fn tamano_de_archivo(ruta: &Path) -> Result<u64, AppError> {
    std::fs::metadata(ruta)
        .map(|m| m.len())
        .map_err(|e| AppError::RespaldoError(format!("{} ({})", ruta.display(), e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::fresh_test_db;

    fn repo() -> SqliteRespaldoRepository {
        SqliteRespaldoRepository::new()
    }

    #[test]
    fn crear_respaldo_genera_un_sqlite_valido() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");

        let resultado = repo().crear_respaldo(&destino).unwrap();

        assert_eq!(resultado.ruta, destino.to_string_lossy().to_string());
        assert!(destino.exists());
        assert!(resultado.tamano_bytes > 0);
        assert!(!resultado.generado_en.is_empty());
    }

    #[test]
    fn crear_respaldo_conserva_los_datos_de_la_base() {
        let _guard = fresh_test_db();
        {
            let conn = DB.lock().unwrap();
            conn.execute("INSERT INTO categorias (categoria) VALUES ('Bebidas')", [])
                .unwrap();
        }

        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");
        repo().crear_respaldo(&destino).unwrap();

        let copia = rusqlite::Connection::open(&destino).unwrap();
        let total: i64 = copia
            .query_row(
                "SELECT COUNT(*) FROM categorias WHERE categoria = 'Bebidas'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 1);
    }

    #[test]
    fn crear_respaldo_sobrescribe_un_archivo_previo() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let destino = dir.path().join("respaldo.db");
        std::fs::write(&destino, b"contenido viejo").unwrap();

        let resultado = repo().crear_respaldo(&destino).unwrap();

        assert!(resultado.tamano_bytes > 0);
        let copia = rusqlite::Connection::open(&destino).unwrap();
        let integridad: String = copia
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
    }

    #[test]
    fn crear_respaldo_rechaza_ruta_vacia() {
        let _guard = fresh_test_db();
        assert!(matches!(
            repo().crear_respaldo(Path::new("")),
            Err(AppError::RespaldoPathInvalido)
        ));
    }

    #[test]
    fn info_expone_la_ruta_de_la_base_datos() {
        let _guard = fresh_test_db();
        let info = repo().info().unwrap();
        assert_eq!(
            info.ruta_base_datos,
            get_db_path().to_string_lossy().to_string()
        );
    }

    fn cantidad(conn: &Connection, condicion: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {}", condicion), [], |row| {
            row.get(0)
        })
        .unwrap()
    }

    /// Takes a backup of the current data and hands back the path.
    fn respaldo_de_ahora(dir: &Path) -> PathBuf {
        let ruta = dir.join("respaldo.db");
        repo().crear_respaldo(&ruta).unwrap();
        ruta
    }

    #[test]
    fn restaurar_devuelve_los_datos_de_la_copia() {
        let _guard = fresh_test_db();
        {
            let conn = DB.lock().unwrap();
            conn.execute("INSERT INTO categorias (categoria) VALUES ('Bebidas')", [])
                .unwrap();
        }

        let dir = tempfile::tempdir().unwrap();
        let respaldo = respaldo_de_ahora(dir.path());

        // The live database moves on after the backup was taken.
        {
            let conn = DB.lock().unwrap();
            conn.execute("INSERT INTO categorias (categoria) VALUES ('Snacks')", [])
                .unwrap();
        }

        let resultado = repo().restaurar(&respaldo).unwrap();

        let conn = DB.lock().unwrap();
        assert_eq!(cantidad(&conn, "categorias WHERE categoria = 'Bebidas'"), 1);
        assert_eq!(cantidad(&conn, "categorias WHERE categoria = 'Snacks'"), 0);
        assert_eq!(resultado.ruta_origen, respaldo.to_string_lossy());
        assert_eq!(resultado.tablas_restauradas, TABLES.len());
        assert!(resultado.registros_restaurados > 0);
    }

    #[test]
    fn restaurar_deja_el_esquema_completo_e_integro() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let respaldo = respaldo_de_ahora(dir.path());

        repo().restaurar(&respaldo).unwrap();

        let conn = DB.lock().unwrap();
        for tabla in TABLES {
            let existe: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    params![tabla],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(existe, 1, "falta la tabla {}", tabla);
        }
        let integridad: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
    }

    #[test]
    fn restaurar_devuelve_las_claves_foraneas_a_su_estado_anterior() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let respaldo = respaldo_de_ahora(dir.path());
        let antes: bool = DB
            .lock()
            .unwrap()
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();

        repo().restaurar(&respaldo).unwrap();

        let despues: bool = DB
            .lock()
            .unwrap()
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(despues, antes);
    }

    #[test]
    fn restaurar_borra_las_tablas_que_la_copia_no_trae() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let respaldo = respaldo_de_ahora(dir.path());
        {
            // A table the backup cannot have: it was created after the copy.
            let conn = DB.lock().unwrap();
            conn.execute_batch("CREATE TABLE tabla_obsoleta (id INTEGER PRIMARY KEY)")
                .unwrap();
        }

        repo().restaurar(&respaldo).unwrap();

        let conn = DB.lock().unwrap();
        assert_eq!(
            cantidad(&conn, "sqlite_master WHERE name = 'tabla_obsoleta'"),
            0
        );
    }

    #[test]
    fn restaurar_no_deja_archivos_si_la_base_no_esta_en_un_archivo() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let respaldo = respaldo_de_ahora(dir.path());

        let resultado = repo().restaurar(&respaldo).unwrap();

        // The test database is in memory, so there is no previous copy to
        // leave next to it.
        assert_eq!(resultado.ruta_respaldo_previo, None);
    }

    #[test]
    fn restaurar_rechaza_un_archivo_inexistente() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            repo().restaurar(&dir.path().join("no-existe.db")),
            Err(AppError::RespaldoInvalido(_))
        ));
    }

    #[test]
    fn restaurar_rechaza_un_archivo_que_no_es_sqlite() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("falso.db");
        std::fs::write(&ruta, b"esto no es una base de datos").unwrap();

        assert!(matches!(
            repo().restaurar(&ruta),
            Err(AppError::RespaldoInvalido(_))
        ));
    }

    #[test]
    fn restaurar_rechaza_un_sqlite_sin_el_esquema_de_la_app() {
        let _guard = fresh_test_db();
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("vacia.db");
        Connection::open(&ruta)
            .unwrap()
            .execute_batch("CREATE TABLE otra (id INTEGER PRIMARY KEY)")
            .unwrap();

        assert!(matches!(
            repo().restaurar(&ruta),
            Err(AppError::RespaldoInvalido(detalle)) if detalle.contains("categorias")
        ));
    }

    #[test]
    fn una_copia_invalida_no_altera_los_datos_actuales() {
        let _guard = fresh_test_db();
        {
            let conn = DB.lock().unwrap();
            conn.execute("INSERT INTO categorias (categoria) VALUES ('Bebidas')", [])
                .unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("falso.db");
        std::fs::write(&ruta, b"esto no es una base de datos").unwrap();

        assert!(matches!(
            repo().restaurar(&ruta),
            Err(AppError::RespaldoInvalido(_))
        ));

        let conn = DB.lock().unwrap();
        assert_eq!(cantidad(&conn, "categorias"), 1);
    }

    #[test]
    fn ruta_del_respaldo_previo_usa_la_carpeta_pedida_y_el_nombre_de_la_base() {
        let ruta = ruta_del_respaldo_previo(
            Path::new("/datos/el-kioskon-app.db"),
            Path::new("/mi-pendrive"),
        );

        assert_eq!(ruta.parent().unwrap(), Path::new("/mi-pendrive"));
        let nombre = ruta.file_name().unwrap().to_str().unwrap();
        assert!(nombre.starts_with("el-kioskon-app.pre-restauracion-"));
        assert!(nombre.ends_with(".db"));
    }

    /// Builds a real file-backed database seeded like the app one.
    fn base_en_un_archivo(ruta: &Path) -> Connection {
        let conn = Connection::open(ruta).unwrap();
        initialize(&conn).unwrap();
        conn.execute("INSERT INTO categorias (categoria) VALUES ('Bebidas')", [])
            .unwrap();
        conn
    }

    #[test]
    fn copiar_datos_actuales_deja_una_copia_de_lo_que_se_pisa() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("el-kioskon-app.db");
        base_en_un_archivo(&ruta);

        let previa = {
            let conn = Connection::open(&ruta).unwrap();
            copiar_datos_actuales(&conn, &ruta, dir.path()).unwrap()
        };

        let previa = previa.expect("una base en un archivo sí deja copia previa");
        assert_eq!(Path::new(&previa).parent().unwrap(), dir.path());
        let nombre = Path::new(&previa).file_name().unwrap().to_str().unwrap();
        assert!(nombre.starts_with("el-kioskon-app.pre-restauracion-"));
        assert!(nombre.ends_with(".db"));
        let copia = Connection::open(&previa).unwrap();
        assert_eq!(
            cantidad(&copia, "categorias WHERE categoria = 'Bebidas'"),
            1
        );
    }

    #[test]
    fn copiar_datos_actuales_va_a_la_carpeta_de_la_copia_y_no_a_la_de_la_base() {
        let dir_base = tempfile::tempdir().unwrap();
        let dir_copia = tempfile::tempdir().unwrap();
        let ruta = dir_base.path().join("el-kioskon-app.db");
        base_en_un_archivo(&ruta);

        let previa = {
            let conn = Connection::open(&ruta).unwrap();
            copiar_datos_actuales(&conn, &ruta, dir_copia.path()).unwrap()
        }
        .expect("una base en un archivo sí deja copia previa");

        // The copy belongs next to the backup that was chosen, which is where
        // whoever asked for the restore is looking.
        assert_eq!(Path::new(&previa).parent().unwrap(), dir_copia.path());
        assert!(!dir_base
            .path()
            .read_dir()
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains("pre-restauracion")));
    }

    #[test]
    fn copiar_datos_actuales_cae_a_la_carpeta_de_la_base_si_la_elegida_no_agrega() {
        let dir_base = tempfile::tempdir().unwrap();
        let dir_copia = tempfile::tempdir().unwrap();
        let ruta = dir_base.path().join("el-kioskon-app.db");
        base_en_un_archivo(&ruta);

        // A path whose parent is a regular file can never be written to, and
        // unlike `chmod` that stays true when the tests run as root.
        let bloqueado = dir_copia.path().join("bloqueado");
        std::fs::write(&bloqueado, b"soy un archivo, no una carpeta").unwrap();
        let carpeta_buena = ruta.parent().unwrap();

        let previa = {
            let conn = Connection::open(&ruta).unwrap();
            copiar_datos_actuales(&conn, &ruta, &bloqueado.join("subcarpeta")).unwrap()
        }
        .expect("una carpeta que no admite escritura no puede frenar el restore");

        // A read-only stick must not block the restore: the copy goes back to
        // the live database's own folder, which the app can always write to.
        assert_eq!(Path::new(&previa).parent().unwrap(), carpeta_buena);
        let copia = Connection::open(&previa).unwrap();
        assert_eq!(
            cantidad(&copia, "categorias WHERE categoria = 'Bebidas'"),
            1
        );
    }

    #[test]
    fn copiar_datos_actuales_falla_si_ninguna_carpeta_sirve() {
        let dir = tempfile::tempdir().unwrap();
        let bloqueado = dir.path().join("bloqueado");
        std::fs::write(&bloqueado, b"soy un archivo, no una carpeta").unwrap();
        let ruta = bloqueado.join("subcarpeta").join("el-kioskon-app.db");

        let conn = Connection::open_in_memory().unwrap();
        let error = copiar_datos_actuales(&conn, &ruta, &bloqueado.join("otra")).unwrap_err();

        // Without a safety copy there is nothing to fall back on, so the
        // restore has to stop instead of replacing the data.
        assert!(matches!(error, AppError::RestaurarRespaldoError(_)));
    }

    #[test]
    fn copiar_datos_actuales_reemplaza_una_copia_previa_del_mismo_sello() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("el-kioskon-app.db");
        base_en_un_archivo(&ruta);
        let conn = Connection::open(&ruta).unwrap();

        let primera = copiar_datos_actuales(&conn, &ruta, dir.path())
            .unwrap()
            .unwrap();
        // Two calls in the same second resolve to the same path, and
        // `VACUUM INTO` refuses to write over an existing file, so the stale
        // copy has to be removed first. Two calls in different seconds write
        // to different paths, which is why this only asserts that the second
        // one produced a usable database.
        let segunda = copiar_datos_actuales(&conn, &ruta, dir.path())
            .unwrap()
            .unwrap();
        drop(conn);

        let copia = Connection::open(&segunda).unwrap();
        let integridad: String = copia
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integridad, "ok");
        assert_eq!(
            cantidad(&copia, "categorias WHERE categoria = 'Bebidas'"),
            1
        );
        assert!(Path::new(&primera).exists());
    }

    #[test]
    fn copiar_datos_actuales_no_hace_nada_sobre_una_base_en_memoria() {
        let conn = Connection::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            copiar_datos_actuales(&conn, Path::new(":memory:"), dir.path()).unwrap(),
            None
        );
    }

    #[test]
    fn objeto_nombre_salta_el_if_not_exists() {
        assert_eq!(
            objeto_nombre("CREATE TABLE IF NOT EXISTS categorias (id INTEGER)").unwrap(),
            "categorias"
        );
        assert_eq!(
            objeto_nombre("CREATE TABLE \"detalle_presupuestos\" (id INTEGER)").unwrap(),
            "detalle_presupuestos"
        );
        assert_eq!(
            objeto_nombre("CREATE TABLE main.ventas (id INTEGER)").unwrap(),
            "ventas"
        );
    }

    #[test]
    fn objeto_nombre_rechaza_un_nombre_que_no_se_pueda_interpolar() {
        // The name lands in a `DROP`/`INSERT` built with format!, so a name
        // that is not a plain identifier has to be refused here rather than
        // quoted into something unexpected.
        assert!(matches!(
            objeto_nombre("CREATE TABLE \"ventas; DROP TABLE audit_logs --\" (id INTEGER)"),
            Err(AppError::RespaldoInvalido(_))
        ));
        assert!(matches!(
            objeto_nombre("SELECT 1"),
            Err(AppError::RespaldoInvalido(_))
        ));
    }
}

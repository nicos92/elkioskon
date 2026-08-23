# Plan: Refactorización de `infrastructure/database/mod.rs` (SOLID / SRP)

## Objetivo

Dividir `src-tauri/src/infrastructure/database/mod.rs` (1206 líneas) en módulos pequeños,
cohesivos y testeables, aplicando SRP y bajo acoplamiento. `mod.rs` pasa a ser punto de
composición y exposición únicamente.

**Alcance acordado**: solo el módulo `database` (no se corrigen los puntos de fuga DIP en
`HomeService`/`CierreService`; quedan como trabajo futuro). Se mantienen re-exports de
compatibilidad para no tocar los 106 sitios que usan `database::DB.lock()` ni los ~12
tests que importan `{reset_test_db, TEST_LOCK}`.

**Restricción dura**: no cambia comportamiento, SQL, orden de migración/seeds, ni
contratos públicos. Ningún archivo fuera de `infrastructure/database/` se modifica.

---

## Etapa 1 — Análisis del estado actual

### Responsabilidades mezcladas en `mod.rs`

| # | Responsabilidad | Código actual |
|---|---|---|
| 1 | Configuración de ruta (`CALISE_DB_PATH`, ProjectDirs, override test `:memory:`) | `get_db_path()` L16–38 |
| 2 | Conexión global singleton + inicialización | `DB` L11–14, `init_database()` |
| 3 | Infraestructura de testing global | `TEST_LOCK`, `reset_test_db()` L40–71 |
| 4 | Config criptográfica ajena a la BD | `BCRYPT_COST` L6–9 |
| 5 | Esquema DDL (~285 líneas: 20 tablas + índices) | `apply_schema()` L142–426 |
| 6 | Migraciones (`ensure_column`, rebuild de `presupuestos`, backfills de datos) | L372–395, 428–477, 489–509 |
| 7 | Mantenimiento/retención | `purge_old_audit_logs()` L479–487 |
| 8 | Seeds (permisos, admin con bcrypt, tipos venta, cliente defecto) | L511–598 |
| 9 | Datos demo (~500 líneas de constantes `DEMO_*`) | L600–1088 |
| 10 | Tests unitarios del migrate | L1090–1205 |

### Problemas detectados

- **SRP**: todo lo anterior vive en un archivo.
- **OCP**: agregar una tabla exige tocar `apply_schema` **y** la lista DROP de
  `reset_test_db()` (duplicación).
- **DIP**: `apply_schema` mezcla DDL + migraciones + seeds en una sola secuencia sin seams;
  solo es testeable vía el singleton global.
- Nota: `user_service.rs` tiene su propia constante privada `BCRYPT_COST = 10` duplicada
  (fuera de alcance; se deja como está).
- El test `all_covers_seeded_permissions` de `permission_code.rs` duplica la lista inline
  (no importa `PERMISSIONS`), por lo que mover esa constante es seguro.

---

## Etapa 2 — Estructura propuesta

```text
infrastructure/database/
├── mod.rs               # SOLO composición: pub mod + re-exports compatibles
├── config.rs            # get_db_path() (ambas variantes cfg) + BCRYPT_COST (cfg intactos)
├── connection.rs        # static DB, init_database(), TEST_LOCK, reset_test_db()
│                        # + pub(crate) fn initialize(conn): secuencia de arranque completa
├── schema.rs            # SCHEMA_SQL (DDL idempotente) + apply_schema(conn) SOLO DDL
│                        # + pub(crate) const TABLES: &[&str] (fuente única para reset_test_db)
├── migrations.rs        # ensure_column(), migrate_presupuestos_estado() (+ sus 2 tests),
│                        # backfills nombrados: run_column_migrations(),
│                        # backfill_stock_updated_at(), backfill_ventas_tipo_venta(),
│                        # backfill_ventas_cliente()
├── maintenance.rs       # purge_old_audit_logs()
└── seeds/
    ├── mod.rs           # orquestador de seeds (run_seeds desglosado por paso)
    ├── permissions.rs   # const PERMISSIONS (46) + seed_permissions
    ├── admin.rs         # seed_admin_user (usa BCRYPT_COST de config)
    ├── tipos_venta.rs   # const TIPOS_VENTA + seed_tipos_venta
    ├── cliente.rs       # seed_cliente_defecto
    └── demo_data.rs     # constantes DEMO_* (~500 líneas) + seed_demo_data
```

### Dependencias

```text
mod.rs ── declara/re-exporta
connection.rs ──> config, schema, migrations, maintenance, seeds
schema.rs / migrations.rs / maintenance.rs / seeds/* ──> funciones puras sobre &Connection
```

- Sin traits nuevos: con un único motor (SQLite) y proceso único, un `ConnectionProvider`
  genérico no aporta sustitución real ("no sobre-abstraer").
- Visibilidad mínima: `pub(crate)` o privado; público solo lo re-exportado por compatibilidad.

### Re-exports de compatibilidad en `mod.rs`

```rust
pub use config::{get_db_path, BCRYPT_COST};
pub use connection::{init_database, reset_test_db, DB, TEST_LOCK};
```

---

## Etapa 3 — Pasos de implementación

1. Crear `config.rs`: mover `get_db_path()` (variantes `#[cfg(test)]`/`#[cfg(not(test))]`)
   y `BCRYPT_COST` con sus atributos cfg intactos.
2. Crear `schema.rs`: extraer el bloque DDL a `SCHEMA_SQL`, `apply_schema()` queda solo con
   el `execute_batch(SCHEMA_SQL)`. Definir `TABLES` con las 20 tablas y un test unitario
   sin DB que verifique que cada `CREATE TABLE IF NOT EXISTS <nombre>` de `SCHEMA_SQL`
   figura en `TABLES` (detección de drift OCP).
3. Crear `migrations.rs`: mover `ensure_column`, `migrate_presupuestos_estado` y sus 2 tests
   existentes (intactos). Nombrar los backfills:
   - `run_column_migrations(conn)`: los 3 `ensure_column` de ventas + índice
     `idx_ventas_cliente_id`.
   - `backfill_stock_updated_at(conn)`: ensure_column stock + UPDATE si hay filas.
   - `backfill_ventas_tipo_venta(conn)` y `backfill_ventas_cliente(conn)`: los dos UPDATE
     que dependen de datos sembrados.
4. Crear `maintenance.rs`: mover `purge_old_audit_logs`.
5. Crear `seeds/`: mover constantes y funciones seed tal cual, un archivo por tema.
6. Reescribir `connection.rs`:
   - `initialize(conn)` ejecuta la secuencia **exactamente** en el orden actual:
     ```text
     schema::apply_schema                       (DDL)
     migrations::backfill_stock_updated_at      (ensure_column stock + UPDATE)
     migrations::run_column_migrations          (ensure_column ventas + índice)
     seeds::seed_tipos_venta
     migrations::backfill_ventas_tipo_venta     (UPDATE → 'Efectivo')
     seeds::seed_permissions / seed_admin_user / seed_cliente_defecto
     migrations::backfill_ventas_cliente        (UPDATE → 'Consumidor Final')
     seeds::seed_demo_data
     maintenance::purge_old_audit_logs
     migrations::migrate_presupuestos_estado
     ```
   - `init_database()` abre conexión + llama `initialize`.
   - `reset_test_db()` usa `schema::TABLES` para generar los DROP (con
     `PRAGMA foreign_keys = OFF`, igual que hoy) y luego llama `initialize()`
     (reemplaza al viejo `apply_schema(...)` completo → mismo efecto neto).
7. Reescribir `mod.rs` como módulo de composición con los re-exports.
8. Test nuevo en `maintenance.rs`: `purge_old_audit_logs` sobre
   `Connection::open_in_memory()` propio (sin el global): fila vieja (>90 días) se borra,
   fila reciente queda.

## Etapa 4 — Corrección de dependencias

- Ajustar imports internos entre los nuevos módulos.
- Verificar con grep que ningún sitio externo referencia items privados movidos
  (`PERMISSIONS`, `TIPOS_VENTA`, `DEMO_*` son privados hoy: sin riesgo).
- No hacer públicos elementos que no lo necesitan.

## Etapa 5 — Validación

```bash
cd src-tauri && cargo check && cargo test && cargo clippy --lib --tests
```

---

## Verificación / criterios de aceptación

- [ ] `mod.rs` < ~20 líneas: solo `pub mod` + re-exports.
- [ ] Misma cantidad de tests pasando que antes (+2 nuevos: drift de schema, purge).
- [ ] Los tests migrados de `migrate_presupuestos_estado` pasan intactos.
- [ ] Los ~12 archivos que importan `database::{DB, reset_test_db, TEST_LOCK}` no cambian.
- [ ] `git status` muestra cambios solo dentro de `src-tauri/src/infrastructure/database/`.
- [ ] `cargo check`, `cargo test`, `cargo clippy --lib --tests` sin errores ni warnings nuevos.

## Trabajo futuro (fuera de este plan)

- Extraer queries SQL crudas de `HomeService` a un `SqliteHomeRepository` + trait en domain.
- Mover el bloque SQL de `CierreService::crear_cierre` a `SqliteCierreRepository` y depender
  de `Arc<dyn CierreRepository>` (hoy guarda el tipo concreto).
- Helpers `with_conn`/`with_conn_mut` para eliminar ~20 duplicaciones del mapeo
  lock-poisoning → `AppError::Internal`.
- Unificar `BCRYPT_COST` duplicado entre `config.rs` y `user_service.rs`.

## Notas

- El orden de arranque es contrato de comportamiento: cualquier reordenamiento puede
  romper bases existentes (los backfills dependen de los seeds previos).
- `reset_test_db()` conserva `PRAGMA foreign_keys = OFF` antes de los DROP.
- AGENTS.md §9.6 documenta el comportamiento de test (`:memory:` + `BCRYPT_COST = 4`);
  no cambia con esta refactorización.

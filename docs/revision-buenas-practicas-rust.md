# Revisión de buenas prácticas Rust — backend `src-tauri`

Revisión del crate Rust contra el *Rust Best Practices Handbook* de Apollo
GraphQL (`.agents/skills/rust-best-practices/references/`). Cubre el análisis
completo, el plan de trabajo acordado y el resultado de aplicarlo.

- **Fecha**: 2026-10-02
- **Alcance**: `src-tauri/src/` (119 archivos `.rs`). Excluidos `gen/`
  (generado) y `target/`.
- **Toolchain**: `rustc 1.97.1`, `clippy 0.1.97`, `rustfmt 1.9.0-stable`
- **Versión del crate**: `el-kioskon-app` 0.4.2
- **Estado**: P0 y `[lints]` aplicados. P1/P2 documentados, sin tocar.

---

## 1. Método

| Paso | Cómo |
| ---- | ---- |
| Baseline de lint | `cargo clippy --all-targets --all-features --locked -- -D warnings` |
| Lint extendido | `cargo clippy --all-targets --locked -- -W clippy::pedantic` (volumen, sin aplicarlo a `[lints]`) |
| Inventario de código | Recorrido de `unwrap`/`expect`/`panic!`, `clone`, `&String`/`&Vec`/`&Option`, construcción de SQL con `format!` |
| Superficie pública | Conteo de `pub fn` / `pub struct` / `pub enum` / `pub trait` / `pub const` y cobertura de `///` |
| Dispatch | Mapa de `dyn` en firmas y de los structs `*AppState` |
| Tests | Conteo de `#[test]`, `#[cfg(test)]`, `automock`, `#[serial]`, `unsafe` |
| Verificación empírica | Escaneo del tz database con `chrono-tz` 2000-2044, y test de regresión que reproduce el panic anterior |

Los conteos de abajo son reproducibles con los comandos de la sección 7.

---

## 2. Veredicto

El código base está **sólido**. `cargo clippy --all-targets --all-features
--locked -- -D warnings` pasa limpio: no había deuda de lint acumulada. Los
hallazgos fueron casi todos de robustez (panics alcanzables) y de
mantenibilidad anticipada, no de estilo.

Prioridades originales:

- **P0** — panics alcanzables desde comandos de producción. Uno era un bug real
  de usuario (hueco de DST a la medianoche).
- **P1** — arquitectura: doble sincronización redundante y violación de capas.
- **P2** — SQL construido a mano y código duplicado.
- **P3** — tooling: falta `[lints]`, hay dependencias muertas.

### Lo que ya estaba bien

No todo necesitaba arreglo, y conviene que quede registrado para que nadie
"optimice" algo que ya es correcto:

| Práctica | Evidencia |
| -------- | --------- |
| Cero `unsafe` | 0 ocurrencias en los 119 archivos |
| Cero TODOs sueltos | 0 `TODO`/`FIXME`/`HACK`/`XXX` |
| `thiserror` bien usado, sin `anyhow` | §4.3/§4.4 del handbook: correcto para una app |
| Inyección de dependencias real | Los 16 traits de repo son `Send + Sync`; 15 de 16 tienen `#[cfg_attr(test, mockall::automock)]` |
| Dispatch dinámico solo donde corresponde | `Arc<dyn Repo>` en el borde del servicio; `&dyn ToSql` en los binds de rusqlite |
| Sin anti-patrones de firma | 0 `&String`, 0 `&Vec<T>`, 0 `&&str` |
| Sin colecciones desechables | 0 `.collect()` construidos solo para `len()`/`is_empty()` |
| Aislamiento de tests correcto | `fresh_test_db()` serializa con `TEST_LOCK` e ignora el poisoning a propósito (§3 de AGENTS.md) |
| Base verde | 247 tests antes de empezar, 0 errores |

**Una salvedad sobre los mocks**: `CierreRepository` es el único trait de
repositorio sin `automock`. No es un bug (el servicio se testea contra el
`SqliteCierreRepository` real, que es lo que importa para una consulta con
`GROUP BY`), pero el conteo correcto es 15/16, no 16/16.

---

## 3. Análisis — P0: panics alcanzables

Había **12** ocurrencias de `unwrap`/`expect` en código de producción,
alcanzables desde comandos. Ninguna tenía justificación escrita.

| # | Ubicación | Código | ¿Podía fallar de verdad? |
| - | --------- | ------ | ------------------------ |
| 1 | `home_service.rs:150`, `cierre_service.rs:150` | `local_to_utc(...).unwrap()` | **Sí. Panic real de usuario** (ver §3.1) |
| 2 | `home_service.rs:136,140`, `cierre_service.rs:37,41` | `and_hms_opt(0, 0, 0).unwrap()` | No (`0h 0m 0s` siempre es wall-time válido), pero API que devuelve `Option` |
| 3 | `user_service.rs:81` | `existing.is_some() && existing.unwrap()` | No hoy. Bomba de relojería |
| 4 | `audit_detail.rs:47` | `serde_json::to_string(self).expect(...)` | No (§6.1) |
| 5 | `connection.rs:14` | `init_database().expect(...)` | No en la práctica |
| 6 | `seeds/admin.rs:16` | `bcrypt::hash(...).expect(...)` | No en la práctica |
| 7 | `seeds/demo_data.rs:139` | `bcrypt::hash(...).expect(...)` | No en la práctica. Dentro de un `for` sobre 6 usuarios |
| 8 | `lib.rs:132` | `.expect("error while running tauri application")` | No en la práctica |

### 3.1 El bug de producción: hueco de DST a la medianoche

`local_to_utc` estaba **duplicado byte a byte** en `home_service.rs:145-153` y
`cierre_service.rs:145-153`, y ambas copias hacían:

```rust
chrono::Local
    .from_local_datetime(dt)
    .earliest()                              // Option
    .or_else(|| chrono::Local.from_local_datetime(dt).latest())
    .unwrap()                                // <-- panica
```

`LocalResult` tiene tres casos: `Single`, `Ambiguous` (el reloj atrasa, la hora
local ocurre dos veces) y **`None`** (el reloj adelanta, la hora local **no
existe**). `earliest()` devuelve `Option`, así que en `None` toda la cadena
—incluido el `.or_else`— es `None` y el `.unwrap()` **entra en panic**.

Impacto cuando eso ocurre:

- `get_home_stats` (el dashboard) entra en panic.
- `crear_cierre` entra en panic — **justo el día que más importa cerrar**.

#### Qué tan real es (medido, no supuesto)

Mi primera versión de este documento afirmaba que esto pasaba "dos noches por
año en Argentina". **Eso es falso**, y la forma de llegar a la respuesta correcta
importa porque define si el arreglo era urgente o hipotético.

Medición: barrido de las 16 500 medianoches de 2000 a 2044 por zona, con
`chrono-tz` (que sí respeta la zona; ver la nota de Windows más abajo).

| Zona | Medianoches inexistentes | Próximas |
| ---- | ------------------------ | -------- |
| `Asia/Beirut` | 45 | anuales, hasta 2044-03-27 |
| `America/Santiago` (Chile) | 44 | anuales, hasta 2044-09-04 |
| `America/Havana` (Cuba) | 43 | anuales, hasta 2044-03-13 |
| `Africa/Cairo` | 36 | anuales, hasta 2044-04-29 |
| `America/Asuncion` | 25 | última: 2024-10-06 |
| `Asia/Tehran` | 21 | anual hasta 2022-03-22 |
| `America/Sao_Paulo` | 19 | anual hasta 2018-11-04 |
| `Pacific/Apia` | 2 | 2010-09-26, **2011-12-30** (Samoa saltó el día entero) |
| `America/Argentina/Buenos_Aires` | **2** | 2007-12-30 y 2008-10-19. **Ninguna después** |

Conclusión real: **Argentina dejó de exponer esto en 2008**, porque su regla
actual de DST transiciona a las 24:00 y no a las 00:00. Chile y Cuba lo hacen
todos los años, sin fecha de fin proyectada. El riesgo para esta app no es
"el kiosco de Buenos Aires un año de estos", sino una máquina configurada en
otra zona: Chile y Cuba son vecinas, y un usuario que viaja lleva su reloj.

Por eso el arreglo se hizo igual. Un panic en el cierre es un día de venta sin
cerrar, y la causa no depende de dónde esté la máquina.

> **Trampa de Windows**: `chrono::Local` en Windows ignora la variable de entorno
> `TZ` y usa las APIs de zona horaria del sistema. Un escaneo por `TZ=...` sobre
> Windows devuelve el mismo resultado para todas las zonas y no mide nada. Por
> eso la medición usa `chrono-tz`, que sí resuelve la zona explícitamente. La
> máquina de desarrollo estaba en `Argentina Standard Time`.

### 3.2 El resto no eran bugs, pero se cambiaron

**`user_service.rs:81`** — `existing.is_some() && existing.unwrap().id != id`.
`existing` es un `Option` local inmutable entre la comprobación y el uso, así que
el `unwrap()` no podía fallar. Se queda igual hasta que alguien mueva esa línea,
y entonces entra en producción sin que ningún test lo note. El capítulo 1.3 del
handbook tiene el patrón directo.

**`and_hms_opt(0,0,0).unwrap()`** — `and_hms_opt` devuelve `Option` justamente
para que el llamador decida. Acá `Option` nunca es `None` para `0h 0m 0s`, así que
no era un bug; se absorbió en el helper para que no quede ningún `unwrap` en el
camino y para que el compilador lo vigile si mañana cambia la hora.

### 3.3 Un bonus del P0: un `#[allow]` que era basura

Al activar `[lints.clippy]` aparecieron nueve errores, y uno no era un lint sino
esto:

```
error: this lint expectation is unfulfilled
  --> src/application/services/presupuesto_service.rs:60
```

El `#[allow(clippy::too_many_arguments)]` de `presupuesto_service::create`
**nunca aplicó**: la función tiene 7 parámetros y el umbral de clippy es
dispara a partir de 8. El de `venta_service::create` sí aplica (tiene 8). El
atributo era ruido que además ocultaba que nadie lo había revisado; se borró. El
de `venta_service` se convirtió a `#[expect]`, que a diferencia de `allow`
avisa si el lint deja de ser necesario (§2.4 del handbook).

---

## 4. Hallazgos fuera de alcance

Registrados, no corregidos en esta tanda (ver §6.2).

### 4.1 P1 — doble sincronización redundante (cap. 6 y 9)

Los 16 `*AppState` guardan el servicio dentro de un `Mutex`:

```rust
// api/commands/venta_commands.rs:12
pub struct VentaAppState {
    pub venta_service: Mutex<VentaService>,
}
```

Pero ese servicio ya guarda `Arc<dyn VentaRepository>` y
`Arc<dyn ClienteRepository>`, y **los 16 traits exigen `Send + Sync`**
(`domain/repositories/venta_repository.rs:6` y los otros 15). O sea: el `Arc` ya
es thread-safe, y el `Mutex` alrededor serializa sin necesidad.

Consecuencias:

1. Cada comando toma un mutex **por dominio durante todo su trabajo, incluido
   el I/O contra SQLite**. Dos ventas simultáneas del mismo dominio se serializan
   aunque no se toquen.
2. Hay **tres niveles de bloqueo** para lo que `Arc<dyn Trait>` ya resuelve:
   `Mutex<AppState>` → `Mutex<DB>` → el lock interno de SQLite.
3. `Mutex<T>: Sync` solo requiere `T: Send`. El compilador lo acepta, así que
   nada delata el sobrante.

### 4.2 P1 — violación de capas

`application/` y `api/` tocan `infrastructure::database::DB` con SQL crudo:

- `cierre_service.rs:44`, `home_service.rs:20`, `cost_update_service.rs:127`
- `articulo_commands.rs`, `stock_commands.rs`, `sub_categoria_commands.rs`,
  `permissions.rs`, `mod.rs`

`HomeService` y `CierreService` directamente **no tienen repositorio**: la capa
de dominio no existe para ellos, el servicio es un `api/commands` con otro
nombre. AGENTS.md §9.9 justifica que `check_permission` consulte la DB directo
(`permissions.rs`), pero eso no cubre los demás.

### 4.3 P2 — SQL construido a mano

`audit_log_repository.rs:64-105` — el filtro de auditoría arma el `WHERE` con
`format!` y **reimplementa el escape de comillas 4 veces**:

```rust
conditions.push(format!("screen = '{}'", screen.replace('\'', "''")));
```

Hoy es correcto (doble comilla simple es el escape de literales en SQLite), pero
la corrección depende de que nadie olvide un `.replace` en el próximo filtro que
se agregue. Las alternativas son placeholders `?` con `params![]`, como ya se
hacen en el resto del crate.

Otros puntos menores del mismo grupo:

- **Placeholders**: `vec!["?"; n].join(", ")` en lugar de
  `presupuesto_repository.rs:369`, `venta_repository.rs:408` y
  `cost_update_repository.rs:100-105` (este último opera sobre toda la tabla
  `stock`: N `String` desechables por llamada).
- **Literales estáticos convertidos a `String` en cada llamada**:
  `"p.estado = ?".to_string()` en `presupuesto_repository.rs:237,243,250,269`.
- **Identificadores interpolados**: `respaldo_repository.rs:395,410` y
  `migrations.rs:9,20`. **No es inyección** — salen de `TABLES` o del
  `sqlite_master` de la base viva, nunca de input externo — pero el nombre de
  tabla no está como constante de tipo.

### 4.4 P2 — `turno_actual` duplicado

`venta_repository.rs:448-465` y `presupuesto_repository.rs:409-426` son copias
idénticas, en la ruta crítica de toda venta y todo presupuesto. Además el
`match` sobre `Option` final es un `unwrap_or(false)` disfrazado (§1.3):

```rust
match es_nocturno_ahora(&config) {
    Some(es_nocturno) => Ok((config.activo, es_nocturno)),
    None => Ok((false, false)),
}
```

No es el caso del *Rule of Three* de §1.8 ("dos ocurrencias están bien"): es
**conocimiento compartido duplicado**. "Cómo se resuelve la medianoche local" y
"cómo se lee la franja nocturna" son decisiones de negocio que si cambian tienen
que cambiar en todos lados a la vez. Por eso el Paso 2 sí las extrae.

### 4.5 P3 — documentación (cap. 8)

- 278 `pub fn` + 159 tipos públicos, **98 líneas de `///`/`//!` en total**, y solo
  **3 items públicos documentados**.
- Sin `#![deny(missing_docs)]` ni `#![warn(missing_docs)]` en `lib.rs`.
- Con `-W clippy::pedantic`: **263** `missing_errors_doc` y **5**
  `missing_panics_doc`.

Parte de esto es discutible — un Tauri app no es una librería pública, y
documentar 437 items para que no los lea nadie tiene costo de mantenimiento. Pero
`AppError::code()` y `user_message()` sí deberían documentar qué es cada cosa:
son el contrato con el frontend.

### 4.6 Mantenimiento: agregar una variante de `AppError` toca 3 lugares

```
error.rs  enum AppError      (+1 variante)
error.rs  fn code()          (+1 brazo de 65)
error.rs  fn user_message()  (+1 brazo de 65)
```

Mismo patrón con `PermissionCode::as_str()` (48 brazos) y `AuditScreen::as_str()`
(15). El mapeo es 1:1, así que una macro tipo `strum::EnumString` lo eliminaría
— pero `user_message()` tiene `format!` y necesita una macro propia.

Lo importante es que los tres `match` son **exhaustivos**: agregar una variante
**falla al compilar**. El costo de mantenimiento es alto pero nunca silencioso,
que es lo que §4.1 del handbook pide.

Contraste útil: los **permisos sí** están doc-verificados.
`docs_consistency.rs` tiene `permission_count_in_docs_matches_the_enum`, que
compara el "48" de AGENTS.md §9.8 contra `PermissionCode::all()`. Las variantes
de `AppError` no tienen ese guard — por eso puede haber un desfase silencioso
entre el contrato del frontend y el enum, que es justo el punto que más
conviene documentar.

---

## 5. Plan de trabajo

Alcance acordado: **P0 (los panics) + `[lints]` selectivo**. P1 y P2 quedan
anotados, no se tocan. **Los cuatro pasos están aplicados.**

### Paso 1 — `[lints]` en `Cargo.toml` + higiene de dependencias

Aplicado. El bloque final:

```toml
[lints.rust]
unsafe_code = "forbid"
# Cargo exige que un grupo tenga prioridad distinta de los lints puntuales de la
# misma tabla; -1 lo deja por debajo de `unsafe_code` sin perder efecto propio.
future_incompatible = { level = "warn", priority = -1 }

[lints.clippy]
# `all` queda en el priority default (0) y los lints puntuales en 1 a propósito:
# en Cargo el priority MÁS ALTO gana, así que al revés el `deny` de `all`
# taparía el `warn` de `unnecessary_wraps`.
all = "deny"
redundant_clone = { level = "deny", priority = 1 }
needless_borrow = { level = "deny", priority = 1 }
large_enum_variant = { level = "deny", priority = 1 }
clone_on_copy = { level = "deny", priority = 1 }
manual_ok_or = { level = "deny", priority = 1 }
map_unwrap_or = { level = "deny", priority = 1 }
needless_collect = { level = "deny", priority = 1 }
unnecessary_wraps = { level = "warn", priority = 1 }
```

**Por qué no `pedantic` a ciegas.** El paso extendido reporta 263
`missing_errors_doc` y 96 `needless_pass_by_value`. Buena parte de estos últimos
son **falsos positivos**: las firmas de comandos Tauri *deben* ser `String`/`Vec`
por ownership porque serde las deserializa — "arreglarlas" rompería el puente con
el frontend. La doc de AGENTS.md §10 ya dice que el lint del frontend es
`pnpm build`; lo mismo vale acá: el lint debe describir el estilo real.

`unnecessary_wraps` queda en `warn` y **hoy no dispara en el crate**: es una
guarda hacia adelante, no una deuda actual.

**Fallos que aparecieron al activar el bloque, y cómo se resolvieron:**

1. `redundant_clone` encontró **8** casos, no 6:
   - 6 en `proveedor_service.rs:80-85`, seis `existing.campo = proveedor.campo.clone()`
     consecutivos → `clone_from()`.
   - 1 en `user_service.rs:119` (`target.clone()` donde `target` no se usa después)
     → mover.
   - 1 en `cost_update_repository.rs:119` (`now.clone()` donde `now` no se usa
     después) → mover.
2. `map_unwrap_or` encontró **6**: cinco `DateTime::parse_from_rfc3339` en
   `user_repository.rs` y uno en `dollar_commands.rs:81` → `map_or_else`.
3. El `#[expect]` de `presupuesto_service` resultó no cumplido (§3.3).
4. `future_incompatible` como grupo chocó con `unsafe_code`: Cargo rechaza
   prioridades iguales entre un grupo y un lint puntual. Resuelto con
   `priority = -1`.

**Higiene:**

| Cambio | Detalle |
| ------ | ------- |
| `once_cell::sync::Lazy` → `std::sync::LazyLock` | `connection.rs:1,13`. Estable desde 1.80; rustc actual 1.97.1. `once_cell` sale de `[dependencies]`. Sigue en el lock como transitiva de `rusqlite`/`tauri`, que es lo correcto |
| `serial_test` → borrar | dev-dep con **0** `#[serial]` en el crate |
| `#[allow]` → `#[expect]` | Solo `venta_service.rs:57`; el de `presupuesto_service` se borró (§3.3) |
| `chrono-tz` → dev-dep | Agregado para poder testear la rama `None` con datos IANA reales (§5, Paso 2) |

### Paso 2 — Helper de rango local compartido

Aplicado. Nuevo módulo `application/services/rango_local.rs`, hermano de
`audit_detail.rs`, con esta firma:

```rust
pub fn rango_utc_del_dia_local(day: NaiveDate) -> Result<(String, String), AppError>
```

Internamente matchea los tres casos de `LocalResult` a través de un core
genérico sobre `TimeZone`:

- `Single(dt)` → ese instante.
- `Ambiguous(primero, _)` → el más temprano. El retroceso del reloj hace que la
  hora ocurra dos veces, y el día arranca con la primera.
- `None` → la hora local **no existe**. Se busca la primera que sí existe, con
  tope de 24 intentos de una hora. Si tampoco aparece, `AppError::Internal`.

**Decisión de negocio, tomada:** la primera opción era devolver un error en vez de
degradar. Se eligió degradar porque un error convierte esas noches en un
dashboard caído y un día que no se puede cerrar, que es exactamente cuando el
cierre importa. Si se prefiere el error, es cambiar el cuerpo de la rama `None`.

`AppError::Internal` y no una variante nueva: agregarla habría toca 3 lugares
(§4.6) y ampliaría el contrato del frontend con un código que nadie maneja, para un
caso que solo puede darse en una zona patológica.

** Consumidores actualizados:**

- `home_service.rs:47` → `rango_utc_del_dia_local(Local::now().date_naive())?`;
  borradas `hoy_utc_range()` y `local_to_utc()`.
- `cierre_service.rs:37` → `rango_utc_del_dia_local(day)?`; borrada `local_to_utc()`.

**Tests (6, todos verdes).** El core genérico existe justamente para esto: con
`Local` no hay forma de elegir zona, y en Windows `TZ` no sirve (§3.1).

| Test | Qué prueba |
| ---- | ---------- |
| `medianoche_inexistente_en_chile_arranca_en_la_primera_hora_existente` | La rama `None` con datos reales: Santiago 2026-09-06 arranca 01:00 y el rango sigue siendo del día pedido |
| `el_codigo_anterior_entaba_en_panic` | Regresión: el `.earliest().or_else(..).unwrap()` anterior entra en panic con esa misma fecha y zona |
| `medianoche_ambigua_toma_la_primera_aparicion` | Rama `Ambiguous` con datos reales: Havana 2026-11-01 |
| `un_dia_sin_cambio_empieza_a_la_medianoche_local` | El caso normal, en tres zonas |
| `el_frontera_entre_dias_es_compartida_en_cualquier_zona` | El invariante fuerte: el `fin` de un día es el `inicio` del siguiente, en 4 zonas × 3 días (normal, con hueco, con ambigüedad) |
| `funciona_para_hoy_en_la_zona_del_sistema` | El camino real de `get_home_stats` |

Los tests tienen una guarda `assert!(matches!(..., LocalResult::None))` que
verifica la premisa antes de probar el comportamiento: si el tz database cambia
y la fecha deja de tener hueco, el test falla con un mensaje claro en vez de
pasar por arriba.

### Paso 3 — `user_service.rs:81`

Aplicado.

```rust
// antes
if existing.is_some() && existing.unwrap().id != id {
// después
if existing.as_ref().is_some_and(|otro| otro.id != id) {
```

Una línea. El mensaje de error no cambia.

### Paso 4 — Documentar la deuda

Este mismo documento es el registro. No va a AGENTS.md a propósito:
`docs_consistency.rs` verifica frases concretas de AGENTS.md (la tabla de
comandos, el conteo de permisos, el conteo de tablas, el nombre del archivo de
base), así que las notas de deuda no lo romperían, pero ensuciarían un
documento que hoy es una especificación y no un changelog.

---

## 6. Veredictos de alcance

### 6.1 Se quedan como están

Quedan **5** ocurrencias de `expect` en producción, todas deliberadas:

| Hallazgo | Veredicto |
| -------- | --------- |
| `audit_detail.rs:47` `.expect()` | **Se queda.** `AuditDetail` es solo `&'static str` + `String` + `Vec<AuditCambio>`, y `AuditCambio` son tres strings. serde_json no puede fallar ahí. §4.2 de Apollo permite `expect` cuando el fallo es imposible. Arreglarlo exige `?` en 12 call sites de 12 archivos para cubrir un failure mode inexistente. Se le agregó un comentario con la invariante |
| 2 × `expect()` de bcrypt en `seeds/admin.rs` y `seeds/demo_data.rs` | **Se quedan.** Viven en `initialize()`, que devuelve `Result<_, rusqlite::Error>`; el error de bcrypt no entra en ese tipo. Arreglarlos cambia la firma de `initialize` → `init_database` → `reset_test_db` → `ensure_db_ready` para cubrir "bcrypt no compiló". Anotado en §6.2 |
| `connection.rs:13` y `lib.rs:132` `.expect()` | **Se quedan.** Son bootstrap: si la DB no abre, la app no tiene nada que hacer y no hay a qué caerse. `lib.rs:132` es el entry point de Tauri |
| `&Option<String>` ×8 en firmas | Fuera. Barato de arreglar (`Option<&str>`), pero sin bug detrás. Anotado |
| 66 × `float_cmp`, 263 × `missing_errors_doc` | Fuera. Caro de revisar en diff, ninguno es un bug |

### 6.2 Deuda registrada

Estos quedan anotados, no corregidos:

1. **Doble serialización** de §4.1 — el `Mutex<Service>` en los 16 `*AppState`.
2. **Violación de capas** de §4.2 — 8 módulos con SQL crudo fuera de
   `infrastructure/`; `HomeService` y `CierreService` sin repositorio.
3. **Escape SQL manual** de §4.3 — 4 reimplementaciones del escape de comillas en
   `audit_log_repository.rs`.
4. **`turno_actual` duplicado** de §4.4.
5. **Coste de 3 lugares por variante de `AppError`** de §4.6.
6. **Cobertura de documentación** de §4.5.
7. **2 `expect()` de bcrypt en el bootstrap** de §6.1.
8. **`&Option<String>` ×8** y 66 `float_cmp`.
9. **`CierreRepository` sin `automock`** (§2).

---

## 7. Reproducir los números

```bash
cd src-tauri

# baseline (debe seguir limpio)
cargo clippy --all-targets --all-features --locked -- -D warnings

# volumen de pedantic (informativo, no bloqueante)
cargo clippy --all-targets --locked -- -W clippy::pedantic

# formato y tests
cargo fmt --check
cargo test
```

Estado actual: clippy limpio, `fmt --check` exit 0, **253 tests** (247 originales
+ 6 nuevos de `rango_local`), 0 errores.

`cargo test` es el que más importa: `docs_consistency.rs` valida que AGENTS.md
siga sincronizado con el código, y el Paso 2 movió firmas.

> Para reproducir la medición de §3.1 hay que usar `chrono-tz`, no `TZ`. En
> Windows `TZ` no cambia la zona de `chrono::Local`.

---

## 8. Observaciones

- La base es inusualmente buena para una app Tauri: la capa de dominio tiene
  traits reales con mocks, no services que hablan directo con la DB. Los
  hallazgos de §4.1 y §4.2 son erosión en los bordes, no un problema de diseño
  de fondo.
- El proyecto ya tiene un patrón de testeo DB bien pensado (`fresh_test_db()` con
  `TEST_LOCK` y el poisoning ignorado a propósito, documentado en AGENTS.md §3).
  Vale la pena mantener ese rigor en el helper de fechas: los tests de
  `rango_local` mantienen la misma exigencia — verificar la premisa antes de
  verificar el comportamiento.
- El repo no tiene CI para Rust más allá de los comandos de AGENTS.md §3. Con
  `redundant_clone` en `deny` (y `redundant_clone` es un lint de *nursery*), un
  upgrade de toolchain puede romper el build sin que nadie haya tocado código.
  Vale la pena registrar eso en AGENTS.md §3.
- La lección del §3.1 es generalizable: **la primera versión de este documento
  afirmaba un bug anual en Argentina sin haberlo medido**, y era falso. La
  medición con `chrono-tz` convirtió una afirmación sin comprobar en una tabla, y esa
  tabla es la que justifica el arreglo. Un hallazgo de panic tiene que
  verificarse contra la plataforma donde corre.
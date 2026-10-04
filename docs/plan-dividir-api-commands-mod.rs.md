# Plan para dividir `api/commands/mod.rs` por dominio

El objetivo es romper el cuello de botella que representa `src-tauri/src/api/commands/mod.rs` (416 líneas) dividiéndolo en archivos por dominio, sin alterar la API pública de comandos Tauri.

## 1. Diagnóstico

El archivo actual mezcla responsabilidades:

- Imports generales y compartidos (1–13)
- Declaración de 16 submódulos (14–30)
- `pub use` masivo desde todos los archivos de comandos (32–80)
- Estado compartido: `AppState`, `impl Default`, `impl AppState` (82–98)
- DTOs/Requests/Responses de usuarios: `CreateUserRequest`, `LoginRequest`, `UpdateUserRequest`, `AddPermissionRequest`, `ChangePasswordRequest`, `UserResponse`, `LoginResponse`, `impl From<User>` (100–159)
- Comandos inline de **usuarios/autenticación**: `ensure_db_ready`, `login`, `create_user`, `get_all_users`, `update_user`, `change_password`, `delete_user` (161–299)
- Comandos inline de **permisos**: `add_permission_to_user`, `remove_permission_from_user`, `get_user_permissions`, `get_all_permissions`, `create_permission` (301–402)
- Helper auxiliar: `permission_code_by_id()` (404–416)

El resto de los dominios ya están separados en sus propios archivos (`articulo_commands.rs`, `categoria_commands.rs`, `venta_commands.rs`, etc.). Los cortes naturales son **usuarios/auth** y **permisos**.

## 2. División propuesta

Se crearán dos nuevos archivos y se reducirá drásticamente `mod.rs`.

| Archivo nuevo | Elementos a extraer | Líneas aprox. |
|---|---|---|
| `src-tauri/src/api/commands/user_commands.rs` | DTOs de usuario (`CreateUserRequest`, `LoginRequest`, `UpdateUserRequest`, `ChangePasswordRequest`, `UserResponse`, `LoginResponse`, `impl From<User>`), comandos `login`, `create_user`, `get_all_users`, `update_user`, `change_password`, `delete_user`. | ~140–150 |
| `src-tauri/src/api/commands/permission_commands.rs` | `AddPermissionRequest`, comandos `add_permission_to_user`, `remove_permission_from_user`, `get_user_permissions`, `get_all_permissions`, `create_permission`, helper `permission_code_by_id()`. | ~120–140 |

## 3. Decisiones de diseño

| Decisión | Propuesta | Justificación |
|---|---|---|
| **Mover `ensure_db_ready`** | Dejarlo en `mod.rs` | Es un comando transversal de inicialización, no pertenece al dominio de usuarios. Además coincide con lo documentado ("inline en mod.rs"). |
| **Mover `AddPermissionRequest`** | A `permission_commands.rs` | Se usa para asignar/quitar permisos. Mantenerlo junto a esos comandos mejora la cohesión del dominio. |
| **DTOs compartidos** | Mantener junto a su dominio | Evitar crear una carpeta `api/dtos/` prematuramente. Si aparece reutilización real entre dominios, podrá extraerse más adelante. |
| **Helper `permission_code_by_id`** | Mover a `permission_commands.rs` | Solo se usa en `add_permission_to_user` y `remove_permission_from_user`. Es un detalle interno de ese dominio. |
| **Preservar `pub use` en `mod.rs`** | Sí | Garantiza retrocompatibilidad total con `lib.rs` (no requiere modificar imports ni el `invoke_handler`). |
| **Imports comunes** | Duplicar lo mínimo | Cada archivo declara sus propias dependencias. Esto es correcto y evita crear un módulo `common.rs` artificial. |

## 4. Estructura resultante

### 4.1 `mod.rs` tras la división

Deberá contener únicamente módulos, re-exports, estado compartido y el comando transversal:

- Declaraciones `pub mod ...` (incluyendo `user_commands` y `permission_commands`)
- `pub use` de todos los comandos (manteniendo exactamente los nombres usados por `lib.rs`)
- `AppState`, `Default` e `impl AppState`
- `#[tauri::command] pub async fn ensure_db_ready()`

**Tamaño objetivo:** ~120–150 líneas (vs 416 actuales).

### 4.2 `user_commands.rs`

Responsabilidades: DTOs de usuario, conversión `User -> UserResponse`, comandos de autenticación/gestión de usuarios.

Imports esperados: `std::sync::Mutex`, `tauri::State`, `crate::api::commands::{permissions::check_permission, AppState}`, `crate::application::services::{log_audit, AuditDetail}`, `crate::domain::entities::{AuditAction, AuditScreen, PermissionCode, User}`, `crate::infrastructure::error::AppError`.

### 4.3 `permission_commands.rs`

Responsabilidades: DTO para asignación de permisos, helper `permission_code_by_id`, comandos de gestión de permisos.

Imports esperados: `rusqlite::params`, `std::sync::Mutex`, `tauri::State`, `crate::api::commands::{permissions::check_permission, AppState}`, `crate::application::services::{log_audit, UserService}`, `crate::domain::entities::{AuditAction, AuditScreen, Permission, PermissionCode, UserPermission}`, `crate::infrastructure::database::DB`, `crate::infrastructure::error::AppError`.

## 5. Impacto en otros archivos

| Archivo | Cambio | Motivo |
|---|---|---|
| `src-tauri/src/api/commands/mod.rs` | Sí (refactor) | Extraer código, añadir módulos nuevos, actualizar `pub use`, reducir archivo. |
| `src-tauri/src/lib.rs` | No | Usa imports y nombres existentes. Los `pub use` se mantienen idénticos. |
| `src-tauri/src/api/commands/permissions.rs` | No | Se conserva tal cual. |
| `src-tauri/src/docs_consistency.rs` | Posible revisión | No debería requerir cambios (comandos registrados iguales). Ejecutar tests para confirmar. |
| Resto de `api/commands/*.rs` | No | No se modifican. |

## 6. Plan de ejecución paso a paso

### Fase 0 – Preparación
1. Verificar estado base con build limpio: `cd src-tauri && cargo build`

### Fase 1 – Crear `user_commands.rs`
1. Crear `src-tauri/src/api/commands/user_commands.rs`
2. Copiar imports, DTOs, `impl From<User>`, y comandos `login`, `create_user`, `get_all_users`, `update_user`, `change_password`, `delete_user`
3. Ajustar imports para referenciar `AppState` desde `crate::api::commands::AppState`

### Fase 2 – Crear `permission_commands.rs`
1. Crear `src-tauri/src/api/commands/permission_commands.rs`
2. Copiar `AddPermissionRequest`, helper `permission_code_by_id`, y comandos de permisos
3. Ajustar imports según corresponda

### Fase 3 – Reducir `mod.rs`
1. Añadir `pub mod user_commands;` y `pub mod permission_commands;` en la sección de módulos
2. Eliminar de `mod.rs` todo lo movido a los nuevos archivos (DTOs, impls, comandos de usuario/permisos)
3. Actualizar bloque `pub use` para re-exportar desde `user_commands` y `permission_commands` los elementos movidos
4. Limpiar imports que queden sin uso

### Fase 4 – Verificación de compilación
1. `cd src-tauri && cargo check`
2. `cd src-tauri && cargo build`

### Fase 5 – Verificación de calidad
1. `cd src-tauri && cargo fmt --check`
2. `cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings`

### Fase 6 – Tests
1. `cd src-tauri && cargo test` (incluir todos los tests, especialmente `docs_consistency`)

## 7. Riesgos y mitigaciones

| Riesgo | Causa | Mitigación |
|---|---|---|
| Imports circulares | Dependencia entre nuevos módulos | `AppState` vive en `mod.rs`. Ambos nuevos archivos lo importan desde `crate::api::commands::AppState`. No hay dependencia entre ellos. |
| Visibilidad incorrecta | Olvidar `pub` en items re-exportados | Usar `pub` en structs, enums, funciones y tipos movidos. Detectable con `cargo check`. |
| Pérdida de atributos `#[tauri::command]` | Error al copiar | Copiar atributos tal cual desde `mod.rs`. |
| Re-exports incompletos | Olvidar algún item en `pub use` | Revisar contra lo extraído. Cualquier omisión falla en `cargo check` o build. |
| Falla en `docs_consistency` | Estructura inesperada | Comandos registrados mantienen mismos nombres. Ejecutar `cargo test` para validar. |

## 8. Criterios de aceptación

- [ ] `user_commands.rs` creado con elementos de usuario/auth
- [ ] `permission_commands.rs` creado con permisos + helper
- [ ] `mod.rs` reducido a módulos + `pub use` + `AppState` + `impls` + `ensure_db_ready` (~≤150 líneas)
- [ ] `cargo check` OK
- [ ] `cargo build` OK
- [ ] `cargo fmt --check` OK
- [ ] `cargo clippy --all-targets --all-features --locked -- -D warnings` OK
- [ ] `cargo test` OK (todos los tests pasan)
- [ ] Superficie de comandos Tauri idéntica (`lib.rs` sin modificaciones)

## 9. Conclusión

Esta división corrige el principal cuello de botella organizativo: `mod.rs` deja de ser un *god module* y vuelve a su rol de barrel. Mejora cohesión, legibilidad y paralelización de PRs, sin introducir sobreingeniería. El riesgo es bajo porque es un refactor puramente estructural con red de seguridad completa (compilación + clippy + tests).
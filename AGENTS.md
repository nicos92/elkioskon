# AGENTS.md - Guía para Agentes de Código

Este documento proporciona instrucciones y convenciones para agentes de código que operan en este repositorio.

---

## 1. Resumen del Proyecto

- **Stack**: Tauri 2 + Vue 3 + TypeScript + Vite + Pinia + Vue Router
- **Package Manager**: pnpm
- **Frontend**: Vue 3 + TypeScript strict mode, con arquitectura limpia (Domain-Driven Design simplificado)
- **Backend**: Rust con arquitectura limpia (Clean Architecture)
- **Base de datos**: SQLite con rusqlite

---

## 2. Arquitectura del Proyecto

### Backend Rust (Clean Architecture)

``` bash
src-tauri/src/
├── domain/           # Entidades y traits de repositorio
│   ├── entities/     # 23 tipos (ver abajo)
│   └── repositories/ # 16 traits + pagination.rs (Page<T>, paginación server-side)
├── application/      # Casos de uso
│   └── services/     # 17 servicios + audit_detail.rs (DTO de auditoría)
├── infrastructure/   # Implementaciones concretas
│   ├── database/     # config, connection, maintenance, migrations, schema, seeds/
│   ├── repositories/ # 16 impls Sqlite*Repository
│   └── error.rs      # AppError enum
└── api/
    └── commands/     # 18 archivos; mod.rs reexporta todos los comandos y AppStates
```

**`domain/entities/`** (23): `articulo`, `audit_log`, `categoria`, `cierre`, `cliente`, `cost_update_item`, `cost_update_operation`, `dollar_quote`, `dollar_rate`, `home`, `nocturno_config`, `permission`, `permission_code`, `presupuesto`, `proveedor`, `respaldo`, `stock`, `stock_preview`, `sub_categoria`, `tipo_venta`, `turno`, `user`, `venta`.

**`domain/repositories/`** (16 traits): `articulo`, `audit_log`, `categoria`, `cierre`, `cliente`, `cost_update`, `dollar_quote`, `nocturno_config`, `presupuesto`, `proveedor`, `respaldo`, `stock`, `sub_categoria`, `tipo_venta`, `user`, `venta`. Más `pagination.rs` con el tipo `Page<T>` que usan los repos paginados.

**`application/services/`** (17): `articulo`, `audit_log`, `categoria`, `cierre`, `cliente`, `cost_update`, `dollar`, `home`, `nocturno_config`, `presupuesto`, `proveedor`, `respaldo`, `stock`, `sub_categoria`, `tipo_venta`, `user`, `venta`.

**`infrastructure/database/`** — 6 módulos, no un archivo único:

| Módulo | Responsabilidad |
| ------ | --------------- |
| `config.rs` | `get_db_path()` y `BCRYPT_COST` (10 normal, 4 en test) |
| `connection.rs` | `DB: Lazy<Mutex<Connection>>`, `init_database()`, helpers de test |
| `schema.rs` | `SCHEMA_SQL` (DDL completo) y `TABLES` (lista de tablas, `#[cfg(test)]`) |
| `migrations.rs` | Migraciones idempotentes para bases preexistentes |
| `maintenance.rs` | `purge_old_audit_logs()` — borra `audit_logs` con más de 90 días |
| `seeds/` | `admin`, `cliente`, `demo_data`, `nocturno_config`, `permissions`, `proveedor`, `tipos_venta` |

**`api/commands/`** (18 archivos): los comandos de usuario (`login`, `create_user`, `get_all_users`, `update_user`, `change_password`, `delete_user`, `get_user_permissions`, `get_all_permissions`, `add_permission_to_user`, `remove_permission_from_user`, `create_permission`), `ensure_db_ready` y `get_home_stats` viven **inline en `mod.rs`**, no en un archivo propio. `permissions.rs` expone el helper `check_permission` que comparten los demás módulos.

### Frontend Vue (Clean Architecture)

``` bash
src/
├── domain/            # Tipos e interfaces
│   ├── entities/      # User, Permission, Proveedor, Categoria, SubCategoria, Articulo, Stock, Cliente, Venta, Presupuesto, PERMISSIONS, businessRules
│   └── interfaces/    # 16 contratos I*Repository (articulo, audit, categoria, cierre, cliente, dollar, home, nocturno, presupuesto, proveedor, respaldo, stock, subCategoria, tipoVenta, user, venta)
├── application/       # Casos de uso
│   └── usecases/      # Login, CreateUser, GetAllUsers, UpdateUser, DeleteUser, ChangePassword, ManagePermissions, ProveedorUseCase, CategoriaUseCase, SubCategoriaUseCase, ArticuloUseCase, StockUseCase, ClienteUseCase, TipoVentaUseCase, VentaUseCase, ConvertirPresupuestoEnVenta, PresupuestoUseCase, CierreUseCase, AuditUseCase, HomeUseCase, DollarUseCase, NocturnoUseCase, RespaldoUseCase
├── infrastructure/    # Implementaciones
│   ├── api/           # 16 repos que implementan los contratos I*Repository, + errorHandler.ts
│   ├── di.ts          # Registra y exporta las 16 instancias de repos (única fuente; los stores las consumen)
│   └── utils/         # currentUser.ts (getCurrentUserId)
└── presentation/      # Capa UI
    ├── layouts/       # MainLayout con sidebar
    ├── pages/         # 20 páginas (ver abajo)
    ├── stores/        # 19 Pinia stores (ver abajo)
    ├── composables/   # usePermissions, usePagination, useCart, useConfirm, useToasts; venta/ (usePreciosVenta, useClienteSeleccion, usePresupuestoOrigen)
    ├── components/    # ui/ (Modal, DataTable, PaginationBar, ConfirmButton, PageHeader, SearchBar, EntityFormModal), venta/ (ArticuloSearch, CartTable, ClienteSelector, NuevoClienteModal, PresupuestoPrintArea, TotalsPanel), Toasts, TopBar, ConfirmDialog
    └── router/        # Vue Router config
```

**`presentation/pages/`** (20): `ActualizarCostoPage`, `ArticulosPage`, `AuditoriaPage`, `CategoriasPage`, `CierresPage`, `ClientesPage`, `DolarPage`, `HomePage`, `LoginPage`, `NuevaVentaPage`, `PermissionsPage`, `PresupuestosPage`, `ProveedoresPage`, `SettingsPage`, `StockPage`, `SubCategoriasPage`, `TiposVentaPage`, `UserPermissionsPage`, `UsersPage`, `VentasPage`.

**`presentation/stores/`** (19): `articulosStore`, `auditStore`, `authStore`, `categoriasStore`, `cierresStore`, `clientesStore`, `dolarStore`, `homeStore`, `nocturnoStore`, `permissionsStore`, `presupuestosStore`, `proveedoresStore`, `respaldoStore`, `stockStore`, `subCategoriasStore`, `themeStore`, `tiposVentaStore`, `usersStore`, `ventasStore`.

---

## 3. Comandos de Build y Desarrollo

### Comandos principales (frontend)

```bash
pnpm dev                          # Inicia el servidor Vite en http://localhost:1420
pnpm build                        # TypeScript check + build de producción
pnpm preview                      # Previsualizar build de producción
```

> Nota: `pnpm build` corre `vue-tsc --noEmit` (typecheck) y luego `vite build`. No hay script de lint ni de tests en `package.json`.

### Comandos Tauri

```bash
pnpm tauri dev                    # Desarrollo Tauri (frontend + backend)
pnpm tauri build                  # Build de producción Tauri
```

### Comandos Rust (directos)

```bash
cd src-tauri && cargo build       # Compilar
cd src-tauri && cargo test        # Ejecutar tests (unitarios + repos con DB en memoria)
cd src-tauri && cargo fmt         # Normalizar formato en todo el crate
cd src-tauri && cargo fmt --check # Verificar formato sin escribir (usar en CI)
cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings
```

> **Por qué los flags completos de clippy**: `--all-targets` incluye los targets de test, donde `--lib --tests` no llega. Los repositorios tienen sus tests inline (`#[cfg(test)] mod tests` en el mismo archivo), así que un comando que no los compila deja sin revisar justo el código que más errores acumula. `--locked` evita resolver dependencias en silencio; `-D warnings` convierte cada lint en error.

### Tests (Rust)

- **Nunca tocan la DB real**: en builds de test `get_db_path()` devuelve `:memory:` y `BCRYPT_COST` = 4. El archivo `el-kioskon-app.db` queda intacto.
- **Tests que tocan la DB** (repositorios, servicios) usan el helper `fresh_test_db()`, que serializa los tests y reconstruye el esquema. Vive en `infrastructure/database/connection.rs` y se reexporta desde `mod.rs`.

```rust
let _guard = fresh_test_db();
```

> **Por qué `fresh_test_db()` y no tomar el lock a mano**: el helper es la única adquisición de `TEST_LOCK` en todo el crate, y el lock es un `static` privado justamente para que el patrón no se pueda reintroducir. Además ignora el poisoning con `unwrap_or_else(PoisonError::into_inner)`: el lock serializa tests, no protege un invariante, y el esquema se reconstruye inmediatamente después de adquirirlo, así que no hay estado que corromper. Ignorarlo evita que un test que falla de verdad se esconda detrás de nueve `PoisonError` que no dicen nada sobre la causa real.

- Tests unitarios puros (sin DB) se escriben como módulos `#[cfg(test)] mod tests` en el mismo archivo: entidades, `AppError`, `PermissionCode`, `schema`.
- `docs_consistency.rs` verifica que `AGENTS.md` no se desvíe del código (comandos, permisos, tablas). Si agregás un comando o permiso y el doc no lo menciona, el test falla.
- `PermissionCode::all()` tiene un test que verifica los 48 permisos contra la lista seed de `infrastructure/database/seeds/permissions.rs`.
- `get_all_permissions_returns_seeded_permissions` compara el set completo contra `PermissionCode::all()`, no contra un número. **No lo edites al agregar un permiso**: se ajusta solo.

---

## 4. Convenciones de Código - TypeScript/Vue

### Estructura de archivos

- Componentes Vue: `PascalCase.vue`
- Archivos TypeScript: `camelCase.ts`
- Tipos/Interfaces: `camelCase.types.ts` o en el mismo archivo del módulo

### Imports

```typescript
// Usar comillas dobles
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { User } from "../../domain/entities";

// Importaciones de relativa
import App from "./App.vue";
import { helper } from "../utils/helper";
```

### Naming Conventions

```typescript
// Variables y funciones: camelCase
const userName = "Juan";
function getUserById(id: string): User {}

// Constantes: UPPER_SNAKE_CASE (para valores de configuración)
const MAX_RETRIES = 3;

// Types/Interfaces/Enums: PascalCase
interface UserProfile { ... }
type ApiResponse<T> = { ... }
enum Status { ... }

// Componentes Vue: PascalCase en el template
<UserCard />, <SettingsDialog />
```

### TypeScript Strict Mode

El proyecto tiene `strict: true` en tsconfig.json. Reglas activas:

- `noUnusedLocals: true` - No declarar variables sin usar
- `noUnusedParameters: true` - No tener parámetros sin usar
- `noFallthroughCasesInSwitch: true` - Todos los casos switch deben break/return

### Componentes Vue 3

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  title: string;
  count?: number;
}>();

const isLoading = ref(false);

const doubledCount = computed(() => (props.count ?? 0) * 2);

async function fetchData() {
  isLoading.value = true;
  try {
    const result = await invoke<string>("command_name", { id: 1 });
  } catch (error) {
    console.error(error);
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <!-- Template aquí -->
</template>

<style scoped>
/* Estilos scoped por defecto */
</style>
```

---

## 5. Convenciones de Código - Rust

### Estructura de archivos Rust

``` bash
src-tauri/src/
├── domain/entities/    # Structs con derive Serialize
├── domain/repositories/# Traits
├── application/services# Lógica de negocio
├── infrastructure/    # Implementaciones concretas
└── api/commands/       # Tauri commands
```

### Estilo de código

- **Indentación**: 4 espacios (no tabs)
- **Llaves**: Same-line para funciones, newline para structs/enums
- **Formato**: no editar el formato a mano; correr `cargo fmt`

```rust
fn greet(name: &str) -> String {
    format!("Hola, {}!", name)
}

struct User {
    name: String,
    age: u32,
}

enum Status {
    Active,
    Inactive,
}
```

### Naming Conventions Codes

- Funciones/variables: `snake_case`
- Structs/Enums/Traits: `PascalCase`
- Constantes: `SCREAMING_SNAKE_CASE`

### Macros y Atributos

```rust
// Los comandos de I/O usan #[tauri::command(async)]
#[tauri::command(async)]
pub fn get_all_users(
    user_id: i64,
    state: State<AppState>,
) -> Result<Vec<UserResponse>, AppError> {
    Ok(vec![])
}

// Los comandos sin I/O usan #[tauri::command]
#[tauri::command]
fn my_command(arg: String) -> Result<String, AppError> {
    Ok(arg)
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}
```

---

## 6. Integración Tauri (Frontend <-> Backend)

### Llamar comandos Rust desde Vue

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<UserResponse>("create_user", {
  userId: 1,
  request: { username: "test", password: "123" }
});
```

### Tipos compartidos

- Definir tipos TypeScript que correspondan a las structs de Rust
- Usar `serde` derive macros en Rust: `#[derive(Serialize, Deserialize)]`
- Los errores se serializan como string: `AppError` implementa `serde::Serialize` y llega a `invoke` como rechazo de la promesa
- Los tipos snake_case en Rust viajan como camelCase al frontend: `user_id` → `userId`. Recordarlo al escribir los tipos TS

---

## 7. Modelos de Base de Datos

### Ubicación del archivo

- **Producción**: `el-kioskon-app.db` en el directorio de datos de `ProjectDirs::from("", "", "com.elkioskon.app")`. Si `ProjectDirs` falla, cae a `./el-kioskon-app.db`.
- **Override**: la variable de entorno `ELKIOSKON_DB_PATH` (path absoluto o relativo) reemplaza la ruta por defecto. Sirve para apuntar a una DB de pruebas sin tocar la real:

```bash
# Linux/macOS
ELKIOSKON_DB_PATH=/tmp/scratch.db pnpm tauri dev
# PowerShell
$env:ELKIOSKON_DB_PATH="C:\temp\scratch.db"; pnpm tauri dev
```

- **Test**: `:memory:` (ver §3).

### Seeds

Al crear el esquema se siembran datos por defecto. **El documento anterior decía que solo se sembraba `admin`; eso es incorrecto.**

| Seed | Qué crea |
| ---- | -------- |
| `permissions.rs` | Los 48 permisos (lista canónica) |
| `admin.rs` | Usuario `admin` / `admin123` con los 48 permisos |
| `proveedor.rs` | Proveedor por defecto |
| `cliente.rs` | Cliente por defecto (usado por ventas sin cliente explícito) |
| `tipos_venta.rs` | Tipos de venta por defecto |
| `nocturno_config.rs` | Fila única de configuración nocturna (desactivada) |
| `demo_data.rs` | **6 usuarios demo**: `vendedor1`/`vendedor1`, `vendedor2`/`vendedor2`, `stock1`/`stock1`, `auditor1`/`auditor1`, `cajero1`/`cajero1`, `gerente1`/`gerente1`, cada uno con un subconjunto de permisos, más proveedores, clientes, artículos y stock de ejemplo |

`demo_data.rs` es lo primero que hay que revisar si un test o una pantalla ven datos inesperados.

### Tablas

21 tablas en total. DDL canónico en `infrastructure/database/schema.rs` (`SCHEMA_SQL`); la constante `TABLES` lo refleja y un test verifica que ambas cosas coincidan.

### Usuarios

- `id`: INTEGER PRIMARY KEY
- `username`: TEXT UNIQUE
- `password`: TEXT (hashed con bcrypt)
- `active`: INTEGER (0/1)
- `created_at`: TEXT (ISO 8601)
- `modified_at`: TEXT (ISO 8601)

### Permisos

- `id`: INTEGER PRIMARY KEY
- `permission`: TEXT UNIQUE
- `created`: TEXT (ISO 8601)

### user_permissions (relación muchos a muchos)

- `user_id`: INTEGER FK
- `permission_id`: INTEGER FK
- `assigned_at`: TEXT (ISO 8601)

### Proveedores

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `cuit`: TEXT UNIQUE (opcional)
- `proveedor`: TEXT NOT NULL
- `nombre`: TEXT NOT NULL
- `tel`, `email`, `observacion`: TEXT (opcionales)

### Clientes

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `nombre`, `apellido`, `telefono`, `email`, `direccion`: TEXT (todos opcionales)
- `created_at`, `updated_at`: TEXT NOT NULL (ISO 8601)

### Categorías

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `categoria`: TEXT UNIQUE

### Sub Categorías

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `sub_categoria`: TEXT UNIQUE
- `id_categoria`: INTEGER NOT NULL FK → categorias(id)

### Artículos

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `articulo`: TEXT NOT NULL UNIQUE
- `cod_articulo`: TEXT NOT NULL UNIQUE
- `id_sub_categoria`: INTEGER NOT NULL FK → sub_categorias(id)
- `id_proveedor`: INTEGER NOT NULL FK → proveedores(id)

### Stock

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `id_articulo`: INTEGER NOT NULL FK → articulos(id)
- `cantidad`: REAL NOT NULL
- `costo`: REAL NOT NULL
- `ganancia`: REAL NOT NULL
- `ganancia_diurna`: REAL NOT NULL DEFAULT 0
- `ganancia_nocturna`: REAL NOT NULL DEFAULT 0

Notas:

- `ganancia_diurna` / `ganancia_nocturna` existen desde la incorporación del recargo nocturno. El precio de venta toma la ganancia diurna o nocturna según si la operación cae dentro de la franja configurada en `nocturno_config`.

### tipos_venta

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `nombre`: TEXT NOT NULL UNIQUE
- `hacia_donde`: TEXT (opcional; a qué se entrega, ej. mostrador/mesa)
- `created_at`: TEXT NOT NULL (ISO 8601)

### Ventas

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `user_id`: INTEGER NOT NULL FK → users(id)
- `fecha`: TEXT NOT NULL
- `total`: REAL NOT NULL
- `descuento`: REAL NOT NULL DEFAULT 0
- `anulada`: INTEGER NOT NULL DEFAULT 0 (soft-delete)
- `observacion`: TEXT (opcional)
- `id_tipo_venta`: INTEGER (opcional) FK → tipos_venta(id)
- `cliente_id`: INTEGER NOT NULL FK → clientes(id)
- `created_at`: TEXT NOT NULL
- `porcentaje_nocturno`: REAL NOT NULL DEFAULT 0 (snapshot del recargo aplicado)

Notas:

- Una venta **sí** decrementa stock (a diferencia de un presupuesto).
- `cliente_id` es `NOT NULL` en la tabla, pero el request de `create_venta` lo recibe opcional: si viene `None`, el servicio usa el cliente por defecto.
- `anular_venta` no borra la fila, marca `anulada = 1`. El stock se repone y se registra auditoría.

### venta_detalle

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `id_venta`: INTEGER NOT NULL FK → ventas(id) ON DELETE CASCADE
- `id_articulo`: INTEGER NOT NULL FK → articulos(id)
- `cantidad`: REAL NOT NULL
- `costo_unitario`: REAL NOT NULL (snapshot del costo al momento de la venta)
- `precio_unitario`: REAL NOT NULL
- `subtotal`: REAL NOT NULL

### Presupuestos

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `user_id`: INTEGER FK → users(id)
- `fecha`: TEXT (ISO 8601)
- `total`: REAL
- `descuento`: REAL (0..=100)
- `estado`: TEXT CHECK (`pendiente` | `aprobado` | `vencido` | `convertido` | `anulado`), default `pendiente`
- `fecha_vencimiento`: TEXT (opcional, input de fecha en frontend; vacío → NULL)
- `observacion`: TEXT (opcional)
- `cliente_id`: INTEGER FK → clientes(id) (opcional/NULLable)
- `created_at`: TEXT (ISO 8601)

### detalle_presupuestos

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `id_presupuesto`: INTEGER FK → presupuestos(id) ON DELETE CASCADE
- `id_articulo`: INTEGER FK → articulos(id)
- `cantidad`, `costo_unitario`, `precio_unitario`, `subtotal`: REAL

Notas:

- Un presupuesto **no decrementa stock** (a diferencia de una venta).
- `cliente_id` y `fecha_vencimiento` son opcionales (`NULL`); a diferencia de `ventas`, que exige cliente.
- `precio_unitario`: si el carrito trae precio, se usa ese; si no, se calcula `costo * (1 + ganancia/100)` leyendo la tabla `stock`.
- Índices: `idx_detalle_presupuestos_id_presupuesto`, `idx_presupuestos_estado`.
- `estado` incluye `'anulado'` (soft-delete). El CHECK con `anulado` se aplica en bases existentes vía la migración idempotente `migrate_presupuestos_estado()` (en `infrastructure/database/migrations.rs`, ejecutada al final de `apply_schema`): reconstruye la tabla conservando `detalle_presupuestos`; es no-op si el SQL ya contiene `anulado`.
- Estados terminales: `convertido` y `anulado` son inmutables; `update_estado` los rechaza con `AppError::PresupuestoEstadoInvalido`.
- `get_all_presupuestos` acepta filtros server-side (`estado`, `fecha_desde`, `fecha_hasta`, `query` por id/cliente/usuario/artículo) vía `PresupuestoFilter`. La conversión a venta es frontend: carga los ítems en el carrito de Nueva Venta (`?presupuesto_id=N`) y tras una venta exitosa marca el presupuesto como `convertido`.

### Cierres

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `fecha`: TEXT NOT NULL UNIQUE
- `dia`, `mes`, `anio`: INTEGER NOT NULL
- `total_costo`, `total_ganancia`, `total_venta`: REAL NOT NULL
- `created_at`: TEXT NOT NULL

### cierre_tipos (desglose por tipo de venta)

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `id_cierre`: INTEGER NOT NULL FK → cierres(id) ON DELETE CASCADE
- `id_tipo_venta`: INTEGER NOT NULL FK → tipos_venta(id)
- `total`: REAL NOT NULL

Notas:

- `fecha` es UNIQUE: un día se cierra una sola vez. `is_dia_cerrado` lo consulta antes de permitir una venta.
- `reabrir_cierre` borra el cierre y su desglose para permitir operar ese día de nuevo.

### cost_update_operations

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `user_id`: INTEGER NOT NULL FK → users(id)
- `porcentaje`: REAL NOT NULL
- `filtro_categoria`, `filtro_sub_categoria`, `filtro_proveedor`: INTEGER (opcionales) FKs a sus tablas
- `affected_count`: INTEGER NOT NULL DEFAULT 0
- `estado`: TEXT NOT NULL DEFAULT `'aplicada'` CHECK (`aplicada` | `deshecha`)
- `created_at`: TEXT NOT NULL
- `undone_at`: TEXT (opcional)

### cost_update_items (snapshot para deshacer)

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `operation_id`: INTEGER NOT NULL FK → cost_update_operations(id) ON DELETE CASCADE
- `id_stock`: INTEGER NOT NULL FK → stock(id)
- `costo_anterior`, `costo_nuevo`: REAL NOT NULL

Notas:

- Una actualización masiva de costos guarda el costo previo de cada fila para poder revertirla con `undo_cost_update`.
- `cleanup_cost_update_operations` purga operaciones antiguas ya deshechas para que la tabla no crezca sin límite.

### audit_logs

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `user_id`: INTEGER NOT NULL
- `username`: TEXT NOT NULL (copia del nombre, sobrevive al borrado del usuario)
- `screen`: TEXT NOT NULL
- `action`: TEXT NOT NULL
- `detail`: TEXT (JSON serializado)
- `created_at`: TEXT NOT NULL

Notas:

- No tiene FK a `users` a propósito: el log es un registro de auditoría y debe sobrevivir al borrado del usuario.
- Retención de 90 días, aplicada por `purge_old_audit_logs()` en `infrastructure/database/maintenance.rs`.

### nocturno_config (singleton)

- `id`: INTEGER PRIMARY KEY CHECK (id = 1) — una sola fila
- `activo`: INTEGER NOT NULL DEFAULT 0
- `porcentaje`: REAL NOT NULL DEFAULT 0
- `hora_inicio`: TEXT NOT NULL DEFAULT `'22:00'`
- `hora_fin`: TEXT NOT NULL DEFAULT `'06:00'`

Notas:

- La franja `hora_inicio`/`hora_fin` cruza la medianoche (22:00 → 06:00), así que la comparación de "está en turno nocturno" tiene que manejar el wrap-around.

### Cotizaciones del dólar (historial circular)

- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `official_buy`: REAL NOT NULL
- `official_sell`: REAL NOT NULL
- `blue_buy`: REAL NOT NULL
- `blue_sell`: REAL NOT NULL
- `timestamp`: DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL

Reglas:

- El sistema conserva como máximo **4 filas** (`MAX_QUOTES` en `infrastructure/repositories/dollar_quote_repository.rs`).
- `save` es atómico (transacción): si `COUNT(*) >= 4` elimina la fila más antigua (`ORDER BY timestamp ASC, id ASC LIMIT 1`) y luego inserta la nueva.
- `find_all` devuelve hasta 4 filas ordenadas `timestamp DESC, id DESC`.
- `delete_by_id` deja temporalmente N-1 filas hasta la próxima ingesta de la API.
- El `timestamp` lo genera SQLite (`CURRENT_TIMESTAMP`, UTC); no se inserta explícitamente.

---

## 8. API Commands (Tauri)

74 comandos registrados en `tauri::generate_handler!` en `src-tauri/src/lib.rs`. **Esa lista es la fuente de verdad**: si agregás un comando, agregalo también acá o el test `docs_consistency` falla.

### Base y home

| Command | Descripción |
| --------- | ------------- |
| `ensure_db_ready` | Inicializa/verifica la DB al arrancar la app (no requiere `user_id`) |
| `get_home_stats` | Métricas del dashboard (ventas del día, ganancia, etc.) |

### Usuarios y permisos

| Command | Descripción |
| --------- | ------------- |
| `login` | Autenticar usuario (no requiere `user_id` previo) |
| `create_user` | Crear nuevo usuario |
| `get_all_users` | Listar todos los usuarios (excluye `admin`) |
| `update_user` | Actualizar usuario |
| `change_password` | Cambiar contraseña (propia o de otro usuario con permiso `cambiar_contrasena_usuario`) |
| `delete_user` | Eliminar usuario |
| `get_user_permissions` | Obtener permisos de un usuario |
| `get_all_permissions` | Listar todos los permisos |
| `add_permission_to_user` | Asignar permiso a usuario |
| `remove_permission_from_user` | Quitar permiso a usuario |
| `create_permission` | Crear nuevo permiso |

### Proveedores

| Command | Descripción |
| --------- | ------------- |
| `get_all_proveedores` | Listar proveedores |
| `get_proveedor_by_id` | Obtener proveedor por id |
| `create_proveedor` | Crear proveedor |
| `update_proveedor` | Actualizar proveedor |
| `delete_proveedor` | Eliminar proveedor |

### Categorías

| Command | Descripción |
| --------- | ------------- |
| `get_all_categorias` | Listar categorías |
| `create_categoria` | Crear categoría |
| `update_categoria` | Actualizar categoría |
| `delete_categoria` | Eliminar categoría (rechaza si tiene sub categorías) |

### Sub categorías

| Command | Descripción |
| --------- | ------------- |
| `get_all_sub_categorias` | Listar sub categorías |
| `get_sub_categorias_by_categoria` | Sub categorías de una categoría |
| `create_sub_categoria` | Crear sub categoría |
| `update_sub_categoria` | Actualizar sub categoría |
| `delete_sub_categoria` | Eliminar sub categoría |

### Artículos

| Command | Descripción |
| --------- | ------------- |
| `get_all_articulos` | Listar artículos |
| `create_articulo` | Crear artículo |
| `update_articulo` | Actualizar artículo |
| `delete_articulo` | Eliminar artículo |

### Stock

| Command | Descripción |
| --------- | ------------- |
| `get_all_stock` | Listar stock |
| `get_stock_by_id` | Obtener stock por id |
| `get_stock_by_articulo` | Obtener stock de un artículo |
| `create_stock` | Crear stock |
| `update_stock` | Actualizar stock |
| `delete_stock` | Eliminar stock |
| `get_precio_venta` | Calcular precio de venta de una fila de stock |

### Actualización masiva de costos

| Command | Descripción |
| --------- | ------------- |
| `get_stock_preview_costo` | Simular el efecto de un cambio de costo por porcentaje y filtros, sin escribir |
| `apply_costo_percentage_stock` | Aplicar el cambio y registrar la operación (para poder deshacer) |
| `get_last_undoable_cost_update` | Última operación aplicada todavía no deshecha, o `null` |
| `undo_cost_update` | Revertir una operación por `operation_id` |
| `cleanup_cost_update_operations` | Purgar operaciones antiguas ya deshechas |

### Auditoría

| Command | Descripción |
| --------- | ------------- |
| `get_audit_logs` | Listar logs de auditoría paginados con filtros |

### Ventas

| Command | Descripción |
| --------- | ------------- |
| `create_venta` | Crear venta (decrementa stock; usa el recargo nocturno si corresponde) |
| `get_all_ventas` | Listar ventas paginadas con detalle (filtros y orden) |
| `get_venta_by_id` | Obtener venta con detalle por id |
| `get_ventas_por_cliente` | Historial de ventas de un cliente |
| `anular_venta` | Anular venta (soft-delete, repone stock) |

### Tipos de venta

| Command | Descripción |
| --------- | ------------- |
| `get_all_tipos_venta` | Listar tipos de venta |
| `create_tipo_venta` | Crear tipo de venta |
| `update_tipo_venta` | Actualizar tipo de venta |
| `delete_tipo_venta` | Eliminar tipo de venta |

### Cierres

| Command | Descripción |
| --------- | ------------- |
| `crear_cierre` | Cerrar el día y calcular totales por tipo de venta |
| `get_all_cierres` | Listar cierres paginados con su desglose |
| `reabrir_cierre` | Reabrir un día cerrado (borra el cierre) |
| `is_dia_cerrado` | Indicar si una fecha está cerrada |

### Clientes

| Command | Descripción |
| --------- | ------------- |
| `get_all_clientes` | Listar clientes |
| `get_cliente_by_id` | Obtener cliente por id |
| `get_cliente_defecto` | Obtener el cliente por defecto (para ventas sin cliente explícito) |
| `crear_cliente` | Crear cliente |
| `actualizar_cliente` | Actualizar cliente |
| `eliminar_cliente` | Eliminar cliente |

### Cotizaciones del dólar

| Command | Descripción |
| --------- | ------------- |
| `get_dollar_quotes` | Obtener historial de cotizaciones del dólar (máx 4, más reciente primero) |
| `fetch_dollar_rates_manual` | Forzar actualización manual contra la API (guarda una fila nueva con rotación) |
| `delete_dollar_quote` | Eliminar una cotización por `id`; devuelve el historial restante |

### Recargo nocturno

| Command | Descripción |
| --------- | ------------- |
| `get_nocturno_config` | Leer la configuración nocturna (singleton) |
| `save_nocturno_config` | Guardar la configuración nocturna (permiso `configurar_recargo_nocturno`) |

### Respaldos

| Command | Descripción |
| --------- | ------------- |
| `crear_respaldo` | Copiar la DB a la ruta elegida por el usuario vía `VACUUM INTO` (permiso `gestionar_respaldos`) |
| `get_respaldo_info` | Tamaño y ruta del archivo de base de datos actual |

### Presupuestos

| Command | Descripción |
| --------- | ------------- |
| `crear_presupuesto` | Crear presupuesto (no decrementa stock; estado `pendiente`; `fecha_vencimiento`/`cliente_id` opcionales) |
| `get_all_presupuestos` | Listar presupuestos paginados con detalle (filtros `estado`, `fecha_desde`, `fecha_hasta`, `query`) |
| `get_presupuesto_by_id` | Obtener presupuesto con detalle por id |
| `cambiar_estado_presupuesto` | Cambiar estado del presupuesto (permiso `generar_presupuesto`; rechaza estados terminales `convertido`/`anulado`) |

---

## 9. Errores Comunes a Evitar

1. **No dejar variables sin usar** - TypeScript lo marca como error
2. **No usar `any`** - Usar tipos específicos o `unknown`
3. **No olvidar el `.value`** al acceder a refs de Vue
4. **En Rust, siempre manejar `Result` con `?` o match**
5. **No hardcodear secrets** - usar variables de entorno
6. **DB global** - `infrastructure::database::DB` es un `Lazy<Mutex<Connection>>` (rusqlite no es `Sync`); todos los repos lo bloquean. El esquema se crea en el primer arranque en el directorio de datos de `ProjectDirs` (`el-kioskon-app.db`), sin migraciones. Se siembran los 48 permisos, el usuario `admin` / `admin123` con todos ellos, y datos demo (ver §7). En builds de test apunta a `:memory:` y `BCRYPT_COST` baja a 4 (ver §3). Para probar contra otra ruta, usar `ELKIOSKON_DB_PATH`.
7. **Cuidado con el panic del lock global** - `DB.lock()` usa `.expect()` en producción a propósito: si el mutex queda envenenado, es un bug real y hay que enterarse. La excepción son los helpers de test (`fresh_test_db`, `reset_test_db`), que ignoran el poisoning para que un fallo no se propague (ver §3).
8. **Sincronizar permisos en 3 lugares** (strings en español snake_case, ej. `ver_usuarios`): Rust `PermissionCode::as_str()` (`domain/entities/permission_code.rs`), TS `PERMISSIONS` (`src/domain/entities/permissions.ts`) y la lista seed (`infrastructure/database/seeds/permissions.rs`). Los helpers de `usePermissions.ts` se generan automáticamente desde las claves de `PERMISSIONS` (`can` + clave en PascalCase, ej. `VIEW_USERS` → `canViewUsers`); NO agregar helpers a mano. El test `all_covers_seeded_permissions` verifica que `PermissionCode::all()` (48) esté sincronizado con la lista seed de Rust (no cubre TS). El test `docs_consistency` verifica que los 48 estén mencionados en este documento. Permisos destacados: `ver_dolar` habilita la pantalla de cotización (actualización **solo manual**, botón "Actualizar ahora" → `fetch_dollar_rates_manual`; no hay polling automático ni eventos de dólar), `configurar_recargo_nocturno` y `gestionar_respaldos`.
9. **No olvidar `user_id` en los comandos** - Todo comando recibe `user_id: i64` como primer argumento, salvo `login` y `ensure_db_ready`. Los de usuarios/permisos usan `AppState` + `UserService::has_permission`; los de dominio (articulo/categoria/...) usan el `check_permission` de `api/commands/permissions.rs`, que consulta la DB directo. Respetar el patrón al agregar comandos
10. **Registrar comandos y estados en `lib.rs`** - `.manage(...)` + `tauri::generate_handler!` + reexport en `api/commands/mod.rs`
11. **Auth en frontend** - Usuario y permisos se persisten en `sessionStorage` (`currentUser`, `userPermissions`). Los repos leen `getCurrentUserId()` (`infrastructure/utils/currentUser.ts`) y lo pasan como `userId` a cada `invoke`. El guard del router llama `authStore.loadFromStorage()`
12. **Repos y capas** - Los 16 repos implementan su contrato `I*Repository` y se registran en `infrastructure/di.ts` (única fuente de instancias; NO instanciar repos fuera de ahí). Los stores delegan en casos de uso de `application/usecases` (patrón módulo usuarios: store → usecase → repo inyectado). `infrastructure/api/index.ts` re-exporta todos los repos y `di.ts` (comodín de importación). El flujo presupuesto→venta vive en `ConvertirPresupuestoEnVenta` (usecase); refresco de stock tras venta/anulación se orquesta desde las páginas
13. **Archivos Rust en snake_case** - El nombre de archivo debe coincidir exactamente con el módulo declarado en `mod.rs` (ej. `categoria_repository.rs` para `pub mod categoria_repository;`). Un desajuste de mayúsculas compila en Windows/macOS por filesystem case-insensitive, pero rompe rust-analyzer y falla en Linux. Al renombrar solo mayúsculas usar `git mv` en dos pasos (nombre temporal → destino) porque `core.ignorecase=true`
14. **Código generado** - `src-tauri/gen/**` (proyecto Android) no se edita a mano
15. **No editar el doc a mano para "arreglarlo"** - Si un dato de este documento está mal, corregí el código y después el doc, y corré `cargo test` para confirmar que `docs_consistency` sigue conforme

---

## 10. IDE Recomendado

- **VS Code** con extensiones:
  - Vue - Official (Volar)
  - Tauri
  - rust-analyzer

> El repositorio **no** tiene ESLint ni Prettier configurados (no hay `.eslintrc*` ni `.prettierrc*`), aunque este documento antes los recomendaba. La verificación de formato y lint del frontend es `pnpm build`, que corre `vue-tsc --noEmit`. Para el backend, `cargo fmt --check` y clippy (ver §3).

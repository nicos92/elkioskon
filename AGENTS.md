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
│   ├── entities/     # User, Permission, PermissionCode, Proveedor, Categoria, SubCategoria, Articulo, Stock, Cliente, Venta, Presupuesto
│   └── repositories/# UserRepository, ArticuloRepository, CategoriaRepository, ProveedorRepository, StockRepository, SubCategoriaRepository, ClienteRepository, VentaRepository, PresupuestoRepository
├── application/     # Casos de uso/Servicios
│   └── services/    # UserService, ArticuloService, CategoriaService, ProveedorService, StockService, SubCategoriaService, ClienteService, VentaService, PresupuestoService
├── infrastructure/  # Implementaciones concretas
│   ├── database/    # Conexión SQLite
│   ├── repositories/# SqliteUserRepository, SqliteArticuloRepository, ...
│   └── error.rs     # AppError enum
└── api/             # Commands de Tauri
    └── commands/    # invoke handlers
```

### Frontend Vue (Clean Architecture)

``` bash
src/
├── domain/            # Tipos e interfaces
│   ├── entities/      # User, Permission, Proveedor, Categoria, SubCategoria, Articulo, Stock, Cliente, Venta, Presupuesto, PERMISSIONS, businessRules
│   └── interfaces/    # 14 contratos I*Repository (user, proveedor, categoria, subCategoria, articulo, stock, cliente, tipoVenta, venta, presupuesto, cierre, auditoria, home, dolar) + VentaQuery, CierreQuery, PresupuestoQuery
├── application/       # Casos de uso
│   └── usecases/      # Login, CreateUser, GetAllUsers, UpdateUser, DeleteUser, ChangePassword, ManagePermissions, ProveedorUseCase, CategoriaUseCase, SubCategoriaUseCase, ArticuloUseCase, StockUseCase, ClienteUseCase, TipoVentaUseCase, VentaUseCase, ConvertirPresupuestoEnVenta, PresupuestoUseCase, CierreUseCase, AuditUseCase, HomeUseCase, DollarUseCase
├── infrastructure/    # Implementaciones
│   ├── api/           # 14 repos (implementan los contratos I*Repository), errorHandler.ts
│   ├── di.ts          # Registra y exporta las instancias de repos (única fuente; los stores la consumen)
│   └── utils/         # currentUser.ts (getCurrentUserId)
└── presentation/      # Capa UI
    ├── layouts/       # MainLayout con sidebar
    ├── pages/         # Login, Home, Users, Proveedores, Categorias, SubCategorias, Articulos, Stock, Permissions, Settings, Ventas, NuevaVenta, Presupuestos, Clientes, Cierres, Dolar, Auditoria
    ├── stores/        # Pinia stores (auth, users, permissions, proveedores, categorias, subCategorias, articulos, stock, theme, clientes, ventas, presupuestos, dolar, cierres, auditoria, home, tiposVenta). Delegan en casos de uso; reciben el repo por constructor (di.ts)
    ├── composables/   # usePermissions, usePagination, useCart, useConfirm, useToasts; venta/ (usePreciosVenta, useClienteSeleccion, usePresupuestoOrigen)
    ├── components/    # ui/ (Modal, DataTable, PaginationBar, ConfirmButton, PageHeader, SearchBar, EntityFormModal), venta/ (ArticuloSearch, CartTable, ClienteSelector, NuevoClienteModal, PresupuestoPrintArea, TotalsPanel), Toasts, TopBar, ConfirmDialog
    └── router/        # Vue Router config
```

---

## 3. Comandos de Build y Desarrollo

### Comandos principales (frontend)

```bash
pnpm dev                          # Inicia el servidor Vite en http://localhost:1420
pnpm build                        # TypeScript check + build de producción
pnpm preview                      # Previsualizar build de producción
```

> Nota: `pnpm build` corre `vue-tsc --noEmit` (typecheck) y luego `vite build`. No hay script de lint ni de tests.

### Comandos Tauri

```bash
pnpm tauri dev                    # Desarrollo Tauri (frontend + backend)
pnpm tauri build                  # Build de producción Tauri
```

### Comandos Rust (directos)

```bash
cd src-tauri && cargo check       # Verificar código sin compilar
cd src-tauri && cargo build        # Compilar
cd src-tauri && cargo test         # Ejecutar tests (unitarios + repos con DB en memoria)
cd src-tauri && cargo clippy --lib --tests  # Linter de Rust
```

### Tests (Rust)

- **Nunca tocan la DB real**: en builds de test `get_db_path()` devuelve `:memory:` y `BCRYPT_COST` = 4. `app.db` queda intacta.
- **Tests que tocan la DB** (repositorios, servicios): adquirir `TEST_LOCK` (serializa los tests de DB) y llamar `reset_test_db()` (borra tablas y recrea esquema + seeds). Ambas viven en `infrastructure/database/mod.rs` bajo `#[cfg(test)]`.

```rust
let _guard = TEST_LOCK.lock().unwrap();
reset_test_db().unwrap();
```

- Tests unitarios puros (sin DB) se escriben como módulos `#[cfg(test)] mod tests` en el mismo archivo: entidades, `AppError`, `PermissionCode`.
- `PermissionCode::all()` tiene un test que verifica los 46 permisos contra la lista seed de `infrastructure/database/mod.rs`.

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
type ApiResponse<T> = { ... };
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
  request: { username: "test", password: "123" }
});
```

### Tipos compartidos

- Definir tipos TypeScript que correspondan a las structs de Rust
- Usar `serde` derive macros en Rust: `#[derive(Serialize, Deserialize)]`
- Los errores se serializan como string: `AppError` implementa `serde::Serialize` y llega a `invoke` como rechazo de la promesa

---

## 7. Modelos de Base de Datos

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

- `id`: INTEGER PRIMARY KEY
- `cuit`: TEXT UNIQUE (opcional)
- `proveedor`: TEXT NOT NULL
- `nombre`: TEXT NOT NULL
- `tel`, `email`, `observacion`: TEXT (opcionales)

### Categorías

- `id`: INTEGER PRIMARY KEY
- `categoria`: TEXT UNIQUE

### Sub Categorías

- `id`: INTEGER PRIMARY KEY
- `sub_categoria`: TEXT UNIQUE
- `id_categoria`: INTEGER FK → categorias(id)

### Artículos

- `id`: INTEGER PRIMARY KEY
- `articulo`: TEXT UNIQUE
- `cod_articulo`: TEXT UNIQUE
- `id_sub_categoria`: INTEGER FK → sub_categorias(id)
- `id_proveedor`: INTEGER FK → proveedores(id)

### Stock

- `id`: INTEGER PRIMARY KEY
- `id_articulo`: INTEGER FK → articulos(id)
- `cantidad`: REAL
- `costo`: REAL
- `ganancia`: REAL

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
- `estado` incluye `'anulado'` (soft-delete). El CHECK con `anulado` se aplica en bases existentes vía la migración idempotente `migrate_presupuestos_estado()` (en `infrastructure/database/mod.rs`, ejecutada al final de `apply_schema`): reconstruye la tabla conservando `detalle_presupuestos`; es no-op si el SQL ya contiene `anulado`.
- Estados terminales: `convertido` y `anulado` son inmutables; `update_estado` los rechaza con `AppError::PresupuestoEstadoInvalido`.
- `get_all_presupuestos` acepta filtros server-side (`estado`, `fecha_desde`, `fecha_hasta`, `query` por id/cliente/usuario/artículo) vía `PresupuestoFilter`. La conversión a venta es frontend: carga los ítems en el carrito de Nueva Venta (`?presupuesto_id=N`) y tras una venta exitosa marca el presupuesto como `convertido`.

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

| Command | Descripción |
| --------- | ------------- |
| `login` | Autenticar usuario |
| `create_user` | Crear nuevo usuario |
| `get_all_users` | Listar todos los usuarios |
| `update_user` | Actualizar usuario |
| `change_password` | Cambiar contraseña (propia o de otro usuario con permiso `cambiar_contrasena_usuario`) |
| `delete_user` | Eliminar usuario |
| `get_user_permissions` | Obtener permisos de un usuario |
| `get_all_permissions` | Listar todos los permisos |
| `add_permission_to_user` | Asignar permiso a usuario |
| `remove_permission_from_user` | Quitar permiso a usuario |
| `create_permission` | Crear nuevo permiso |
| `get_all_proveedores` | Listar proveedores |
| `get_proveedor_by_id` | Obtener proveedor por id |
| `create_proveedor` | Crear proveedor |
| `update_proveedor` | Actualizar proveedor |
| `delete_proveedor` | Eliminar proveedor |
| `get_all_categorias` | Listar categorías |
| `create_categoria` | Crear categoría |
| `update_categoria` | Actualizar categoría |
| `delete_categoria` | Eliminar categoría |
| `get_all_sub_categorias` | Listar sub categorías |
| `get_sub_categorias_by_categoria` | Sub categorías de una categoría |
| `create_sub_categoria` | Crear sub categoría |
| `update_sub_categoria` | Actualizar sub categoría |
| `delete_sub_categoria` | Eliminar sub categoría |
| `get_all_articulos` | Listar artículos |
| `create_articulo` | Crear artículo |
| `update_articulo` | Actualizar artículo |
| `delete_articulo` | Eliminar artículo |
| `get_all_stock` | Listar stock |
| `get_stock_by_id` | Obtener stock por id |
| `get_stock_by_articulo` | Obtener stock de un artículo |
| `create_stock` | Crear stock |
| `update_stock` | Actualizar stock |
| `delete_stock` | Eliminar stock |
| `get_precio_venta` | Calcular precio de venta |
| `crear_presupuesto` | Crear presupuesto (no decrementa stock; estado `pendiente`; `fecha_vencimiento`/`cliente_id` opcionales) |
| `get_all_presupuestos` | Listar presupuestos paginados con detalle (filtros `estado`, `fecha_desde`, `fecha_hasta`, `query`) |
| `get_presupuesto_by_id` | Obtener presupuesto con detalle por id |
| `cambiar_estado_presupuesto` | Cambiar estado del presupuesto (permiso `generar_presupuesto`; rechaza estados terminales `convertido`/`anulado`) |
| `get_dollar_quotes` | Obtener historial de cotizaciones del dólar (máx 4, más reciente primero) |
| `fetch_dollar_rates_manual` | Forzar actualización manual contra la API (guarda una fila nueva con rotación) |
| `delete_dollar_quote` | Eliminar una cotización por `id` del historial |

---

## 9. Errores Comunes a Evitar

1. **No dejar variables sin usar** - TypeScript lo marca como error
2. **No usar `any`** - Usar tipos específicos o `unknown`
3. **No olvidar el `.value`** al acceder a refs de Vue
4. **En Rust, siempre manejar `Result` con `?` o match**
5. **No hardcodear secrets** - usar variables de entorno
6. **DB global** - `infrastructure::database::DB` es un `Lazy<Mutex<Connection>>` (rusqlite no es `Sync`); todos los repos lo bloquean. El esquema se crea en el primer arranque en el directorio de datos de `ProjectDirs` (`app.db`), sin migraciones. Se siembran 46 permisos y el usuario `admin` / `admin123` con todos los permisos. En builds de test apunta a `:memory:` y `BCRYPT_COST` baja a 4 (ver Tests en §3)
7. **Sincronizar permisos en 3 lugares** (strings en español snake_case, ej. `ver_usuarios`): Rust `PermissionCode::as_str()` (`domain/entities/permission_code.rs`), TS `PERMISSIONS` (`src/domain/entities/permissions.ts`) y la lista seed (`infrastructure/database/mod.rs`). Los helpers de `usePermissions.ts` se generan automáticamente desde las claves de `PERMISSIONS` (`can` + clave en PascalCase, ej. `VIEW_USERS` → `canViewUsers`); NO agregar helpers a mano. El test `all_covers_seeded_permissions` verifica que `PermissionCode::all()` (46) esté sincronizado con la lista seed de Rust (no cubre TS). El permiso nuevo `ver_dolar` permite acceder a la pantalla de cotización del dólar; la actualización es **solo manual** (botón "Actualizar ahora" → `fetch_dollar_rates_manual`). No hay polling automático ni eventos de dólar.
8. **No olvidar `user_id` en los comandos** - Todo comando recibe `user_id: i64` como primer argumento. Los de usuarios/permisos usan `AppState` + `UserService::has_permission`; los de dominio (articulo/categoria/...) duplican un `check_permission` propio que consulta la DB directo. Respetar el patrón al agregar comandos
9. **Registrar comandos y estados en `lib.rs`** - `.manage(...)` + `tauri::generate_handler!` en `src-tauri/src/lib.rs`
10. **Auth en frontend** - Usuario y permisos se persisten en `sessionStorage` (`currentUser`, `userPermissions`). Los repos leen `getCurrentUserId()` (`infrastructure/utils/currentUser.ts`) y lo pasan como `userId` a cada `invoke`. El guard del router llama `authStore.loadFromStorage()`
11. **Repos y capas** - Los 14 repos implementan su contrato `I*Repository` y se registran en `infrastructure/di.ts` (única fuente de instancias; NO instanciar repos fuera de ahí). Los stores delegan en casos de uso de `application/usecases` (patrón módulo usuarios: store → usecase → repo inyectado). `infrastructure/api/index.ts` re-exporta todos los repos y `di.ts` (comodín de importación). El flujo presupuesto→venta vive en `ConvertirPresupuestoEnVenta` (usecase); refresco de stock tras venta/anulación se orquesta desde las páginas
12. **Archivos Rust en snake_case** - El nombre de archivo debe coincidir exactamente con el módulo declarado en `mod.rs` (ej. `categoria_repository.rs` para `pub mod categoria_repository;`). Un desajuste de mayúsculas compila en Windows/macOS por filesystem case-insensitive, pero rompe rust-analyzer y falla en Linux. Al renombrar solo mayúsculas usar `git mv` en dos pasos (nombre temporal → destino) porque `core.ignorecase=true`
13. **Código generado** - `src-tauri/gen/**` (proyecto Android) no se edita a mano

---

## 10. IDE Recomendado

- **VS Code** con extensiones:
  - Vue - Official (Volar)
  - Tauri
  - rust-analyzer
  - ESLint
  - Prettier

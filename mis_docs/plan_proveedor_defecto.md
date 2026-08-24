# Plan: Proveedor por defecto "Sin Proveedor"

## Objetivo

Crear un seed idempotente que genere un proveedor por defecto llamado **"Sin Proveedor"**,
replicando el patrón del cliente por defecto **"Consumidor Final"**. Este registro:

- No se puede **eliminar** ni **modificar** (a diferencia del cliente, que sí permite edición).
- Sirve como proveedor asignado a los artículos que no tienen un proveedor real.
- Se preselecciona automáticamente al crear un artículo nuevo.

## Estado actual

- `articulos.id_proveedor` es `NOT NULL` → todo artículo requiere proveedor.
- En `ArticulosPage.vue` el listado ya muestra el texto de fallback `"Sin proveedor"`
  cuando no encuentra el proveedor en el mapa (línea ~107), pero ese valor no existe
  como registro real.
- El patrón cliente por defecto está implementado en:
  - Seed: `src-tauri/src/infrastructure/database/seeds/cliente.rs`
  - Protección: `ClienteService::delete` (`AppError::NoSePuedeEliminarClienteDefecto`)
  - UI: badge + guard en `ClientesPage.vue`, helpers `isDefaultClient` /
    `DEFAULT_CLIENT_LABEL` en `src/domain/entities/businessRules.ts`

## Cambios Backend (Rust)

### 1. Nuevo seed `seeds/proveedor.rs`

Archivo nuevo `src-tauri/src/infrastructure/database/seeds/proveedor.rs`:

```rust
use rusqlite::Connection;

pub(crate) fn seed_proveedor_defecto(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO proveedores (proveedor, nombre)
         SELECT 'Sin Proveedor', 'Sin Proveedor'
         WHERE NOT EXISTS (
             SELECT 1 FROM proveedores WHERE proveedor = 'Sin Proveedor'
         )",
        [],
    )?;
    Ok(())
}
```

- Idempotente (mismo patrón que `cliente.rs`): si se ejecuta varias veces no duplica.
- Registrar en `seeds/mod.rs`: `mod proveedor;` + re-export.
- Llamarlo en `connection.rs::initialize()` justo después de `seed_cliente_defecto`
  y antes de `seed_demo_data`. Corre también sobre instalaciones existentes al arrancar
  (no requiere migración).

### 2. Entidad `domain/entities/proveedor.rs`

- Constante pública `DEFAULT_PROVEEDOR_NOMBRE: &str = "Sin Proveedor"`.
- Método `is_default(&self) -> bool` comparando `self.proveedor`.
- Tests unitarios: true para "Sin Proveedor", false para otros.

La identificación es por nombre; es estable porque la edición queda bloqueada
(misma debilidad aceptada en el patrón cliente: un usuario no puede crear otro
proveedor con exactamente ese nombre y esperar eliminarlo).

### 3. Errores `infrastructure/error.rs`

Dos variantes nuevas (con entradas en `code()` y `user_message()`):

| Variante | code | mensaje usuario |
| -------- | ----- | --------------- |
| `NoSePuedeModificarProveedorDefecto` | `no_se_puede_modificar_proveedor_defecto` | No se puede modificar el proveedor 'Sin Proveedor'. |
| `NoSePuedeEliminarProveedorDefecto` | `no_se_puede_eliminar_proveedor_defecto` | No se puede eliminar el proveedor 'Sin Proveedor'. |

El frontend muestra `message` del error vía `toErrorMessage`, así que el toast
sale sin cambios extra.

### 4. Servicio `application/services/proveedor_service.rs`

- `update()`: tras cargar `existing`, si `existing.is_default()` → error.
- `delete()`: tras cargar `existing`, si `existing.is_default()` → error.
- No hace falta agregar `find_default()` al trait `ProveedorRepository`:
  el servicio ya carga la entidad y compara con `is_default()`. Menos superficie de cambio.
- Tests de servicio:
  - modificar el default falla (`NoSePuedeModificarProveedorDefecto`)
  - eliminar el default falla (`NoSePuedeEliminarProveedorDefecto`)
  - eliminar un proveedor normal sigue funcionando

## Cambios Frontend

### 5. Dominio `src/domain/entities/businessRules.ts`

```ts
export const DEFAULT_PROVEEDOR = "Sin Proveedor";

export function isDefaultProveedor(proveedor: Proveedor): boolean {
  return proveedor.proveedor === DEFAULT_PROVEEDOR;
}
```

(`index.ts` ya re-exporta `businessRules.ts` con `export *`.)

### 6. `ProveedoresPage.vue`

- Badge "por defecto" junto a la razón social (reutilizar estilo `.default-badge`
  de ClientesPage).
- Ocultar botón Editar y ConfirmButton de eliminar cuando `isDefaultProveedor(proveedor)`.
- Guards defensivos en `handleUpdate` / `handleDelete` con toast de error.

### 7. `ArticulosPage.vue`

En `openCreateModal()`: preseleccionar el id del proveedor por defecto:

```ts
const def = proveedoresStore.proveedores.find(isDefaultProveedor);
newIdProveedor.value = def?.id ?? null;
```

El usuario puede cambiar la selección en el select. Si el store aún no cargó,
queda `null` y la validación existente exige elegir uno.

## Efectos colaterales verificados

- Los tests Rust usan `reset_test_db()` → `initialize()` → corre el seed nuevo;
  ningún test aserta conteos totales de proveedores, así que no se rompen.
- HomePage cuenta `total_proveedores`: ahora incluye el default (+1). Informativo, aceptado.
- La actualización masiva de costos filtra por `filtro_proveedor`; "Sin Proveedor"
  aparece como opción más en el select, comportamiento deseado.

## Verificación

```bash
cd src-tauri && cargo test && cargo clippy --lib --tests
pnpm build   # vue-tsc --noEmit + vite build
```

## Checklist

- [ ] Seed idempotente creado y registrado
- [ ] Seed ejecutado en `initialize()` antes de demo data
- [ ] Entidad con `is_default()` + tests
- [ ] Variantes de error con code + user_message
- [ ] Servicio bloquea update/delete del default + tests
- [ ] `isDefaultProveedor` / `DEFAULT_PROVEEDOR` en dominio TS
- [ ] ProveedoresPage: badge + botones ocultos + guards
- [ ] ArticulosPage: preselección al crear
- [ ] `cargo test` + `cargo clippy` OK
- [ ] `pnpm build` OK

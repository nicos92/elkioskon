# Implementación: Auditoría con detalle estructurado (JSON antes/después)

Documento de referencia para **replicar** en una aplicación prácticamente idéntica
los cambios realizados para el requerimiento `mis_docs/auditorio_detalles.md`.

## Objetivo

Reemplazar los IDs internos poco descriptivos (ej. `id_articulo` en stock) por
códigos/nombres legibles, y guardar en `audit_logs.detail` un **JSON estructurado
con antes/después** en los eventos `modificar`, de modo que la pantalla de Auditoría
pueda renderizar un desglose legible ("Antes → Después").

## Decisiones confirmadas

- **No hay cambios de esquema**: `detail` ya es `TEXT`. Solo se enriquece el contenido.
- **Dos formatos** en `detail`:
  - Texto plano descriptivo → eventos `nuevo`/`eliminar` y detalles simples.
  - JSON → eventos `modificar` y el guardado de recargo nocturno.
- **Históricos de stock con IDs crudos**: se dejan como están (sin migración).
- Los `eliminar` resuelven el nombre/código **antes** de borrar (vía `get_by_id`
  del service; si no existiera, fallback al ID).

### Formato JSON

```json
{
  "tipo": "stock",
  "descripcion": "Coca Cola 1.5L (cód. CC1500)",
  "cambios": [
    { "campo": "cantidad", "antes": "50", "valor": "40" },
    { "campo": "costo", "antes": "100", "valor": "110" }
  ]
}
```

`tipo` usado por dominio (constante `&'static str`): `stock`, `recargo_nocturno`,
`usuario`, `proveedor`, `categoria`, `sub_categoria`, `articulo`, `cliente`,
`tipo_venta`, `presupuesto`. Todos los valores de `cambios` son strings.

---

## Backend (Rust)

### 1. Nuevo módulo — `src-tauri/src/application/services/audit_detail.rs`

Fuente completa (nuevo archivo):

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditCambio {
    pub campo: &'static str,
    pub antes: String,
    pub valor: String,
}

impl AuditCambio {
    pub fn new(campo: &'static str, antes: impl ToString, valor: impl ToString) -> Self {
        Self {
            campo,
            antes: antes.to_string(),
            valor: valor.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditDetail {
    pub tipo: &'static str,
    pub descripcion: String,
    pub cambios: Vec<AuditCambio>,
}

impl AuditDetail {
    pub fn new(tipo: &'static str, descripcion: impl Into<String>) -> Self {
        Self {
            tipo,
            descripcion: descripcion.into(),
            cambios: Vec::new(),
        }
    }

    pub fn cambio(
        mut self,
        campo: &'static str,
        antes: impl ToString,
        valor: impl ToString,
    ) -> Self {
        self.cambios.push(AuditCambio::new(campo, antes, valor));
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serializar AuditDetail a JSON")
    }
}

pub fn opt_str(val: &Option<String>) -> String {
    val.clone().unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Cambio {
        campo: String,
        antes: String,
        valor: String,
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        tipo: String,
        descripcion: String,
        cambios: Vec<Cambio>,
    }

    #[test]
    fn to_json_persiste_campos_y_cambios() {
        let json = AuditDetail::new("recargo_nocturno", "Recargo nocturno")
            .cambio("activo", false, true)
            .cambio("porcentaje", 0.0, 12.5)
            .cambio("hora_inicio", "22:00", "23:00")
            .cambio("hora_fin", "06:00", "07:00")
            .to_json();

        let payload: Payload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.tipo, "recargo_nocturno");
        assert_eq!(payload.descripcion, "Recargo nocturno");
        assert_eq!(payload.cambios.len(), 4);
        assert_eq!(payload.cambios[0].campo, "activo");
        assert_eq!(payload.cambios[0].antes, "false");
        assert_eq!(payload.cambios[0].valor, "true");
        assert_eq!(payload.cambios[2].campo, "hora_inicio");
        assert_eq!(payload.cambios[2].valor, "23:00");
    }

    #[test]
    fn to_json_genera_objeto_con_campos_esperados() {
        let json = AuditDetail::new("categoria", "Bebidas")
            .cambio("categoria", "Bebidas", "Bebidas y aguas")
            .to_json();

        assert!(json.starts_with('{'));
        assert!(json.contains("\"tipo\":\"categoria\""));
        assert!(json.contains("\"descripcion\":\"Bebidas\""));
        assert!(json.contains("\"campo\":\"categoria\""));
        assert!(json.contains("\"antes\":\"Bebidas\""));
        assert!(json.contains("\"valor\":\"Bebidas y aguas\""));
    }

    #[test]
    fn opt_str_muestra_guion_cuando_none() {
        assert_eq!(opt_str(&None), "-");
        assert_eq!(opt_str(&Some("abc".to_string())), "abc");
    }
}
```

Registrarlo en `src-tauri/src/application/services/mod.rs`:

```rust
pub mod audit_detail;
...
pub use audit_detail::{opt_str, AuditCambio, AuditDetail};
```

### 2. Patrón para `modificar` (antes/después)

En cada comando `update_*`:

1. `use` de los nuevos ítems: `crate::application::services::{log_audit, opt_str, AuditDetail, XService}`.
2. Antes de mutar: `let antes = service.get_by_id(request.id)?;`
3. Tras mutar: armar el detail y pasar `Some(detail)` a `log_audit`.

```rust
let antes = service.get_by_id(request.id)?;
let result = service.update(request.id, request.categoria)?;
let detail = AuditDetail::new("categoria", format!("{} (id {})", result.categoria, result.id))
    .cambio("categoria", &antes.categoria, &result.categoria)
    .to_json();
log_audit(user_id, AuditScreen::Categorias, AuditAction::Update, Some(detail))?;
```

- `cambio()` toma `impl ToString` en ambos lados → para campos `String` pasar
  **por referencia** (`&antes.x`, `&result.x`). Si se pasa el `String` por valor se
  produce `E0382 use of partially moved value` al devolver `Ok(result)`.
- `bool`, `f64`, `i64`, `&str`, `String` (por ref) y `Option<String>` (vía `opt_str`)
  se serializan bien.
- `opt_str(&opcional)` muestra `"-"` si es `None`.

### 3. Patrón para `eliminar` (resolver antes de borrar)

```rust
let antes = service.get_by_id(id)?;
service.delete(id)?;
log_audit(
    user_id,
    AuditScreen::Categorias,
    AuditAction::Delete,
    Some(format!("Categoría eliminada: {} (id {})", antes.categoria, id)),
)?;
```

### 4. Resolver nombres/códigos (helpers con consulta directa a `DB`)

Para stock, artículos, sub-categorías y permisos se agregan helpers `fn` en el
mismo archivo del comando, consultando la tabla relacionada con `rusqlite`.

Stock (`stock_commands.rs`), núcleo del requerimiento (usa `cod_articulo` en vez del ID):

```rust
fn articulo_label(id_articulo: i64) -> Result<String, AppError> {
    query_articulo_label("SELECT articulo, cod_articulo FROM articulos WHERE id = ?1", id_articulo)
}

fn articulo_label_by_stock(id_stock: i64) -> Result<String, AppError> {
    query_articulo_label(
        "SELECT a.articulo, a.cod_articulo
         FROM stock s JOIN articulos a ON a.id = s.id_articulo
         WHERE s.id = ?1",
        id_stock,
    )
}

fn query_articulo_label(sql: &str, param: i64) -> Result<String, AppError> {
    let conn = DB.lock().map_err(|e| AppError::Internal(e.to_string()))?;

    let (articulo, cod): (String, String) = conn
        .query_row(sql, params![param], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(format!("{} (cód. {})", articulo, cod))
}
```

Requiere `use rusqlite::params;` y `use crate::infrastructure::database::DB;`.
Helpers análogos agregados por archivo:

| Archivo | Helper | Consulta |
| ------------- | ------------- | ------------- |
| `stock_commands.rs` | `articulo_label`, `articulo_label_by_stock` | `articulos.articulo` + `cod_articulo` (JOIN en stock para la baja) |
| `articulo_commands.rs` | `sub_categoria_name`, `proveedor_name` | `sub_categorias.sub_categoria`, `proveedores.nombre` |
| `sub_categoria_commands.rs` | `categoria_name` | `categorias.categoria` |
| `mod.rs` (usuarios/permisos) | `permission_code_by_id` | `permissions.permission` |
| `proveedor_commands.rs` | `proveedor_label` | — (arma label desde la entidad) |
| `cliente_commands.rs` | `cliente_label(&Cliente)` | — (arma label `Nombre Apellido`) |
| `presupuesto_commands.rs` | `cliente_label(&Option<String>, &Option<String>)` | — (label del snapshot de cliente) |
| `venta_commands.rs` | `venta_cliente_label(&Option<String>, &Option<String>)` | — (label del snapshot de cliente) |

> Nota: `cliente_label` de `cliente_commands.rs` recibe `&Cliente`; la de
> presupuesto/venta recibe los dos `&Option<String>` del snapshot. Son helpers
> locales distintos (mismo nombre en archivos distintos, sin conflicto).

### 5. Resumen por archivo de comandos

| Archivo | `nuevo` | `modificar` | `eliminar` |
| ------------- | ------------- | ------------- | ------------- |
| `stock_commands.rs` | "Stock creado: {label}, cantidad=..., costo=..., ganancia=..." | JSON `stock` (cantidad, costo, ganancia) | "Stock eliminado: {label}" (label vía JOIN pre-delete) |
| `nocturno_commands.rs` | — | JSON `recargo_nocturno` (activo, porcentaje, hora_inicio, hora_fin) con `service.get()` antes de `save` | — |
| `mod.rs` (usuarios) | "Usuario creado: {username} (id {id})" | JSON `usuario` (username, activo); change_password con username | "Usuario eliminado: {username} (id {id})" |
| `mod.rs` (permisos) | "Permiso: {code} (id {id})" | asignar/revocar: "Permiso {code} asignado/quitado al usuario {username} (id {id})" | — |
| `proveedor_commands.rs` | "Proveedor creado: {proveedor_label}" | JSON `proveedor` (proveedor, nombre, cuit, tel, email, observacion) | "Proveedor eliminado: {label}" |
| `categoria_commands.rs` | "Categoría creada: {cat} (id {id})" | JSON `categoria` | "Categoría eliminada: {cat} (id {id})" |
| `sub_categoria_commands.rs` | "Sub categoría creada: {sub} (id {id}) de categoría {categoria}" | JSON `sub_categoria` (sub_categoria, categoria) | "Sub categoría eliminada: {sub} (id {id}) de categoría {categoria}" |
| `articulo_commands.rs` | "Artículo creado: {art} (cód. {cod}) sub-categoría {x}, proveedor {y} (id {id})" | JSON `articulo` (articulo, cod_articulo, sub_categoria, proveedor) | "Artículo eliminado: {art} (cód. {cod}) (id {id})" |
| `cliente_commands.rs` | "Cliente creado: {label} (id {id})" | JSON `cliente` (nombre, apellido, telefono, email, direccion) | "Cliente eliminado: {label} (id {id})" |
| `tipo_venta_commands.rs` | "Tipo de venta creado: {nombre} (id {id})" | JSON `tipo_venta` (nombre, hacia_donde) | "Tipo de venta eliminado: {nombre} (id {id})" |
| `presupuesto_commands.rs` | "Presupuesto creado: id {id}, total=${total}, {n} ítems, cliente {label}" | JSON `presupuesto` (solo `estado` en cambiar_estado) | — |
| `venta_commands.rs` | "Venta creada: id {id}, total=${total}, {n} ítems, cliente {label}" | anular: "Venta {id} anulada (total=..., clientes...)" | — |
| `dollar_commands.rs` | — | fetch manual con valores (oficial/blue buy/sell) | borrado con valores de la cotización |

### 6. Errores típicos de compilación

- `E0382 use of partially moved value` → campos `String` pasados por valor a
  `cambio()`; agregar `&` al `antes`/`valor` cuando la entidad se devuelve después.
- `E0277` en `serde_json::to_string` → la struct `AuditDetail` necesita
  `#[derive(Serialize)]` (no olvidarla al copiar).
- `E0277` en tests de `audit_detail` → no derivar `Deserialize` sobre tipos con
  `campo: &'static str`; declarar structs locales `Cambio`/`Payload` con `String`
  (así está en el código de referencia).

---

## Frontend (Vue)

### 1. Nuevo archivo — `src/presentation/utils/auditDetail.ts`

Fuente completa (nuevo archivo):

```ts
export interface AuditCambio {
  campo: string;
  antes: string;
  valor: string;
}

export interface AuditDetail {
  tipo: string;
  descripcion: string;
  cambios: AuditCambio[];
}

const FIELD_LABELS: Record<string, string> = {
  cantidad: "Cantidad",
  costo: "Costo",
  ganancia: "Ganancia",
  activo: "Activo",
  porcentaje: "Porcentaje",
  hora_inicio: "Hora inicio",
  hora_fin: "Hora fin",
  username: "Usuario",
  proveedor: "Proveedor",
  nombre: "Nombre",
  cuit: "CUIT",
  tel: "Teléfono",
  email: "Email",
  observacion: "Observación",
  sub_categoria: "Sub categoría",
  categoria: "Categoría",
  articulo: "Artículo",
  cod_articulo: "Código",
  apellido: "Apellido",
  telefono: "Teléfono",
  direccion: "Dirección",
  hacia_donde: "Hacia dónde",
  estado: "Estado",
};

export function parseAuditDetail(detail: string | null): AuditDetail | null {
  if (!detail) return null;
  const trimmed = detail.trim();
  if (!trimmed.startsWith("{")) return null;
  try {
    const parsed: unknown = JSON.parse(trimmed);
    if (typeof parsed !== "object" || parsed === null) return null;
    const { tipo, descripcion, cambios } = parsed as AuditDetail;
    if (typeof tipo !== "string" || typeof descripcion !== "string") return null;
    if (!Array.isArray(cambios)) return null;
    return { tipo, descripcion, cambios };
  } catch {
    return null;
  }
}

export function auditFieldLabel(campo: string): string {
  return FIELD_LABELS[campo] ?? campo.replace(/_/g, " ");
}

export function formatAuditValue(campo: string, value: string): string {
  if (campo === "activo") {
    if (value === "true") return "Sí";
    if (value === "false") return "No";
  }
  if (campo === "porcentaje") return `${value}%`;
  return value;
}
```

`parseAuditDetail` devuelve `null` para detalle legacy/texto plano → la UI lo
muestra como texto (retrocompatible).

### 2. Render en `src/presentation/pages/AuditoriaPage.vue`

En `<script setup>`:

```ts
import { ref, computed, onMounted } from "vue";
import {
    parseAuditDetail,
    auditFieldLabel,
    formatAuditValue,
} from "../utils/auditDetail";

const details = computed(
    () =>
        new Map(
            auditStore.logs.map(
                (log) => [log.id, parseAuditDetail(log.detail)] as const,
            ),
        ),
);
```

> Ojo con `as const` en el tuple del `Map` (`[log.id, ...] as const`); sin él,
> TypeScript puede no inferir el par `[K, V]` para el constructor de `Map`.

En `<template>`, celda de Detalle:

```html
<td>
    <template v-if="details.get(log.id)">
        <div class="audit-detail">
            <div class="audit-desc">
                {{ details.get(log.id)!.descripcion }}
            </div>
            <div
                v-for="cambio in details.get(log.id)!.cambios"
                :key="cambio.campo"
                class="audit-change"
            >
                <span class="audit-field">
                    {{ auditFieldLabel(cambio.campo) }}
                </span>
                <span class="audit-value audit-old">
                    {{ formatAuditValue(cambio.campo, cambio.antes) }}
                </span>
                <span class="audit-arrow">→</span>
                <span class="audit-value audit-new">
                    {{ formatAuditValue(cambio.campo, cambio.valor) }}
                </span>
            </div>
        </div>
    </template>
    <span v-else>{{ log.detail || "-" }}</span>
</td>
```

Estilos scoped (copiar en `<style scoped>`):

```css
.audit-detail {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.85rem;
}

.audit-desc {
    font-weight: 600;
    margin-bottom: 0.15rem;
}

.audit-change {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
}

.audit-field {
    color: var(--color-text-muted);
    min-width: 6.5rem;
}

.audit-arrow {
    color: var(--color-primary);
}

.audit-value {
    white-space: nowrap;
}

.audit-old {
    text-decoration: line-through;
    color: var(--color-text-muted);
}

.audit-new {
    font-weight: 600;
    color: var(--color-success);
}
```

---

## Verificación

```bash
cd src-tauri && cargo check
cd src-tauri && cargo test          # incluye los 3 tests de audit_detail.rs
cd src-tauri && cargo clippy --lib --tests
cd .. && pnpm build                 # vue-tsc --noEmit + vite build
```

## Notas para replicar en la otra app

- La aplicación destino debe tener el sistema de auditoría ya funcionando
  (`audit_logs` + `log_audit` + pantalla Auditoría). Si no, replicar primero
  `mis_docs/plan_auditoria.md`.
- `serde_json` y `rusqlite` ya están en las deps del proyecto; no hace falta
  agregar dependencias.
- No requiere migración de DB: `detail` es `TEXT` desde el inicio.
- Los eventos `consultar` no se tocan (siguen sin registrarse o como estén).
- Si la otra app tiene menos/más dominios, el patrón es el mismo por comando:
  1. `nuevo` → texto descriptivo con nombres en vez de IDs.
  2. `modificar` → `antes = service.get_by_id(...)` + `AuditDetail::new(...).cambio(...)`.
  3. `eliminar` → resolver label con `get_by_id` antes del borrado.
- Commits de referencia en la rama `auditoria`:
  `d70233f`, `ffc89ec`, `a119723`, `80da354`, `b93945a`.
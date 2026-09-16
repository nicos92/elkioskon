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
export interface User {
  id: number;
  username: string;
  active: boolean;
  created_at: string;
  modified_at: string;
  permissions?: string[];
}

export interface Permission {
  id: number;
  permission: string;
  created: string;
}

export interface UserPermission {
  id: number;
  permission: string;
  created: string;
  assigned_at: string;
}

export interface Proveedor {
  id: number;
  cuit?: string;
  proveedor: string;
  nombre: string;
  tel?: string;
  email?: string;
  observacion?: string;
}

export interface Cliente {
  id: number;
  nombre?: string;
  apellido?: string;
  telefono?: string;
  email?: string;
  direccion?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateClienteRequest {
  nombre?: string;
  apellido?: string;
  telefono?: string;
  email?: string;
  direccion?: string;
}

export interface UpdateClienteRequest {
  id: number;
  nombre?: string;
  apellido?: string;
  telefono?: string;
  email?: string;
  direccion?: string;
}

export interface Categoria {
  id: number;
  categoria: string;
}

export interface TipoVenta {
  id: number;
  nombre: string;
  hacia_donde: string | null;
  created_at: string;
}

export interface CreateTipoVentaRequest {
  nombre: string;
  hacia_donde?: string;
}

export interface UpdateTipoVentaRequest {
  id: number;
  nombre: string;
  hacia_donde?: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface CreateUserRequest {
  username: string;
  password: string;
}

export interface UpdateUserRequest {
  id: number;
  username: string;
  active: boolean;
}

export interface AddPermissionRequest {
  user_id: number;
  permission_id: number;
}

export interface ChangePasswordRequest {
  target_user_id: number;
  current_password?: string | null;
  new_password: string;
}

export interface CreateProveedorRequest {
  proveedor: string;
  nombre: string;
  cuit?: string;
  tel?: string;
  email?: string;
  observacion?: string;
}

export interface UpdateProveedorRequest {
  id: number;
  proveedor: string;
  nombre: string;
  cuit?: string;
  tel?: string;
  email?: string;
  observacion?: string;
}

export interface CreateCategoriaRequest {
  categoria: string;
}

export interface UpdateCategoriaRequest {
  id: number;
  categoria: string;
}

export interface SubCategoria {
  id: number;
  sub_categoria: string;
  id_categoria: number;
}

export interface CreateSubCategoriaRequest {
  sub_categoria: string;
  id_categoria: number;
}

export interface UpdateSubCategoriaRequest {
  id: number;
  sub_categoria: string;
  id_categoria: number;
}

export interface Articulo {
  id: number;
  articulo: string;
  cod_articulo: string;
  id_sub_categoria: number;
  id_proveedor: number;
}

export interface CreateArticuloRequest {
  articulo: string;
  cod_articulo: string;
  id_sub_categoria: number;
  id_proveedor: number;
}

export interface UpdateArticuloRequest {
  id: number;
  articulo: string;
  cod_articulo: string;
  id_sub_categoria: number;
  id_proveedor: number;
}

export interface Stock {
  id: number;
  id_articulo: number;
  cantidad: number;
  costo: number;
  ganancia: number;
  updated_at?: string;
}

export interface CreateStockRequest {
  id_articulo: number;
  cantidad: number;
  costo: number;
  ganancia: number;
}

export interface UpdateStockRequest {
  id: number;
  cantidad: number;
  costo: number;
  ganancia: number;
}

export interface AuditLog {
  id: number;
  user_id: number;
  username: string;
  screen: string;
  action: string;
  detail: string | null;
  created_at: string;
}

export interface AuditLogFilters {
  user_id?: number;
  screen?: string;
  action?: string;
  from?: string;
  to?: string;
  limit?: number;
  offset?: number;
}

export interface AuditLogPage {
  items: AuditLog[];
  total: number;
  limit: number;
  offset: number;
}

export interface Venta {
  id: number;
  user_id: number;
  fecha: string;
  total: number;
  descuento: number;
  anulada: boolean;
  observacion: string | null;
  created_at: string;
  porcentaje_nocturno: number;
}

export interface VentaDetalle {
  id: number;
  id_venta: number;
  id_articulo: number;
  cantidad: number;
  costo_unitario: number;
  precio_unitario: number;
  subtotal: number;
}

export interface VentaDetalleConArticulo {
  id: number;
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  cantidad: number;
  costo_unitario: number;
  precio_unitario: number;
  subtotal: number;
}

export interface VentaWithDetalle {
  id: number;
  user_id: number;
  username: string;
  fecha: string;
  subtotal: number;
  descuento: number;
  total: number;
  anulada: boolean;
  observacion: string | null;
  tipo_venta: string | null;
  cliente_id: number;
  cliente_nombre: string | null;
  cliente_apellido: string | null;
  created_at: string;
  porcentaje_nocturno: number;
  items: VentaDetalleConArticulo[];
}

export interface CreateVentaDetalleRequest {
  id_articulo: number;
  cantidad: number;
  precio_unitario?: number;
}

export interface CreateVentaRequest {
  items: CreateVentaDetalleRequest[];
  descuento?: number;
  observacion?: string;
  id_tipo_venta?: number;
  cliente_id?: number;
}

export interface VentaPage {
  items: VentaWithDetalle[];
  total: number;
  limit: number;
  offset: number;
}

export type PresupuestoEstado =
  | "pendiente"
  | "aprobado"
  | "vencido"
  | "convertido"
  | "anulado";

export interface Presupuesto {
  id: number;
  user_id: number;
  fecha: string;
  total: number;
  descuento: number;
  estado: PresupuestoEstado;
  fecha_vencimiento: string | null;
  observacion: string | null;
  cliente_id: number | null;
  created_at: string;
}

export interface PresupuestoDetalleConArticulo {
  id: number;
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  cantidad: number;
  costo_unitario: number;
  precio_unitario: number;
  subtotal: number;
}

export interface PresupuestoWithDetalle {
  id: number;
  user_id: number;
  username: string;
  fecha: string;
  subtotal: number;
  descuento: number;
  total: number;
  estado: PresupuestoEstado;
  fecha_vencimiento: string | null;
  observacion: string | null;
  cliente_id: number | null;
  cliente_nombre: string | null;
  cliente_apellido: string | null;
  created_at: string;
  items: PresupuestoDetalleConArticulo[];
}

export interface CreatePresupuestoDetalleRequest {
  id_articulo: number;
  cantidad: number;
  precio_unitario?: number;
}

export interface CreatePresupuestoRequest {
  items: CreatePresupuestoDetalleRequest[];
  descuento?: number;
  observacion?: string;
  fecha_vencimiento?: string;
  cliente_id?: number;
}

export interface PresupuestoPage {
  items: PresupuestoWithDetalle[];
  total: number;
  limit: number;
  offset: number;
}

export interface CambiarEstadoPresupuestoRequest {
  id: number;
  estado: PresupuestoEstado;
}

export interface Cierre {
  id: number;
  fecha: string;
  dia: number;
  mes: number;
  anio: number;
  total_costo: number;
  total_ganancia: number;
  total_venta: number;
  created_at: string;
}

export interface CierreTipo {
  id_tipo_venta: number;
  tipo_venta: string;
  total: number;
}

export interface CierreWithTipos extends Cierre {
  tipos: CierreTipo[];
}

export interface CierrePage {
  items: CierreWithTipos[];
  total: number;
  limit: number;
  offset: number;
}

export interface CrearCierreRequest {
  fecha: string;
}

export interface DollarQuote {
  id: number;
  official_buy: number;
  official_sell: number;
  blue_buy: number;
  blue_sell: number;
  timestamp: string;
}

export interface NocturnoConfig {
  activo: boolean;
  porcentaje: number;
  hora_inicio: string;
  hora_fin: string;
}

export interface StockBajoItem {
  id_stock: number;
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  cantidad: number;
}

export interface SubCategoriaInfo {
  id: number;
  sub_categoria: string;
}

export interface CategoriaConSub {
  id: number;
  categoria: string;
  sub_categorias: SubCategoriaInfo[];
}

export interface HomeStats {
  total_articulos: number;
  articulos_con_stock: number;
  total_usuarios: number;
  usuarios_activos: number;
  usuarios_inactivos: number;
  total_proveedores: number;
  total_categorias: number;
  total_sub_categorias: number;
  ventas_hoy: number;
  total_ventas_hoy: number;
  stock_bajo: StockBajoItem[];
  categorias: CategoriaConSub[];
}

export interface StockPreview {
  id_stock: number;
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  categoria: string;
  sub_categoria: string;
  proveedor: string;
  costo_actual: number;
  ganancia: number;
  costo_nuevo: number;
  cantidad: number;
}

export interface ApplyCostoPercentageRequest {
  porcentaje: number;
  id_categoria: number | null;
  id_sub_categoria: number | null;
  id_proveedor: number | null;
}

export interface ApplyCostoPercentageResult {
  updated_count: number;
}

export interface CostUpdateOperationResponse {
  id: number;
  porcentaje: number;
  affected_count: number;
  categoria_nombre: string | null;
  sub_categoria_nombre: string | null;
  proveedor_nombre: string | null;
  created_at: string;
}

export interface UndoOperationResult {
  restored_count: number;
}

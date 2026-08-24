import type { Cliente, Proveedor } from "./types";

export const DEFAULT_CLIENT_NOMBRE = "Consumidor";
export const DEFAULT_CLIENT_APELLIDO = "Final";
export const DEFAULT_CLIENT_LABEL = "Consumidor Final";

export function isDefaultClient(cliente: Cliente): boolean {
  return (
    cliente.nombre === DEFAULT_CLIENT_NOMBRE &&
    cliente.apellido === DEFAULT_CLIENT_APELLIDO
  );
}

export const DEFAULT_PROVEEDOR = "Sin Proveedor";

export function isDefaultProveedor(proveedor: Proveedor): boolean {
  return proveedor.proveedor === DEFAULT_PROVEEDOR;
}

export function calcularPrecioVenta(costo: number, ganancia: number): number {
  return costo * (1 + ganancia / 100);
}

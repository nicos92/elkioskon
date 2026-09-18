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

export function margenEfectivo(
  ganancia: number,
  gananciaDiurna: number,
  gananciaNocturna: number,
  activo: boolean,
  esNocturno: boolean,
): number {
  if (!activo) return ganancia;
  if (esNocturno) return gananciaNocturna > 0 ? gananciaNocturna : ganancia;
  return gananciaDiurna > 0 ? gananciaDiurna : ganancia;
}

export function minutosDesdeMedianoche(hhmm: string): number | null {
  const match = /^(\d{2}):(\d{2})$/.exec(hhmm.trim());
  if (!match) return null;
  const hora = Number(match[1]);
  const minuto = Number(match[2]);
  if (hora > 23 || minuto > 59) return null;
  return hora * 60 + minuto;
}

export function esHorarioNocturno(
  ahoraMinutos: number,
  inicioMinutos: number,
  finMinutos: number,
): boolean {
  if (inicioMinutos === finMinutos) return false;
  if (inicioMinutos < finMinutos) {
    return ahoraMinutos >= inicioMinutos && ahoraMinutos < finMinutos;
  }
  return ahoraMinutos >= inicioMinutos || ahoraMinutos < finMinutos;
}

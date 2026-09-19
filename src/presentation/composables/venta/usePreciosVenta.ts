import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import {
  calcularPrecioVenta,
  esHorarioNocturno,
  margenEfectivo,
  minutosDesdeMedianoche,
} from "../../../domain/entities";
import type { CartItem, CartSourceItem } from "../useCart";
import type { useArticulosStore } from "../../stores";
import type { useNocturnoStore } from "../../stores";
import type { useStockStore } from "../../stores";

export interface UsePreciosVentaOptions {
  stockStore: ReturnType<typeof useStockStore>;
  articulosStore: ReturnType<typeof useArticulosStore>;
  nocturnoStore: ReturnType<typeof useNocturnoStore>;
  getCart: () => CartItem[];
}

function horaActualEnMinutos(): number {
  const d = new Date();
  return d.getHours() * 60 + d.getMinutes();
}

export function usePreciosVenta({
  stockStore,
  articulosStore,
  nocturnoStore,
  getCart,
}: UsePreciosVentaOptions) {
  const minutosActuales = ref(horaActualEnMinutos());
  let relojTimer: number | undefined;

  const esNocturnoActual = computed(() => {
    const cfg = nocturnoStore.config;
    if (!cfg.activo) return false;
    const inicio = minutosDesdeMedianoche(cfg.hora_inicio);
    const fin = minutosDesdeMedianoche(cfg.hora_fin);
    if (inicio === null || fin === null) return false;
    return esHorarioNocturno(minutosActuales.value, inicio, fin);
  });

  const articulosVendibles = computed<CartSourceItem[]>(() => {
    const cfg = nocturnoStore.config;
    return stockStore.stocks.map((s) => {
      const articulo = articulosStore.articulos.find(
        (a) => a.id === s.id_articulo,
      );
      const margen = margenEfectivo(
        s.ganancia,
        s.ganancia_diurna,
        s.ganancia_nocturna,
        cfg.activo,
        esNocturnoActual.value,
      );
      return {
        id_articulo: s.id_articulo,
        cod_articulo: articulo?.cod_articulo || "-",
        articulo: articulo?.articulo || "Sin artículo",
        stockDisponible: s.cantidad,
        precioVenta: calcularPrecioVenta(s.costo, margen),
      };
    });
  });

  function repreciarCarrito(items: CartItem[]) {
    const cfg = nocturnoStore.config;
    for (const item of items) {
      const s = stockStore.stocks.find(
        (stock) => stock.id_articulo === item.id_articulo,
      );
      if (!s) continue;
      const margen = margenEfectivo(
        s.ganancia,
        s.ganancia_diurna,
        s.ganancia_nocturna,
        cfg.activo,
        esNocturnoActual.value,
      );
      item.precio = calcularPrecioVenta(s.costo, margen);
      item.subtotal = item.cantidad * item.precio;
    }
  }

  function refrescarTurno() {
    if (!document.hidden) {
      minutosActuales.value = horaActualEnMinutos();
    }
  }

  watch(
    () => [
      nocturnoStore.config.activo,
      nocturnoStore.config.hora_inicio,
      nocturnoStore.config.hora_fin,
      minutosActuales.value,
    ],
    () => {
      repreciarCarrito(getCart());
    },
  );

  onMounted(() => {
    relojTimer = window.setInterval(() => {
      minutosActuales.value = horaActualEnMinutos();
    }, 30000);
    window.addEventListener("focus", refrescarTurno);
    document.addEventListener("visibilitychange", refrescarTurno);
  });

  onUnmounted(() => {
    if (relojTimer !== undefined) {
      window.clearInterval(relojTimer);
    }
    window.removeEventListener("focus", refrescarTurno);
    document.removeEventListener("visibilitychange", refrescarTurno);
  });

  return {
    articulosVendibles,
    esNocturnoActual,
    minutosActuales,
    repreciarCarrito,
  };
}
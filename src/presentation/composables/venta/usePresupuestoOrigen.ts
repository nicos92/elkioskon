import { ref } from "vue";
import type { Ref } from "vue";
import type { Router } from "vue-router";
import type {
  Cliente,
  PresupuestoWithDetalle,
} from "../../../domain/entities";
import type { CartItem, CartSourceItem } from "../useCart";
import type { useClientesStore } from "../../stores";
import type { usePresupuestosStore } from "../../stores";

export type CargarPresupuestoResult =
  | "ok"
  | "no-encontrado"
  | "estado-terminal"
  | "error";

export interface UsePresupuestoOrigenOptions {
  router: Router;
  presupuestosStore: ReturnType<typeof usePresupuestosStore>;
  clientesStore: ReturnType<typeof useClientesStore>;
  getArticulosVendibles: () => CartSourceItem[];
  clienteSeleccionado: Ref<Cliente | null>;
  clienteDefecto: Ref<Cliente | null>;
  descuento: Ref<number>;
  observacion: Ref<string>;
  fechaVencimiento: Ref<string>;
  setItems: (items: CartItem[]) => void;
}

export function usePresupuestoOrigen(options: UsePresupuestoOrigenOptions) {
  const {
    router,
    presupuestosStore,
    clientesStore,
    getArticulosVendibles,
    clienteSeleccionado,
    clienteDefecto,
    descuento,
    observacion,
    fechaVencimiento,
    setItems,
  } = options;

  const presupuestoOrigen = ref<number | null>(null);
  const precargandoPresupuesto = ref(false);

  function quitarPresupuesto() {
    presupuestoOrigen.value = null;
    router.replace({ name: "nueva-venta" });
  }

  async function cargarPresupuesto(
    id: number,
  ): Promise<CargarPresupuestoResult> {
    precargandoPresupuesto.value = true;
    try {
      const presupuesto: PresupuestoWithDetalle | null =
        await presupuestosStore.getPresupuestoById(id);
      if (!presupuesto) return "no-encontrado";
      if (
        presupuesto.estado === "convertido" ||
        presupuesto.estado === "anulado"
      ) {
        router.replace({ name: "nueva-venta" });
        return "estado-terminal";
      }
      descuento.value = presupuesto.descuento || 0;
      observacion.value = presupuesto.observacion || "";
      fechaVencimiento.value = presupuesto.fecha_vencimiento || "";
      if (presupuesto.cliente_id) {
        const cliente = clientesStore.clientes.find(
          (c) => c.id === presupuesto.cliente_id,
        );
        clienteSeleccionado.value = cliente ?? clienteDefecto.value;
      } else {
        clienteSeleccionado.value = clienteDefecto.value;
      }
      const items: CartItem[] = [];
      for (const item of presupuesto.items) {
        const stock = getArticulosVendibles().find(
          (s) => s.id_articulo === item.id_articulo,
        );
        items.push({
          id_articulo: item.id_articulo,
          cod_articulo: item.cod_articulo,
          articulo: item.articulo,
          stockDisponible: stock?.stockDisponible ?? 0,
          cantidad: item.cantidad,
          precio: item.precio_unitario,
          subtotal: item.subtotal,
        });
      }
      setItems(items);
      presupuestoOrigen.value = id;
      return "ok";
    } catch {
      return "error";
    } finally {
      precargandoPresupuesto.value = false;
    }
  }

  return {
    presupuestoOrigen,
    precargandoPresupuesto,
    cargarPresupuesto,
    quitarPresupuesto,
  };
}
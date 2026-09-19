<script setup lang="ts">
import type { Cliente } from "../../../domain/entities";
import type { CartItem } from "../../composables/useCart";
import { clienteLabel } from "../../utils/cliente";
import PrintArea from "../ui/PrintArea.vue";

defineProps<{
  fecha: string;
  cliente: Cliente | null;
  items: CartItem[];
  subtotal: number;
  descuento: number;
  descuentoMonto: number;
  total: number;
  observacion: string;
}>();
</script>

<template>
    <PrintArea
        titulo="Presupuesto"
        :info="
            [
                { label: 'Fecha', value: fecha },
                ...(cliente
                    ? [{ label: 'Cliente', value: clienteLabel(cliente) }]
                    : []),
            ]
        "
        :items="
            items.map((item) => ({
                codigo: item.cod_articulo,
                articulo: item.articulo,
                cantidad: item.cantidad,
                precio: item.precio,
                subtotal: item.subtotal,
            }))
        "
        :subtotal="subtotal"
        :descuento="descuento"
        :descuento-monto="descuentoMonto"
        :total="total"
        :observacion="observacion"
    />
</template>
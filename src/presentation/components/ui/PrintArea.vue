<script setup lang="ts">
import { formatMoney } from "../../utils/format";

interface PrintRow {
    codigo: string;
    articulo: string;
    cantidad: number;
    precio: number;
    subtotal: number;
}

interface PrintInfo {
    label: string;
    value: string;
}

defineProps<{
    titulo: string;
    info?: PrintInfo[];
    items: PrintRow[];
    subtotal: number;
    descuento?: number;
    descuentoMonto?: number;
    total: number;
    observacion?: string;
    nota?: string;
}>();
</script>

<template>
    <Teleport to="body">
        <div class="print-area" id="print-area">
            <h1>{{ titulo }}</h1>
            <p v-for="linea in info" :key="linea.label">
                {{ linea.label }}: {{ linea.value }}
            </p>
            <p v-if="observacion">Observación: {{ observacion }}</p>
            <div class="print-summary">
                <p class="print-line">Subtotal: {{ formatMoney(subtotal) }}</p>
                <p v-if="descuento && descuento > 0" class="print-line">
                    Descuento ({{ descuento }}%):
                    −{{ formatMoney(descuentoMonto ?? 0) }}
                </p>
                <p class="print-total">Total: {{ formatMoney(total) }}</p>
                <p v-if="nota" class="print-obs">{{ nota }}</p>
            </div>
            <table>
                <thead>
                    <tr>
                        <th>Código</th>
                        <th>Artículo</th>
                        <th>Cantidad</th>
                        <th>Precio</th>
                        <th>Subtotal</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="item in items" :key="item.codigo">
                        <td>{{ item.codigo }}</td>
                        <td>{{ item.articulo }}</td>
                        <td>{{ item.cantidad }}</td>
                        <td>{{ formatMoney(item.precio) }}</td>
                        <td>{{ formatMoney(item.subtotal) }}</td>
                    </tr>
                </tbody>
            </table>
        </div>
    </Teleport>
</template>
<script setup lang="ts">
import type { CartItem } from "../../composables/useCart";
import { formatMoney } from "../../utils/format";

defineProps<{
  items: CartItem[];
  stockWarning: (item: CartItem) => boolean;
}>();

const emit = defineEmits<{
  "update-cantidad": [item: CartItem, value: number];
  "update-precio": [item: CartItem, value: number];
  remove: [idArticulo: number];
}>();

function toNumber(value: string): number {
  const n = Number(value);
  return Number.isNaN(n) ? 0 : n;
}
</script>

<template>
    <table v-if="items.length > 0" class="cart-table">
        <thead>
            <tr>
                <th>Código</th>
                <th>Artículo</th>
                <th>Cantidad</th>
                <th>Precio</th>
                <th>Subtotal</th>
                <th>Acciones</th>
            </tr>
        </thead>
        <tbody>
            <tr
                v-for="item in items"
                :key="item.id_articulo"
                :class="{ 'stock-warning-row': stockWarning(item) }"
            >
                <td>{{ item.cod_articulo }}</td>
                <td>
                    {{ item.articulo }}
                    <span v-if="stockWarning(item)" class="stock-warning">
                        ⚠ Stock insuficiente ({{ item.stockDisponible }})
                    </span>
                </td>
                <td>
                    <input
                        :value="item.cantidad"
                        type="number"
                        step="0.01"
                        min="0.01"
                        @input="
                            emit(
                                'update-cantidad',
                                item,
                                toNumber(
                                    ($event.target as HTMLInputElement).value,
                                ),
                            )
                        "
                        class="cart-input"
                    />
                </td>
                <td>
                    <input
                        :value="item.precio"
                        type="number"
                        step="0.01"
                        min="0"
                        @input="
                            emit(
                                'update-precio',
                                item,
                                toNumber(
                                    ($event.target as HTMLInputElement).value,
                                ),
                            )
                        "
                        class="cart-input"
                    />
                </td>
                <td>{{ formatMoney(item.subtotal) }}</td>
                <td>
                    <button
                        @click="emit('remove', item.id_articulo)"
                        class="btn-icon btn-danger"
                        title="Quitar"
                    >
                        <img src="/svg/trash.svg" alt="Quitar" />
                    </button>
                </td>
            </tr>
        </tbody>
    </table>
</template>

<style scoped>
.cart-table {
    width: 100%;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
}

.cart-table th,
.cart-table td {
    padding: 0.7rem 0.8rem;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
}

.cart-table th {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-secondary);
}

.cart-table tbody tr:last-child td {
    border-bottom: none;
}

.cart-table tbody td {
    vertical-align: middle;
}

.cart-input {
    width: 84px;
    padding: 0.55rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    text-align: right;
    font-variant-numeric: tabular-nums;
}

.cart-input:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 1px;
}

.btn-icon {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
}

.btn-icon img {
    width: 18px;
    height: 18px;
}

.btn-danger:hover {
    opacity: 0.7;
}

.btn-icon:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 2px;
    border-radius: 4px;
}

.stock-warning-row td {
    background: rgba(var(--color-warning-rgb), 0.08);
}

.stock-warning {
    display: block;
    color: var(--color-warning);
    font-size: 0.8rem;
}
</style>

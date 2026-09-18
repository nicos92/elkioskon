<script setup lang="ts">
import { formatMoney } from "../../utils/format";

defineProps<{
  subtotal: number;
  descuento: number;
  descuentoMonto: number;
  total: number;
}>();

const emit = defineEmits<{
  "update:descuento": [value: number];
}>();
</script>

<template>
    <div class="total-box">
        <span class="total-label">Subtotal</span>
        <span class="total-value">{{ formatMoney(subtotal) }}</span>
    </div>
    <div class="total-box">
        <span class="total-label">Descuento</span>
        <div class="descuento-row">
            <input
                :value="descuento"
                type="number"
                step="0.01"
                min="0"
                max="100"
                class="descuento-input"
                @input="
                    emit(
                        'update:descuento',
                        Number(($event.target as HTMLInputElement).value),
                    )
                "
            />
            <span>%</span>
            <span v-if="descuentoMonto > 0" class="descuento-monto">
                −{{ formatMoney(descuentoMonto) }}
            </span>
        </div>
    </div>
    <div class="total-box total-final">
        <span class="total-label">Total</span>
        <span class="total-value">{{ formatMoney(total) }}</span>
    </div>
</template>

<style scoped>
.total-box {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    border-radius: 8px;
    background: var(--color-surface-2);
    min-width: 180px;
}

.total-final {
    background: var(--color-success);
    color: var(--color-on-success);
}

.total-label {
    font-size: 0.85rem;
    text-transform: uppercase;
    opacity: 0.8;
}

.total-value {
    font-size: 1.4rem;
    font-weight: 700;
}

.descuento-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.1rem;
    font-weight: 600;
}

.descuento-input {
    width: 80px;
    padding: 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
}

.descuento-monto {
    color: var(--color-danger);
    font-size: 0.9rem;
}
</style>

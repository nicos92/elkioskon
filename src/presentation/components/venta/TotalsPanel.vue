<script setup lang="ts">
import { computed } from "vue";
import { formatMoney } from "../../utils/format";

const props = withDefaults(
    defineProps<{
        subtotal: number;
        descuento: number;
        descuentoMonto: number;
        total: number;
        count?: number;
    }>(),
    { count: 0 },
);

const emit = defineEmits<{
    "update:descuento": [value: number];
}>();

const itemsLabel = computed(() => {
    if (props.count === 0) return "Sin artículos";
    return props.count === 1 ? "1 artículo" : `${props.count} artículos`;
});
</script>

<template>
    <div class="total-box">
        <div class="tape-line tape-line-items">
            <span class="tape-static">{{ itemsLabel }}</span>
        </div>

        <div class="tape-line">
            <span class="tape-static">Subtotal</span>
            <span class="tape-value">{{ formatMoney(subtotal) }}</span>
        </div>

        <div class="tape-line tape-line-descuento">
            <span class="tape-static">Descuento</span>
            <div class="descuento-row">
                <span v-if="descuentoMonto > 0" class="descuento-monto">
                    −{{ formatMoney(descuentoMonto) }}
                </span>
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
                <span class="descuento-unidad">%</span>
            </div>
        </div>

        <div class="tape-line tape-total">
            <span class="tape-total-label">Total</span>
            <span class="tape-total-value">{{ formatMoney(total) }}</span>
        </div>
    </div>
</template>

<style scoped>
.total-box {
    display: flex;
    flex-direction: column;
    font-variant-numeric: tabular-nums;
}

.tape-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.15rem 0;
}

.tape-line-items {
    margin-bottom: 0.15rem;
}

.tape-static {
    font-size: 0.85rem;
    color: var(--color-text-secondary);
}

.tape-line-items .tape-static {
    font-size: 0.78rem;
}

.tape-value {
    font-size: 0.95rem;
    font-weight: 600;
}

.tape-line-descuento {
    align-items: center;
}

.descuento-row {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.35rem;
}

.descuento-input {
    width: 64px;
    padding: 0.3rem 0.45rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    text-align: right;
    font-variant-numeric: tabular-nums;
}

.descuento-input:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 1px;
}

.descuento-unidad {
    font-size: 0.9rem;
    font-weight: 600;
}

.descuento-monto {
    color: var(--color-warning);
    font-size: 0.9rem;
    font-weight: 600;
}

.tape-total {
    align-items: baseline;
    margin-top: 0.35rem;
    padding-top: 0.6rem;
    border-top: 2px solid var(--color-text);
}

.tape-total-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-text);
}

.tape-total-value {
    font-size: clamp(1.7rem, 1.2rem + 1.5vw, 2.3rem);
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
}
</style>
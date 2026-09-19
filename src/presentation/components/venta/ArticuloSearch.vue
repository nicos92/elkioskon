<script setup lang="ts">
import { ref } from "vue";
import type { CartSourceItem } from "../../composables/useCart";
import { formatMoney } from "../../utils/format";

defineProps<{
  query: string;
  results: CartSourceItem[];
}>();

const emit = defineEmits<{
  "update:query": [value: string];
  select: [idArticulo: number];
  enter: [];
}>();

const inputEl = ref<HTMLInputElement | null>(null);

function focus() {
  inputEl.value?.focus();
}

defineExpose({ focus });
</script>

<template>
    <label class="scan-label" for="scan-input">
        Código o nombre de artículo
    </label>
    <input
        id="scan-input"
        ref="inputEl"
        :value="query"
        type="text"
        class="scan-input"
        autocomplete="off"
        placeholder="Escanee o escriba y presione Enter…"
        @input="
            emit('update:query', ($event.target as HTMLInputElement).value)
        "
        @keydown.enter.prevent="emit('enter')"
    />
    <div v-if="results.length > 0" class="search-results">
        <button
            v-for="result in results"
            :key="result.id_articulo"
            type="button"
            @click="emit('select', result.id_articulo)"
            class="search-result-item"
        >
            <span class="result-code">{{ result.cod_articulo }}</span>
            <span class="result-name">{{ result.articulo }}</span>
            <span class="result-stock">
                Stock: {{ result.stockDisponible }}
            </span>
            <span class="result-precio">
                {{ formatMoney(result.precioVenta) }}
            </span>
        </button>
    </div>
    <div
        v-if="query.trim() && results.length === 0"
        class="empty-state small"
    >
        Sin coincidencias
    </div>
</template>

<style scoped>
.scan-label {
    display: block;
    margin-bottom: 0.4rem;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--color-text-secondary);
}

.scan-input {
    width: 100%;
    padding: 0.9rem 1rem;
    font-size: 1.125rem;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    box-sizing: border-box;
    background: var(--color-surface-2);
    color: var(--color-text);
}

.scan-input::placeholder {
    color: var(--color-text-muted);
    opacity: 0.85;
}

.scan-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 30%, transparent);
    background: var(--color-surface);
}

.search-results {
    margin-top: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
}

.search-result-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    width: 100%;
    padding: 0.6rem 0.85rem;
    background: var(--color-surface);
    border: none;
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    text-align: left;
    color: var(--color-text);
    font-variant-numeric: tabular-nums;
}

.search-result-item:last-child {
    border-bottom: none;
}

.search-result-item:hover {
    background: var(--color-surface-2);
}

.search-result-item:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: -2px;
}

.result-code {
    font-weight: 600;
    min-width: 110px;
}

.result-name {
    flex: 1;
}

.result-stock {
    color: var(--color-text-muted);
    min-width: 90px;
}

.result-precio {
    font-weight: 600;
    min-width: 80px;
    text-align: right;
}

.empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-muted);
}

.empty-state.small {
    padding: 1rem;
}
</style>
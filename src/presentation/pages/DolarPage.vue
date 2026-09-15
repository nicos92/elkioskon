<script setup lang="ts">
import { onMounted } from "vue";
import { useDolarStore } from "../stores";
import { useToasts } from "../composables/useToasts";
import { useConfirm } from "../composables/useConfirm";
import { formatMoney } from "../utils/format";
import { formatTimestamp } from "../utils/date";
import type { DollarQuote } from "../../domain/entities";

const dolarStore = useDolarStore();
const { error: toastError, success: toastSuccess } = useToasts();
const { confirm } = useConfirm();

async function handleRefresh() {
  const ok = await dolarStore.fetchManual();
  if (ok) {
    toastSuccess("Cotización actualizada.");
  } else {
    toastError(dolarStore.error || "No se pudo actualizar la cotización.");
  }
}

async function handleDelete(quote: DollarQuote) {
  const ok = await confirm({
    message: "¿Está seguro de eliminar esta cotización?",
  });
  if (!ok) return;

  const deleted = await dolarStore.deleteQuote(quote.id);
  if (deleted) {
    toastSuccess("Cotización eliminada.");
  } else {
    toastError(dolarStore.error || "No se pudo eliminar la cotización.");
  }
}

onMounted(async () => {
  await dolarStore.fetchQuotes();
});
</script>

<template>
    <div class="dolar-page">
        <div class="page-header">
            <h1>Cotización del Dólar</h1>
        </div>

        <div class="dolar-bar">
            <button
                class="btn-primary"
                :disabled="dolarStore.updating"
                @click="handleRefresh"
            >
                {{ dolarStore.updating ? "Actualizando..." : "Actualizar ahora" }}
            </button>
            <div class="toolbar-spacer"></div>
            <span v-if="dolarStore.lastUpdated" class="last-updated">
                Última actualización:
                {{ new Date(dolarStore.lastUpdated).toLocaleTimeString() }}
            </span>
        </div>

        <div v-if="dolarStore.loading" class="loading">Cargando...</div>

        <div v-if="dolarStore.error" class="error-banner">
            {{ dolarStore.error }}
        </div>

        <div v-if="dolarStore.latest" class="cards">
            <div class="rate-card">
                <div class="rate-card-header">
                    <span class="rate-type">Dólar Oficial</span>
                    <span class="rate-date">
                        {{ formatTimestamp(dolarStore.latest.timestamp) }}
                    </span>
                </div>
                <div class="rate-values">
                    <div class="rate-value">
                        <span class="rate-label">Compra</span>
                        <span class="rate-amount">
                            {{ formatMoney(dolarStore.latest.official_buy) }}
                        </span>
                    </div>
                    <div class="rate-value">
                        <span class="rate-label">Venta</span>
                        <span class="rate-amount">
                            {{ formatMoney(dolarStore.latest.official_sell) }}
                        </span>
                    </div>
                </div>
            </div>

            <div class="rate-card">
                <div class="rate-card-header">
                    <span class="rate-type">Dólar Blue</span>
                    <span class="rate-date">
                        {{ formatTimestamp(dolarStore.latest.timestamp) }}
                    </span>
                </div>
                <div class="rate-values">
                    <div class="rate-value">
                        <span class="rate-label">Compra</span>
                        <span class="rate-amount">
                            {{ formatMoney(dolarStore.latest.blue_buy) }}
                        </span>
                    </div>
                    <div class="rate-value">
                        <span class="rate-label">Venta</span>
                        <span class="rate-amount">
                            {{ formatMoney(dolarStore.latest.blue_sell) }}
                        </span>
                    </div>
                </div>
            </div>
        </div>

        <div class="history-section">
            <h2>Historial de cotizaciones</h2>

            <div
                v-if="!dolarStore.loading && dolarStore.quotes.length === 0"
                class="empty-state"
            >
                No hay cotizaciones registradas.
            </div>

            <div class="table-wrapper">
                <table
                    v-if="dolarStore.quotes.length > 0"
                    class="data-table"
                >
                    <thead>
                        <tr>
                            <th>Fecha</th>
                            <th>Oficial Compra</th>
                            <th>Oficial Venta</th>
                            <th>Blue Compra</th>
                            <th>Blue Venta</th>
                            <th>Acciones</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            v-for="quote in dolarStore.quotes"
                            :key="quote.id"
                        >
                            <td>{{ formatTimestamp(quote.timestamp) }}</td>
                            <td>{{ formatMoney(quote.official_buy) }}</td>
                            <td>{{ formatMoney(quote.official_sell) }}</td>
                            <td>{{ formatMoney(quote.blue_buy) }}</td>
                            <td>{{ formatMoney(quote.blue_sell) }}</td>
                            <td>
                                <button
                                    class="btn-delete"
                                    :disabled="dolarStore.updating"
                                    @click="handleDelete(quote)"
                                >
                                    Eliminar
                                </button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dolar-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;
}

.page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
}

.page-header h1 {
    margin: 0;
}

.dolar-bar {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 1.5rem;
}

.toolbar-spacer {
    flex: 1;
}

.last-updated {
    color: var(--color-text-muted);
    font-size: 0.9rem;
}

.btn-primary {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-primary:hover:not(:disabled) {
    background: var(--color-secondary);
}

.btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn-delete {
    background: transparent;
    color: var(--color-danger);
    border: 1px solid rgba(var(--color-danger-rgb), 0.4);
    padding: 0.4rem 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
}

.btn-delete:hover:not(:disabled) {
    background: rgba(var(--color-danger-rgb), 0.1);
}

.btn-delete:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
    margin-bottom: 1.5rem;
}

.rate-card {
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
}

.rate-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    gap: 0.5rem;
}

.rate-type {
    font-weight: 600;
    font-size: 1.05rem;
}

.rate-date {
    color: var(--color-text-muted);
    font-size: 0.85rem;
    text-align: right;
}

.rate-values {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
}

.rate-value {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.rate-label {
    color: var(--color-text-muted);
    font-size: 0.85rem;
}

.rate-amount {
    font-size: 1.5rem;
    font-weight: 700;
}

.history-section {
    margin-top: 2rem;
}

.history-section h2 {
    margin-bottom: 1rem;
    font-size: 1.1rem;
}

.table-wrapper {
    overflow-x: auto;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.data-table {
    width: 100%;
    background: var(--color-surface);
}

.data-table th,
.data-table td {
    padding: 1rem;
    text-align: left;
}

.data-table th {
    background: var(--color-surface-2);
    font-weight: 600;
}

.error-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
}

.loading,
.empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-muted);
}
</style>

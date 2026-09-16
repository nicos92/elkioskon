<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useAuditStore } from "../stores";
import { usePagination } from "../composables/usePagination";
import PaginationBar from "../components/PaginationBar.vue";
import { dayStartToISO, dayEndToISO } from "../utils/date";
import {
    parseAuditDetail,
    auditFieldLabel,
    formatAuditValue,
} from "../utils/auditDetail";
import type { AuditLogFilters } from "../../domain/entities";

const auditStore = useAuditStore();

const details = computed(
    () =>
        new Map(
            auditStore.logs.map(
                (log) => [log.id, parseAuditDetail(log.detail)] as const,
            ),
        ),
);

const screen = ref("");
const action = ref("");
const from = ref("");
const to = ref("");

const screens = [
    "Usuarios",
    "Proveedores",
    "Clientes",
    "Categorias",
    "SubCategorias",
    "Articulos",
    "Stock",
    "Ventas",
    "Tipos de Venta",
    "Permisos",
    "Auditoria",
    "Cierres del día",
];

const actions = ["nuevo", "modificar", "consultar", "eliminar"];

function buildFilters(limit: number, offset: number): AuditLogFilters {
    const filters: AuditLogFilters = {
        limit,
        offset,
    };
    if (screen.value) filters.screen = screen.value;
    if (action.value) filters.action = action.value;
    if (from.value) filters.from = dayStartToISO(from.value);
    if (to.value) filters.to = dayEndToISO(to.value);
    return filters;
}

const pagination = usePagination({
    fetch: async (limit, off) => {
        await auditStore.fetchLogs(buildFilters(limit, off));
        if (auditStore.logs.length === 0 && off > 0 && auditStore.total > 0) {
            const lastPage = Math.max(1, Math.ceil(auditStore.total / limit));
            const newOffset = (lastPage - 1) * limit;
            offset.value = newOffset;
            await auditStore.fetchLogs(buildFilters(limit, newOffset));
        }
    },
    getTotal: () => auditStore.total,
});

const {
    pageSize,
    pageSizeOptions,
    offset,
    currentPage,
    totalPages,
    pages,
    goToPage,
    handlePageSizeChange,
    refresh,
} = pagination;

async function handleSearch() {
    offset.value = 0;
    await refresh();
}

function resetFilters() {
    screen.value = "";
    action.value = "";
    from.value = "";
    to.value = "";
    handleSearch();
}

function formatDate(iso: string): string {
    const date = new Date(iso);
    if (isNaN(date.getTime())) return iso;
    return date.toLocaleString();
}

onMounted(() => {
    refresh();
});
</script>

<template>
    <div class="auditoria-page">
        <div class="page-header">
            <h1>Auditoría de Acciones</h1>
        </div>

        <div class="filters-bar">
            <select v-model="screen" class="filter-input">
                <option value="">Todas las pantallas</option>
                <option v-for="s in screens" :key="s" :value="s">
                    {{ s }}
                </option>
            </select>
            <select v-model="action" class="filter-input">
                <option value="">Todas las acciones</option>
                <option v-for="a in actions" :key="a" :value="a">
                    {{ a }}
                </option>
            </select>
            <div class="filter-date">
                <input v-model="from" type="date" class="filter-input" />
                <span>→</span>
                <input v-model="to" type="date" class="filter-input" />
            </div>
            <select
                v-model="pageSize"
                class="filter-input page-size-select"
                @change="handlePageSizeChange"
                title="Registros por página"
            >
                <option v-for="n in pageSizeOptions" :key="n" :value="n">
                    {{ n }} por página
                </option>
            </select>
            <button @click="handleSearch" class="btn-primary">Buscar</button>
            <button @click="resetFilters" class="btn-secondary">
                Limpiar
            </button>
        </div>

        <div v-if="auditStore.loading" class="loading">Cargando...</div>

        <div v-if="auditStore.error" class="error-banner">
            {{ auditStore.error }}
        </div>

        <div class="table-wrapper">
        <table v-if="!auditStore.loading" class="data-table">
            <thead>
                <tr>
                    <th>Fecha y Hora</th>
                    <th>Usuario</th>
                    <th>Pantalla</th>
                    <th>Acción</th>
                    <th>Detalle</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="log in auditStore.logs" :key="log.id">
                    <td>{{ formatDate(log.created_at) }}</td>
                    <td>{{ log.username || `(id ${log.user_id})` }}</td>
                    <td>{{ log.screen }}</td>
                    <td>
                        <span
                            :class="['badge', `badge-${log.action}`]"
                        >
                            {{ log.action }}
                        </span>
                    </td>
                    <td>
                        <template v-if="details.get(log.id)">
                            <div class="audit-detail">
                                <div class="audit-desc">
                                    {{ details.get(log.id)!.descripcion }}
                                </div>
                                <div
                                    v-for="cambio in details.get(log.id)!.cambios"
                                    :key="cambio.campo"
                                    class="audit-change"
                                >
                                    <span class="audit-field">
                                        {{ auditFieldLabel(cambio.campo) }}
                                    </span>
                                    <span class="audit-value audit-old">
                                        {{ formatAuditValue(cambio.campo, cambio.antes) }}
                                    </span>
                                    <span class="audit-arrow">→</span>
                                    <span class="audit-value audit-new">
                                        {{ formatAuditValue(cambio.campo, cambio.valor) }}
                                    </span>
                                </div>
                            </div>
                        </template>
                        <span v-else>{{ log.detail || "-" }}</span>
                    </td>
                </tr>
            </tbody>
        </table>
        </div>

        <div
            v-if="!auditStore.loading && auditStore.logs.length === 0"
            class="empty-state"
        >
            No hay registros de auditoría con los filtros seleccionados
        </div>

        <PaginationBar
            v-if="!auditStore.loading && auditStore.total > 0"
            :current-page="currentPage"
            :total-pages="totalPages"
            :pages="pages"
            :count="auditStore.logs.length"
            :total="auditStore.total"
            label="registros"
            @go="goToPage"
        />
    </div>
</template>

<style scoped>
.auditoria-page {
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

.filters-bar {
    display: flex;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 1.5rem;
}

.filter-input {
    padding: 0.6rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.95rem;
}

select.filter-input {
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path d='M1 1l5 5 5-5' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    padding-right: 2.5rem;
}

.filter-date {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.btn-primary {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 0.6rem 1.25rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-primary:hover:not(:disabled) {
    background: var(--color-secondary);
}

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.6rem 1.25rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.table-wrapper {
    overflow-x: auto;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.badge {
    display: inline-block;
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: capitalize;
}

.badge-nuevo {
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
    color: var(--color-success);
}

.badge-modificar {
    background: color-mix(in srgb, var(--color-warning) 12%, transparent);
    color: var(--color-warning);
}

.badge-consultar {
    background: color-mix(in srgb, var(--color-primary) 12%, transparent);
    color: var(--color-primary);
}

.badge-eliminar {
    background: rgba(var(--color-danger-rgb), 0.15);
    color: var(--color-danger);
}

.audit-detail {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.85rem;
}

.audit-desc {
    font-weight: 600;
    margin-bottom: 0.15rem;
}

.audit-change {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
}

.audit-field {
    color: var(--color-text-muted);
    min-width: 6.5rem;
}

.audit-arrow {
    color: var(--color-primary);
}

.audit-value {
    white-space: nowrap;
}

.audit-old {
    text-decoration: line-through;
    color: var(--color-text-muted);
}

.audit-new {
    font-weight: 600;
    color: var(--color-success);
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

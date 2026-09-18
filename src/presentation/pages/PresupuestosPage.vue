<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { usePresupuestosStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import { useConfirm } from "../composables/useConfirm";
import { formatMoney } from "../utils/format";
import type { PresupuestoEstado, PresupuestoWithDetalle } from "../../domain/entities";
import { DEFAULT_CLIENT_LABEL } from "../../domain/entities";

const router = useRouter();
const presupuestosStore = usePresupuestosStore();
const { canGenerarPresupuesto, canCreateVenta } = usePermissions();
const { error: toastError, success: toastSuccess } = useToasts();
const { confirm } = useConfirm();

const showDetailModal = ref(false);
const selectedPresupuesto = ref<PresupuestoWithDetalle | null>(null);

const estadoFilter = ref<PresupuestoEstado | "">("");
const fechaDesde = ref("");
const fechaHasta = ref("");
const searchQuery = ref("");

const estadoOptions: PresupuestoEstado[] = [
  "pendiente",
  "aprobado",
  "vencido",
  "convertido",
  "anulado",
];

const estadoLabels: Record<PresupuestoEstado, string> = {
  pendiente: "Pendiente",
  aprobado: "Aprobado",
  vencido: "Vencido",
  convertido: "Convertido",
  anulado: "Anulado",
};

const currentPage = computed(
  () => Math.floor(presupuestosStore.offset / presupuestosStore.limit) + 1,
);
const totalPages = computed(() =>
  Math.max(1, Math.ceil(presupuestosStore.total / presupuestosStore.limit)),
);

async function fetchPage(offset = 0) {
  await presupuestosStore.fetchPresupuestos({
    limit: presupuestosStore.limit,
    offset,
    estado: estadoFilter.value || undefined,
    fecha_desde: fechaDesde.value || undefined,
    fecha_hasta: fechaHasta.value || undefined,
    query: searchQuery.value.trim() || undefined,
  });
}

onMounted(async () => {
  if (canGenerarPresupuesto()) {
    await fetchPage(0);
  }
});

watch([estadoFilter, fechaDesde, fechaHasta], () => {
  fetchPage(0);
});

function applySearch() {
  fetchPage(0);
}

async function goToPage(page: number) {
  if (page < 1 || page > totalPages.value || page === currentPage.value) return;
  await fetchPage((page - 1) * presupuestosStore.limit);
}

function clearFilters() {
  estadoFilter.value = "";
  fechaDesde.value = "";
  fechaHasta.value = "";
  searchQuery.value = "";
  fetchPage(0);
}

function isTerminal(presupuesto: PresupuestoWithDetalle): boolean {
  return presupuesto.estado === "convertido" || presupuesto.estado === "anulado";
}

function estadoClass(estado: PresupuestoEstado): string {
  return `status-${estado}`;
}

function openDetailModal(presupuesto: PresupuestoWithDetalle) {
  selectedPresupuesto.value = presupuesto;
  showDetailModal.value = true;
}

function clienteNombre(presupuesto: PresupuestoWithDetalle): string {
  const name = [presupuesto.cliente_nombre, presupuesto.cliente_apellido]
    .filter(Boolean)
    .join(" ");
  return name || DEFAULT_CLIENT_LABEL;
}

async function handleConvertir(presupuesto: PresupuestoWithDetalle) {
  if (isTerminal(presupuesto) || !canCreateVenta()) return;
  if (
    !(await confirm({
      message: `¿Convertir el presupuesto N° ${presupuesto.id} en venta? Se cargarán sus artículos en el carrito de Nueva Venta.`,
      confirmText: "Convertir a venta",
    }))
  ) {
    return;
  }
  router.push({ name: "nueva-venta", query: { presupuesto_id: presupuesto.id } });
}

async function handleAnular(presupuesto: PresupuestoWithDetalle) {
  if (isTerminal(presupuesto)) return;
  if (
    !(await confirm({
      message: `¿Anular el presupuesto N° ${presupuesto.id}? Esta acción no se puede deshacer.`,
      confirmText: "Anular presupuesto",
    }))
  ) {
    return;
  }
  const success = await presupuestosStore.cambiarEstado(presupuesto.id, "anulado");
  if (success) {
    toastSuccess(`Presupuesto N° ${presupuesto.id} anulado.`);
  } else {
    toastError(presupuestosStore.error || "No se pudo anular el presupuesto.");
  }
}
</script>

<template>
    <div class="presupuestos-page">
        <div class="page-header">
            <h1>Gestión de Presupuestos</h1>
        </div>

        <div class="filters-bar">
            <div class="filter-group">
                <label>Estado</label>
                <select v-model="estadoFilter" class="filter-select">
                    <option value="">Todos</option>
                    <option
                        v-for="estado in estadoOptions"
                        :key="estado"
                        :value="estado"
                    >
                        {{ estadoLabels[estado] }}
                    </option>
                </select>
            </div>
            <div class="filter-group">
                <label>Desde</label>
                <input v-model="fechaDesde" type="date" class="filter-input" />
            </div>
            <div class="filter-group">
                <label>Hasta</label>
                <input v-model="fechaHasta" type="date" class="filter-input" />
            </div>
            <div class="filter-group filter-query">
                <label>Buscar</label>
                <input
                    v-model="searchQuery"
                    type="text"
                    placeholder="N°, cliente, usuario o artículo..."
                    class="filter-input"
                    @keydown.enter.prevent="applySearch"
                />
            </div>
            <button class="btn-secondary" @click="applySearch">Buscar</button>
            <button class="btn-secondary" @click="clearFilters">Limpiar</button>
        </div>

        <div v-if="presupuestosStore.loading" class="loading">Cargando...</div>

        <div v-if="presupuestosStore.error" class="error-banner">
            {{ presupuestosStore.error }}
        </div>

        <div class="table-wrapper">
            <table v-if="!presupuestosStore.loading" class="data-table">
                <thead>
                    <tr>
                        <th>N°</th>
                        <th>Fecha</th>
                        <th>Vencimiento</th>
                        <th>Cliente</th>
                        <th>Usuario</th>
                        <th>Items</th>
                        <th>Subtotal</th>
                        <th>Desc.</th>
                        <th>Total</th>
                        <th>Estado</th>
                        <th>Acciones</th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="presupuesto in presupuestosStore.presupuestos"
                        :key="presupuesto.id"
                    >
                        <td>{{ presupuesto.id }}</td>
                        <td>{{ new Date(presupuesto.fecha).toLocaleString() }}</td>
                        <td>{{ presupuesto.fecha_vencimiento || "—" }}</td>
                        <td>{{ clienteNombre(presupuesto) }}</td>
                        <td>{{ presupuesto.username }}</td>
                        <td>{{ presupuesto.items.length }}</td>
                        <td>{{ formatMoney(presupuesto.subtotal) }}</td>
                        <td>
                            {{ presupuesto.descuento > 0 ? `${presupuesto.descuento}%` : "—" }}
                        </td>
                        <td>{{ formatMoney(presupuesto.total) }}</td>
                        <td>
                            <span :class="['status', estadoClass(presupuesto.estado)]">
                                {{ estadoLabels[presupuesto.estado] }}
                            </span>
                        </td>
                        <td class="actions">
                            <button
                                @click="openDetailModal(presupuesto)"
                                class="btn-icon"
                                title="Ver detalle"
                            >
                                <img src="/svg/article.svg" alt="Ver detalle" />
                            </button>
                            <button
                                v-if="canCreateVenta() && !isTerminal(presupuesto)"
                                @click="handleConvertir(presupuesto)"
                                class="btn-icon"
                                title="Convertir a venta"
                            >
                                <img src="/svg/thunder.svg" alt="Convertir a venta" />
                            </button>
                            <button
                                v-if="!isTerminal(presupuesto)"
                                @click="handleAnular(presupuesto)"
                                class="btn-icon btn-danger"
                                title="Anular presupuesto"
                            >
                                <img src="/svg/trash.svg" alt="Anular presupuesto" />
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div
            v-if="!presupuestosStore.loading && presupuestosStore.presupuestos.length === 0"
            class="empty-state"
        >
            No hay presupuestos que coincidan con los filtros
        </div>

        <div
            v-if="!presupuestosStore.loading && totalPages > 1"
            class="pagination"
        >
            <button
                class="btn-secondary"
                :disabled="currentPage <= 1"
                @click="goToPage(currentPage - 1)"
            >
                Anterior
            </button>
            <span class="pagination-info">
                Página {{ currentPage }} de {{ totalPages }} —
                {{ presupuestosStore.total }} presupuestos
            </span>
            <button
                class="btn-secondary"
                :disabled="currentPage >= totalPages"
                @click="goToPage(currentPage + 1)"
            >
                Siguiente
            </button>
        </div>

        <div
            v-if="showDetailModal && selectedPresupuesto"
            class="modal-overlay"
            @click.self="showDetailModal = false"
        >
            <div class="modal modal-large">
                <h2>Presupuesto N° {{ selectedPresupuesto.id }}</h2>
                <p class="detail-meta">
                    Fecha: {{ new Date(selectedPresupuesto.fecha).toLocaleString() }}
                    — Usuario: {{ selectedPresupuesto.username }}
                    <span
                        :class="['status', estadoClass(selectedPresupuesto.estado)]"
                    >
                        {{ estadoLabels[selectedPresupuesto.estado] }}
                    </span>
                </p>
                <p class="detail-meta">
                    Cliente: {{ clienteNombre(selectedPresupuesto) }}
                </p>
                <p v-if="selectedPresupuesto.fecha_vencimiento" class="detail-meta">
                    Vencimiento: {{ selectedPresupuesto.fecha_vencimiento }}
                </p>
                <p v-if="selectedPresupuesto.observacion" class="detail-meta">
                    Observación: {{ selectedPresupuesto.observacion }}
                </p>
                <table class="data-table">
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
                        <tr
                            v-for="item in selectedPresupuesto.items"
                            :key="item.id"
                        >
                            <td>{{ item.cod_articulo }}</td>
                            <td>{{ item.articulo }}</td>
                            <td>{{ item.cantidad }}</td>
                            <td>{{ formatMoney(item.precio_unitario) }}</td>
                            <td>{{ formatMoney(item.subtotal) }}</td>
                        </tr>
                    </tbody>
                </table>
                <div class="cart-totals">
                    <p>Subtotal: {{ formatMoney(selectedPresupuesto.subtotal) }}</p>
                    <p v-if="selectedPresupuesto.descuento > 0">
                        Descuento ({{ selectedPresupuesto.descuento }}%):
                        −{{ formatMoney((selectedPresupuesto.subtotal * selectedPresupuesto.descuento) / 100) }}
                    </p>
                    <p class="cart-total">
                        Total: {{ formatMoney(selectedPresupuesto.total) }}
                    </p>
                </div>
                <div class="modal-actions">
                    <button
                        @click="showDetailModal = false"
                        class="btn-secondary"
                    >
                        Cerrar
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.presupuestos-page {
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
    align-items: flex-end;
    gap: 1rem;
    flex-wrap: wrap;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: var(--color-surface);
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.1);
}

.filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
}

.filter-group label {
    font-size: 0.85rem;
    color: var(--color-text-muted);
    font-weight: 500;
}

.filter-query {
    flex: 1;
    min-width: 220px;
}

.filter-input,
.filter-select {
    padding: 0.6rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 0.95rem;
    background: var(--color-surface);
    color: var(--color-text);
    box-sizing: border-box;
}

.filter-select {
    min-width: 150px;
}

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1.5rem;
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
    box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.1);
}

.status {
    font-weight: 500;
}

.status-pendiente {
    color: var(--color-warning);
}

.status-aprobado {
    color: var(--color-success);
}

.status-vencido {
    color: var(--color-danger);
}

.status-convertido {
    color: var(--color-primary);
}

.status-anulado {
    color: var(--color-danger);
}

.actions {
    display: flex;
    gap: 0.5rem;
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

.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal {
    background: var(--color-surface);
    padding: 2rem;
    border-radius: 12px;
    width: 100%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
}

.modal-large {
    max-width: 720px;
}

.modal h2 {
    margin: 0 0 1.5rem;
}

.detail-meta {
    margin-bottom: 0.5rem;
    color: var(--color-text-muted);
}

.cart-totals {
    margin-top: 1rem;
    text-align: right;
}

.cart-totals p {
    margin: 0.25rem 0;
}

.cart-total {
    font-size: 1.1rem;
    font-weight: 600;
}

.modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
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

.pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-top: 1.5rem;
}

.pagination button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.pagination-info {
    color: var(--color-text-muted);
}
</style>

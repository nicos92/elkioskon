<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useVentasStore, useStockStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import { useConfirm } from "../composables/useConfirm";
import { formatMoney } from "../utils/format";
import type { VentaWithDetalle } from "../../domain/entities";
import { DEFAULT_CLIENT_LABEL } from "../../domain/entities";
import PrintArea from "../components/ui/PrintArea.vue";

const router = useRouter();
const ventasStore = useVentasStore();
const { canCreateVenta, canAnularVenta, canGenerarPresupuesto } = usePermissions();
const { error: toastError, success: toastSuccess } = useToasts();
const { confirm } = useConfirm();

const showDetailModal = ref(false);
const selectedVenta = ref<VentaWithDetalle | null>(null);

const searchQuery = ref("");

const currentPage = computed(
    () => Math.floor(ventasStore.offset / ventasStore.limit) + 1,
);
const totalPages = computed(() =>
    Math.max(1, Math.ceil(ventasStore.total / ventasStore.limit)),
);

const filteredVentas = computed(() => {
    const query = searchQuery.value.toLowerCase().trim();
    if (!query) return ventasStore.ventas;
    return ventasStore.ventas.filter(
        (v) =>
            v.username.toLowerCase().includes(query) ||
            v.items.some(
                (i) =>
                    i.articulo.toLowerCase().includes(query) ||
                    i.cod_articulo.toLowerCase().includes(query),
            ),
    );
});

onMounted(async () => {
    await Promise.all([
        ventasStore.fetchVentas({ limit: 50, offset: 0 }),
        ventasStore.checkDiaCerrado(),
    ]);
});

async function goToPage(page: number) {
    if (page < 1 || page > totalPages.value || page === currentPage.value) return;
    await ventasStore.fetchVentas({
        limit: ventasStore.limit,
        offset: (page - 1) * ventasStore.limit,
    });
}

function irANuevaVenta() {
    router.push({ name: "nueva-venta" });
}

function openDetailModal(venta: VentaWithDetalle) {
    selectedVenta.value = venta;
    showDetailModal.value = true;
}

async function handleAnular(id: number) {
    if (
        !(await confirm({
            message:
                "¿Está seguro de anular esta venta? Se devolverán las cantidades al stock.",
            confirmText: "Anular venta",
        }))
    ) {
        return;
    }
    const success = await ventasStore.anularVenta(id);
    if (success) {
        toastSuccess("Venta anulada y stock restaurado.");
        await useStockStore().fetchStock();
    } else {
        toastError(ventasStore.error || "No se pudo anular la venta.");
    }
}

function generarPdfDetalle() {
    if (!selectedVenta.value) return;
    window.print();
}

function clienteNombre(venta: VentaWithDetalle): string {
    const name = [venta.cliente_nombre, venta.cliente_apellido]
        .filter(Boolean)
        .join(" ");
    return name || DEFAULT_CLIENT_LABEL;
}
</script>

<template>
    <div class="ventas-page">
        <div class="page-header">
            <h1>Gestión de Ventas</h1>
            <button
                v-if="canCreateVenta()"
                @click="irANuevaVenta"
                class="btn-primary"
                :disabled="ventasStore.diaCerrado"
            >
                Nueva Venta
            </button>
        </div>

        <div v-if="ventasStore.diaCerrado" class="dia-cerrado-banner">
            Día cerrado, no se pueden ingresar más ventas.
        </div>

        <div class="search-bar">
            <input
                v-model="searchQuery"
                type="text"
                placeholder="Buscar por usuario o artículo..."
                class="search-input"
            />
        </div>

        <div v-if="ventasStore.loading" class="loading">Cargando...</div>

        <div v-if="ventasStore.error" class="error-banner">
            {{ ventasStore.error }}
        </div>

        <div class="table-wrapper">
        <table v-if="!ventasStore.loading" class="data-table">
            <thead>
                <tr>
                    <th>N°</th>
                    <th>Fecha</th>
                    <th>Usuario</th>
                    <th>Tipo</th>
                    <th>Items</th>
                    <th>Subtotal</th>
                    <th>Desc.</th>
                    <th>Total</th>
                    <th>Estado</th>
                    <th>Acciones</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="venta in filteredVentas" :key="venta.id">
                    <td>{{ venta.id }}</td>
                    <td>{{ new Date(venta.fecha).toLocaleString() }}</td>
                    <td>{{ venta.username }}</td>
                    <td>{{ venta.tipo_venta || "Efectivo" }}</td>
                    <td>{{ venta.items.length }}</td>
                    <td>{{ formatMoney(venta.subtotal) }}</td>
                    <td>{{ venta.descuento > 0 ? `${venta.descuento}%` : "—" }}</td>
                    <td>{{ formatMoney(venta.total) }}</td>
                    <td>
                        <span
                            :class="
                                venta.anulada
                                    ? 'status-anulada'
                                    : 'status-activa'
                            "
                        >
                            {{ venta.anulada ? "Anulada" : "Activa" }}
                        </span>
                    </td>
                    <td class="actions">
                        <button
                            @click="openDetailModal(venta)"
                            class="btn-icon"
                            title="Ver detalle"
                        >
                            <img src="/svg/article.svg" alt="Ver detalle" />
                        </button>
                        <button
                            v-if="canAnularVenta() && !venta.anulada"
                            @click="handleAnular(venta.id)"
                            class="btn-icon btn-danger"
                            title="Anular venta"
                        >
                            <img src="/svg/trash.svg" alt="Anular venta" />
                        </button>
                    </td>
                </tr>
            </tbody>
        </table>
        </div>

        <div
            v-if="!ventasStore.loading && filteredVentas.length === 0"
            class="empty-state"
        >
            No hay ventas que coincidan con la búsqueda
        </div>

        <div v-if="!ventasStore.loading && totalPages > 1" class="pagination">
            <button
                class="btn-secondary"
                :disabled="currentPage <= 1"
                @click="goToPage(currentPage - 1)"
            >
                Anterior
            </button>
            <span class="pagination-info">
                Página {{ currentPage }} de {{ totalPages }} — {{ ventasStore.total }} ventas
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
            v-if="showDetailModal && selectedVenta"
            class="modal-overlay"
            @click.self="showDetailModal = false"
        >
            <div class="modal modal-large">
                <h2>Venta N° {{ selectedVenta.id }}</h2>
                <p class="detail-meta">
                    Fecha: {{ new Date(selectedVenta.fecha).toLocaleString() }}
                    — Usuario: {{ selectedVenta.username }}
                    <span
                        :class="
                            selectedVenta.anulada
                                ? 'status-anulada'
                                : 'status-activa'
                        "
                    >
                        {{ selectedVenta.anulada ? "Anulada" : "Activa" }}
                    </span>
                </p>
                <p class="detail-meta">
                    Cliente: {{ clienteNombre(selectedVenta) }}
                </p>
                <p v-if="selectedVenta.observacion" class="detail-meta">
                    Observación: {{ selectedVenta.observacion }}
                </p>
                <p v-if="selectedVenta.tipo_venta" class="detail-meta">
                    Tipo de venta: {{ selectedVenta.tipo_venta }}
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
                            v-for="item in selectedVenta.items"
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
                    <p>Subtotal: {{ formatMoney(selectedVenta.subtotal) }}</p>
                    <p v-if="selectedVenta.descuento > 0">
                        Descuento ({{ selectedVenta.descuento }}%):
                        −{{ formatMoney((selectedVenta.subtotal * selectedVenta.descuento) / 100) }}
                    </p>
                    <p class="cart-total">
                        Total: {{ formatMoney(selectedVenta.total) }}
                    </p>
                </div>
                <div class="modal-actions">
                    <button
                        v-if="canGenerarPresupuesto()"
                        @click="generarPdfDetalle"
                        class="btn-secondary"
                    >
                        Generar PDF
                    </button>
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

    <PrintArea
        v-if="selectedVenta"
        :titulo="`Venta N° ${selectedVenta.id}`"
        :info="[
            {
                label: 'Fecha',
                value: new Date(selectedVenta.fecha).toLocaleString(),
            },
            { label: 'Usuario', value: selectedVenta.username },
            { label: 'Cliente', value: clienteNombre(selectedVenta) },
        ]"
        :items="
            selectedVenta.items.map((item) => ({
                codigo: item.cod_articulo,
                articulo: item.articulo,
                cantidad: item.cantidad,
                precio: item.precio_unitario,
                subtotal: item.subtotal,
            }))
        "
        :subtotal="selectedVenta.subtotal"
        :descuento="selectedVenta.descuento"
        :descuento-monto="
            (selectedVenta.subtotal * selectedVenta.descuento) / 100
        "
        :total="selectedVenta.total"
        :observacion="selectedVenta.observacion || undefined"
        :nota="
            `Estado: ${selectedVenta.anulada ? 'Anulada' : 'Activa'}`
        "
    />
</template>

<style scoped>
.ventas-page {
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

.dia-cerrado-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1.5rem;
    font-weight: 600;
}

.search-bar {
    margin-bottom: 1.5rem;
}

.search-input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
    background: var(--color-surface);
    color: var(--color-text);
}

.search-input:focus {
    outline: none;
    border-color: var(--color-primary);
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

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.table-wrapper {
    overflow-x: auto;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.1);
}

.status-activa {
    color: var(--color-success);
    font-weight: 500;
}

.status-anulada {
    color: var(--color-danger);
    font-weight: 500;
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

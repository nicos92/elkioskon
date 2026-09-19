<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  useVentasStore,
  useStockStore,
  useArticulosStore,
  useTiposVentaStore,
  useClientesStore,
  usePresupuestosStore,
  useNocturnoStore,
} from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import { useCart } from "../composables/useCart";
import type { CartItem } from "../composables/useCart";
import { usePreciosVenta } from "../composables/venta/usePreciosVenta";
import { useClienteSeleccion } from "../composables/venta/useClienteSeleccion";
import { usePresupuestoOrigen } from "../composables/venta/usePresupuestoOrigen";
import type {
  CreatePresupuestoRequest,
  CreateVentaRequest,
} from "../../domain/entities";
import ArticuloSearch from "../components/venta/ArticuloSearch.vue";
import CartTable from "../components/venta/CartTable.vue";
import ClienteSelector from "../components/venta/ClienteSelector.vue";
import NuevoClienteModal from "../components/venta/NuevoClienteModal.vue";
import PresupuestoPrintArea from "../components/venta/PresupuestoPrintArea.vue";
import TotalsPanel from "../components/venta/TotalsPanel.vue";

const router = useRouter();
const ventasStore = useVentasStore();
const stockStore = useStockStore();
const articulosStore = useArticulosStore();
const tiposVentaStore = useTiposVentaStore();
const clientesStore = useClientesStore();
const presupuestosStore = usePresupuestosStore();
const nocturnoStore = useNocturnoStore();
const {
  canVenderSinStock,
  canGenerarPresupuesto,
  canViewClientes,
  canCreateCliente,
} = usePermissions();
const { error: toastError, success: toastSuccess, warning: toastWarning } = useToasts();

const articuloSearchRef = ref<InstanceType<typeof ArticuloSearch> | null>(null);

const observacion = ref("");
const tipoVentaId = ref<number | null>(null);
const fechaVencimiento = ref("");

watch(
    () => tiposVentaStore.tipos,
    (tipos) => {
        if (tipoVentaId.value === null && tipos.length > 0) {
            const efectivo = tipos.find((t) => t.nombre === "Efectivo");
            tipoVentaId.value = efectivo ? efectivo.id : tipos[0].id;
        }
    },
    { immediate: true },
);

const precios = usePreciosVenta({
    stockStore,
    articulosStore,
    nocturnoStore,
    getCart: () => cart.value,
});

const { esNocturnoActual } = precios;

const cartLogic = useCart({
    getVendibles: () => precios.articulosVendibles.value,
    canVenderSinStock,
    getTipoVentaId: () => tipoVentaId.value,
    focusInput: () => articuloSearchRef.value?.focus(),
});

const {
    cart,
    searchQuery,
    descuento,
    searchResults,
    carritoSubtotal,
    descuentoMonto,
    carritoTotal,
    carritoValido,
    presupuestoValido,
    focusSearch,
    addArticuloById,
    onSearchEnter,
    removeArticulo,
    vaciarCarrito,
    updateSubtotal,
    stockWarning,
    setItems,
    resetCart,
} = cartLogic;

const cliente = useClienteSeleccion({ clientesStore });

const presupuesto = usePresupuestoOrigen({
    router,
    presupuestosStore,
    clientesStore,
    getArticulosVendibles: () => precios.articulosVendibles.value,
    clienteSeleccionado: cliente.clienteSeleccionado,
    clienteDefecto: cliente.clienteDefecto,
    descuento,
    observacion,
    fechaVencimiento,
    setItems,
});

const {
    presupuestoOrigen,
    precargandoPresupuesto,
    quitarPresupuesto,
} = presupuesto;

const {
    clienteSeleccionado,
    clienteQuery,
    mostrandoClientes,
    mostrarModalNuevoCliente,
    clientesFiltrados,
    seleccionarCliente,
    quitarCliente,
    abrirModalNuevoCliente,
    crearClienteRapido,
} = cliente;

watch(precios.esNocturnoActual, (esNocturno, antes) => {
    if (esNocturno && !antes) {
        toastWarning("Precios nocturnos activos.");
    }
});

const fechaHoy = computed(() => new Date().toLocaleDateString());
const cartCount = computed(() => cart.value.length);

onMounted(async () => {
    await Promise.all([
        stockStore.fetchStock(),
        articulosStore.fetchArticulos(),
        tiposVentaStore.fetchTiposVenta(),
        nocturnoStore.fetchConfig(),
        canViewClientes() ? clientesStore.fetchClientes() : Promise.resolve(),
    ]);
    await ventasStore.checkDiaCerrado();
    if (canViewClientes()) {
        await cliente.cargarDefecto();
    }
    const presupuestoId = router.currentRoute.value.query.presupuesto_id;
    if (presupuestoId) {
        const resultado = await presupuesto.cargarPresupuesto(
            Number(presupuestoId),
        );
        if (resultado === "no-encontrado") {
            toastError("No se pudo cargar el presupuesto.");
        } else if (resultado === "estado-terminal") {
            toastError(
                "El presupuesto no se puede convertir porque su estado no lo permite.",
            );
        } else if (resultado === "ok") {
            toastSuccess(`Presupuesto N° ${presupuestoId} cargado.`);
        } else {
            toastError("No se pudo cargar el presupuesto.");
        }
    }
    focusSearch();
});

function resetForm() {
    resetCart();
    observacion.value = "";
    const efectivo = tiposVentaStore.tipos.find((t) => t.nombre === "Efectivo");
    tipoVentaId.value = efectivo
        ? efectivo.id
        : tiposVentaStore.tipos[0]?.id ?? null;
    quitarCliente();
}

async function handleCreate() {
    if (!carritoValido.value) return;
    const request: CreateVentaRequest = {
        items: cart.value.map((item) => ({
            id_articulo: item.id_articulo,
            cantidad: item.cantidad,
            precio_unitario: item.precio,
        })),
        descuento: descuento.value || 0,
        observacion: observacion.value.trim() || undefined,
        id_tipo_venta: tipoVentaId.value || undefined,
        cliente_id: clienteSeleccionado.value?.id,
    };
    const venta = await ventasStore.convertirPresupuestoEnVenta(
        request,
        presupuestoOrigen.value ?? undefined,
    );
    if (venta) {
        toastSuccess(`Venta N° ${venta.venta.id} registrada.`);
        if (presupuestoOrigen.value) {
            const presupuestoId = presupuestoOrigen.value;
            if (venta.presupuestoConvertido) {
                toastSuccess(
                    `Presupuesto N° ${presupuestoId} marcado como convertido.`,
                );
            } else {
                toastError(
                    "La venta se registró, pero no se pudo marcar el presupuesto como convertido.",
                );
            }
            presupuestoOrigen.value = null;
            router.replace({ name: "nueva-venta" });
            await presupuestosStore.fetchPresupuestos();
        }
        await stockStore.fetchStock();
        resetForm();
        focusSearch();
    } else {
        toastError(ventasStore.error || "No se pudo registrar la venta.");
    }
}

async function handleGuardarPresupuesto() {
    if (!carritoValido.value) return;
    const request: CreatePresupuestoRequest = {
        items: cart.value.map((item) => ({
            id_articulo: item.id_articulo,
            cantidad: item.cantidad,
            precio_unitario: item.precio,
        })),
        descuento: descuento.value || 0,
        observacion: observacion.value.trim() || undefined,
        fecha_vencimiento: fechaVencimiento.value.trim() || undefined,
        cliente_id: clienteSeleccionado.value?.id,
    };
    const presupuesto = await presupuestosStore.crearPresupuesto(request);
    if (presupuesto) {
        toastSuccess(`Presupuesto N° ${presupuesto.id} guardado.`);
        fechaVencimiento.value = "";
        focusSearch();
    } else {
        toastError(
            presupuestosStore.error || "No se pudo guardar el presupuesto.",
        );
    }
}

function cancelar() {
    router.push({ name: "ventas" });
}

function onCantidadChange(item: CartItem, value: number) {
    item.cantidad = value;
    updateSubtotal(item);
}

function onPrecioChange(item: CartItem, value: number) {
    item.precio = value;
    updateSubtotal(item);
}

function generarPdf() {
    if (cart.value.length === 0) return;
    window.print();
}
</script>

<template>
    <div class="nueva-venta-page">
        <header class="page-header">
            <div class="header-left">
                <button
                    type="button"
                    @click="cancelar"
                    class="btn-secondary btn-volver"
                >
                    ← Volver
                </button>
                <h1>Nueva Venta</h1>
            </div>
            <div
                v-if="nocturnoStore.config.activo"
                class="turno-chip"
                :class="esNocturnoActual ? 'turno-nocturno' : 'turno-diurno'"
                :title="
                    esNocturnoActual
                        ? 'Ganancia nocturna activa'
                        : 'Ganancia diurna activa'
                "
            >
                <span v-if="esNocturnoActual">Turno noche</span>
                <span v-else>Turno día</span>
            </div>
        </header>

        <div v-if="ventasStore.diaCerrado" class="dia-cerrado-banner">
            Día cerrado, no se pueden ingresar más ventas.
        </div>

        <div v-if="presupuestoOrigen" class="presupuesto-banner">
            <span>
                Presupuesto N° {{ presupuestoOrigen }} cargado
                <template v-if="precargandoPresupuesto"> (cargando...)</template>
            </span>
            <button
                type="button"
                class="btn-secondary"
                @click="quitarPresupuesto"
            >
                Quitar
            </button>
        </div>

        <div class="workspace">
            <section class="scan-column">
                <div class="panel scan-panel">
                    <ArticuloSearch
                        ref="articuloSearchRef"
                        :query="searchQuery"
                        :results="searchResults"
                        @update:query="searchQuery = $event"
                        @select="addArticuloById"
                        @enter="onSearchEnter"
                    />
                </div>

                <div class="panel cart-panel">
                    <div class="cart-header">
                        <h2>
                            Artículos
                            <span v-if="cartCount > 0" class="cart-count">
                                {{ cartCount }}
                            </span>
                        </h2>
                        <button
                            v-if="cart.length > 0"
                            type="button"
                            @click="vaciarCarrito"
                            class="btn-ghost"
                        >
                            Vaciar
                        </button>
                    </div>

                    <CartTable
                        :items="cart"
                        :stock-warning="stockWarning"
                        @update-cantidad="onCantidadChange"
                        @update-precio="onPrecioChange"
                        @remove="removeArticulo"
                    />

                    <div v-if="cart.length === 0" class="empty-state">
                        <p class="empty-title">No hay artículos todavía</p>
                        <p class="empty-hint">
                            Escanee o escriba un código y presione Enter.
                        </p>
                    </div>
                </div>
            </section>

            <aside class="drawer-column">
                <div class="drawer">
                    <div class="drawer-commit">
                        <TotalsPanel
                            :subtotal="carritoSubtotal"
                            :descuento="descuento"
                            :descuento-monto="descuentoMonto"
                            :total="carritoTotal"
                            :count="cartCount"
                            @update:descuento="descuento = $event"
                        />
                        <div class="drawer-actions">
                            <button
                                type="button"
                                @click="handleCreate"
                                class="btn-registrar"
                                :disabled="
                                    !carritoValido || ventasStore.diaCerrado
                                "
                            >
                                Registrar Venta
                            </button>
                            <button
                                v-if="canGenerarPresupuesto()"
                                type="button"
                                @click="handleGuardarPresupuesto"
                                class="btn-secondary btn-guardar-presupuesto"
                                :disabled="!presupuestoValido"
                            >
                                Guardar Presupuesto
                            </button>
                        </div>
                    </div>

                    <div class="drawer-details">
                        <div class="form-group">
                            <label>Tipo de venta</label>
                            <select v-model.number="tipoVentaId" class="tipo-select">
                                <option
                                    v-for="tipo in tiposVentaStore.tipos"
                                    :key="tipo.id"
                                    :value="tipo.id"
                                >
                                    {{ tipo.nombre }}
                                </option>
                            </select>
                        </div>
                        <ClienteSelector
                            v-if="canViewClientes()"
                            :clientes="clientesFiltrados"
                            :query="clienteQuery"
                            :show="mostrandoClientes"
                            :selected="clienteSeleccionado"
                            :can-create="canCreateCliente()"
                            @update:query="clienteQuery = $event"
                            @update:show="mostrandoClientes = $event"
                            @select="seleccionarCliente"
                            @clear="quitarCliente"
                            @create="abrirModalNuevoCliente"
                        />
                        <div class="form-group">
                            <label>Observación</label>
                            <input
                                v-model="observacion"
                                type="text"
                                placeholder="Opcional"
                            />
                        </div>
                        <div
                            v-if="canGenerarPresupuesto()"
                            class="form-group"
                        >
                            <label>Vencimiento del presupuesto</label>
                            <input
                                v-model="fechaVencimiento"
                                type="date"
                                placeholder="Opcional"
                            />
                        </div>
                    </div>

                    <div class="drawer-footer">
                        <button
                            v-if="canGenerarPresupuesto()"
                            type="button"
                            @click="generarPdf"
                            class="btn-ghost"
                            :disabled="cart.length === 0"
                        >
                            PDF
                        </button>
                        <button
                            type="button"
                            @click="cancelar"
                            class="btn-ghost"
                        >
                            Cancelar
                        </button>
                    </div>
                </div>
            </aside>
        </div>

        <div v-if="ventasStore.error" class="error-banner">
            {{ ventasStore.error }}
        </div>

        <NuevoClienteModal
            v-model="mostrarModalNuevoCliente"
            :error="clientesStore.error"
            @submit="crearClienteRapido"
        />

        <PresupuestoPrintArea
            :fecha="fechaHoy"
            :cliente="clienteSeleccionado"
            :items="cart"
            :subtotal="carritoSubtotal"
            :descuento="descuento"
            :descuento-monto="descuentoMonto"
            :total="carritoTotal"
            :observacion="observacion"
        />
    </div>
</template>

<style scoped>
.nueva-venta-page {
    padding: 1.25rem;
    background: var(--color-bg);
    min-height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.header-left h1 {
    margin: 0;
    font-size: 1.5rem;
}

.turno-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.3rem 0.85rem;
    border-radius: 999px;
    font-size: 0.78rem;
    font-weight: 600;
    white-space: nowrap;
}

.turno-diurno {
    color: var(--color-text-muted);
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
}

.turno-nocturno {
    color: #fff;
    background: var(--color-warning);
}

.dia-cerrado-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-weight: 600;
}

.presupuesto-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    color: var(--color-primary);
    background: color-mix(in srgb, var(--color-primary) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-weight: 600;
}

.presupuesto-banner .btn-secondary {
    padding: 0.35rem 0.75rem;
    font-size: 0.85rem;
}

.workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 360px;
    gap: 1rem;
    align-items: start;
    flex: 1;
}

.scan-column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
}

.panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 1.25rem;
}

.scan-panel {
    padding: 1rem 1.25rem 1.25rem;
}

.cart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
}

.cart-header h2 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.05rem;
    margin: 0;
}

.cart-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.4rem;
    height: 1.4rem;
    padding: 0 0.4rem;
    border-radius: 999px;
    background: var(--color-surface-2);
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    font-weight: 600;
}

.drawer-column {
    position: sticky;
    top: 1rem;
    align-self: start;
}

.drawer {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 1.25rem;
    max-height: calc(100vh - 56px - 2rem);
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
}

.drawer-commit {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.drawer-actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border);
}

.btn-registrar {
    width: 100%;
    background: var(--color-success);
    color: var(--color-on-success);
    border: none;
    padding: 0.9rem 1rem;
    border-radius: 8px;
    font-size: 1.05rem;
    font-weight: 700;
    cursor: pointer;
}

.btn-registrar:hover:not(:disabled) {
    filter: brightness(1.08);
}

.btn-registrar:disabled {
    opacity: 0.55;
    cursor: not-allowed;
}

.btn-guardar-presupuesto {
    width: 100%;
}

.drawer-details {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border);
}

.drawer-details .form-group {
    margin: 0;
}

.drawer-footer {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border);
}

.btn-ghost {
    background: none;
    border: none;
    color: var(--color-text-muted);
    padding: 0.5rem;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    font-weight: 500;
}

.btn-ghost:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-surface-2);
}

.btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-volver {
    padding: 0.5rem 1rem;
}

.empty-state {
    text-align: center;
    padding: 2.5rem 1rem;
    color: var(--color-text-muted);
    border: 1px dashed var(--color-border);
    border-radius: 8px;
}

.empty-title {
    font-weight: 600;
    margin-bottom: 0.25rem;
    color: var(--color-text-secondary);
}

.empty-hint {
    font-size: 0.9rem;
}

.error-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 8px;
}

button:focus-visible,
input:focus-visible,
select:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 2px;
}

@media (max-width: 900px) {
    .nueva-venta-page {
        padding: 0.75rem;
    }

    .workspace {
        grid-template-columns: 1fr;
    }

    .drawer-column {
        position: static;
    }

    .drawer {
        max-height: none;
        overflow: visible;
        padding-bottom: 18rem;
    }

    .drawer-commit {
        position: fixed;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 15;
        background: var(--color-surface);
        border-top: 2px solid var(--color-border);
        box-shadow: 0 -4px 12px rgba(var(--color-shadow-rgb), 0.12);
        padding: 0.6rem 0.75rem 0.75rem;
    }
}
</style>
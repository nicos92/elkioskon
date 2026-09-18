<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
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
import type { CartItem, CartSourceItem } from "../composables/useCart";
import type {
  Cliente,
  CreateClienteRequest,
  CreatePresupuestoRequest,
  CreateVentaRequest,
} from "../../domain/entities";
import {
  calcularPrecioVenta,
  esHorarioNocturno,
  margenEfectivo,
  minutosDesdeMedianoche,
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
const presupuestoOrigen = ref<number | null>(null);
const precargandoPresupuesto = ref(false);

const clienteSeleccionado = ref<Cliente | null>(null);
const clienteDefecto = ref<Cliente | null>(null);
const clienteQuery = ref("");
const mostrandoClientes = ref(false);
const mostrarModalNuevoCliente = ref(false);

const clientesFiltrados = computed<Cliente[]>(() => {
  const query = clienteQuery.value.trim().toLowerCase();
  const base = clientesStore.clientes;
  if (!query) return base;
  return base.filter(
    (c) =>
      (c.nombre || "").toLowerCase().includes(query) ||
      (c.apellido || "").toLowerCase().includes(query) ||
      (c.telefono || "").toLowerCase().includes(query) ||
      (c.email || "").toLowerCase().includes(query),
  );
});

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

function horaActualEnMinutos(): number {
    const d = new Date();
    return d.getHours() * 60 + d.getMinutes();
}

const minutosActuales = ref(horaActualEnMinutos());
let relojTimer: number | undefined;

const esNocturnoActual = computed(() => {
    const cfg = nocturnoStore.config;
    if (!cfg.activo) return false;
    const inicio = minutosDesdeMedianoche(cfg.hora_inicio);
    const fin = minutosDesdeMedianoche(cfg.hora_fin);
    if (inicio === null || fin === null) return false;
    return esHorarioNocturno(minutosActuales.value, inicio, fin);
});

const articulosVendibles = computed<CartSourceItem[]>(() => {
  const cfg = nocturnoStore.config;
  return stockStore.stocks.map((s) => {
    const articulo = articulosStore.articulos.find(
      (a) => a.id === s.id_articulo,
    );
    const margen = margenEfectivo(
      s.ganancia,
      s.ganancia_diurna,
      s.ganancia_nocturna,
      cfg.activo,
      esNocturnoActual.value,
    );
    return {
      id_articulo: s.id_articulo,
      cod_articulo: articulo?.cod_articulo || "-",
      articulo: articulo?.articulo || "Sin artículo",
      stockDisponible: s.cantidad,
      precioVenta: calcularPrecioVenta(s.costo, margen),
    };
  });
});

const cartLogic = useCart({
  getVendibles: () => articulosVendibles.value,
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

watch(esNocturnoActual, (esNocturno, antes) => {
  if (esNocturno && !antes) {
    toastWarning("Precios nocturnos activos.");
  }
});

watch(
  () => [
    nocturnoStore.config.activo,
    nocturnoStore.config.hora_inicio,
    nocturnoStore.config.hora_fin,
    minutosActuales.value,
  ],
  () => {
    const cfg = nocturnoStore.config;
    for (const item of cart.value) {
      const s = stockStore.stocks.find(
        (stock) => stock.id_articulo === item.id_articulo,
      );
      if (!s) continue;
      const margen = margenEfectivo(
        s.ganancia,
        s.ganancia_diurna,
        s.ganancia_nocturna,
        cfg.activo,
        esNocturnoActual.value,
      );
      item.precio = calcularPrecioVenta(s.costo, margen);
      item.subtotal = item.cantidad * item.precio;
    }
  },
);

const fechaHoy = computed(() => new Date().toLocaleDateString());

onMounted(async () => {
  await Promise.all([
    stockStore.fetchStock(),
    articulosStore.fetchArticulos(),
    tiposVentaStore.fetchTiposVenta(),
    nocturnoStore.fetchConfig(),
    canViewClientes() ? clientesStore.fetchClientes() : Promise.resolve(),
  ]);
  minutosActuales.value = horaActualEnMinutos();
  relojTimer = window.setInterval(() => {
    minutosActuales.value = horaActualEnMinutos();
  }, 30000);
  window.addEventListener("focus", refrescarTurno);
  document.addEventListener("visibilitychange", refrescarTurno);
  await ventasStore.checkDiaCerrado();
  if (canViewClientes()) {
    const clienteDef = await clientesStore.getClienteDefecto();
    clienteDefecto.value = clienteDef;
    clienteSeleccionado.value = clienteDef;
  }
  const presupuestoId = router.currentRoute.value.query.presupuesto_id;
  if (presupuestoId) {
    await cargarPresupuesto(Number(presupuestoId));
  }
  focusSearch();
});

function refrescarTurno() {
  if (!document.hidden) {
    minutosActuales.value = horaActualEnMinutos();
  }
}

onUnmounted(() => {
  if (relojTimer !== undefined) {
    window.clearInterval(relojTimer);
  }
  window.removeEventListener("focus", refrescarTurno);
  document.removeEventListener("visibilitychange", refrescarTurno);
});

function quitarPresupuesto() {
  presupuestoOrigen.value = null;
  router.replace({ name: "nueva-venta" });
}

async function cargarPresupuesto(id: number) {
  precargandoPresupuesto.value = true;
  try {
    const presupuesto = await presupuestosStore.getPresupuestoById(id);
    if (!presupuesto) {
      toastError("No se pudo cargar el presupuesto.");
      return;
    }
    if (
      presupuesto.estado === "convertido" ||
      presupuesto.estado === "anulado"
    ) {
      toastError(
        "El presupuesto no se puede convertir porque su estado no lo permite.",
      );
      router.replace({ name: "nueva-venta" });
      return;
    }
    descuento.value = presupuesto.descuento || 0;
    observacion.value = presupuesto.observacion || "";
    fechaVencimiento.value = presupuesto.fecha_vencimiento || "";
    if (presupuesto.cliente_id) {
      const cliente = clientesStore.clientes.find(
        (c) => c.id === presupuesto.cliente_id,
      );
      clienteSeleccionado.value = cliente ?? clienteDefecto.value;
    } else {
      clienteSeleccionado.value = clienteDefecto.value;
    }
    const items: CartItem[] = [];
    for (const item of presupuesto.items) {
      const stock = articulosVendibles.value.find(
        (s) => s.id_articulo === item.id_articulo,
      );
      items.push({
        id_articulo: item.id_articulo,
        cod_articulo: item.cod_articulo,
        articulo: item.articulo,
        stockDisponible: stock?.stockDisponible ?? 0,
        cantidad: item.cantidad,
        precio: item.precio_unitario,
        subtotal: item.subtotal,
      });
    }
    setItems(items);
    presupuestoOrigen.value = id;
    toastSuccess(`Presupuesto N° ${id} cargado.`);
  } catch {
    toastError("No se pudo cargar el presupuesto.");
  } finally {
    precargandoPresupuesto.value = false;
  }
}

function resetForm() {
  resetCart();
  observacion.value = "";
  const efectivo = tiposVentaStore.tipos.find((t) => t.nombre === "Efectivo");
  tipoVentaId.value = efectivo
    ? efectivo.id
    : tiposVentaStore.tipos[0]?.id ?? null;
  clienteSeleccionado.value = clienteDefecto.value;
  clienteQuery.value = "";
  mostrandoClientes.value = false;
}

function seleccionarCliente(cliente: Cliente) {
  clienteSeleccionado.value = cliente;
  clienteQuery.value = "";
  mostrandoClientes.value = false;
}

function quitarCliente() {
  clienteSeleccionado.value = clienteDefecto.value;
  clienteQuery.value = "";
  mostrandoClientes.value = false;
}

function abrirModalNuevoCliente() {
  clientesStore.error = null;
  mostrarModalNuevoCliente.value = true;
}

async function crearClienteRapido(request: CreateClienteRequest) {
  const nuevoCliente = await clientesStore.crearCliente(request);
  if (nuevoCliente) {
    seleccionarCliente(nuevoCliente);
    mostrarModalNuevoCliente.value = false;
  }
}

function onCantidadChange(item: CartItem, value: number) {
  item.cantidad = value;
  updateSubtotal(item);
}

function onPrecioChange(item: CartItem, value: number) {
  item.precio = value;
  updateSubtotal(item);
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

function generarPdf() {
  if (cart.value.length === 0) return;
  window.print();
}
</script>

<template>
    <div class="nueva-venta-page">
        <div class="page-header">
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
        </div>

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

        <div
            v-if="nocturnoStore.config.activo"
            class="turno-banner"
            :class="esNocturnoActual ? 'turno-nocturno' : 'turno-diurno'"
        >
            <span v-if="esNocturnoActual">
                Precios nocturnos activos: se aplica la ganancia nocturna de
                cada artículo.
            </span>
            <span v-else>
                Precios diurnos activos: se aplica la ganancia diurna de cada
                artículo.
            </span>
        </div>

        <div class="venta-section header-section">
            <div class="form-group obs-group">
                <label>Observación</label>
                <input
                    v-model="observacion"
                    type="text"
                    placeholder="Opcional"
                />
            </div>
            <div class="form-group obs-group">
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
            <div class="form-group obs-group">
                <label>Vencimiento presupuesto</label>
                <input
                    v-model="fechaVencimiento"
                    type="date"
                    placeholder="Opcional"
                />
            </div>

            <div class="acciones">
                <button
                    v-if="canGenerarPresupuesto()"
                    type="button"
                    @click="generarPdf"
                    class="btn-tertiary"
                    :disabled="cart.length === 0"
                >
                    PDF
                </button>
                <button
                    v-if="canGenerarPresupuesto()"
                    type="button"
                    @click="handleGuardarPresupuesto"
                    class="btn-presupuesto"
                    :disabled="!presupuestoValido"
                >
                    Guardar Presupuesto
                </button>
                <button
                    type="button"
                    @click="cancelar"
                    class="btn-secondary"
                >
                    Cancelar
                </button>
                <button
                    type="button"
                    @click="handleCreate"
                    class="btn-success"
                    :disabled="!carritoValido || ventasStore.diaCerrado"
                >
                    Registrar Venta
                </button>
            </div>
        </div>

        <div class="venta-section totals-section">
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
            <TotalsPanel
                :subtotal="carritoSubtotal"
                :descuento="descuento"
                :descuento-monto="descuentoMonto"
                :total="carritoTotal"
                @update:descuento="descuento = $event"
            />
        </div>

        <div class="venta-section search-section">
            <div class="cart-header">
                <h2>Artículos de la venta</h2>
                <button
                    v-if="cart.length > 0"
                    type="button"
                    @click="vaciarCarrito"
                    class="btn-secondary"
                >
                    Vaciar
                </button>
            </div>
            <ArticuloSearch
                ref="articuloSearchRef"
                :query="searchQuery"
                :results="searchResults"
                @update:query="searchQuery = $event"
                @select="addArticuloById"
                @enter="onSearchEnter"
            />
            <CartTable
                :items="cart"
                :stock-warning="stockWarning"
                @update-cantidad="onCantidadChange"
                @update-precio="onPrecioChange"
                @remove="removeArticulo"
            />
            <div v-if="cart.length === 0" class="empty-state">
                Agregue artículos a la venta
            </div>
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
    padding: 1rem;
    background: var(--color-bg);
    min-height: 100%;
}

.page-header {
    margin-bottom: 1rem;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
}

.header-left h1 {
    margin: 0;
}

.dia-cerrado-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
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
    border-radius: 6px;
    margin-bottom: 1rem;
    font-weight: 600;
}

.presupuesto-banner .btn-secondary {
    padding: 0.35rem 0.75rem;
    font-size: 0.85rem;
}

.turno-banner {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 0.9rem;
    border-radius: 999px;
    margin-bottom: 0.75rem;
    font-size: 0.8rem;
    font-weight: 600;
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

.venta-section {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1rem;
    box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.1);
}

.header-section {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
}

.obs-group {
    flex: 1;
    min-width: 240px;
    margin: 0;
}

.acciones {
    display: flex;
    gap: 0.75rem;
}

.totals-section {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
}

.form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-sizing: border-box;
    background: var(--color-surface);
    color: var(--color-text);
}

.tipo-select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-sizing: border-box;
    background: var(--color-surface);
    color: var(--color-text);
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path d='M1 1l5 5 5-5' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    padding-right: 2.5rem;
}

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn-tertiary {
    background: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-danger);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-tertiary:disabled {
    opacity: 0.6;
    border: none;
    cursor: not-allowed;
}

.btn-presupuesto {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-presupuesto:hover:not(:disabled) {
    background: var(--color-secondary);
}

.btn-presupuesto:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn-volver {
    padding: 0.5rem 1rem;
}

.cart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
}

.cart-header h2 {
    font-size: 1.4rem;
    margin: 0;
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

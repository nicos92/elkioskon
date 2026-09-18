<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useStockStore, useArticulosStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import PageHeader from "../components/ui/PageHeader.vue";
import SearchBar from "../components/ui/SearchBar.vue";
import DataTable from "../components/ui/DataTable.vue";
import EntityFormModal from "../components/ui/EntityFormModal.vue";
import ConfirmButton from "../components/ui/ConfirmButton.vue";
import type {
    Stock,
    CreateStockRequest,
    UpdateStockRequest,
} from "../../domain/entities";
import { calcularPrecioVenta } from "../../domain/entities";

const stockStore = useStockStore();
const articulosStore = useArticulosStore();
const router = useRouter();
const { canCreateStock, canUpdateStock, canDeleteStock } = usePermissions();

const showModal = ref(false);
const modalMode = ref<"create" | "edit">("create");
const selectedStock = ref<Stock | null>(null);

const formIdArticulo = ref<number | null>(null);
const formArticuloNombre = ref("");
const formCantidad = ref(0);
const formCosto = ref(0);
const formGanancia = ref(0);
const formGananciaDiurna = ref(0);
const formGananciaNocturna = ref(0);

const preciosVenta = ref<Map<number, number>>(new Map());

const searchQuery = ref("");

const loading = computed(
    () => stockStore.loading || articulosStore.loading,
);

const filteredStock = computed(() => {
    const query = searchQuery.value.toLowerCase().trim();
    if (!query) return stockCompletos.value;
    return stockCompletos.value.filter(
        (s) =>
            s.codArticulo.toLowerCase().includes(query) ||
            s.articuloNombre.toLowerCase().includes(query),
    );
});

const formPreviewPrecioVenta = computed(() => {
    return calcularPrecioVenta(formCosto.value, formGanancia.value);
});

const stockCompletos = computed(() => {
    return stockStore.stocks.map((s) => {
        const articulo = articulosStore.articulos.find(
            (a) => a.id === s.id_articulo,
        );
        const precioVenta =
            preciosVenta.value.get(s.id) ||
            calcularPrecioVenta(s.costo, s.ganancia);
        return {
            ...s,
            articuloNombre: articulo?.articulo || "Sin artículo",
            codArticulo: articulo?.cod_articulo || "-",
            precioVenta,
        };
    });
});

const articulosDisponibles = computed(() => {
    return articulosStore.articulos.filter((a) => {
        const tieneStock = stockStore.stocks.some(
            (s) => s.id_articulo === a.id,
        );
        return !tieneStock;
    });
});

onMounted(async () => {
    await Promise.all([
        stockStore.fetchStock(),
        articulosStore.fetchArticulos(),
    ]);
    for (const stock of stockStore.stocks) {
        const precio = await stockStore.getPrecioVenta(stock.id);
        if (precio !== null) {
            preciosVenta.value.set(stock.id, precio);
        }
    }
});

function openCreateModal() {
    modalMode.value = "create";
    selectedStock.value = null;
    formIdArticulo.value = null;
    formCantidad.value = 0;
    formCosto.value = 0;
    formGanancia.value = 0;
    formGananciaDiurna.value = 0;
    formGananciaNocturna.value = 0;
    showModal.value = true;
}

function openEditModal(stock: (typeof stockCompletos.value)[0]) {
    modalMode.value = "edit";
    selectedStock.value = stock;
    formIdArticulo.value = stock.id_articulo;
    formArticuloNombre.value = stock.articuloNombre;
    formCantidad.value = stock.cantidad;
    formCosto.value = stock.costo;
    formGanancia.value = stock.ganancia;
    formGananciaDiurna.value = stock.ganancia_diurna;
    formGananciaNocturna.value = stock.ganancia_nocturna;
    showModal.value = true;
}

async function handleCreate() {
    if (!formIdArticulo.value) return;
    const request: CreateStockRequest = {
        id_articulo: formIdArticulo.value,
        cantidad: formCantidad.value,
        costo: formCosto.value,
        ganancia: formGanancia.value,
        ganancia_diurna: formGananciaDiurna.value,
        ganancia_nocturna: formGananciaNocturna.value,
    };
    const success = await stockStore.createStock(request);
    if (success) {
        showModal.value = false;
        await articulosStore.fetchArticulos();
    }
}

async function handleUpdate() {
    if (!selectedStock.value) return;
    const request: UpdateStockRequest = {
        id: selectedStock.value.id,
        cantidad: formCantidad.value,
        costo: formCosto.value,
        ganancia: formGanancia.value,
        ganancia_diurna: formGananciaDiurna.value,
        ganancia_nocturna: formGananciaNocturna.value,
    };
    const success = await stockStore.updateStock(request);
    if (success) {
        preciosVenta.value.set(
            selectedStock.value.id,
            formPreviewPrecioVenta.value,
        );
        showModal.value = false;
    }
}

async function handleDelete(id: number) {
    const success = await stockStore.deleteStock(id);
    if (!success) {
        useToasts().error(
            stockStore.error || "No se pudo eliminar el stock.",
        );
    }
}
</script>

<template>
    <div class="stock-page">
        <PageHeader title="Gestión de Stock">
            <button
                v-if="canUpdateStock()"
                @click="router.push({ name: 'actualizar-costo' })"
                class="btn-secondary"
            >
                Actualizar Precios de Costo
            </button>
            <button
                v-if="canCreateStock()"
                @click="openCreateModal"
                class="btn-primary"
            >
                Crear Stock
            </button>
        </PageHeader>

        <SearchBar
            v-model="searchQuery"
            placeholder="Buscar por código o artículo..."
        />

        <div v-if="stockStore.error" class="error-banner">
            {{ stockStore.error }}
        </div>

        <DataTable
            :columns="['Código', 'Artículo', 'Cantidad', 'Costo', 'Ganancia %', 'Ganancia Diurna %', 'Ganancia Nocturna %', 'Precio Venta', 'Acciones']"
            :loading="loading"
            :count="filteredStock.length"
            empty="No hay stock que coincida con la búsqueda"
        >
            <tr v-for="stock in filteredStock" :key="stock.id">
                <td>{{ stock.codArticulo }}</td>
                <td>{{ stock.articuloNombre }}</td>
                <td>{{ stock.cantidad }}</td>
                <td>${{ stock.costo.toFixed(2) }}</td>
                <td>{{ stock.ganancia }}%</td>
                <td>{{ stock.ganancia_diurna }}%</td>
                <td>{{ stock.ganancia_nocturna }}%</td>
                <td>${{ stock.precioVenta.toFixed(2) }}</td>
                <td class="actions">
                    <button
                        v-if="canUpdateStock()"
                        @click="openEditModal(stock)"
                        class="btn-icon"
                        title="Editar"
                    >
                        <img src="/svg/edit.svg" alt="Editar" />
                    </button>
                    <ConfirmButton
                        v-if="canDeleteStock()"
                        message="¿Está seguro de eliminar este stock?"
                        @confirmed="handleDelete(stock.id)"
                    />
                </td>
            </tr>
        </DataTable>

        <EntityFormModal
            v-model="showModal"
            :title="modalMode === 'create' ? 'Crear Stock' : 'Editar Stock'"
            :error="stockStore.error"
            :submit-label="modalMode === 'create' ? 'Crear' : 'Guardar'"
            :disable-submit="modalMode === 'create' && !formIdArticulo"
            @submit="modalMode === 'create' ? handleCreate() : handleUpdate()"
        >
            <div v-if="modalMode === 'create'" class="form-group">
                <label>Artículo</label>
                <select v-model="formIdArticulo" required>
                    <option :value="null" disabled>
                        Seleccione un artículo
                    </option>
                    <option
                        v-for="art in articulosDisponibles"
                        :key="art.id"
                        :value="art.id"
                    >
                        {{ art.cod_articulo }} - {{ art.articulo }}
                    </option>
                </select>
            </div>
            <div v-if="modalMode === 'edit'" class="form-group">
                <label>Artículo</label>
                <input
                    type="text"
                    :value="formArticuloNombre"
                    disabled
                />

            </div>
            <div class="form-group">
                <label>Cantidad</label>
                <input
                    v-model.number="formCantidad"
                    type="number"
                    step="0.01"
                    required
                />
            </div>
            <div class="form-group">
                <label>Costo</label>
                <input
                    v-model.number="formCosto"
                    type="number"
                    step="0.01"
                    min="0"
                    required
                />
            </div>
            <div class="form-group">
                <label>Ganancia (%)</label>
                <input
                    v-model.number="formGanancia"
                    type="number"
                    step="0.01"
                    min="0"
                    required
                />
            </div>
            <div class="form-group">
                <label>Ganancia diurna (%)</label>
                <input
                    v-model.number="formGananciaDiurna"
                    type="number"
                    step="0.01"
                    min="0"
                />
            </div>
            <div class="form-group">
                <label>Ganancia nocturna (%)</label>
                <input
                    v-model.number="formGananciaNocturna"
                    type="number"
                    step="0.01"
                    min="0"
                />
            </div>
            <template #extra>
                <div class="preview-precio">
                    Precio de Venta: ${{
                        formPreviewPrecioVenta.toFixed(2)
                    }}
                </div>
            </template>
        </EntityFormModal>
    </div>
</template>

<style scoped>
.stock-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;
}

.preview-precio {
    background: var(--color-surface-2);
    padding: 0.75rem;
    border-radius: 6px;
    text-align: center;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 1rem;
}
</style>

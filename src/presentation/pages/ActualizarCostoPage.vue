<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
    useStockStore,
    useCategoriasStore,
    useSubCategoriasStore,
    useProveedoresStore,
} from "../stores";
import { useConfirm } from "../composables/useConfirm";
import { useToasts } from "../composables/useToasts";
import PageHeader from "../components/ui/PageHeader.vue";
import DataTable from "../components/ui/DataTable.vue";
import type { StockPreview } from "../../domain/entities";

const router = useRouter();
const stockStore = useStockStore();
const categoriasStore = useCategoriasStore();
const subCategoriasStore = useSubCategoriasStore();
const proveedoresStore = useProveedoresStore();
const { confirm } = useConfirm();
const toasts = useToasts();

const idCategoria = ref<number | null>(null);
const idSubCategoria = ref<number | null>(null);
const idProveedor = ref<number | null>(null);
const porcentaje = ref<number>(0);

const preview = ref<StockPreview[]>([]);
const loadingPreview = ref(false);
const applying = ref(false);
const applied = ref(false);

const subCategoriasFiltradas = computed(() => {
    if (!idCategoria.value) return subCategoriasStore.subCategorias;
    return subCategoriasStore.subCategorias.filter(
        (s) => s.id_categoria === idCategoria.value,
    );
});

const porcentajeValido = computed(() => {
    return porcentaje.value !== 0 && porcentaje.value > -100 && porcentaje.value <= 100 && isFinite(porcentaje.value);
});

const tienePreview = computed(() => preview.value.length > 0);

const previewResumen = computed(() => {
    if (preview.value.length === 0) return null;
    const totalActual = preview.value.reduce((sum, p) => sum + p.costo_actual, 0);
    const totalNuevo = preview.value.reduce((sum, p) => sum + p.costo_nuevo, 0);
    return {
        cantidad: preview.value.length,
        totalActual,
        totalNuevo,
        diferencia: totalNuevo - totalActual,
    };
});

onMounted(async () => {
    await Promise.all([
        categoriasStore.fetchCategorias(),
        subCategoriasStore.fetchSubCategorias(),
        proveedoresStore.fetchProveedores(),
        stockStore.fetchLastUndoable(),
    ]);
});

function resetPreview() {
    preview.value = [];
    applied.value = false;
}

function handleCategoriaChange() {
    idSubCategoria.value = null;
    resetPreview();
}

function handleSubCategoriaChange() {
    resetPreview();
}

function handleProveedorChange() {
    resetPreview();
}

function handlePorcentajeChange() {
    resetPreview();
}

async function loadPreview() {
    if (!porcentajeValido.value) return;
    loadingPreview.value = true;
    preview.value = [];
    applied.value = false;
    try {
        preview.value = await stockStore.getStockPreviewCosto(
            porcentaje.value,
            idCategoria.value,
            idSubCategoria.value,
            idProveedor.value,
        );
    } finally {
        loadingPreview.value = false;
    }
}

async function handleApply() {
    if (!tienePreview.value || !porcentajeValido.value) return;

    const label = porcentaje.value > 0 ? "aumentar" : "reducir";
    const confirmed = await confirm({
        title: "Confirmar actualización",
        message: `¿Está seguro de ${label} el precio de costo de ${preview.value.length} artículos en un ${porcentaje.value}%?`,
        confirmText: "Aplicar",
        variant: "danger",
    });

    if (!confirmed) return;

    applying.value = true;
    try {
        const result = await stockStore.applyCostoPercentage({
            porcentaje: porcentaje.value,
            id_categoria: idCategoria.value,
            id_sub_categoria: idSubCategoria.value,
            id_proveedor: idProveedor.value,
        });
        if (result) {
            toasts.success(
                `Se actualizaron ${result.updated_count} artículos correctamente.`,
            );
            applied.value = true;
            await stockStore.fetchStock();
        }
    } finally {
        applying.value = false;
    }
}

function goBack() {
    router.push({ name: "stock" });
}

const undoing = ref(false);

async function handleUndo() {
    if (!stockStore.lastOperation) return;

    const confirmed = await confirm({
        title: "Deshacer actualización",
        message: `¿Está seguro de deshacer la actualización del ${stockStore.lastOperation.porcentaje}% (${stockStore.lastOperation.affected_count} artículos)? Se restaurarán los precios de costo anteriores.`,
        confirmText: "Deshacer",
        variant: "danger",
    });

    if (!confirmed) return;

    undoing.value = true;
    try {
        const result = await stockStore.undoCostUpdate();
        if (result) {
            toasts.success(
                `Se restauraron ${result.restored_count} artículos correctamente.`,
            );
            await stockStore.fetchStock();
        }
    } finally {
        undoing.value = false;
    }
}
</script>

<template>
    <div class="actualizar-costo-page">
        <PageHeader title="Actualizar Precios de Costo">
            <button @click="goBack" class="btn-secondary">
                Volver al Stock
            </button>
        </PageHeader>

        <div v-if="stockStore.error" class="error-banner">
            {{ stockStore.error }}
        </div>

        <div class="filters-panel">
            <div class="form-group">
                <label>Categoría</label>
                <select
                    v-model="idCategoria"
                    @change="handleCategoriaChange"
                >
                    <option :value="null">Todas</option>
                    <option
                        v-for="cat in categoriasStore.categorias"
                        :key="cat.id"
                        :value="cat.id"
                    >
                        {{ cat.categoria }}
                    </option>
                </select>
            </div>

            <div class="form-group">
                <label>Subcategoría</label>
                <select
                    v-model="idSubCategoria"
                    @change="handleSubCategoriaChange"
                >
                    <option :value="null">Todas</option>
                    <option
                        v-for="sub in subCategoriasFiltradas"
                        :key="sub.id"
                        :value="sub.id"
                    >
                        {{ sub.sub_categoria }}
                    </option>
                </select>
            </div>

            <div class="form-group">
                <label>Proveedor</label>
                <select
                    v-model="idProveedor"
                    @change="handleProveedorChange"
                >
                    <option :value="null">Todos</option>
                    <option
                        v-for="prov in proveedoresStore.proveedores"
                        :key="prov.id"
                        :value="prov.id"
                    >
                        {{ prov.proveedor }}
                    </option>
                </select>
            </div>

            <div class="form-group porcentaje-group">
                <label>Porcentaje de ajuste</label>
                <div class="porcentaje-input-wrapper">
                    <input
                        v-model.number="porcentaje"
                        type="number"
                        step="0.01"
                        min="-99.99"
                        max="100"
                        placeholder="Ej: 20 o -10"
                        class="porcentaje-input"
                        @input="handlePorcentajeChange"
                    />
                    <span class="porcentaje-symbol">%</span>
                </div>
            </div>
        </div>

        <div class="actions-bar">
            <button
                class="btn-primary"
                :disabled="!porcentajeValido || loadingPreview"
                @click="loadPreview"
            >
                {{ loadingPreview ? "Cargando..." : "Vista Previa" }}
            </button>
            <button
                v-if="tienePreview && !applied"
                class="btn-primary"
                :disabled="applying"
                @click="handleApply"
            >
                {{ applying ? "Aplicando..." : "Aplicar Cambios" }}
            </button>
            <button
                v-if="stockStore.lastOperation"
                class="btn-undo"
                :disabled="undoing"
                @click="handleUndo"
            >
                {{ undoing ? "Deshaciendo..." : "Deshacer última actualización" }}
            </button>
            <span v-if="applied" class="applied-message">
                Cambios aplicados correctamente.
            </span>
        </div>

        <div v-if="tienePreview" class="preview-section">
            <div class="preview-summary">
                <span class="summary-count">
                    {{ previewResumen?.cantidad }} artículos serán
                    {{ porcentaje > 0 ? "aumentados" : porcentaje < 0 ? "reducidos" : "afectados" }}
                </span>
                <span class="summary-detail">
                    Costo actual total: ${{ previewResumen?.totalActual.toFixed(2) }} →
                    Costo nuevo total: ${{ previewResumen?.totalNuevo.toFixed(2) }}
                    (<span :class="previewResumen?.diferencia! >= 0 ? 'text-success' : 'text-danger'">
                        {{ previewResumen?.diferencia! >= 0 ? "+" : "" }}${{ previewResumen?.diferencia.toFixed(2) }}
                    </span>)
                </span>
            </div>

            <DataTable
                :columns="[
                    'Código',
                    'Artículo',
                    'Categoría',
                    'Subcat.',
                    'Proveedor',
                    'Costo Actual',
                    'Costo Nuevo',
                    'Diferencia',
                ]"
                :loading="loadingPreview"
                :count="preview.length"
                empty="No hay artículos para mostrar"
            >
                <tr v-for="item in preview" :key="item.id_stock">
                    <td>{{ item.cod_articulo }}</td>
                    <td>{{ item.articulo }}</td>
                    <td>{{ item.categoria }}</td>
                    <td>{{ item.sub_categoria }}</td>
                    <td>{{ item.proveedor }}</td>
                    <td>${{ item.costo_actual.toFixed(2) }}</td>
                    <td class="costo-nuevo">
                        ${{ item.costo_nuevo.toFixed(2) }}
                    </td>
                    <td
                        :class="
                            item.costo_nuevo >= item.costo_actual
                                ? 'text-success'
                                : 'text-danger'
                        "
                    >
                        {{
                            item.costo_nuevo >= item.costo_actual ? "+" : ""
                        }}${{ (item.costo_nuevo - item.costo_actual).toFixed(2) }}
                    </td>
                </tr>
            </DataTable>
        </div>
    </div>
</template>

<style scoped>
.actualizar-costo-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;
}

.filters-panel {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 1.5rem;
    background: var(--color-surface);
    border-radius: 8px;
    border: 1px solid var(--color-border);
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
}

.form-group label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-secondary);
}

.form-group select,
.form-group input {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.9rem;
}

.porcentaje-group {
    min-width: 180px;
}

.porcentaje-input-wrapper {
    display: flex;
    align-items: center;
    gap: 0.25rem;
}

.porcentaje-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface-2);
    color: var(--color-text);
    font-size: 0.9rem;
}

.porcentaje-symbol {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text-secondary);
}

.actions-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.preview-section {
    margin-top: 1rem;
}

.preview-summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    margin-bottom: 1rem;
    flex-wrap: wrap;
    gap: 0.5rem;
}

.summary-count {
    font-weight: 600;
    font-size: 1rem;
}

.summary-detail {
    font-size: 0.9rem;
    color: var(--color-text-secondary);
}

.costo-nuevo {
    font-weight: 600;
}

.text-success {
    color: var(--color-success);
}

.text-danger {
    color: var(--color-danger);
}

.applied-message {
    color: var(--color-success);
    font-weight: 500;
}

.btn-undo {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-warning);
    border-radius: 6px;
    background: transparent;
    color: var(--color-warning);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.2s, color 0.2s;
}

.btn-undo:hover:not(:disabled) {
    background: var(--color-warning);
    color: var(--color-on-primary);
}

.btn-undo:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}
</style>

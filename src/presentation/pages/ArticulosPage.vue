<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
    useArticulosStore,
    useSubCategoriasStore,
    useProveedoresStore,
    useCategoriasStore,
} from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import PageHeader from "../components/ui/PageHeader.vue";
import SearchBar from "../components/ui/SearchBar.vue";
import DataTable from "../components/ui/DataTable.vue";
import EntityFormModal from "../components/ui/EntityFormModal.vue";
import ConfirmButton from "../components/ui/ConfirmButton.vue";
import type {
    Articulo,
    Categoria,
    Proveedor,
    SubCategoria,
    CreateArticuloRequest,
    UpdateArticuloRequest,
} from "../../domain/entities";
import { isDefaultProveedor } from "../../domain/entities";

const articulosStore = useArticulosStore();
const subCategoriasStore = useSubCategoriasStore();
const proveedoresStore = useProveedoresStore();
const categoriasStore = useCategoriasStore();
const { canCreateArticulo, canUpdateArticulo, canDeleteArticulo } =
    usePermissions();

const showCreateModal = ref(false);
const showEditModal = ref(false);
const selectedArticulo = ref<Articulo | null>(null);

const newArticulo = ref("");
const newCodArticulo = ref("");
const newIdSubCategoria = ref<number | null>(null);
const newIdProveedor = ref<number | null>(null);

const editArticulo = ref("");
const editCodArticulo = ref("");
const editIdSubCategoria = ref<number | null>(null);
const editIdProveedor = ref<number | null>(null);

const searchQuery = ref("");

const loading = computed(
    () =>
        articulosStore.loading ||
        subCategoriasStore.loading ||
        proveedoresStore.loading,
);

const filteredArticulos = computed(() => {
    const query = searchQuery.value.toLowerCase().trim();
    if (!query) return articulosCompletos.value;
    return articulosCompletos.value.filter(
        (art) =>
            art.cod_articulo.toLowerCase().includes(query) ||
            art.articulo.toLowerCase().includes(query) ||
            art.categoriaNombre.toLowerCase().includes(query) ||
            art.subCategoriaNombre.toLowerCase().includes(query) ||
            art.proveedorNombre.toLowerCase().includes(query),
    );
});

const subCategoriaPorId = computed(() => {
    const map = new Map<number, SubCategoria>();
    for (const s of subCategoriasStore.subCategorias) {
        map.set(s.id, s);
    }
    return map;
});

const categoriaPorId = computed(() => {
    const map = new Map<number, Categoria>();
    for (const c of categoriasStore.categorias) {
        map.set(c.id, c);
    }
    return map;
});

const proveedorPorId = computed(() => {
    const map = new Map<number, Proveedor>();
    for (const p of proveedoresStore.proveedores) {
        map.set(p.id, p);
    }
    return map;
});

const articulosCompletos = computed(() => {
    return articulosStore.articulos
        .slice()
        .sort((a, b) => b.cod_articulo.localeCompare(a.cod_articulo))
        .map((a) => {
            const subCat = subCategoriaPorId.value.get(a.id_sub_categoria);
            const cat = subCat
                ? categoriaPorId.value.get(subCat.id_categoria)
                : undefined;
            const prov = proveedorPorId.value.get(a.id_proveedor);
            return {
                ...a,
                subCategoriaNombre:
                    subCat?.sub_categoria || "Sin sub categoría",
                categoriaNombre: cat?.categoria || "Sin categoría",
                proveedorNombre: prov?.proveedor || "Sin proveedor",
            };
        });
});

const subCategoriasConCategoria = computed(() => {
    return subCategoriasStore.subCategorias.map((sc) => ({
        ...sc,
        label: `${categoriaPorId.value.get(sc.id_categoria)?.categoria || ""} > ${sc.sub_categoria}`,
    }));
});

onMounted(async () => {
    await Promise.all([
        articulosStore.fetchArticulos(),
        subCategoriasStore.fetchSubCategorias(),
        categoriasStore.fetchCategorias(),
        proveedoresStore.fetchProveedores(),
    ]);
});

function openCreateModal() {
    newArticulo.value = "";
    newCodArticulo.value = "";
    newIdSubCategoria.value = null;
    const proveedorDefecto = proveedoresStore.proveedores.find(
        isDefaultProveedor,
    );
    newIdProveedor.value = proveedorDefecto?.id ?? null;
    showCreateModal.value = true;
}

function openEditModal(art: (typeof articulosCompletos.value)[0]) {
    selectedArticulo.value = art;
    editArticulo.value = art.articulo;
    editCodArticulo.value = art.cod_articulo;
    editIdSubCategoria.value = art.id_sub_categoria;
    editIdProveedor.value = art.id_proveedor;
    showEditModal.value = true;
}

async function handleCreate() {
    if (!newIdSubCategoria.value || !newIdProveedor.value) return;
    const request: CreateArticuloRequest = {
        articulo: newArticulo.value,
        cod_articulo: newCodArticulo.value,
        id_sub_categoria: newIdSubCategoria.value,
        id_proveedor: newIdProveedor.value,
    };
    const success = await articulosStore.createArticulo(request);
    if (success) {
        showCreateModal.value = false;
    }
}

async function handleUpdate() {
    if (
        !selectedArticulo.value ||
        !editIdSubCategoria.value ||
        !editIdProveedor.value
    )
        return;
    const request: UpdateArticuloRequest = {
        id: selectedArticulo.value.id,
        articulo: editArticulo.value,
        cod_articulo: editCodArticulo.value,
        id_sub_categoria: editIdSubCategoria.value,
        id_proveedor: editIdProveedor.value,
    };
    const success = await articulosStore.updateArticulo(request);
    if (success) {
        showEditModal.value = false;
    }
}

async function handleDelete(id: number) {
    const success = await articulosStore.deleteArticulo(id);
    if (!success) {
        useToasts().error(
            articulosStore.error || "No se pudo eliminar el artículo.",
        );
    }
}
</script>

<template>
    <div class="articulos-page">
        <PageHeader title="Gestión de Artículos">
            <button
                v-if="canCreateArticulo()"
                @click="openCreateModal"
                class="btn-primary"
            >
                Crear Artículo
            </button>
        </PageHeader>

        <SearchBar
            v-model="searchQuery"
            placeholder="Buscar por código, artículo, categoría, subcategoría o proveedor..."
        />

        <div v-if="articulosStore.error" class="error-banner">
            {{ articulosStore.error }}
        </div>

        <DataTable
            :columns="['Código', 'Artículo', 'Categoría', 'Sub Categoría', 'Proveedor', 'Acciones']"
            :loading="loading"
            :count="filteredArticulos.length"
            empty="No hay artículos que coincidan con la búsqueda"
        >
            <tr v-for="art in filteredArticulos" :key="art.id">
                <td>{{ art.cod_articulo }}</td>
                <td>{{ art.articulo }}</td>
                <td>{{ art.categoriaNombre }}</td>
                <td>{{ art.subCategoriaNombre }}</td>
                <td>{{ art.proveedorNombre }}</td>
                <td class="actions">
                    <button
                        v-if="canUpdateArticulo()"
                        @click="openEditModal(art)"
                        class="btn-icon"
                        title="Editar"
                    >
                        <img src="/svg/edit.svg" alt="Editar" />
                    </button>
                    <ConfirmButton
                        v-if="canDeleteArticulo()"
                        message="¿Está seguro de eliminar este artículo?"
                        @confirmed="handleDelete(art.id)"
                    />
                </td>
            </tr>
        </DataTable>

        <EntityFormModal
            v-model="showCreateModal"
            title="Crear Artículo"
            :error="articulosStore.error"
            submit-label="Crear"
            :disable-submit="!newIdSubCategoria || !newIdProveedor"
            @submit="handleCreate"
        >
            <div class="form-group">
                <label>Código</label>
                <input v-model="newCodArticulo" type="text" required />
            </div>
            <div class="form-group">
                <label>Artículo</label>
                <input v-model="newArticulo" type="text" required />
            </div>
            <div class="form-group">
                <label>Sub Categoría</label>
                <select v-model="newIdSubCategoria" required>
                    <option :value="null" disabled>
                        Seleccione una sub categoría
                    </option>
                    <option
                        v-for="sc in subCategoriasConCategoria"
                        :key="sc.id"
                        :value="sc.id"
                    >
                        {{ sc.label }}
                    </option>
                </select>
            </div>
            <div class="form-group">
                <label>Proveedor</label>
                <select v-model="newIdProveedor" required>
                    <option :value="null" disabled>
                        Seleccione un proveedor
                    </option>
                    <option
                        v-for="prov in proveedoresStore.proveedores"
                        :key="prov.id"
                        :value="prov.id"
                    >
                        {{ prov.proveedor }}
                    </option>
                </select>
            </div>
        </EntityFormModal>

        <EntityFormModal
            v-model="showEditModal"
            title="Editar Artículo"
            :error="articulosStore.error"
            submit-label="Guardar"
            @submit="handleUpdate"
        >
            <div class="form-group">
                <label>Código</label>
                <input v-model="editCodArticulo" type="text" required />
            </div>
            <div class="form-group">
                <label>Artículo</label>
                <input v-model="editArticulo" type="text" required />
            </div>
            <div class="form-group">
                <label>Sub Categoría</label>
                <select v-model="editIdSubCategoria" required>
                    <option
                        v-for="sc in subCategoriasConCategoria"
                        :key="sc.id"
                        :value="sc.id"
                    >
                        {{ sc.label }}
                    </option>
                </select>
            </div>
            <div class="form-group">
                <label>Proveedor</label>
                <select v-model="editIdProveedor" required>
                    <option
                        v-for="prov in proveedoresStore.proveedores"
                        :key="prov.id"
                        :value="prov.id"
                    >
                        {{ prov.proveedor }}
                    </option>
                </select>
            </div>
        </EntityFormModal>
    </div>
</template>

<style scoped>
.articulos-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;
}
</style>

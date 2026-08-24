<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useProveedoresStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import PageHeader from "../components/ui/PageHeader.vue";
import DataTable from "../components/ui/DataTable.vue";
import EntityFormModal from "../components/ui/EntityFormModal.vue";
import ConfirmButton from "../components/ui/ConfirmButton.vue";
import type {
    Proveedor,
    CreateProveedorRequest,
    UpdateProveedorRequest,
} from "../../domain/entities";
import { isDefaultProveedor, DEFAULT_PROVEEDOR } from "../../domain/entities";

const proveedoresStore = useProveedoresStore();
const { canCreateProveedor, canUpdateProveedor, canDeleteProveedor } =
    usePermissions();

const showCreateModal = ref(false);
const showEditModal = ref(false);
const selectedProveedor = ref<Proveedor | null>(null);

const newProveedor = ref("");
const newNombre = ref("");
const newCuit = ref("");
const newTel = ref("");
const newEmail = ref("");
const newObservacion = ref("");

const editProveedor = ref("");
const editNombre = ref("");
const editCuit = ref("");
const editTel = ref("");
const editEmail = ref("");
const editObservacion = ref("");

onMounted(async () => {
    await proveedoresStore.fetchProveedores();
});

function openCreateModal() {
    newProveedor.value = "";
    newNombre.value = "";
    newCuit.value = "";
    newTel.value = "";
    newEmail.value = "";
    newObservacion.value = "";
    showCreateModal.value = true;
}

function openEditModal(proveedor: Proveedor) {
    selectedProveedor.value = proveedor;
    editProveedor.value = proveedor.proveedor;
    editNombre.value = proveedor.nombre;
    editCuit.value = proveedor.cuit || "";
    editTel.value = proveedor.tel || "";
    editEmail.value = proveedor.email || "";
    editObservacion.value = proveedor.observacion || "";
    showEditModal.value = true;
}

async function handleCreate() {
    const request: CreateProveedorRequest = {
        proveedor: newProveedor.value,
        nombre: newNombre.value,
        cuit: newCuit.value.toString() || undefined,
        tel: newTel.value.toString() || undefined,
        email: newEmail.value || undefined,
        observacion: newObservacion.value || undefined,
    };
    const success = await proveedoresStore.createProveedor(request);
    if (success) {
        showCreateModal.value = false;
    }
}

async function handleUpdate() {
    if (!selectedProveedor.value) return;
    if (isDefaultProveedor(selectedProveedor.value)) {
        useToasts().error(
            `No se puede modificar el proveedor '${DEFAULT_PROVEEDOR}'.`,
        );
        return;
    }
    const request: UpdateProveedorRequest = {
        id: selectedProveedor.value.id,
        proveedor: editProveedor.value,
        nombre: editNombre.value,
        cuit: editCuit.value.toString() || undefined,
        tel: editTel.value.toString() || undefined,
        email: editEmail.value || undefined,
        observacion: editObservacion.value || undefined,
    };
    const success = await proveedoresStore.updateProveedor(request);
    if (success) {
        showEditModal.value = false;
    }
}

async function handleDelete(id: number) {
    const proveedor = proveedoresStore.proveedores.find((p) => p.id === id);
    if (proveedor && isDefaultProveedor(proveedor)) {
        useToasts().error(
            `No se puede eliminar el proveedor '${DEFAULT_PROVEEDOR}'.`,
        );
        return;
    }
    const success = await proveedoresStore.deleteProveedor(id);
    if (!success) {
        useToasts().error(
            proveedoresStore.error || "No se pudo eliminar el proveedor.",
        );
    }
}
</script>

<template>
    <div class="proveedores-page">
        <PageHeader title="Gestión de Proveedores">
            <button
                v-if="canCreateProveedor()"
                @click="openCreateModal"
                class="btn-primary"
            >
                Crear Proveedor
            </button>
        </PageHeader>

        <div v-if="proveedoresStore.error" class="error-banner">
            {{ proveedoresStore.error }}
        </div>

        <DataTable
            :columns="['Razón Social', 'Nombre', 'CUIT', 'Teléfono', 'Email', 'Acciones']"
            :loading="proveedoresStore.loading"
            :count="proveedoresStore.proveedores.length"
            empty="No hay proveedores registrados"
        >
            <tr
                v-for="proveedor in proveedoresStore.proveedores"
                :key="proveedor.id"
            >
                <td>
                    {{ proveedor.proveedor }}
                    <span
                        v-if="isDefaultProveedor(proveedor)"
                        class="default-badge"
                    >
                        por defecto
                    </span>
                </td>
                <td>{{ proveedor.nombre }}</td>
                <td>{{ proveedor.cuit || "-" }}</td>
                <td>{{ proveedor.tel || "-" }}</td>
                <td>{{ proveedor.email || "-" }}</td>
                <td class="actions">
                    <button
                        v-if="
                            canUpdateProveedor() &&
                            !isDefaultProveedor(proveedor)
                        "
                        @click="openEditModal(proveedor)"
                        class="btn-icon"
                        title="Editar"
                    >
                        <img src="/svg/edit.svg" alt="Editar" />
                    </button>
                    <ConfirmButton
                        v-if="
                            canDeleteProveedor() &&
                            !isDefaultProveedor(proveedor)
                        "
                        message="¿Está seguro de eliminar este proveedor?"
                        @confirmed="handleDelete(proveedor.id)"
                    />
                </td>
            </tr>
        </DataTable>

        <EntityFormModal
            v-model="showCreateModal"
            title="Crear Proveedor"
            :error="proveedoresStore.error"
            submit-label="Crear"
            @submit="handleCreate"
        >
            <div class="form-group">
                <label>Razón Social</label>
                <input v-model="newProveedor" type="text" required />
            </div>
            <div class="form-group">
                <label>Nombre</label>
                <input v-model="newNombre" type="text" required />
            </div>
            <div class="form-group">
                <label>CUIT</label>
                <input v-model="newCuit" type="number" maxlength="11" />
            </div>
            <div class="form-group">
                <label>Teléfono</label>
                <input v-model="newTel" type="number" />
            </div>
            <div class="form-group">
                <label>Email</label>
                <input v-model="newEmail" type="email" />
            </div>
            <div class="form-group">
                <label>Observación</label>
                <textarea v-model="newObservacion" rows="3"></textarea>
            </div>
        </EntityFormModal>

        <EntityFormModal
            v-model="showEditModal"
            title="Editar Proveedor"
            :error="proveedoresStore.error"
            submit-label="Guardar"
            @submit="handleUpdate"
        >
            <div class="form-group">
                <label>Razón Social</label>
                <input v-model="editProveedor" type="text" required />
            </div>
            <div class="form-group">
                <label>Nombre</label>
                <input v-model="editNombre" type="text" required />
            </div>
            <div class="form-group">
                <label>CUIT</label>
                <input v-model="editCuit" type="number" maxlength="11" />
            </div>
            <div class="form-group">
                <label>Teléfono</label>
                <input v-model="editTel" type="number" />
            </div>
            <div class="form-group">
                <label>Email</label>
                <input v-model="editEmail" type="email" />
            </div>
            <div class="form-group">
                <label>Observación</label>
                <textarea v-model="editObservacion" rows="3"></textarea>
            </div>
        </EntityFormModal>
    </div>
</template>

<style scoped>
.proveedores-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;
}

.default-badge {
    display: inline-block;
    margin-left: 0.5rem;
    padding: 0.15rem 0.5rem;
    font-size: 0.7rem;
    border-radius: 999px;
    background: rgba(102, 126, 234, 0.15);
    color: #9A7EDD;
}
</style>

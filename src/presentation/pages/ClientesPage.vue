<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useClientesStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import { clienteLabel } from "../utils/cliente";
import PageHeader from "../components/ui/PageHeader.vue";
import DataTable from "../components/ui/DataTable.vue";
import EntityFormModal from "../components/ui/EntityFormModal.vue";
import ConfirmButton from "../components/ui/ConfirmButton.vue";
import type {
    Cliente,
    CreateClienteRequest,
    UpdateClienteRequest,
} from "../../domain/entities";
import {
    isDefaultClient,
    DEFAULT_CLIENT_LABEL,
} from "../../domain/entities";

const clientesStore = useClientesStore();
const { canCreateCliente, canUpdateCliente, canDeleteCliente } =
    usePermissions();

const showCreateModal = ref(false);
const showEditModal = ref(false);
const selectedCliente = ref<Cliente | null>(null);

const newNombre = ref("");
const newApellido = ref("");
const newTelefono = ref("");
const newEmail = ref("");
const newDireccion = ref("");

const editNombre = ref("");
const editApellido = ref("");
const editTelefono = ref("");
const editEmail = ref("");
const editDireccion = ref("");

const newFormInvalid = computed(
    () =>
        !newNombre.value.trim() &&
        !newApellido.value.trim() &&
        !newTelefono.value.trim() &&
        !newEmail.value.trim() &&
        !newDireccion.value.trim(),
);

const editFormInvalid = computed(
    () =>
        !editNombre.value.trim() &&
        !editApellido.value.trim() &&
        !editTelefono.value.trim() &&
        !editEmail.value.trim() &&
        !editDireccion.value.trim(),
);

onMounted(async () => {
    await clientesStore.fetchClientes();
});

function openCreateModal() {
    newNombre.value = "";
    newApellido.value = "";
    newTelefono.value = "";
    newEmail.value = "";
    newDireccion.value = "";
    showCreateModal.value = true;
}

function openEditModal(cliente: Cliente) {
    selectedCliente.value = cliente;
    editNombre.value = cliente.nombre || "";
    editApellido.value = cliente.apellido || "";
    editTelefono.value = cliente.telefono || "";
    editEmail.value = cliente.email || "";
    editDireccion.value = cliente.direccion || "";
    showEditModal.value = true;
}

async function handleCreate() {
    const request: CreateClienteRequest = {
        nombre: newNombre.value.trim() || undefined,
        apellido: newApellido.value.trim() || undefined,
        telefono: newTelefono.value.trim() || undefined,
        email: newEmail.value.trim() || undefined,
        direccion: newDireccion.value.trim() || undefined,
    };
    const success = await clientesStore.crearCliente(request);
    if (success) {
        showCreateModal.value = false;
    }
}

async function handleUpdate() {
    if (!selectedCliente.value) return;
    const request: UpdateClienteRequest = {
        id: selectedCliente.value.id,
        nombre: editNombre.value.trim() || undefined,
        apellido: editApellido.value.trim() || undefined,
        telefono: editTelefono.value.trim() || undefined,
        email: editEmail.value.trim() || undefined,
        direccion: editDireccion.value.trim() || undefined,
    };
    const success = await clientesStore.actualizarCliente(request);
    if (success) {
        showEditModal.value = false;
    }
}

async function handleDelete(cliente: Cliente) {
    if (isDefaultClient(cliente)) {
        useToasts().error(
            `No se puede eliminar el cliente '${DEFAULT_CLIENT_LABEL}'.`,
        );
        return;
    }
    const success = await clientesStore.eliminarCliente(cliente.id);
    if (!success) {
        useToasts().error(
            clientesStore.error || "No se pudo eliminar el cliente.",
        );
    }
}
</script>

<template>
    <div class="clientes-page">
        <PageHeader title="Gestión de Clientes">
            <button
                v-if="canCreateCliente()"
                @click="openCreateModal"
                class="btn-primary"
            >
                Crear Cliente
            </button>
        </PageHeader>

        <div v-if="clientesStore.error" class="error-banner">
            {{ clientesStore.error }}
        </div>

        <DataTable
            :columns="['Cliente', 'Teléfono', 'Email', 'Dirección', 'Acciones']"
            :loading="clientesStore.loading"
            :count="clientesStore.clientes.length"
            empty="No hay clientes registrados"
        >
            <tr v-for="cliente in clientesStore.clientes" :key="cliente.id">
                <td>
                    {{ clienteLabel(cliente) }}
                    <span
                        v-if="isDefaultClient(cliente)"
                        class="default-badge"
                    >
                        por defecto
                    </span>
                </td>
                <td>{{ cliente.telefono || "-" }}</td>
                <td>{{ cliente.email || "-" }}</td>
                <td>{{ cliente.direccion || "-" }}</td>
                <td class="actions">
                    <button
                        v-if="canUpdateCliente()"
                        @click="openEditModal(cliente)"
                        class="btn-icon"
                        title="Editar"
                    >
                        <img src="/svg/edit.svg" alt="Editar" />
                    </button>
                    <ConfirmButton
                        v-if="canDeleteCliente() && !isDefaultClient(cliente)"
                        message="¿Está seguro de eliminar este cliente?"
                        @confirmed="handleDelete(cliente)"
                    />
                </td>
            </tr>
        </DataTable>

        <EntityFormModal
            v-model="showCreateModal"
            title="Crear Cliente"
            :error="clientesStore.error"
            submit-label="Crear"
            :disable-submit="newFormInvalid"
            @submit="handleCreate"
        >
            <div class="form-group">
                <label>Nombre</label>
                <input v-model="newNombre" type="text" />
            </div>
            <div class="form-group">
                <label>Apellido</label>
                <input v-model="newApellido" type="text" />
            </div>
            <div class="form-group">
                <label>Teléfono</label>
                <input v-model="newTelefono" type="text" />
            </div>
            <div class="form-group">
                <label>Email</label>
                <input v-model="newEmail" type="email" />
            </div>
            <div class="form-group">
                <label>Dirección</label>
                <input v-model="newDireccion" type="text" />
            </div>
            <template #extra>
                <div v-if="newFormInvalid" class="validation-hint">
                    El cliente debe contar al menos con un dato de contacto o
                    identificación.
                </div>
            </template>
        </EntityFormModal>

        <EntityFormModal
            v-model="showEditModal"
            title="Editar Cliente"
            :error="clientesStore.error"
            submit-label="Guardar"
            :disable-submit="editFormInvalid"
            @submit="handleUpdate"
        >
            <div class="form-group">
                <label>Nombre</label>
                <input v-model="editNombre" type="text" />
            </div>
            <div class="form-group">
                <label>Apellido</label>
                <input v-model="editApellido" type="text" />
            </div>
            <div class="form-group">
                <label>Teléfono</label>
                <input v-model="editTelefono" type="text" />
            </div>
            <div class="form-group">
                <label>Email</label>
                <input v-model="editEmail" type="email" />
            </div>
            <div class="form-group">
                <label>Dirección</label>
                <input v-model="editDireccion" type="text" />
            </div>
            <template #extra>
                <div v-if="editFormInvalid" class="validation-hint">
                    El cliente debe contar al menos con un dato de contacto o
                    identificación.
                </div>
            </template>
        </EntityFormModal>
    </div>
</template>

<style scoped>
.clientes-page {
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
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
}

.validation-hint {
    color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) 30%, transparent);
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    font-size: 0.85rem;
}
</style>

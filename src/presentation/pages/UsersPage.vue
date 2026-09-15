<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useUsersStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import { useConfirm } from "../composables/useConfirm";
import type { User } from "../../domain/entities";

const router = useRouter();
const usersStore = useUsersStore();
const { error: toastError, success: toastSuccess } = useToasts();
const {
    canCreateUser,
    canUpdateUser,
    canDeleteUser,
    canChangeUserPassword,
    canAssignPermission,
    canRemovePermission,
} = usePermissions();
const { confirm } = useConfirm();

const showCreateModal = ref(false);
const showEditModal = ref(false);
const showPasswordModal = ref(false);
const selectedUser = ref<User | null>(null);

const newUsername = ref("");
const newPassword = ref("");
const editUsername = ref("");
const editActive = ref(true);

const passwordTargetId = ref(0);
const passwordNew = ref("");
const passwordConfirm = ref("");
const passwordError = ref<string | null>(null);
const isSavingPassword = ref(false);

onMounted(async () => {
    await usersStore.fetchUsers();
});

async function handleCreate() {
    const success = await usersStore.createUser(
        newUsername.value,
        newPassword.value,
    );
    if (success) {
        showCreateModal.value = false;
        newUsername.value = "";
        newPassword.value = "";
    }
}

function openEditModal(user: User) {
    selectedUser.value = user;
    editUsername.value = user.username;
    editActive.value = user.active;
    showEditModal.value = true;
}

function openPasswordModal(user: User) {
    passwordTargetId.value = user.id;
    passwordNew.value = "";
    passwordConfirm.value = "";
    passwordError.value = null;
    showPasswordModal.value = true;
}

async function handleChangePassword() {
    passwordError.value = null;
    if (!passwordNew.value) {
        passwordError.value = "Ingrese la nueva contraseña.";
        return;
    }
    if (passwordNew.value !== passwordConfirm.value) {
        passwordError.value = "Las contraseñas no coinciden.";
        return;
    }

    isSavingPassword.value = true;
    const success = await usersStore.changePassword(
        passwordTargetId.value,
        passwordNew.value,
    );
    isSavingPassword.value = false;

    if (success) {
        showPasswordModal.value = false;
        toastSuccess("Contraseña actualizada correctamente.");
    } else {
        passwordError.value =
            usersStore.error || "No se pudo cambiar la contraseña.";
    }
}

async function handleUpdate() {
    if (!selectedUser.value) return;
    const success = await usersStore.updateUser(
        selectedUser.value.id,
        editUsername.value,
        editActive.value,
    );
    if (success) {
        showEditModal.value = false;
    }
}

async function handleDelete(id: number) {
    if (await confirm({ message: "¿Está seguro de eliminar este usuario?" })) {
        const success = await usersStore.deleteUser(id);
        if (!success) {
            toastError(usersStore.error || "No se pudo eliminar el usuario.");
        }
    }
}

function openPermissionsModal(user: User) {
    router.push({ name: "user-permissions", params: { id: user.id } });
}
</script>

<template>
    <div class="users-page">
        <div class="page-header">
            <h1>Gestión de Usuarios</h1>
            <button
                v-if="canCreateUser()"
                @click="showCreateModal = true"
                class="btn-primary"
            >
                Crear Usuario
            </button>
        </div>

        <div v-if="usersStore.loading" class="loading">Cargando...</div>

        <div v-if="usersStore.error" class="error-banner">
            {{ usersStore.error }}
        </div>

        <div class="table-wrapper">
        <table v-if="!usersStore.loading" class="data-table">
            <thead>
                <tr>
                    <th>Usuario</th>
                    <th>Activo</th>
                    <th>Fecha Creación</th>
                    <th>Acciones</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="user in usersStore.users" :key="user.id">
                    <td>{{ user.username }}</td>
                    <td>
                        <span
                            :class="
                                user.active
                                    ? 'status-active'
                                    : 'status-inactive'
                            "
                        >
                            {{ user.active ? "Sí" : "No" }}
                        </span>
                    </td>
                    <td>
                        {{ new Date(user.created_at).toLocaleDateString() }}
                    </td>
                    <td class="actions">
                        <button
                            v-if="
                                canAssignPermission() || canRemovePermission()
                            "
                            @click="openPermissionsModal(user)"
                            class="btn-icon"
                            title="Permisos"
                        >
                            <img src="/svg/permissions.svg" alt="Permisos" />
                        </button>
                        <button
                            v-if="canChangeUserPassword()"
                            @click="openPasswordModal(user)"
                            class="btn-icon"
                            title="Cambiar contraseña"
                        >
                            <img
                                src="/svg/lock.svg"
                                alt="Cambiar contraseña"
                            />
                        </button>
                        <button
                            v-if="canUpdateUser()"
                            @click="openEditModal(user)"
                            class="btn-icon"
                            title="Editar"
                        >
                            <img src="/svg/edit.svg" alt="Editar" />
                        </button>
                        <button
                            v-if="canDeleteUser()"
                            @click="handleDelete(user.id)"
                            class="btn-icon btn-danger"
                            title="Eliminar"
                        >
                            <img src="/svg/trash.svg" alt="Eliminar" />
                        </button>
                    </td>
                </tr>
            </tbody>
        </table>
        </div>

        <div
            v-if="!usersStore.loading && usersStore.users.length === 0"
            class="empty-state"
        >
            No hay usuarios registrados
        </div>

        <div
            v-if="showCreateModal"
            class="modal-overlay"
            @click.self="showCreateModal = false"
        >
            <div class="modal">
                <h2>Crear Usuario</h2>
                <form @submit.prevent="handleCreate">
                    <div class="form-group">
                        <label>Usuario</label>
                        <input v-model="newUsername" type="text" required />
                    </div>
                    <div class="form-group">
                        <label>Contraseña</label>
                        <input v-model="newPassword" type="password" required />
                    </div>
                    <div v-if="usersStore.error" class="error-message">
                        {{ usersStore.error }}
                    </div>
                    <div class="modal-actions">
                        <button
                            type="button"
                            @click="showCreateModal = false"
                            class="btn-secondary"
                        >
                            Cancelar
                        </button>
                        <button type="submit" class="btn-primary">Crear</button>
                    </div>
                </form>
            </div>
        </div>

        <div
            v-if="showEditModal"
            class="modal-overlay"
            @click.self="showEditModal = false"
        >
            <div class="modal">
                <h2>Editar Usuario</h2>
                <form @submit.prevent="handleUpdate">
                    <div class="form-group">
                        <label>Usuario</label>
                        <input v-model="editUsername" type="text" required />
                    </div>
                    <div class="form-group">
                        <label>
                            <input v-model="editActive" type="checkbox" />
                            Usuario Activo
                        </label>
                    </div>
                    <div v-if="usersStore.error" class="error-message">
                        {{ usersStore.error }}
                    </div>
                    <div class="modal-actions">
                        <button
                            type="button"
                            @click="showEditModal = false"
                            class="btn-secondary"
                        >
                            Cancelar
                        </button>
                        <button type="submit" class="btn-primary">
                            Guardar
                        </button>
                    </div>
                </form>
            </div>
        </div>

        <div
            v-if="showPasswordModal"
            class="modal-overlay"
            @click.self="showPasswordModal = false"
        >
            <div class="modal">
                <h2>Cambiar contraseña</h2>
                <form @submit.prevent="handleChangePassword">
                    <div class="form-group">
                        <label>Nueva contraseña</label>
                        <input
                            v-model="passwordNew"
                            type="password"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label>Repetir nueva contraseña</label>
                        <input
                            v-model="passwordConfirm"
                            type="password"
                            required
                        />
                    </div>
                    <div v-if="passwordError" class="error-message">
                        {{ passwordError }}
                    </div>
                    <div class="modal-actions">
                        <button
                            type="button"
                            @click="showPasswordModal = false"
                            class="btn-secondary"
                        >
                            Cancelar
                        </button>
                        <button
                            type="submit"
                            class="btn-primary"
                            :disabled="isSavingPassword"
                        >
                            {{ isSavingPassword ? "Guardando..." : "Guardar" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>

<style scoped>
.users-page {
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

.btn-primary {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-primary:hover {
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
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.status-active {
    color: var(--color-success);
    font-weight: 500;
}

.status-inactive {
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
    max-width: 400px;
}

.modal h2 {
    margin: 0 0 1.5rem;
}

.form-group {
    margin-bottom: 1rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
}

.form-group input[type="text"],
.form-group input[type="password"] {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
}

.modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
}

.error-message {
    color: var(--color-danger);
    margin-bottom: 1rem;
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

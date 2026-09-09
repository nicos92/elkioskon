<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useUsersStore, usePermissionsStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";
import { useToasts } from "../composables/useToasts";
import type { User } from "../../domain/entities";

const route = useRoute();
const router = useRouter();
const usersStore = useUsersStore();
const permissionsStore = usePermissionsStore();
const { canAssignPermission, canRemovePermission } = usePermissions();
const { error: toastError, success: toastSuccess } = useToasts();

const userId = computed(() => Number(route.params.id));
const selectedUser = ref<User | null>(null);

const assignedPermissions = computed(() => {
    return permissionsStore.getUserPermissions(userId.value);
});

const availablePermissions = computed(() => {
    const assignedIds = new Set(
        assignedPermissions.value.map((p) => p.id),
    );
    return permissionsStore.allPermissions.filter(
        (p) => !assignedIds.has(p.id),
    );
});

onMounted(async () => {
    if (usersStore.users.length === 0) {
        await usersStore.fetchUsers();
    }
    selectedUser.value =
        usersStore.users.find((u) => u.id === userId.value) || null;
    await permissionsStore.fetchAllPermissions();
    await permissionsStore.fetchUserPermissions(userId.value);
});

async function addPermission(permissionId: number) {
    const success = await permissionsStore.addPermission(
        userId.value,
        permissionId,
    );
    if (success) {
        toastSuccess("Permiso asignado correctamente.");
    } else {
        toastError(
            permissionsStore.error || "No se pudo asignar el permiso.",
        );
    }
}

async function removePermission(permissionId: number) {
    const success = await permissionsStore.removePermission(
        userId.value,
        permissionId,
    );
    if (success) {
        toastSuccess("Permiso removido correctamente.");
    } else {
        toastError(
            permissionsStore.error || "No se pudo quitar el permiso.",
        );
    }
}

function goBack() {
    router.push({ name: "users" });
}
</script>

<template>
    <div class="permissions-page">
        <div class="page-header">
            <h1>Permisos de {{ selectedUser?.username || "..." }}</h1>
            <button @click="goBack" class="btn-secondary">
                Volver a Usuarios
            </button>
        </div>

        <div v-if="permissionsStore.loading" class="loading">
            Cargando permisos...
        </div>

        <div v-if="permissionsStore.error" class="error-banner">
            {{ permissionsStore.error }}
        </div>

        <div v-if="!permissionsStore.loading" class="permissions-grid">
            <div class="permission-section">
                <h3>
                    Permisos Asignados
                    <span class="badge">{{
                        assignedPermissions.length
                    }}</span>
                </h3>
                <ul class="permission-list">
                    <li
                        v-for="perm in assignedPermissions"
                        :key="perm.id"
                    >
                        <div class="perm-info">
                            <span class="perm-name">{{
                                perm.permission
                            }}</span>
                            <span class="perm-date"
                                >Asignado:
                                {{
                                    new Date(
                                        perm.assigned_at,
                                    ).toLocaleString()
                                }}</span
                            >
                        </div>
                        <button
                            v-if="canRemovePermission()"
                            @click="removePermission(perm.id)"
                            class="btn-remove"
                            title="Quitar permiso"
                        >
                            &times;
                        </button>
                    </li>
                    <li
                        v-if="assignedPermissions.length === 0"
                        class="empty"
                    >
                        Sin permisos asignados
                    </li>
                </ul>
            </div>

            <div class="permission-section">
                <h3>
                    Permisos Disponibles
                    <span class="badge">{{
                        availablePermissions.length
                    }}</span>
                </h3>
                <ul class="permission-list">
                    <li
                        v-for="perm in availablePermissions"
                        :key="perm.id"
                    >
                        <span class="perm-name">{{
                            perm.permission
                        }}</span>
                        <button
                            v-if="canAssignPermission()"
                            @click="addPermission(perm.id)"
                            class="btn-add"
                            title="Asignar permiso"
                        >
                            +
                        </button>
                    </li>
                    <li
                        v-if="availablePermissions.length === 0"
                        class="empty"
                    >
                        Todos los permisos asignados
                    </li>
                </ul>
            </div>
        </div>
    </div>
</template>

<style scoped>
.permissions-page {
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

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-secondary:hover {
    opacity: 0.8;
}

.permissions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
}

.permission-section {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.permission-section h3 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.badge {
    background: var(--color-primary);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
}

.permission-list {
    list-style: none;
    padding: 0;
    margin: 0;
    max-height: calc(100vh - 240px);
    overflow-y: auto;
}

.permission-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0.5rem;
    border-bottom: 1px solid var(--color-border);
}

.permission-list li:last-child {
    border-bottom: none;
}

.permission-list li.empty {
    color: var(--color-text-muted);
    font-style: italic;
    justify-content: center;
}

.perm-info {
    display: flex;
    flex-direction: column;
}

.perm-name {
    font-weight: 500;
}

.perm-date {
    font-size: 0.75rem;
    color: var(--color-text-muted);
}

.btn-add {
    background: #48bb78;
    color: white;
    border: none;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1rem;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
}

.btn-add:hover {
    background: #38a169;
}

.btn-remove {
    background: var(--color-danger);
    color: white;
    border: none;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1.1rem;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
}

.btn-remove:hover {
    opacity: 0.8;
}

.loading {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-muted);
}

.error-banner {
    color: var(--color-danger);
    background: rgba(var(--color-danger-rgb), 0.1);
    border: 1px solid rgba(var(--color-danger-rgb), 0.3);
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
}
</style>

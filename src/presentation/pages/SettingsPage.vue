<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { getVersion } from "@tauri-apps/api/app";

import { useAuthStore } from "../stores";
import { useThemeStore } from "../stores/themeStore";
import { useToasts } from "../composables/useToasts";

const router = useRouter();

const authStore = useAuthStore();
const themeStore = useThemeStore();
const { success: toastSuccess } = useToasts();

const appVersion = ref("");

onMounted(async () => {
    try {
        appVersion.value = await getVersion();
    } catch {
        appVersion.value = "0.3.0";
    }
});

const showPasswordModal = ref(false);
const currentPassword = ref("");
const newPassword = ref("");
const confirmPassword = ref("");
const isSaving = ref(false);
const formError = ref<string | null>(null);

const theme = computed({
    get: () => themeStore.mode,
    set: (value) => themeStore.setMode(value),
});

function openPasswordModal() {
    formError.value = null;
    currentPassword.value = "";
    newPassword.value = "";
    confirmPassword.value = "";
    showPasswordModal.value = true;
}

async function handleChangePassword() {
    formError.value = null;
    if (!currentPassword.value || !newPassword.value) {
        formError.value = "Complete todos los campos.";
        return;
    }
    if (newPassword.value !== confirmPassword.value) {
        formError.value = "Las contraseñas no coinciden.";
        return;
    }

    isSaving.value = true;
    const success = await authStore.changeOwnPassword(
        currentPassword.value,
        newPassword.value,
    );
    isSaving.value = false;

    if (!success) {
        formError.value =
            authStore.error || "No se pudo cambiar la contraseña.";
        return;
    }

    showPasswordModal.value = false;
    toastSuccess("Contraseña cambiada correctamente.");
    authStore.logout();
    router.push({ name: "login", query: { passwordChanged: "1" } });
}

function handleLogout() {
    authStore.logout();
    router.push({ name: "login" });
}
</script>

<template>
    <div class="settings-page">
        <h1>Configuración</h1>

        <div class="settings-section">
            <h3>Cuenta</h3>
            <div class="setting-item">
                <span class="setting-label">Usuario:</span>
                <span class="setting-value">{{
                    authStore.user?.username
                }}</span>
            </div>
            <div class="setting-item">
                <span class="setting-label">Estado:</span>
                <span class="setting-value">{{
                    authStore.user?.active ? "Activo" : "Inactivo"
                }}</span>
            </div>
            <div class="setting-item">
                <button
                    @click="openPasswordModal"
                    class="btn-secondary"
                >
                    Cambiar contraseña
                </button>
            </div>
        </div>

        <div class="settings-section">
            <h3>Aplicación</h3>
            <div class="setting-item">
                <span class="setting-label">Versión:</span>
                <span class="setting-value">{{ appVersion }}</span>
            </div>
            <div class="setting-item">
                <span class="setting-label">Tema:</span>
                <select v-model="theme" class="setting-select">
                    <option value="light">Claro</option>
                    <option value="dark">Oscuro</option>
                    <option value="system">Sistema</option>
                </select>
            </div>
        </div>

        <div class="settings-section">
            <h3>Sesión</h3>
            <button @click="handleLogout" class="btn-danger">
                Cerrar Sesión
            </button>
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
                        <label>Contraseña actual</label>
                        <input
                            v-model="currentPassword"
                            type="password"
                            required
                        />
                    </div>
                    <div class="form-group">
                        <label>Nueva contraseña</label>
                        <input v-model="newPassword" type="password" required />
                    </div>
                    <div class="form-group">
                        <label>Repetir nueva contraseña</label>
                        <input
                            v-model="confirmPassword"
                            type="password"
                            required
                        />
                    </div>
                    <div v-if="formError" class="error-message">
                        {{ formError }}
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
                            :disabled="isSaving"
                        >
                            {{ isSaving ? "Guardando..." : "Guardar" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>

<style scoped>
.settings-page {
    padding: 2rem;
    max-width: 100%;
    background: var(--color-bg);
    min-height: 100%;
}

h1 {
    margin: 0 0 2rem;
}

.settings-section {
    background: var(--color-surface);
    padding: 1.5rem;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    margin-bottom: 1.5rem;
}

.settings-section h3 {
    margin: 0 0 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--color-border);
}

.setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 0;
}

.setting-label {
    color: var(--color-text-muted);
}

.setting-value {
    font-weight: 500;
}

.setting-select {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path d='M1 1l5 5 5-5' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>");
    background-repeat: no-repeat;
    background-position: right 0.75rem center;
    padding-right: 2.5rem;
}

.btn-danger {
    background: var(--color-danger);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-danger:hover {
    background: var(--color-danger-dark);
}

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
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

.btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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

</style>

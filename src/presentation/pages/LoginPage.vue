<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useAuthStore } from "../stores";

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const username = ref("");
const password = ref("");
const isLoading = ref(false);
const dbReady = ref(false);

const passwordChanged = computed(() => route.query.passwordChanged === "1");

onMounted(async () => {
    try {
        await authStore.ensureDbReady();
    } finally {
        dbReady.value = true;
    }
});

async function handleLogin() {
    if (!username.value || !password.value || !dbReady.value) return;

    isLoading.value = true;
    const success = await authStore.login(username.value, password.value);
    isLoading.value = false;

    if (success) {
        router.push({ name: "home" });
    }
}
</script>

<template>
    <div class="login-container">
        <div class="login-inner">
            <h1 v-if="!dbReady" class="app-name">El Kioskon App</h1>

            <transition name="fade-slide">
                <div v-if="dbReady" class="login-card">
                    <h2>Iniciar Sesión</h2>

                    <div v-if="passwordChanged" class="success-message">
                        Contraseña cambiada correctamente. Vuelva a ingresar.
                    </div>

                    <form @submit.prevent="handleLogin">
                        <div class="form-group">
                            <label for="username">Usuario</label>
                            <input
                                id="username"
                                v-model="username"
                                type="text"
                                placeholder="Ingrese su usuario"
                                :disabled="isLoading"
                            />
                        </div>

                        <div class="form-group">
                            <label for="password">Contraseña</label>
                            <input
                                id="password"
                                v-model="password"
                                type="password"
                                placeholder="Ingrese su contraseña"
                                :disabled="isLoading"
                            />
                        </div>

                        <div v-if="authStore.error" class="error-message">
                            {{ authStore.error }}
                        </div>

                        <button
                            type="submit"
                            :disabled="isLoading || !username || !password"
                        >
                            {{ isLoading ? "Iniciando sesión..." : "Entrar" }}
                        </button>
                    </form>
                </div>
            </transition>
        </div>
    </div>
</template>

<style scoped>
.login-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--color-bg);
    position: relative;
    overflow: hidden;
}

.login-inner {
    position: relative;
    z-index: 1;
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    align-items: center;
}

.app-name {
    margin: 0;
    font-size: 2.75rem;
    font-weight: 700;
    text-align: center;
    color: color-mix(in srgb, var(--color-primary) 67%, transparent);
}

.login-container::before {
    content: "";
    position: absolute;
    width: 520px;
    height: 520px;
    left: 50%;
    top: 50%;
    transform: translate(-70%, -50%);
    background-image: url("/tienda-1024.png");
    background-repeat: no-repeat;
    background-size: contain;
    background-position: center;
    opacity: 0.12;
    pointer-events: none;
}

.login-card {
    background: color-mix(in srgb, var(--color-surface) 75%, transparent);
    padding: 2rem;
    border-radius: 12px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    width: 100%;
    max-width: 400px;
    backdrop-filter: blur(4px);
}

h2 {
    margin: 0 0 1.5rem;
    text-align: center;
    color: var(--color-text);
}

.form-group {
    margin-bottom: 1rem;
}

label {
    display: block;
    margin-bottom: 0.5rem;
    color: var(--color-text);
    font-weight: 500;
}

input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
    background: var(--color-surface);
    color: var(--color-text);
}

input:focus {
    outline: none;
    border-color: var(--color-primary);
}

button {
    width: 100%;
    padding: 0.75rem;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    margin-top: 0.5rem;
}

button:hover:not(:disabled) {
    background: var(--color-secondary);
}

button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.error-message {
    color: var(--color-danger);
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: #fed7d7;
    border-radius: 6px;
    text-align: center;
}

.success-message {
    color: #2f855a;
    margin-bottom: 1rem;
    padding: 0.75rem;
    background: #c6f6d5;
    border-radius: 6px;
    text-align: center;
}

.fade-slide-enter-active {
    transition: opacity 0.45s ease, transform 0.45s ease;
}

.fade-slide-leave-active {
    transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-slide-enter-from {
    opacity: 0;
    transform: translateY(24px);
}

.fade-slide-leave-to {
    opacity: 0;
    transform: translateY(12px);
}

</style>

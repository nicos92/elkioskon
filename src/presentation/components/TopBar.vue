<script setup lang="ts">
import { computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useAuthStore } from "../stores";
import { useConfirm } from "../composables/useConfirm";

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const { confirm } = useConfirm();

const PAGE_TITLES: Record<string, string> = {
  home: "Inicio",
  users: "Usuarios",
  proveedores: "Proveedores",
  categorias: "Categorías",
  "sub-categorias": "Sub Categorías",
  articulos: "Artículos",
  stock: "Stock",
  ventas: "Ventas",
  "nueva-venta": "Nueva Venta",
  "tipos-venta": "Tipos de Venta",
  permissions: "Permisos",
  auditoria: "Auditoría",
  cierres: "Cierres del día",
  settings: "Configuración",
};

const pageTitle = computed(() => {
  const name = String(route.name ?? "");
  return PAGE_TITLES[name] ?? name;
});

const username = computed(() => authStore.user?.username ?? "");

const initials = computed(() => {
  const parts = username.value
    .split(/[\s._-]+/)
    .filter((part) => part.length > 0);
  if (parts.length === 0) return "?";
  const first = parts[0][0];
  const last = parts.length > 1 ? parts[parts.length - 1][0] : "";
  return (first + last).toUpperCase();
});

async function handleLogout() {
  const ok = await confirm({
    title: "Cerrar sesión",
    message: "¿Está seguro de que desea cerrar la sesión?",
    confirmText: "Cerrar sesión",
    cancelText: "Cancelar",
    variant: "danger",
  });
  if (!ok) return;
  authStore.logout();
  router.push({ name: "login" });
}
</script>

<template>
    <header class="top-bar">
        <h1 class="page-title">{{ pageTitle }}</h1>
        <div class="user-section">
            <span class="avatar" aria-hidden="true">{{ initials }}</span>
            <span class="username">{{ username }}</span>
            <button
                @click="handleLogout"
                class="logout-btn"
                title="Cerrar Sesión"
            >
                <img
                    src="/svg/logout.svg"
                    alt="Cerrar Sesión"
                    class="nav-icon"
                />
            </button>
        </div>
    </header>
</template>

<style scoped>
.top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 1.5rem;
    background: var(--color-sidebar);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    position: sticky;
    top: 0;
    z-index: 20;
    width: 100%;
}

.page-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.user-section {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
}

.avatar {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: var(--color-primary);
    color: var(--color-on-primary);
    font-size: 0.8rem;
    font-weight: 600;
    flex-shrink: 0;
}

.username {
    font-size: 0.9rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 220px;
}

.logout-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
}

.logout-btn img {
    width: 20px;
    height: 20px;
}
</style>

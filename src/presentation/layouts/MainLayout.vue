<script setup lang="ts">
import { ref, computed } from "vue";
import { useRoute } from "vue-router";
import { useAuthStore } from "../stores";
import TopBar from "../components/TopBar.vue";

const route = useRoute();
const authStore = useAuthStore();

const isSidebarOpen = ref(true);

const menuItems = computed(() => {
    const items = [
        { name: "home", label: "Inicio", icon: "home", permission: null },
        {
            name: "users",
            label: "Usuarios",
            icon: "users",
            permission: "ver_usuarios",
        },
        {
            name: "proveedores",
            label: "Proveedores",
            icon: "proveedor",
            permission: "ver_proveedor",
        },
        {
            name: "clientes",
            label: "Clientes",
            icon: "cliente",
            permission: "ver_clientes",
        },
        {
            name: "categorias",
            label: "Categorías",
            icon: "category",
            permission: "ver_categorias",
        },
        {
            name: "sub-categorias",
            label: "Sub Categorías",
            icon: "subcategory",
            permission: "ver_sub_categorias",
        },
        {
            name: "articulos",
            label: "Artículos",
            icon: "article",
            permission: "ver_articulos",
        },
        {
            name: "stock",
            label: "Stock",
            icon: "stock",
            permission: "ver_stock",
        },
        {
            name: "ventas",
            label: "Ventas",
            icon: "ventas",
            permission: "ver_ventas",
        },
        {
            name: "tipos-venta",
            label: "Tipos de Venta",
            icon: "card",
            permission: "ver_tipos_venta",
        },
        {
            name: "presupuestos",
            label: "Presupuestos",
            icon: "card",
            permission: "generar_presupuesto",
        },
        {
            name: "permissions",
            label: "Permisos",
            icon: "lock",
            permission: "ver_permisos",
        },
        {
            name: "auditoria",
            label: "Auditoría",
            icon: "audit",
            permission: "ver_auditoria",
        },
        {
            name: "cierres",
            label: "Cierres del día",
            icon: "calendar",
            permission: "ver_cierres",
        },
        {
            name: "dolar",
            label: "Dólar",
            icon: "dollar",
            permission: "ver_dolar",
        },
        {
            name: "settings",
            label: "Configuración",
            icon: "settigns",
            permission: null,
        },
    ];

    return items.filter((item) => {
        if (!item.permission) return true;
        return authStore.hasPermission(item.permission);
    });
});

function isActive(name: string): boolean {
    return route.name === name;
}
</script>

<template>
    <div class="main-layout">
        <aside :class="['sidebar', { collapsed: !isSidebarOpen }]">
            <div class="sidebar-header">
                <h2 v-if="isSidebarOpen">
                    El Kioskon App
                </h2>
                <button
                    @click="isSidebarOpen = !isSidebarOpen"
                    class="toggle-btn"
                >
                    {{ isSidebarOpen ? "◀" : "▶" }}
                </button>
            </div>

            <nav class="sidebar-nav">
                <router-link
                    v-for="item in menuItems"
                    :key="item.name"
                    :to="{ name: item.name }"
                    :class="['nav-item', { active: isActive(item.name) }]"
                >
                    <img
                        :src="`/svg/${item.icon}.svg`"
                        :alt="item.label"
                        class="nav-icon"
                    />
                    <span v-if="isSidebarOpen" class="nav-label">{{
                        item.label
                    }}</span>
                </router-link>
            </nav>

            <div class="sidebar-footer">
                <span class="footer-hint">Kioskon App</span>
            </div>
        </aside>

        <main class="main-content">
            <TopBar />
            <div class="page-container">
                <router-view />
            </div>
        </main>
    </div>
</template>

<style scoped>
.main-layout {
    display: flex;
    height: 100vh;
    background: var(--color-bg);
}

.sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 56px;
    padding: 0 1rem;
    border-bottom: 1px solid #334155;
    position: sticky;
    top: 0;
    z-index: 10;
    background: #1e293b;
}

.sidebar-header h2 {
    margin: 0;
    font-size: 1.2rem;
    white-space: nowrap;
}

.toggle-btn {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 1rem;
    padding: 0.25rem;
}

.sidebar {
    width: 250px;
    background: #1e293b;
    color: white;
    display: flex;
    flex-direction: column;
    transition: width 0.3s ease;
    overflow: hidden;
}

.sidebar.collapsed {
    width: 60px;
}

.sidebar-nav {
    flex: 1;
    padding: 1rem 0;
    overflow-y: auto;
}

.nav-item {
    display: flex;
    align-items: center;
    padding: 0.75rem 1rem;
    color: #94a3b8;
    text-decoration: none;
    transition: all 0.2s;
}

.nav-item:hover {
    background: #334155;
    color: white;
}

.nav-item.active {
    background: #667e99;
    color: white;
}

.nav-icon {
    width: 20px;
    height: 20px;
}

.nav-label {
    margin-left: 0.5rem;
    white-space: nowrap;
}

.sidebar-footer {
    padding: 1rem;
    border-top: 1px solid #334155;
}

.footer-hint {
    font-size: 0.8rem;
    color: #475569;
}

.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
}

.page-container {
    flex: 1;
    overflow-y: auto;
}
</style>

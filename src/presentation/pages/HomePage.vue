<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useAuthStore, useHomeStore } from "../stores";
import { usePermissions } from "../composables/usePermissions";

const authStore = useAuthStore();
const {
    canViewArticulos,
    canViewUsers,
    canViewProveedores,
    canViewVentas,
    canViewCategorias,
    canViewSubCategorias,
} = usePermissions();
const homeStore = useHomeStore();

onMounted(async () => {
    await homeStore.fetchStats();
});

const totalArticulos = computed(() => homeStore.stats?.total_articulos ?? 0);
const articulosConStock = computed(() => homeStore.stats?.articulos_con_stock ?? 0);
const totalUsuarios = computed(() => homeStore.stats?.total_usuarios ?? 0);
const usuariosActivos = computed(() => homeStore.stats?.usuarios_activos ?? 0);
const usuariosInactivos = computed(() => homeStore.stats?.usuarios_inactivos ?? 0);
const totalProveedores = computed(() => homeStore.stats?.total_proveedores ?? 0);
const totalCategorias = computed(() => homeStore.stats?.total_categorias ?? 0);
const totalSubCategorias = computed(() => homeStore.stats?.total_sub_categorias ?? 0);

const ventasDelDia = computed(() => homeStore.stats?.ventas_hoy ?? 0);
const totalVentasDelDia = computed(() => homeStore.stats?.total_ventas_hoy ?? 0);

function stat(hasPermission: boolean, value: string | number): string {
    return hasPermission ? String(value) : "—";
}

const categoriasConSubcategorias = computed(() => {
    return (homeStore.stats?.categorias ?? []).map((cat) => ({
        id: cat.id,
        categoria: cat.categoria,
        subCategorias: cat.sub_categorias,
    }));
});

const articulosBajoStock = computed(() => {
    return (homeStore.stats?.stock_bajo ?? []).map((s) => ({
        id: s.id_stock,
        articuloNombre: s.articulo,
        codArticulo: s.cod_articulo,
        cantidad: s.cantidad,
    }));
});
</script>

<template>
    <div class="home-page">
        <h1>Bienvenido</h1>
        <p>
            Has iniciado sesión como:
            <strong>{{ authStore.user?.username }}</strong>
        </p>

        <div v-if="homeStore.loading" class="loading">Cargando datos...</div>

        <template v-else>
            <div class="stats-cards">
                <div class="stat-card">
                    <div class="stat-icon primary">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                            />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewArticulos(), totalArticulos)
                        }}</span>
                        <span class="stat-label">Artículos</span>
                        <span v-if="canViewArticulos()" class="stat-sub">
                            {{ articulosConStock }} con stock
                        </span>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon success">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"
                            />
                            <circle cx="9" cy="7" r="4" />
                            <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                            <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewUsers(), totalUsuarios)
                        }}</span>
                        <span class="stat-label">Usuarios</span>
                        <span v-if="canViewUsers()" class="stat-sub">
                            {{ usuariosActivos }} activos · {{ usuariosInactivos }} inactivos
                        </span>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon warning">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path d="M1 3h15v13H1z" />
                            <path d="M16 8h4l3 3v5h-7V8z" />
                            <circle cx="5.5" cy="18.5" r="2.5" />
                            <circle cx="18.5" cy="18.5" r="2.5" />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewProveedores(), totalProveedores)
                        }}</span>
                        <span class="stat-label">Proveedores</span>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon danger">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <line x1="12" y1="1" x2="12" y2="23" />
                            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewVentas(), ventasDelDia)
                        }}</span>
                        <span class="stat-label">Ventas</span>
                        <span v-if="canViewVentas()" class="stat-sub">
                            Hoy · ${{ totalVentasDelDia.toFixed(2) }}
                        </span>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon primary">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                            />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewCategorias(), totalCategorias)
                        }}</span>
                        <span class="stat-label">Categorías</span>
                    </div>
                </div>

                <div class="stat-card">
                    <div class="stat-icon warning">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="24"
                            height="24"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        >
                            <path
                                d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                            />
                            <line x1="12" y1="11" x2="12" y2="17" />
                            <line x1="9" y1="14" x2="15" y2="14" />
                        </svg>
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">{{
                            stat(canViewSubCategorias(), totalSubCategorias)
                        }}</span>
                        <span class="stat-label">Sub Categorías</span>
                    </div>
                </div>
            </div>

            <div class="card full-width">
                <h3>Stock Bajo</h3>
                <p class="card-description">
                    Artículos con menos de 10 unidades
                </p>
                <div class="table-container">
                    <table
                        v-if="
                            !homeStore.loading &&
                            articulosBajoStock.length > 0
                        "
                        class="low-stock-table"
                    >
                        <thead>
                            <tr>
                                <th>Código</th>
                                <th>Artículo</th>
                                <th>Cantidad</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr
                                v-for="item in articulosBajoStock"
                                :key="item.id"
                            >
                                <td>{{ item.codArticulo }}</td>
                                <td>{{ item.articuloNombre }}</td>
                                <td
                                    class="quantity-cell"
                                    :class="{ critical: item.cantidad < 5 }"
                                >
                                    {{ item.cantidad }}
                                </td>
                            </tr>
                        </tbody>
                    </table>
                    <div
                        v-if="
                            !homeStore.loading &&
                            articulosBajoStock.length === 0
                        "
                        class="empty-state"
                    >
                        No hay artículos con stock bajo
                    </div>
                </div>
            </div>

            <div class="card full-width">
                <h3>Categorías y Sub Categorías</h3>
                <div class="categories-list">
                    <div
                        v-for="cat in categoriasConSubcategorias"
                        :key="cat.id"
                        class="category-item"
                    >
                        <div class="category-name">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="16"
                                height="16"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                                />
                            </svg>
                            {{ cat.categoria }}
                        </div>
                        <div class="subcategories">
                            <span
                                v-for="subCat in cat.subCategorias"
                                :key="subCat.id"
                                class="subcategory-tag"
                            >
                                {{ subCat.sub_categoria }}
                            </span>
                            <span
                                v-if="cat.subCategorias.length === 0"
                                class="no-subcategories"
                            >
                                Sin sub categorías
                            </span>
                        </div>
                    </div>
                    <div
                        v-if="
                            !homeStore.loading &&
                            categoriasConSubcategorias.length === 0
                        "
                        class="empty-state"
                    >
                        No hay categorías registradas
                    </div>
                </div>
            </div>
        </template>
    </div>
</template>

<style scoped>
.home-page {
    padding: 2rem;
    background: var(--color-bg);
    min-height: 100%;

}

h1 {
    margin: 0 0 0.5rem;
    color: var(--color-text);
}

p {
    color: var(--color-text-muted);
    margin-bottom: 2rem;
}

.loading {
    text-align: center;
    padding: 2rem;
    color: var(--color-text-muted);
}

.stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
}

.stat-card {
    background: var(--color-surface);
    padding: 1.5rem;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    display: flex;
    align-items: center;
    gap: 1rem;
}

.stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.stat-icon.primary {
    background: color-mix(in srgb, var(--color-primary) 15%, var(--color-surface));
    color: var(--color-primary);
}

.stat-icon.success {
    background: color-mix(in srgb, var(--color-success) 15%, var(--color-surface));
    color: var(--color-success);
}

.stat-icon.warning {
    background: color-mix(in srgb, var(--color-warning) 15%, var(--color-surface));
    color: var(--color-warning);
}

.stat-icon.danger {
    background: color-mix(in srgb, var(--color-danger) 15%, var(--color-surface));
    color: var(--color-danger);
}

.stat-info {
    display: flex;
    flex-direction: column;
}

.stat-value {
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--color-text);
}

.stat-label {
    font-size: 0.875rem;
    color: var(--color-text-muted);
}

.stat-sub {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin-top: 0.25rem;
}

.card {
    background: var(--color-surface);
    padding: 1.5rem;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.card.full-width {
    width: 100%;
    margin-bottom: 1.5rem;
}

.card h3 {
    margin: 0 0 0.5rem;
    color: var(--color-primary);
}

.card p {
    margin: 0 0 1rem;
    color: var(--color-text-muted);
    font-size: 0.9rem;
}

.card-description {
    margin-top: -0.5rem;
    margin-bottom: 1rem;
}

.categories-list {
    max-height: 300px;
    overflow-y: auto;
}

.category-item {
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--color-border);
}

.category-item:last-child {
    border-bottom: none;
}

.category-name {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 0.5rem;
}

.subcategories {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding-left: 1.5rem;
}

.subcategory-tag {
    background: var(--color-surface-2);
    color: var(--color-primary);
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-size: 0.8rem;
}

.no-subcategories {
    color: var(--color-text-muted);
    font-size: 0.8rem;
    font-style: italic;
}

.table-container {
    max-height: 300px;
    overflow: auto;
}

.low-stock-table {
    width: 100%;
    border-collapse: collapse;
}

.low-stock-table th,
.low-stock-table td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--color-border);
}

.low-stock-table th {
    font-weight: 600;
    color: var(--color-text-muted);
    font-size: 0.875rem;
    background: var(--color-surface-2);
}

.low-stock-table td {
    color: var(--color-text);
}

.quantity-cell {
    font-weight: 600;
    color: var(--color-warning);
}

.quantity-cell.critical {
    color: var(--color-danger);
}

.config-card {
    margin-top: 0;
}

.card-link {
    display: inline-block;
    padding: 0.5rem 1rem;
    background: var(--color-primary);
    color: var(--color-on-primary);
    text-decoration: none;
    border-radius: 6px;
    font-size: 0.9rem;
}

.card-link:hover {
    background: var(--color-secondary);
}

.empty-state {
    text-align: center;
    padding: 1.5rem;
    color: var(--color-text-muted);
    font-style: italic;
}
</style>

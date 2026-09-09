import { createRouter, createWebHistory } from "vue-router";
import { useAuthStore } from "../stores";
import { PERMISSIONS } from "../../domain/entities";

const routes = [
  {
    path: "/login",
    name: "login",
    component: () => import("../pages/LoginPage.vue"),
    meta: { requiresAuth: false },
  },
  {
    path: "/",
    component: () => import("../layouts/MainLayout.vue"),
    meta: { requiresAuth: true },
    children: [
      {
        path: "",
        name: "home",
        component: () => import("../pages/HomePage.vue"),
      },
      {
        path: "users",
        name: "users",
        component: () => import("../pages/UsersPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_USERS },
      },
      {
        path: "users/:id/permissions",
        name: "user-permissions",
        component: () => import("../pages/UserPermissionsPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_USERS },
      },
      {
        path: "proveedores",
        name: "proveedores",
        component: () => import("../pages/ProveedoresPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_PROVEEDORES },
      },
      {
        path: "clientes",
        name: "clientes",
        component: () => import("../pages/ClientesPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_CLIENTES },
      },
      {
        path: "categorias",
        name: "categorias",
        component: () => import("../pages/CategoriasPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_CATEGORIAS },
      },
      {
        path: "sub-categorias",
        name: "sub-categorias",
        component: () => import("../pages/SubCategoriasPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_SUB_CATEGORIAS },
      },
      {
        path: "articulos",
        name: "articulos",
        component: () => import("../pages/ArticulosPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_ARTICULOS },
      },
      {
        path: "stock",
        name: "stock",
        component: () => import("../pages/StockPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_STOCK },
      },
      {
        path: "stock/actualizar-costo",
        name: "actualizar-costo",
        component: () => import("../pages/ActualizarCostoPage.vue"),
        meta: { permission: PERMISSIONS.UPDATE_STOCK },
      },
      {
        path: "ventas",
        name: "ventas",
        component: () => import("../pages/VentasPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_VENTAS },
      },
      {
        path: "ventas/nueva",
        name: "nueva-venta",
        component: () => import("../pages/NuevaVentaPage.vue"),
        meta: { permission: PERMISSIONS.CREATE_VENTA },
      },
      {
        path: "tipos-venta",
        name: "tipos-venta",
        component: () => import("../pages/TiposVentaPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_TIPOS_VENTA },
      },
      {
        path: "presupuestos",
        name: "presupuestos",
        component: () => import("../pages/PresupuestosPage.vue"),
        meta: { permission: PERMISSIONS.GENERAR_PRESUPUESTO },
      },
      {
        path: "permissions",
        name: "permissions",
        component: () => import("../pages/PermissionsPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_PERMISSIONS },
      },
      {
        path: "auditoria",
        name: "auditoria",
        component: () => import("../pages/AuditoriaPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_AUDITORIA },
      },
      {
        path: "cierres",
        name: "cierres",
        component: () => import("../pages/CierresPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_CIERRES },
      },
      {
        path: "dolar",
        name: "dolar",
        component: () => import("../pages/DolarPage.vue"),
        meta: { permission: PERMISSIONS.VIEW_DOLAR },
      },
      {
        path: "settings",
        name: "settings",
        component: () => import("../pages/SettingsPage.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore();
  authStore.loadFromStorage();

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next({ name: "login" });
  } else if (to.name === "login" && authStore.isAuthenticated) {
    next({ name: "home" });
  } else if (to.meta.permission && typeof to.meta.permission === "string") {
    if (authStore.hasPermission(to.meta.permission)) {
      next();
    } else {
      next({ name: "home" });
    }
  } else {
    next();
  }
});

export default router;

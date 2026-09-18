import { computed, nextTick, ref } from "vue";

export interface CartItem {
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  stockDisponible: number;
  cantidad: number;
  precio: number;
  subtotal: number;
}

export interface CartSourceItem {
  id_articulo: number;
  cod_articulo: string;
  articulo: string;
  stockDisponible: number;
  precioVenta: number;
}

export interface UseCartOptions {
  getVendibles: () => CartSourceItem[];
  canVenderSinStock: () => boolean;
  getTipoVentaId: () => number | null;
  focusInput?: () => void;
}

export function useCart({
  getVendibles,
  canVenderSinStock,
  getTipoVentaId,
  focusInput,
}: UseCartOptions) {
  const cart = ref<CartItem[]>([]);
  const searchQuery = ref("");
  const descuento = ref<number>(0);

  const searchResults = computed<CartSourceItem[]>(() => {
    const query = searchQuery.value.trim().toLowerCase();
    const inCart = new Set(cart.value.map((c) => c.id_articulo));
    const base = getVendibles().filter((a) => !inCart.has(a.id_articulo));
    if (!query) return [];
    return base
      .filter(
        (a) =>
          a.cod_articulo.toLowerCase().includes(query) ||
          a.articulo.toLowerCase().includes(query),
      )
      .slice(0, 20);
  });

  const carritoSubtotal = computed(() =>
    cart.value.reduce((acc, item) => acc + item.subtotal, 0),
  );

  const descuentoMonto = computed(() => {
    const d = Number.isFinite(descuento.value) ? descuento.value : 0;
    return (carritoSubtotal.value * d) / 100;
  });

  const carritoTotal = computed(() => carritoSubtotal.value - descuentoMonto.value);

  const descuentoValido = computed(
    () =>
      descuento.value === null ||
      (descuento.value >= 0 && descuento.value <= 100),
  );

  const carritoValido = computed(
    () =>
      cart.value.length > 0 &&
      descuentoValido.value &&
      getTipoVentaId() !== null &&
      cart.value.every((i) => i.cantidad > 0 && i.precio >= 0),
  );

  const presupuestoValido = computed(
    () =>
      cart.value.length > 0 &&
      descuentoValido.value &&
      cart.value.every((i) => i.cantidad > 0 && i.precio >= 0),
  );

  function focusSearch() {
    nextTick(() => {
      focusInput?.();
    });
  }

  function addArticuloById(idArticulo: number) {
    const articulo = getVendibles().find((a) => a.id_articulo === idArticulo);
    if (!articulo) return;

    const existing = cart.value.find((c) => c.id_articulo === idArticulo);
    if (existing) {
      existing.cantidad += 1;
      existing.subtotal = existing.cantidad * existing.precio;
    } else {
      cart.value.push({
        id_articulo: articulo.id_articulo,
        cod_articulo: articulo.cod_articulo,
        articulo: articulo.articulo,
        stockDisponible: articulo.stockDisponible,
        cantidad: 1,
        precio: articulo.precioVenta,
        subtotal: articulo.precioVenta,
      });
    }

    searchQuery.value = "";
    focusSearch();
  }

  function onSearchEnter() {
    const query = searchQuery.value.trim().toLowerCase();
    if (!query) return;
    const exact = getVendibles().find(
      (a) => a.cod_articulo.toLowerCase() === query,
    );
    if (exact) {
      addArticuloById(exact.id_articulo);
      return;
    }
    if (searchResults.value.length === 1) {
      addArticuloById(searchResults.value[0].id_articulo);
    }
  }

  function removeArticulo(idArticulo: number) {
    cart.value = cart.value.filter((c) => c.id_articulo !== idArticulo);
  }

  function vaciarCarrito() {
    cart.value = [];
  }

  function updateSubtotal(item: CartItem) {
    item.subtotal = item.cantidad * item.precio;
  }

  function stockWarning(item: CartItem): boolean {
    return item.cantidad > item.stockDisponible && !canVenderSinStock();
  }

  function setItems(items: CartItem[]) {
    cart.value = items;
  }

  function resetCart() {
    cart.value = [];
    descuento.value = 0;
    searchQuery.value = "";
  }

  return {
    cart,
    searchQuery,
    descuento,
    searchResults,
    carritoSubtotal,
    descuentoMonto,
    carritoTotal,
    descuentoValido,
    carritoValido,
    presupuestoValido,
    focusSearch,
    addArticuloById,
    onSearchEnter,
    removeArticulo,
    vaciarCarrito,
    updateSubtotal,
    stockWarning,
    setItems,
    resetCart,
  };
}

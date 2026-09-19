import { computed, ref } from "vue";
import type {
  Cliente,
  CreateClienteRequest,
} from "../../../domain/entities";
import type { useClientesStore } from "../../stores";

export interface UseClienteSeleccionOptions {
  clientesStore: ReturnType<typeof useClientesStore>;
}

export function useClienteSeleccion({
  clientesStore,
}: UseClienteSeleccionOptions) {
  const clienteSeleccionado = ref<Cliente | null>(null);
  const clienteDefecto = ref<Cliente | null>(null);
  const clienteQuery = ref("");
  const mostrandoClientes = ref(false);
  const mostrarModalNuevoCliente = ref(false);

  const clientesFiltrados = computed<Cliente[]>(() => {
    const query = clienteQuery.value.trim().toLowerCase();
    const base = clientesStore.clientes;
    if (!query) return base;
    return base.filter(
      (c) =>
        (c.nombre || "").toLowerCase().includes(query) ||
        (c.apellido || "").toLowerCase().includes(query) ||
        (c.telefono || "").toLowerCase().includes(query) ||
        (c.email || "").toLowerCase().includes(query),
    );
  });

  async function cargarDefecto() {
    const def = await clientesStore.getClienteDefecto();
    clienteDefecto.value = def;
    clienteSeleccionado.value = def;
  }

  function seleccionarCliente(cliente: Cliente) {
    clienteSeleccionado.value = cliente;
    clienteQuery.value = "";
    mostrandoClientes.value = false;
  }

  function quitarCliente() {
    clienteSeleccionado.value = clienteDefecto.value;
    clienteQuery.value = "";
    mostrandoClientes.value = false;
  }

  function abrirModalNuevoCliente() {
    clientesStore.error = null;
    mostrarModalNuevoCliente.value = true;
  }

  async function crearClienteRapido(
    request: CreateClienteRequest,
  ): Promise<Cliente | null> {
    const nuevoCliente = await clientesStore.crearCliente(request);
    if (nuevoCliente) {
      seleccionarCliente(nuevoCliente);
      mostrarModalNuevoCliente.value = false;
    }
    return nuevoCliente;
  }

  return {
    clienteSeleccionado,
    clienteDefecto,
    clienteQuery,
    mostrandoClientes,
    mostrarModalNuevoCliente,
    clientesFiltrados,
    cargarDefecto,
    seleccionarCliente,
    quitarCliente,
    abrirModalNuevoCliente,
    crearClienteRapido,
  };
}
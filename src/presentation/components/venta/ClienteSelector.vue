<script setup lang="ts">
import type { Cliente } from "../../../domain/entities";
import { clienteLabel } from "../../utils/cliente";

defineProps<{
  clientes: Cliente[];
  query: string;
  show: boolean;
  selected: Cliente | null;
  canCreate: boolean;
}>();

const emit = defineEmits<{
  "update:query": [value: string];
  "update:show": [value: boolean];
  select: [cliente: Cliente];
  clear: [];
  create: [];
}>();

function onInput(event: Event) {
  emit("update:query", (event.target as HTMLInputElement).value);
  emit("update:show", true);
}

function onBlur() {
  setTimeout(() => {
    emit("update:show", false);
  }, 150);
}
</script>

<template>
    <div class="form-group obs-group cliente-group">
        <label>Cliente</label>
        <div class="cliente-selector">
            <input
                :value="query"
                type="text"
                placeholder="Buscar por nombre, apellido o teléfono..."
                @focus="emit('update:show', true)"
                @input="onInput"
                @blur="onBlur"
            />
            <button
                v-if="canCreate"
                type="button"
                class="btn-nuevo-cliente"
                title="Crear nuevo cliente"
                @click="emit('create')"
            >
                + Nuevo
            </button>
        </div>
        <div v-if="show" class="cliente-dropdown">
            <button
                v-for="cliente in clientes"
                :key="cliente.id"
                type="button"
                class="cliente-option"
                @mousedown.prevent="emit('select', cliente)"
            >
                <span class="cliente-nombre">
                    {{ clienteLabel(cliente) }}
                </span>
                <span v-if="cliente.telefono" class="cliente-tel">
                    {{ cliente.telefono }}
                </span>
            </button>
            <div v-if="clientes.length === 0" class="cliente-empty">
                Sin coincidencias
            </div>
        </div>
        <div v-if="selected && !show" class="cliente-seleccionado">
            <span>{{ clienteLabel(selected) }}</span>
            <button
                type="button"
                class="cliente-limpiar"
                title="Usar Consumidor Final"
                @click="emit('clear')"
            >
                ×
            </button>
        </div>
    </div>
</template>

<style scoped>
.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
}

.form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-sizing: border-box;
    background: var(--color-surface);
    color: var(--color-text);
}

.obs-group {
    flex: 1;
    min-width: 240px;
    margin: 0;
}

.cliente-group {
    position: relative;
    min-width: 280px;
}

.cliente-selector {
    display: flex;
    gap: 0.5rem;
}

.cliente-selector input {
    flex: 1;
}

.btn-nuevo-cliente {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
    font-weight: 500;
}

.btn-nuevo-cliente:hover {
    background: var(--color-border);
}

.cliente-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 30;
    max-height: 220px;
    overflow-y: auto;
    background: var(--color-surface);
    border: 2px solid var(--color-primary);
    border-radius: 6px;
    margin-top: 0rem;
    box-shadow: 0 4px 12px rgba(var(--color-shadow-rgb), 0.15);
}

.cliente-option {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    text-align: left;
    padding: 0.6rem 0.75rem;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text);
}

.cliente-option:hover {
    background: var(--color-surface-2);
}

.cliente-nombre {
    font-weight: 500;
}

.cliente-tel {
    color: var(--color-text-muted);
    font-size: 0.85rem;
}

.cliente-empty {
    padding: 0.6rem 0.75rem;
    color: var(--color-text-muted);
    font-size: 0.9rem;
}

.cliente-seleccionado {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--color-surface-2);
    border-radius: 6px;
    font-weight: 500;
}

.cliente-limpiar {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    font-size: 1.1rem;
    line-height: 1;
    padding: 0 0.25rem;
}

.cliente-limpiar:hover {
    color: var(--color-danger);
}

.btn-nuevo-cliente:focus-visible,
.cliente-option:focus-visible,
.cliente-limpiar:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 1px;
}

.cliente-selector input:focus-visible {
    outline: 2px solid var(--color-secondary);
    outline-offset: 1px;
}
</style>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { CreateClienteRequest } from "../../../domain/entities";

const props = defineProps<{
  modelValue: boolean;
  error: string | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  submit: [request: CreateClienteRequest];
}>();

const nuevoNombre = ref("");
const nuevoApellido = ref("");
const nuevoTelefono = ref("");
const nuevoEmail = ref("");
const nuevoDireccion = ref("");

const nuevoClienteInvalido = computed(
  () =>
    !nuevoNombre.value.trim() &&
    !nuevoApellido.value.trim() &&
    !nuevoTelefono.value.trim() &&
    !nuevoEmail.value.trim() &&
    !nuevoDireccion.value.trim(),
);

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      nuevoNombre.value = "";
      nuevoApellido.value = "";
      nuevoTelefono.value = "";
      nuevoEmail.value = "";
      nuevoDireccion.value = "";
    }
  },
);

function close() {
  emit("update:modelValue", false);
}

function onSubmit() {
  if (nuevoClienteInvalido.value) return;
  emit("submit", {
    nombre: nuevoNombre.value.trim() || undefined,
    apellido: nuevoApellido.value.trim() || undefined,
    telefono: nuevoTelefono.value.trim() || undefined,
    email: nuevoEmail.value.trim() || undefined,
    direccion: nuevoDireccion.value.trim() || undefined,
  });
}
</script>

<template>
    <div
        v-if="modelValue"
        class="modal-overlay"
        @click.self="close"
    >
        <div class="modal-card">
            <div class="modal-header">
                <h2>Nuevo cliente</h2>
                <button type="button" class="btn-icon" @click="close">
                    ×
                </button>
            </div>
            <form @submit.prevent="onSubmit">
                <div class="form-group">
                    <label>Nombre</label>
                    <input v-model="nuevoNombre" type="text" placeholder="Nombre" />
                </div>
                <div class="form-group">
                    <label>Apellido</label>
                    <input v-model="nuevoApellido" type="text" placeholder="Apellido" />
                </div>
                <div class="form-group">
                    <label>Teléfono</label>
                    <input v-model="nuevoTelefono" type="text" placeholder="Teléfono" />
                </div>
                <div class="form-group">
                    <label>Email</label>
                    <input v-model="nuevoEmail" type="email" placeholder="Email" />
                </div>
                <div class="form-group">
                    <label>Dirección</label>
                    <input v-model="nuevoDireccion" type="text" placeholder="Dirección" />
                </div>
                <div v-if="error" class="error-text">
                    {{ error }}
                </div>
                <div class="modal-actions">
                    <button
                        type="button"
                        class="btn-secondary"
                        @click="close"
                    >
                        Cancelar
                    </button>
                    <button
                        type="submit"
                        class="btn-primary"
                        :disabled="nuevoClienteInvalido"
                    >
                        Crear y seleccionar
                    </button>
                </div>
            </form>
        </div>
    </div>
</template>

<style scoped>
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
}

.modal-card {
    background: var(--color-surface);
    border-radius: 12px;
    padding: 1.5rem;
    width: 100%;
    max-width: 420px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 30px rgba(var(--color-shadow-rgb), 0.3);
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.modal-header h2 {
    margin: 0;
}

.modal-header .btn-icon {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.25rem;
    color: var(--color-text-muted);
    padding: 0.25rem;
}

.form-group {
    margin-bottom: 1rem;
}

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

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
}

.btn-primary {
    background: var(--color-primary);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
}

.btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.75rem 1rem;
    border-radius: 6px;
    cursor: pointer;
}

.error-text {
    color: var(--color-danger);
    margin-top: 0.5rem;
    font-size: 0.9rem;
}
</style>

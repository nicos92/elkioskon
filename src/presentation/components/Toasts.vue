<script setup lang="ts">
import { useToasts } from "../composables/useToasts";

const { toasts } = useToasts();
</script>

<template>
  <TransitionGroup name="toast" tag="div" class="toasts-container">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      class="toast"
      :class="toast.type"
    >
      {{ toast.message }}
    </div>
  </TransitionGroup>
</template>

<style scoped>
.toasts-container {
  position: fixed;
  top: 1rem;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  z-index: 2000;
  max-width: 360px;
}

.toast {
  padding: 0.75rem 1rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.15);
  font-size: 0.9rem;
  color: #fff;
}

.toast.error {
  background: var(--color-danger);
}

.toast.success {
  background: var(--color-success);
  color: var(--color-on-success);
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(1rem);
}
</style>

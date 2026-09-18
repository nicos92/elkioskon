<script setup lang="ts">
import Toasts from "./presentation/components/Toasts.vue";
import ConfirmDialog from "./presentation/components/ConfirmDialog.vue";
</script>

<template>
  <router-view />
  <Toasts />
  <ConfirmDialog />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

button,
input,
select,
textarea {
  font: inherit;
  color: inherit;
}

button,
select {
  appearance: none;
  -webkit-appearance: none;
}

select {
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path d='M1 1l5 5 5-5' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'/></svg>");
  background-repeat: no-repeat;
  background-position: right 0.75rem center;
  padding-right: 2.5rem;
}

button:not(.btn-icon).btn-primary,
button:not(.btn-icon).btn-secondary,
button:not(.btn-icon).btn-danger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1.2;
}

.btn-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-secondary);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-success {
  background: var(--color-success);
  color: var(--color-on-success);
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
}

.btn-success:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-success:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--color-surface-2);
  color: var(--color-text);
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.form-group input,
.form-group select,
.form-group textarea {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-sizing: border-box;
  background: var(--color-surface);
  color: var(--color-text);
}

.actions {
  display: flex;
  gap: 0.5rem;
}

.data-table {
  width: 100%;
  background: var(--color-surface);
}

.data-table th,
.data-table td {
  padding: 1rem;
  text-align: left;
}

.data-table th {
  background: var(--color-surface-2);
  font-weight: 600;
}

.data-table tbody td {
  border-bottom: 1px solid var(--color-border);
}

.data-table tbody tr:last-child td {
  border-bottom: none;
}

.data-table tbody tr:hover td {
  background: var(--color-surface-2);
}

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.25rem;
}

.btn-icon img {
  width: 18px;
  height: 18px;
}

.btn-danger:hover {
  opacity: 0.7;
}

.error-banner {
  color: var(--color-danger);
  background: rgba(var(--color-danger-rgb), 0.1);
  border: 1px solid rgba(var(--color-danger-rgb), 0.3);
  padding: 0.75rem 1rem;
  border-radius: 6px;
  margin-bottom: 1rem;
}

.error-message {
  color: var(--color-danger);
  margin-bottom: 1rem;
}

.loading,
.empty-state {
  text-align: center;
  padding: 2rem;
  color: var(--color-text-muted);
}

html,
body,
#app {
  height: 100%;
  overflow: hidden;
  background-color: var(--color-bg);

}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: var(--color-bg);
  color: var(--color-text);
}

body {
  transition: background-color 0.2s ease, color 0.2s ease;
}

html {
  --color-primary: #00796b;
  --color-on-primary: #ffffff;
  --color-sidebar: #263238;
  --color-sidebar-text-muted: #cfd8dc;
  --color-success: #1de9b6;
  --color-success-dark: #00bfa5;
  --color-success-rgb: 29, 233, 182;
  --color-on-success: #004d40;
  --color-warning: #e53935;
  --color-warning-rgb: 229, 57, 53;
}

html[data-theme="light"] {
  color-scheme: light;
  --color-bg: #eceff1;
  --color-surface: #ffffff;
  --color-surface-2: #cfd8dc;
  --color-text: #263238;
  --color-text-muted: #607d8b;
  --color-text-secondary: #607d8b;
  --color-border: #90a4ae;
  --color-sidebar: #cfd8dc;
  --color-sidebar-text-muted: #455a64;
  --color-primary: #00796b;
  --color-on-primary: #ffffff;
  --color-secondary: #00695c;
  --color-success: #1de9b6;
  --color-success-dark: #00bfa5;
  --color-success-rgb: 29, 233, 182;
  --color-on-success: #004d40;
  --color-warning: #e53935;
  --color-warning-rgb: 229, 57, 53;
  --color-danger: #d32f2f;
  --color-danger-dark: #c62828;
  --color-danger-rgb: 211, 47, 47;
  --color-shadow-rgb: 38, 50, 56;
}

html[data-theme="dark"] {
  color-scheme: dark;
  --color-bg: #263238;
  --color-surface: #37474f;
  --color-surface-2: #455a64;
  --color-text: #eceff1;
  --color-text-muted: #90a4ae;
  --color-text-secondary: #90a4ae;
  --color-border: #546e7a;
  --color-sidebar: #37474f;
  --color-sidebar-text-muted: #cfd8dc;
  --color-primary: #009688;
  --color-on-primary: #ffffff;
  --color-secondary: #26a69a;
  --color-success: #1de9b6;
  --color-success-dark: #00bfa5;
  --color-success-rgb: 29, 233, 182;
  --color-on-success: #004d40;
  --color-warning: #ef5350;
  --color-warning-rgb: 239, 83, 80;
  --color-danger: #f44336;
  --color-danger-dark: #e53935;
  --color-danger-rgb: 244, 67, 54;
  --color-shadow-rgb: 0, 0, 0;
}

.print-area {
  display: none;
}

@media print {
  body > * {
    display: none !important;
  }

  body > .print-area {
    display: block !important;
  }

  .print-area * {
    display: revert !important;
  }

  .print-area {
    padding: 1rem;
  }

  .print-summary {
    border: 1px solid #000;
    padding: 0.75rem 1rem;
    margin: 0.75rem 0;
  }

  .print-area table {
    width: 100%;
    border-collapse: collapse;
  }

  .print-area th,
  .print-area td {
    border: 1px solid #000;
    padding: 0.5rem;
    text-align: left;
  }

  .print-total {
    font-weight: bold;
    text-align: right;
  }

  .print-line {
    text-align: right;
    margin: 0.25rem 0;
  }

  .print-obs {
    margin-top: 1rem;
  }
}
</style>

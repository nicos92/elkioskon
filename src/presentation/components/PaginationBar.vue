<script setup lang="ts">
type PageItem = number | "...";

defineProps<{
  currentPage: number;
  totalPages: number;
  pages: PageItem[];
  count: number;
  total: number;
  label: string;
}>();

const emit = defineEmits<{
  go: [page: PageItem];
}>();
</script>

<template>
    <div class="pagination">
        <span class="pagination-info">
            Mostrando {{ count }} de {{ total }} {{ label }}
        </span>
        <div class="pagination-buttons">
            <button
                @click="emit('go', currentPage - 1)"
                :disabled="currentPage <= 1"
                class="page-nav-btn"
            >
                ‹ Anterior
            </button>
            <button
                v-for="p in pages"
                :key="p"
                :class="['page-btn', { active: p === currentPage }]"
                :disabled="p === '...'"
                @click="emit('go', p)"
            >
                {{ p }}
            </button>
            <button
                @click="emit('go', currentPage + 1)"
                :disabled="currentPage >= totalPages"
                class="page-nav-btn"
            >
                Siguiente ›
            </button>
        </div>
    </div>
</template>

<style scoped>
.pagination {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1.5rem;
}

.pagination-info {
    color: var(--color-text-muted);
    font-size: 0.9rem;
    white-space: nowrap;
    text-align: center;
}

.pagination-buttons {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    justify-content: center;
    flex-wrap: wrap;
}

.page-nav-btn {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
}

.page-nav-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.page-btn {
    min-width: 2.2rem;
    height: 2.2rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.9rem;
    cursor: pointer;
}

.page-btn:disabled {
    border: none;
    background: none;
    cursor: default;
}

.page-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-on-primary);
    font-weight: 600;
}
</style>

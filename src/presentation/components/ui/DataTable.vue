<script setup lang="ts">
defineProps<{
    columns: string[];
    loading: boolean;
    empty: string;
    count: number;
}>();
</script>

<template>
    <div v-if="loading" class="loading">Cargando...</div>
    <div v-else-if="count === 0" class="empty-state">{{ empty }}</div>
    <div v-else class="table-wrapper">
        <table class="data-table">
            <thead>
                <tr>
                    <th v-for="col in columns" :key="col">{{ col }}</th>
                </tr>
            </thead>
            <tbody>
                <slot />
            </tbody>
        </table>
    </div>
</template>

<style scoped>
.table-wrapper {
    overflow-x: auto;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(var(--color-shadow-rgb), 0.1);
}
</style>

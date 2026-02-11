<template>
    <table class="w-full border-collapse">
        <thead>
            <tr class="border-b border-para">
                <th v-for="column in columns" :key="column.name"
                    class="px-6 py-3.5 text-left text-xs font-medium text-para-light tracking-wide">
                    {{ column.name }}
                </th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="(record, index) in records" :key="index"
                class="border-b border-para/50 hover:bg-light/50 transition-colors">
                <td v-for="column in columns" :key="column.name" class="px-6 py-3.5 text-sm text-para-light">
                    {{ record[column.name] }}
                </td>
            </tr>
        </tbody>
    </table>
</template>

<script setup>
const props = defineProps({
    records: {
        type: Array,
        default: () => []
    },
    columns: {
        type: Array,
        default: () => []
    }
});
</script>
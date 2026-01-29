<template>
    <table class="w-full h-fit border-collapse">
        <thead>
            <tr class="border-b border-para">
                <th v-for="column in columns" :key="column"
                    class="px-6 py-4 text-left text-sm font-semibold text-main uppercase tracking-wider bg-light">
                    {{ column }}
                </th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="(record, index) in records" :key="index"
                class="border-b border-para hover:bg-light transition-colors duration-150">
                <td v-for="column in columns" :key="column" class="px-6 py-4 text-sm text-gray-300">
                    {{ record[column] }}
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
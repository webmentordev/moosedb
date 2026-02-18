<template>
    <section>
        <table class="w-full border-collapse">
            <thead>
                <tr class="border-b border-para">
                    <th v-for="column in columns" :key="column.name"
                        class="px-6 py-3.5 text-left text-xs font-medium text-para-light tracking-wide">
                        {{ column.name }}
                    </th>
                </tr>
            </thead>
            <tbody v-if="records.length > 0">
                <tr v-for="(record, index) in records" :key="index"
                    class="border-b border-para/50 hover:bg-light/50 transition-colors">
                    <td v-for="column in columns" :key="column.name" class="px-6 py-3.5 text-sm text-para-light">
                        <template v-if="isDateTimeColumn(column.name)">
                            {{ formatDateTime(record[column.name]) }}
                        </template>
                        <template v-else-if="shouldTruncate(column.name, record[column.name])">
                            <span v-if="!isExpanded(index, column.name)">
                                {{ truncateText(record[column.name]) }}
                                <button @click="toggleExpand(index, column.name)"
                                    class="ml-1 text-blue-600 hover:text-blue-800 font-medium">
                                    read more
                                </button>
                            </span>
                            <span v-else>
                                {{ record[column.name] }}
                                <button @click="toggleExpand(index, column.name)"
                                    class="ml-1 text-blue-600 hover:text-blue-800 font-medium">
                                    show less
                                </button>
                            </span>
                        </template>
                        <template v-else>
                            {{ record[column.name] }}
                        </template>
                    </td>
                </tr>
            </tbody>
        </table>
        <p v-if="records.length == 0" class="py-2 px-4 my-3 bg-dark w-fit rounded-2xl m-auto">Collection is empty!</p>
    </section>
</template>

<script setup>
import { ref } from 'vue';

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

const expandedCells = ref({});

const isDateTimeColumn = (columnName) => {
    return columnName === 'created_at' || columnName === 'updated_at';
};

const formatDateTime = (dateString) => {
    if (!dateString) return '';
    const date = new Date(dateString.replace(' ', 'T') + 'Z');
    return new Intl.DateTimeFormat('en-US', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    }).format(date);
};

const shouldTruncate = (columnName, value) => {
    const excludedColumns = ['id', 'created_at', 'updated_at'];
    if (excludedColumns.includes(columnName)) return false;
    return value && String(value).length > 20;
};

const truncateText = (text) => {
    return String(text).substring(0, 20) + '...';
};

const isExpanded = (rowIndex, columnName) => {
    return expandedCells.value[`${rowIndex}-${columnName}`] || false;
};

const toggleExpand = (rowIndex, columnName) => {
    const key = `${rowIndex}-${columnName}`;
    expandedCells.value[key] = !expandedCells.value[key];
};
</script>
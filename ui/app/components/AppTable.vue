<template>
    <table class="w-full h-fit border-collapse">
        <thead>
            <tr class="border-b border-para">
                <th 
                    v-for="key in keys" 
                    :key="key" 
                    class="px-6 py-4 text-left text-sm font-semibold text-main uppercase tracking-wider bg-light"
                >
                    {{ key }}
                </th>
            </tr>
        </thead>
        <tbody>
            <tr 
                v-for="(record, index) in records" 
                :key="index"
                class="border-b border-para hover:bg-light transition-colors duration-150"
            >
                <td 
                    v-for="key in keys" 
                    :key="key" 
                    class="px-6 py-4 text-sm text-gray-300"
                >
                    {{ record[key] }}
                </td>
            </tr>
        </tbody>
    </table>
</template>

<script setup>
    const props = defineProps({
        records: {
            type: Array,
            default: () => [
                { name: 'John', age: 30, city: 'New York' },
                { name: 'Jane', age: 25, city: 'London' },
                { name: 'Bob', age: 35, city: 'Paris' }
            ]
        }
    });
    const keys = props.records.length > 0 ? Object.keys(props.records[0]) : [];
</script>
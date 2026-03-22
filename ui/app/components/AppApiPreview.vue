<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full" v-if="modelValue">
        <div class="w-full h-full flex justify-end" @click.self="$emit('update:modelValue', false)">
            <div class="w-200 h-full border-l border-white/10 bg-light flex flex-col overflow-hidden">

                <div class="flex items-center justify-between px-6 py-4 border-b border-white/10 shrink-0">
                    <div class="flex items-center gap-3">
                        <span class="text-lg font-semibold">API Preview</span>
                        <code class="text-xs bg-dark/10 px-2 py-1 rounded-lg">{{ collection_name }}</code>
                    </div>
                    <button @click="$emit('update:modelValue', false)"
                        class="text-gray-400 hover:text-gray-200 text-xl leading-none">✕</button>
                </div>

                <div class="flex flex-1 overflow-hidden">
                    <div class="w-56 shrink-0 border-r border-white/10 overflow-y-auto py-3">
                        <div v-for="endpoint in endpoints" :key="endpoint.id" @click="active_id = endpoint.id"
                            :class="['flex items-center gap-2 px-4 py-2.5 cursor-pointer text-sm transition-colors', active_id === endpoint.id ? 'bg-blue-600/15 text-blue-400' : 'hover:bg-dark/10 text-gray-300']">
                            <span :class="['text-xs font-bold w-10 shrink-0', methodColor(endpoint.method)]">
                                {{ endpoint.method }}
                            </span>
                            <span class="truncate">{{ endpoint.label }}</span>
                        </div>
                    </div>

                    <div class="flex-1 overflow-y-auto px-6 py-5" v-if="active">
                        <div class="flex items-center gap-3 mb-1">
                            <span :class="['text-xs font-bold px-2 py-1 rounded', methodBadge(active.method)]">
                                {{ active.method }}
                            </span>
                            <code class="text-sm text-gray-300">{{ active.url }}</code>
                        </div>
                        <p class="text-sm text-gray-400 mt-2 mb-5">{{ active.description }}</p>

                        <template v-if="active.query_params">
                            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">Query params
                            </h3>
                            <div class="border border-white/10 rounded-xl overflow-hidden mb-5">
                                <div v-for="(param, i) in active.query_params" :key="param.name"
                                    :class="['flex items-start gap-3 px-4 py-3 text-sm', i !== active.query_params.length - 1 ? 'border-b border-white/10' : '']">
                                    <code class="text-blue-400 w-24 shrink-0">{{ param.name }}</code>
                                    <span class="text-gray-500 w-16 shrink-0 text-xs mt-0.5">{{ param.type }}</span>
                                    <span class="text-gray-300 text-xs">{{ param.description }}</span>
                                </div>
                            </div>
                        </template>

                        <template v-if="active.body">
                            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">Request body
                            </h3>
                            <div class="relative mb-5">
                                <pre
                                    class="bg-dark/20 border border-white/10 rounded-xl px-4 py-4 text-xs text-gray-200 overflow-x-auto leading-relaxed">{{ active.body }}</pre>
                                <button @click="copy(active.body, 'body')"
                                    class="absolute top-2.5 right-3 text-xs text-gray-500 hover:text-gray-300 transition-colors">
                                    {{ copied === 'body' ? '✓ copied' : 'copy' }}
                                </button>
                            </div>
                        </template>

                        <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 mb-2">Response</h3>
                        <div class="relative">
                            <pre
                                class="bg-dark/20 border border-white/10 rounded-xl px-4 py-4 text-xs text-gray-200 overflow-x-auto leading-relaxed">
        {{ active.response }}</pre>
                            <button @click="copy(active.response, 'response')"
                                class="absolute top-2.5 right-3 text-xs text-gray-500 hover:text-gray-300 transition-colors">
                                {{ copied === 'response' ? '✓ copied' : 'copy' }}
                            </button>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    </div>
</template>

<script setup>
const props = defineProps({
    modelValue: { type: Boolean, default: false },
    collection_id: { type: String, required: true },
    collection_name: { type: String, default: '' },
    columns: { type: Array, default: () => [] }
});

defineEmits(['update:modelValue']);

const active_id = ref('list');
const copied = ref(null);

const dataColumns = computed(() =>
    props.columns.filter(c => !['id', 'created_at', 'updated_at'].includes(c.name))
);

function exampleValue(col) {
    switch (col.field_type) {
        case 'VARCHAR': return `"example_${col.name}"`;
        case 'TEXT': return `"<p>Example content</p>"`;
        case 'INTEGER': return '1';
        case 'DECIMAL': return '1.5';
        case 'BOOLEAN': return 'true';
        case 'DATETIME':
        case 'TIMESTAMP': return '"2025-01-01T00:00:00"';
        case 'FILE': return `[{ "filename": "file.jpg", "mime_type": "image/jpeg", "data": "<base64>" }]`;
        default: return '"value"';
    }
}

function exampleRecord(includeSystem = true) {
    const fields = dataColumns.value.map(col => `    "${col.name}": ${exampleValue(col)}`);
    if (includeSystem) {
        fields.unshift(`    "id": "moo${randomAlpha(12)}"`);
        fields.push(`    "created_at": "2025-01-01 00:00:00"`);
        fields.push(`    "updated_at": "2025-01-01 00:00:00"`);
    }
    return `{\n${fields.join(',\n')}\n}`;
}

function randomAlpha(n) {
    const chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    return Array.from({ length: n }, () => chars[Math.floor(Math.random() * chars.length)]).join('');
}

const endpoints = computed(() => [
    {
        id: 'list',
        method: 'GET',
        label: 'List records',
        url: `/api/records/${props.collection_id}`,
        description: `Returns a paginated list of all records in the ${props.collection_name || 'collection'}.`,
        query_params: [
            { name: 'page', type: 'number', description: 'Page number, starting from 1. Defaults to 1.' },
            { name: 'items', type: 'number', description: 'Records per page. Defaults to 100, max 10000.' }
        ],
        body: null,
        response: JSON.stringify({
            success: true,
            message: `Retrieved 2 of 2 total records from '${props.collection_name}' (page 1/1)`,
            records: [
                JSON.parse(exampleRecord(true)),
                JSON.parse(exampleRecord(true))
            ],
            pagination: {
                current_page: 1,
                items_per_page: 100,
                total_records: 2,
                total_pages: 1,
                records_shown: 2,
                has_next_page: false,
                has_prev_page: false,
                next_page: null,
                prev_page: null
            }
        }, null, 2)
    },
    {
        id: 'single',
        method: 'GET',
        label: 'Get one record',
        url: `/api/records/${props.collection_id}/{record_id}`,
        description: 'Returns a single record by its ID.',
        query_params: null,
        body: null,
        response: JSON.stringify({
            success: true,
            message: `Record retrieved from '${props.collection_name}'`,
            record: JSON.parse(exampleRecord(true))
        }, null, 2)
    }
]);

const active = computed(() => endpoints.value.find(e => e.id === active_id.value) ?? null);

function methodColor(method) {
    return method === 'GET' ? 'text-green-400' : 'text-blue-400';
}

function methodBadge(method) {
    return method === 'GET'
        ? 'bg-green-500/15 text-green-400'
        : 'bg-blue-500/15 text-blue-400';
}

async function copy(text, key) {
    try {
        await navigator.clipboard.writeText(text);
        copied.value = key;
        setTimeout(() => { copied.value = null; }, 1800);
    } catch { }
}
</script>
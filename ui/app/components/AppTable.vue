<template>
    <section>
        <div v-if="selectedRows.size > 0" class="flex items-center gap-3 px-4 py-2.5 mb-2 bg-dark rounded-xl">
            <span class="text-xs text-para-light">{{ selectedRows.size }} selected</span>
            <button @click="deleteSelected"
                class="ml-auto flex items-center gap-1.5 px-3 py-1.5 text-xs text-red-400 border border-red-400/40 rounded-lg hover:bg-red-400/10 transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6l-1 14H6L5 6" />
                    <path d="M10 11v6M14 11v6" />
                    <path d="M9 6V4h6v2" />
                </svg>
                Delete selected
            </button>
        </div>

        <table class="w-full border-collapse">
            <thead>
                <tr class="border-b border-para">
                    <th class="px-4 py-3.5 w-10">
                        <input type="checkbox" :checked="isAllSelected" :indeterminate="isIndeterminate"
                            @change="toggleSelectAll" class="w-3.5 h-3.5 rounded accent-para cursor-pointer" />
                    </th>
                    <th v-for="column in columns" :key="column.name"
                        class="px-6 py-3.5 text-left text-xs font-medium text-para-light tracking-wide">
                        {{ column.name }}
                    </th>
                </tr>
            </thead>
            <tbody v-if="records.length > 0">
                <tr v-for="(record, index) in records" :key="index"
                    :class="['border-b border-para/50 hover:bg-light/50 transition-colors', selectedRows.has(index) ? 'bg-light/30' : '']">
                    <td class="px-4 py-3.5 w-10">
                        <input type="checkbox" :checked="selectedRows.has(index)" @change="toggleRow(index)"
                            class="w-3.5 h-3.5 rounded accent-para cursor-pointer" />
                    </td>
                    <td v-for="column in columns" :key="column.name" class="px-6 py-3.5 text-sm text-para-light">
                        <template v-if="isDateTimeColumn(column.name)">
                            {{ formatDateTime(record[column.name]) }}
                        </template>
                        <template v-else-if="column.field_type === 'FILE'">
                            <button v-if="record[column.name]" type="button"
                                @click="openFilePreview(record[column.name])"
                                class="inline-flex items-center text-para-light hover:text-para transition-colors">
                                <img :src="getFileCount(record[column.name]) > 1
                                    ? 'https://api.iconify.design/mdi:file-multiple-outline.svg?color=%23dbdbdb'
                                    : 'https://api.iconify.design/line-md:file.svg?color=%23dbdbdb'"
                                    class="w-5 h-5 opacity-70 hover:opacity-100 transition-opacity" alt="File" />
                            </button>
                            <span v-else class="text-para-light/30">—</span>
                        </template>
                        <template v-else-if="column.field_type === 'TEXT'">
                            <span
                                v-if="!isExpanded(index, column.name) && shouldTruncate(column.name, record[column.name])"
                                v-html="truncateText(record[column.name])" @click="toggleExpand(index, column.name)"
                                class="cursor-pointer hover:text-para">
                            </span>
                            <span v-else v-html="record[column.name]"
                                @click="shouldTruncate(column.name, record[column.name]) && toggleExpand(index, column.name)"
                                :class="{ 'cursor-pointer hover:text-para': shouldTruncate(column.name, record[column.name]) }">
                            </span>
                        </template>
                        <template v-else-if="shouldTruncate(column.name, record[column.name])">
                            <span @click="toggleExpand(index, column.name)" class="cursor-pointer hover:text-para">
                                {{ isExpanded(index, column.name) ? record[column.name] :
                                    truncateText(record[column.name]) }}
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

    <div v-if="preview_files.length"
        class="fixed inset-0 z-60 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        @click.self="preview_files = []">
        <div class="bg-light border border-white/10 rounded-2xl p-6 w-full max-w-lg shadow-xl">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-lg font-semibold">Files</h2>
                <button @click="preview_files = []"
                    class="text-gray-400 hover:text-white text-xl leading-none">✕</button>
            </div>
            <div class="grid grid-cols-3 gap-3">
                <a v-for="(file, index) in preview_files" :key="index" :href="file.url" target="_blank"
                    rel="noopener noreferrer"
                    class="flex flex-col items-center gap-2 p-3 rounded-xl bg-dark/10 border border-white/5 hover:bg-dark/30 transition-colors">
                    <div class="w-12 h-12 flex items-center justify-center rounded-lg bg-dark/20">
                        <span class="text-2xl">{{ fileIcon(file.name) }}</span>
                    </div>
                    <span class="text-xs text-gray-300 text-center break-all leading-tight line-clamp-2">{{ file.name
                    }}</span>
                </a>
            </div>
        </div>
    </div>
</template>

<script setup>
const { authFetch } = useAuthFetch();

const props = defineProps({
    records: {
        type: Array,
        default: () => []
    },
    columns: {
        type: Array,
        default: () => []
    },
    collectionId: {
        type: String,
        required: true
    }
});

const emit = defineEmits(['fetch-data']);

const expandedCells = ref({});
const selectedRows = ref(new Set());
const preview_files = ref([]);

const isAllSelected = computed(() =>
    props.records.length > 0 && selectedRows.value.size === props.records.length
);

const isIndeterminate = computed(() =>
    selectedRows.value.size > 0 && selectedRows.value.size < props.records.length
);

const toggleSelectAll = () => {
    if (isAllSelected.value) {
        selectedRows.value = new Set();
    } else {
        selectedRows.value = new Set(props.records.map((_, i) => i));
    }
};

const toggleRow = (index) => {
    const updated = new Set(selectedRows.value);
    updated.has(index) ? updated.delete(index) : updated.add(index);
    selectedRows.value = updated;
};

const deleteSelected = async () => {
    const ids = [...selectedRows.value].map(i => props.records[i].id);

    try {
        const data = await authFetch('/admin/api/delete-collection-records', {
            method: 'POST',
            body: {
                collection_id: props.collectionId,
                record_ids: ids
            }
        });

        if (data.success) {
            selectedRows.value = new Set();
            emit('fetch-data');
        } else {
            console.error('Delete failed:', data.message);
        }
    } catch (err) {
        console.error('Delete request failed:', err);
    }
};

function getFileCount(raw) {
    try {
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed.length : 1;
    } catch {
        return 1;
    }
}

function openFilePreview(raw) {
    let paths = [];
    try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed)) paths = parsed;
        else paths = [raw];
    } catch {
        paths = [raw];
    }
    preview_files.value = paths.map(p => ({
        name: p.split('/').pop(),
        url: window.location.origin + '/' + p
    }));
}

function fileIcon(filename) {
    const ext = filename.split('.').pop().toLowerCase();
    if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'avif'].includes(ext)) return '🖼️';
    if (['mp4', 'mov', 'avi', 'webm', 'mkv'].includes(ext)) return '🎬';
    if (['mp3', 'wav', 'ogg', 'flac', 'aac'].includes(ext)) return '🎵';
    if (['pdf'].includes(ext)) return '📄';
    if (['doc', 'docx'].includes(ext)) return '📝';
    if (['xls', 'xlsx', 'csv'].includes(ext)) return '📊';
    if (['zip', 'rar', 'tar', 'gz', '7z'].includes(ext)) return '🗜️';
    if (['js', 'ts', 'py', 'rs', 'go', 'html', 'css', 'json'].includes(ext)) return '💻';
    return '📁';
}

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
    return String(text).substring(0, 15) + '...';
};

const isExpanded = (rowIndex, columnName) => {
    return expandedCells.value[`${rowIndex}-${columnName}`] || false;
};

const toggleExpand = (rowIndex, columnName) => {
    const key = `${rowIndex}-${columnName}`;
    expandedCells.value[key] = !expandedCells.value[key];
};
</script>
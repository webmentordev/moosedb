<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full" v-if="record_show">
        <div class="w-full h-full flex justify-end" @click.self="closeModal">
            <div class="w-137.5 h-full border-l border-white/10 bg-light p-6 overflow-y-auto">
                <h1 class="text-2xl">Create new record</h1>
                <p>Record ID and creation time will be auto generated.</p>
                <div class="mt-3">
                    <div v-for="column in columns.filter(col => col.name !== 'id' && col.name !== 'created_at' && col.name !== 'updated_at')"
                        :key="column.name" class="flex flex-col mb-4">
                        <AppLabel :text="column.name" />

                        <AppInput v-if="column.field_type === 'VARCHAR'" v-model="form_data[column.name]" type="text"
                            :placeholder="`Enter ${column.name}`" />

                        <ClientOnly v-else-if="column.field_type === 'TEXT'">
                            <Quill v-model:content="form_data[column.name]" />
                        </ClientOnly>

                        <AppInput v-else-if="column.field_type === 'INTEGER'" v-model.number="form_data[column.name]"
                            type="number" :placeholder="`Enter ${column.name}`" />

                        <AppInput v-else-if="column.field_type === 'DECIMAL'" v-model.number="form_data[column.name]"
                            type="number" step="0.01" :placeholder="`Enter ${column.name}`" />

                        <label v-else-if="column.field_type === 'BOOLEAN'" class="flex items-center gap-2 mt-2">
                            <input type="checkbox" v-model="form_data[column.name]" class="w-4 h-4" />
                            <span class="text-sm">{{ column.name }}</span>
                        </label>

                        <AppInput v-else-if="column.field_type === 'DATETIME' || column.field_type === 'TIMESTAMP'"
                            v-model="form_data[column.name]" type="datetime-local"
                            :placeholder="`Enter ${column.name}`" />

                        <AppInput v-else v-model="form_data[column.name]" type="text"
                            :placeholder="`Enter ${column.name}`" />
                    </div>

                    <button @click="createRecord" :disabled="loading"
                        class="w-full mt-6 py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-xl font-semibold">
                        {{ loading ? 'Creating...' : 'Create Record' }}
                    </button>

                    <div v-if="message.text"
                        :class="['mt-4 p-3 rounded-lg', message.success ? 'bg-green-500/20 text-green-700' : 'bg-red-500/20 text-red-700']">
                        {{ message.text }}
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
const props = defineProps({
    columns: {
        type: Array,
        default: () => []
    },
    record_show: {
        type: Boolean,
        default: false
    },
    collection_id: {
        type: String,
        required: true
    }
});

const emit = defineEmits(['update:record_show', 'fetch-data']);
function closeModal() {
    emit('update:record_show', false);
}

const form_data = ref({});
const loading = ref(false);
const message = ref({
    text: "",
    success: false
});

const { authFetch } = useAuthFetch();

watch(() => props.columns, (newColumns) => {
    if (newColumns && newColumns.length > 0) {
        const initialData = {};
        newColumns.forEach(column => {
            if (column.field_type === 'BOOLEAN') {
                initialData[column.name] = false;
            } else if (column.field_type === 'TEXT') {
                initialData[column.name] = '';
            } else {
                initialData[column.name] = null;
            }
        });
        form_data.value = initialData;
    }
}, { immediate: true });

async function createRecord() {
    loading.value = true;
    message.value = { text: "", success: false };

    try {
        const response = await authFetch('/admin/api/create-record', {
            method: 'POST',
            body: JSON.stringify({
                collection_id: props.collection_id,
                data: form_data.value
            })
        });

        if (response.success) {
            message.value = {
                text: response.message,
                success: true
            };
            emit('fetch-data');
            setTimeout(() => {
                closeModal();
            }, 1500);
        } else {
            message.value = {
                text: response.message,
                success: false
            };
        }
    } catch (error) {
        message.value = {
            text: error.message || 'Failed to create record',
            success: false
        };
    } finally {
        loading.value = false;
    }
}
</script>
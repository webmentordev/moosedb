<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full" v-if="show">
        <div class="w-full h-full flex justify-end" @click.self="closeModal">
            <div class="w-137.5 h-full border-l border-white/10 bg-light p-6 overflow-y-auto">
                <h1 class="text-2xl">Update collection</h1>
                <div class="mt-3">
                    <div class="flex flex-col">
                        <AppLabel text="Collection name" />
                        <AppInput v-model="name" type="text" placeholder="e.g Books" />
                        <AlertsAlertError v-if="errors.name" :error="errors.name" />
                    </div>

                    <button
                        class="flex items-center justify-center py-2 px-3 border rounded-xl mt-4 bg-main w-full"
                        @click="dropdown = !dropdown">
                        + add new fields
                    </button>

                    <div class="p-3 bg-dark rounded-lg mt-4 grid grid-cols-3 gap-3" v-show="dropdown">
                        <button
                            @click="addField(column)"
                            v-for="column in fieldTypes"
                            :key="column.id"
                            class="bg-light rounded-lg py-2 px-3 border border-white/10">
                            {{ column.name }}
                        </button>
                    </div>

                    <div v-if="fields.length > 0" class="mt-4 space-y-3">
                        <div
                            v-for="(field, index) in fields"
                            :key="index"
                            class="p-4 bg-dark/10 rounded-lg border border-white/10">
                            <div class="flex justify-between items-start mb-3">
                                <div class="flex items-center gap-2">
                                    <h3 class="font-semibold">{{ getFieldTypeName(field.type) }}</h3>
                                    <span v-if="field._existing" class="text-xs py-0.5 px-2 bg-blue-500/20 text-blue-400 rounded-full">existing</span>
                                    <span v-else class="text-xs py-0.5 px-2 bg-green-500/20 text-green-400 rounded-full">new</span>
                                </div>
                                <button @click="removeField(index)" class="text-red-500 hover:text-red-700">✕</button>
                            </div>

                            <div class="space-y-3">
                                <div class="flex flex-col">
                                    <AppLabel text="Field name" />
                                    <AppInput
                                        v-model="field.title"
                                        type="text"
                                        placeholder="e.g title, author, isbn"
                                        :disabled="field._existing" />
                                </div>

                                <div class="flex items-center gap-4">
                                    <label class="flex items-center gap-2">
                                        <input type="checkbox" v-model="field.unique" />
                                        <span class="text-sm">Unique</span>
                                    </label>
                                    <label class="flex items-center gap-2">
                                        <input type="checkbox" v-model="field.nullable" />
                                        <span class="text-sm">Nullable</span>
                                    </label>
                                </div>

                                <div v-if="field.type === 'VARCHAR'" class="grid grid-cols-2 gap-3">
                                    <div class="flex flex-col">
                                        <AppLabel text="Min length" />
                                        <AppInput v-model.number="field.min" type="number" placeholder="Optional" />
                                    </div>
                                    <div class="flex flex-col">
                                        <AppLabel text="Max length" />
                                        <AppInput v-model.number="field.max" type="number" placeholder="Optional" />
                                    </div>
                                </div>

                                <div v-if="field.type === 'INTEGER' || field.type === 'DECIMAL'" class="grid grid-cols-2 gap-3">
                                    <div class="flex flex-col">
                                        <AppLabel text="Min value" />
                                        <AppInput v-model.number="field.min" type="number" placeholder="Optional" />
                                    </div>
                                    <div class="flex flex-col">
                                        <AppLabel text="Max value" />
                                        <AppInput v-model.number="field.max" type="number" placeholder="Optional" />
                                    </div>
                                </div>

                                <div v-if="field.type === 'FILE'" class="flex flex-col">
                                    <AppLabel text="Allowed extensions" />
                                    <AppInput
                                        v-model="field.allowedExtensions"
                                        type="text"
                                        placeholder="e.g jpg,png,pdf (leave empty for all)" />
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="full-separator"></div>
                    <p class="mt-2 text-sm">
                        <code>id</code>, <code>created_at</code> and <code>updated_at</code> are system fields and cannot be modified.
                    </p>
                    <p class="mt-1 text-sm text-yellow-500/80">
                        Removing a field is permanent. File fields will have their uploaded files deleted.
                    </p>

                    <button
                        @click="updateCollection"
                        :disabled="loading"
                        class="w-full mt-6 py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-xl font-semibold">
                        {{ loading ? 'Updating...' : 'Update Collection' }}
                    </button>

                    <div
                        v-if="message.text"
                        :class="['mt-4 p-3 rounded-lg', message.success ? 'bg-green-500/20 text-green-700' : 'bg-red-500/20 text-red-700']">
                        {{ message.text }}
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
const SYSTEM_FIELDS = ['id', 'created_at', 'updated_at'];

const props = defineProps({
    show: {
        type: Boolean,
        default: false
    },
    collectionId: {
        type: String,
        required: true
    },
    collectionName: {
        type: String,
        required: true
    },
    columns: {
        type: Array,
        default: () => []
    }
});

const emit = defineEmits(['update:show', 'fetch-data']);

function closeModal() {
    emit('update:show', false);
}

const { authFetch } = useAuthFetch();
const dropdown = ref(false);
const loading = ref(false);
const name = ref('');
const fields = ref([]);
const errors = ref({ name: null, count: 0 });
const message = ref({ text: '', success: false });

const fieldTypes = [
    { id: 1, name: 'Plain text', type: 'VARCHAR' },
    { id: 2, name: 'Rich text', type: 'TEXT' },
    { id: 3, name: 'Boolean', type: 'BOOLEAN' },
    { id: 4, name: 'Integer', type: 'INTEGER' },
    { id: 5, name: 'Decimal', type: 'DECIMAL' },
    { id: 6, name: 'DateTime', type: 'DATETIME' },
    { id: 7, name: 'File upload', type: 'FILE' },
];

watch(() => props.show, (val) => {
    if (val) {
        initForm();
    }
});

function initForm() {
    name.value = props.collectionName;
    message.value = { text: '', success: false };
    errors.value = { name: null, count: 0 };
    dropdown.value = false;

    fields.value = props.columns
        .filter(col => !SYSTEM_FIELDS.includes(col.name))
        .map(col => ({
            title: col.name,
            type: col.field_type,
            unique: false,
            nullable: false,
            min: null,
            max: null,
            allowedExtensions: '',
            _existing: true
        }));
}

function addField(column) {
    fields.value.push({
        title: '',
        type: column.type,
        unique: false,
        nullable: false,
        min: null,
        max: null,
        allowedExtensions: '',
        _existing: false
    });
    dropdown.value = false;
}

function removeField(index) {
    fields.value.splice(index, 1);
}

function getFieldTypeName(type) {
    const match = fieldTypes.find(c => c.type === type);
    return match ? match.name : type;
}

function validateForm() {
    errors.value = { name: null, count: 0 };

    if (!name.value.trim()) {
        errors.value.name = 'Collection name is required';
        errors.value.count++;
    }

    if (fields.value.length === 0) {
        message.value = { text: 'Please add at least one field', success: false };
        errors.value.count++;
    }

    for (const field of fields.value) {
        if (!field.title.trim()) {
            message.value = { text: 'All fields must have a name', success: false };
            errors.value.count++;
            break;
        }
    }

    return errors.value.count === 0;
}

async function updateCollection() {
    message.value = { text: '', success: false };

    if (!validateForm()) return;

    loading.value = true;

    try {
        const payload = {
            collection_id: props.collectionId,
            collection_name: name.value.trim(),
            fields: fields.value.map(field => ({
                title: field.title.trim(),
                type: field.type,
                unique: field.unique,
                nullable: field.nullable,
                ...(field.min !== null && field.min !== undefined && field.min !== '' && { min: field.min }),
                ...(field.max !== null && field.max !== undefined && field.max !== '' && { max: field.max }),
                ...(field.type === 'FILE' && field.allowedExtensions?.trim() !== '' && { allowed_extensions: field.allowedExtensions.trim() })
            }))
        };

        const response = await authFetch('/admin/api/update-collection', {
            method: 'POST',
            body: JSON.stringify(payload)
        });

        if (response.success) {
            message.value = { text: response.message, success: true };
            emit('fetch-data', true);
        } else {
            message.value = { text: response.message, success: false };
        }
    } catch (error) {
        message.value = { text: `Error: ${error?.data?.message ?? error}`, success: false };
    } finally {
        loading.value = false;
    }
}
</script>
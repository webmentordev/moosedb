<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full" v-if="show">
        <div class="w-full h-full flex justify-end" @click.self="closeModal">
            <div class="w-137.5 h-full border-l border-white/10 bg-light p-6 overflow-y-auto">
                <h1 class="text-2xl">New collection</h1>
                <div class="mt-3">
                    <div class="flex flex-col">
                        <AppLabel text="Collection name"/>
                        <AppInput v-model="name" type="text" placeholder='e.g Books' />
                        <AlertsAlertError v-if="errors.name" :error="errors.name" />
                    </div>

                    <button 
                        class="flex items-center justify-center py-2 px-3 border rounded-xl mt-4 bg-main w-full" 
                        @click="dropdown = !dropdown"
                    >
                        + add new fields
                    </button>
                    
                    <div class="p-3 bg-dark rounded-lg mt-4 grid grid-cols-3 gap-3" v-show="dropdown">
                        <button 
                            @click="addField(column)" 
                            v-for="column in columns" 
                            :key="column.id"
                            class="bg-light rounded-lg py-2 px-3 border border-white/10"
                        >
                            {{ column.name }}
                        </button>
                    </div>
                    <div v-if="fields.length > 0" class="mt-4 space-y-3">
                        <div 
                            v-for="(field, index) in fields" 
                            :key="index"
                            class="p-4 bg-dark/10 rounded-lg border border-white/10"
                        >
                            <div class="flex justify-between items-start mb-3">
                                <h3 class="font-semibold">{{ getFieldTypeName(field.type) }}</h3>
                                <button 
                                    @click="removeField(index)"
                                    class="text-red-500 hover:text-red-700"
                                >
                                    ✕
                                </button>
                            </div>

                            <div class="space-y-3">
                                <div class="flex flex-col">
                                    <AppLabel text="Field name"/>
                                    <AppInput 
                                        v-model="field.title" 
                                        type="text" 
                                        placeholder='e.g title, author, isbn' 
                                    />
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
                                <div v-if="field.type === 'VARCHAR' || field.type === 'TEXT'" class="grid grid-cols-2 gap-3">
                                    <div class="flex flex-col">
                                        <AppLabel text="Min length"/>
                                        <AppInput 
                                            v-model.number="field.min" 
                                            type="number" 
                                            placeholder='Optional' 
                                        />
                                    </div>
                                    <div class="flex flex-col">
                                        <AppLabel text="Max length"/>
                                        <AppInput 
                                            v-model.number="field.max" 
                                            type="number" 
                                            placeholder='Optional' 
                                        />
                                    </div>
                                </div>
                                <div v-if="field.type === 'INTEGER' || field.type === 'DECIMAL'" class="grid grid-cols-2 gap-3">
                                    <div class="flex flex-col">
                                        <AppLabel text="Min value"/>
                                        <AppInput 
                                            v-model.number="field.min" 
                                            type="number" 
                                            placeholder='Optional' 
                                        />
                                    </div>
                                    <div class="flex flex-col">
                                        <AppLabel text="Max value"/>
                                        <AppInput 
                                            v-model.number="field.max" 
                                            type="number" 
                                            placeholder='Optional' 
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="full-separator"></div>
                    <p class="mt-2 text-sm">
                        <code>id (auto increment)</code>, <code>created_at</code> and <code>updated_at</code> will be auto generated.
                    </p>
                    <button 
                        @click="createCollection"
                        :disabled="loading"
                        class="w-full mt-6 py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-xl font-semibold"
                    >
                        {{ loading ? 'Creating...' : 'Create Collection' }}
                    </button>
                    <div v-if="message.text" :class="['mt-4 p-3 rounded-lg', message.success ? 'bg-green-500/20 text-green-700' : 'bg-red-500/20 text-red-700']">
                        {{ message.text }}
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
    const props = defineProps({
        show: {
            type: Boolean,
            default: false
        }
    });
    const emit = defineEmits(['update:show']);
    function closeModal() {
        emit('update:show', false);
    }


    const { authFetch } = useAuthFetch();
    const name = ref("");
    const dropdown = ref(false);
    const loading = ref(false);
    const errors = ref({
        name: null,
        count: 0
    });
    const message = ref({
        text: "",
        success: false
    });

    const columns = ref([
        {
            id: 1,
            name: "Plain text",
            type: "VARCHAR",
        },
        {
            id: 2,
            name: "Rich text",
            type: "TEXT",
        },
        {
            id: 3,
            name: "Boolean",
            type: "BOOLEAN",
        },
        {
            id: 4,
            name: "Integer",
            type: "INTEGER",
        },
        {
            id: 5,
            name: "Decimal",
            type: "DECIMAL",
        },
        {
            id: 6,
            name: "DateTime",
            type: "DATETIME",
        }
    ]);

    const fields = ref([]);

    function addField(column) {
        fields.value.push({
            title: "",
            type: column.type,
            unique: false,
            nullable: false,
            min: null,
            max: null
        });
        dropdown.value = false;
    }

    function removeField(index) {
        fields.value.splice(index, 1);
    }

    function getFieldTypeName(type) {
        const column = columns.value.find(c => c.type === type);
        return column ? column.name : type;
    }

    function validateForm() {
        errors.value = {
            name: null,
            count: 0
        };

        if (!name.value.trim()) {
            errors.value.name = "Collection name is required";
            errors.value.count++;
        }

        if (fields.value.length === 0) {
            message.value = {
                text: "Please add at least one field",
                success: false
            };
            errors.value.count++;
        }
        for (const field of fields.value) {
            if (!field.title.trim()) {
                message.value = {
                    text: "All fields must have a name",
                    success: false
                };
                errors.value.count++;
                break;
            }
        }

        return errors.value.count === 0;
    }

    async function createCollection() {
        message.value = { text: "", success: false };

        if (!validateForm()) {
            return;
        }

        loading.value = true;

        try {
            const payload = {
                collection: name.value.trim(),
                fields: fields.value.map(field => ({
                    title: field.title.trim(),
                    type: field.type,
                    unique: field.unique,
                    nullable: field.nullable,
                    ...(field.min !== null && field.min !== undefined && field.min !== "" && { min: field.min }),
                    ...(field.max !== null && field.max !== undefined && field.max !== "" && { max: field.max })
                }))
            };

            const response = await authFetch('/admin/api/create-collection', {
                method: 'POST',
                body: JSON.stringify(payload)
            });

            const data = response;

            if (data.success) {
                message.value = {
                    text: data.message,
                    success: true
                };
                setTimeout(() => {
                    name.value = "";
                    fields.value = [];
                }, 2000);
            } else {
                message.value = {
                    text: data.message,
                    success: false
                };
            }
        } catch (error) {
            message.value = {
                text: `Error: ${error.data.message}`,
                success: false
            };
        } finally {
            loading.value = false;
        }
    }
</script>
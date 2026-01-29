<template>
    <div class="w-full h-full flex">
        <AppList :collections="collections" :active="active_tb" @refetch-api="fetch_collections" />
        <AlertsSuccess v-if="success_message" :message="success_message" />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <div class="p-3" v-if="collections.length">
            <div class="flex items-center">
                <h1 class="text-lg">Collections</h1>
                <span class="mx-3">/ <strong class="ml-2">{{ active_tb_name }}</strong></span>
                <button @click="show = true"
                    class="w-7.5 h-7.5 flex items-center justify-center bg-red-600/5 rounded-full"><img
                        src="https://api.iconify.design/ic:baseline-delete-forever.svg?color=%23e01b24"
                        width="20"></button>
            </div>

            <!-- <ClientOnly>
                <Quill v-model:content="body" />
            </ClientOnly>
            
            {{ body }} -->

            <AppTable :records="records" :columns="columns" />


            <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-40 w-full h-full" v-if="show">
                <div class="w-full h-full flex justify-end" @click.self="show = false">
                    <div class="w-137.5 h-full border-l border-white/10 bg-light p-6 overflow-y-auto">
                        <h1 class="text-2xl">Delete: <code v-if="active_tb_name">{{ active_tb_name }}</code></h1>
                        <p>This action is irreversible. Deleted collections cannot be recovered.</p>
                        <button @click="delete_collection" :disabled="processing"
                            class="w-full mt-6 py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-xl font-semibold">
                            {{ processing ? 'deleting...' : 'Confirm, delete the collection' }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
        <div class="p-3" v-else>
            <p>No collections data exist at the moment.</p>
        </div>
    </div>
</template>

<script setup>
// definePageMeta({
//     middleware: 'auth'
// });
const errors = ref({
    message: null,
    count: 0
});

const body = ref("");
const records = ref([]);
const columns = ref([]);

const { authFetch } = useAuthFetch();
const show = ref(false);
const success_message = ref(null);
const processing = ref(false);
const collections = ref([]);
const route = useRoute();
const active_tb = ref(null);
const active_tb_name = ref(null);
if (route.query.tb) {
    active_tb.value = route.query.tb;
    await get_collection(active_tb.value)
}
watch(() => route.query.tb, async (newTb) => {
    active_tb.value = newTb || null;
    if (active_tb.value) {
        await get_collection(active_tb.value)
    }
    const value = collections.value.find(item => item.table_id == active_tb.value);
    active_tb_name.value = value.table_name;
});

await fetch_collections();

async function fetch_collections() {
    try {
        const response = await authFetch('/admin/api/collections');
        if (response.success) {
            if (response.collections.length > 0) {
                collections.value = response.collections;
                if (!active_tb.value) {
                    active_tb.value = response.collections[0].table_id;
                    active_tb_name.value = response.collections[0].table_name;
                    await get_collection(active_tb.value);
                }
            }
        } else {
            errors.value.message = response.message;
        }
    } catch (error) {
        errors.value.message = error.data.message;
    }
}

async function delete_collection() {
    resetValues();
    processing.value = true;
    try {
        const response = await authFetch('/admin/api/delete-collection', {
            method: "POST",
            body: {
                collection_id: active_tb.value
            }
        });
        if (response.success) {
            success_message.value = response.message;
            await fetch_collections();
        } else {
            errors.value.message = response.message;
        }
    } catch (error) {
        errors.value.message = error.data.message;
    } finally {
        processing.value = false;
        show.value = false;
    }
}

async function get_collection(table_id) {
    try {
        const response = await authFetch('/admin/api/get-collection-records', {
            method: "POST",
            body: {
                collection_id: table_id
            }
        });
        if (response.success) {
            columns.value = response.columns;
            records.value = response.records;
        } else {
            errors.value.message = response.message;
        }
    } catch (error) {
        errors.value.message = error.data.message;
    }
}


function resetValues() {
    success_message.value = null;
}
</script>
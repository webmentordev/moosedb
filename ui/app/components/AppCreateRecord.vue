<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full" v-if="record_show">
        <div class="w-full h-full flex justify-end" @click.self="closeModal">
            <div class="w-137.5 h-full border-l border-white/10 bg-light p-6 overflow-y-auto">
                <h1 class="text-2xl">Create new record</h1>
                <div class="mt-3">
                    <div class="flex flex-col">
                        <AppLabel text="Collection name" />
                        <AppInput v-model="name" type="text" placeholder='e.g Books' />
                        <AlertsAlertError v-if="errors.name" :error="errors.name" />
                    </div>

                    <!-- <ClientOnly>
                        <Quill v-model:content="body" />
                    </ClientOnly>
                    
                    {{ body }} -->

                    <button class="flex items-center justify-center py-2 px-3 border rounded-xl mt-4 bg-main w-full"
                        @click="dropdown = !dropdown">
                        + create record
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
const props = defineProps({
    columns: {
        type: Array,
        default: []
    },
    record_show: {
        type: Boolean,
        default: false
    }
});

const emit = defineEmits(['update:record_show']);
function closeModal() {
    emit('update:record_show', false);
}

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

const { authFetch } = useAuthFetch();
</script>
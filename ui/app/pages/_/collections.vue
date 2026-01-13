<template>
    <div class="w-full h-full flex">
        <AppList :collections="collections" :active="active_tb" />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <div class="p-3">
            <h1 class="text-lg">Collections</h1>
        </div>
    </div>
</template>

<script setup>
    const { authFetch } = useAuthFetch();
    definePageMeta({
        middleware: 'auth'
    });
    const errors = ref({
        message: null,
        count: 0
    });
    const collections = ref([]);
    const route = useRoute();
    const active_tb = ref(null);
    if(route.query.tb){
        active_tb.value = route.query.tb;
    }
    watch(() => route.query.tb, (newTb) => {
        active_tb.value = newTb || null;
        console.log(active_tb.value);
    });

    try{
        const response = await authFetch('/admin/api/collections');
        if(response.success){
            if(response.collections.length > 0){
                collections.value = response.collections;
                active_tb.value = response.collections[0].table_id;
                console.log(collections.value);
            }
        }else{
            errors.value.message = response.message;
        }
    }catch(error){
        errors.value.message = error.data.message;
    }
</script>
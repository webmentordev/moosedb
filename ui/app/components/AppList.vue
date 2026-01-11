<template>
    <section class="w-55 bg-light p-3 flex flex-col h-full border-r border-white/5">
        <AppCreateCollection v-model:show="show" />
        <AppInput placeholder="Search collection..." />
        <div class="grid grid-cols-1 gap-3 my-3 py-3 border-y border-white/5" v-if="collections.length > 0">
            <AppLink v-for="item in collections" :link='"/_/collections?tb="+item.table_id' :title="item.table_name" />
        </div>
        <p v-else class="my-4 pb-3 text-center border-b border-white/5">No collection exist</p>

        <button class="text-gray-300 flex items-center py-2 px-3 border border-white rounded-xl" @click="show = true">
            <img src="https://api.iconify.design/mdi:table-large-plus.svg?color=%23ffffff" width="18">
            <span class="ml-2 text-sm">+ collection</span>
        </button>
        <AlertsError v-if="errors.message" :message="errors.message" />
    </section>
</template>

<script setup>
    const { authFetch } = useAuthFetch();
    const show = ref(false);
    const errors = ref({
        message: null,
        count: 0
    });

    const collections = ref([]);

    try{
        const response = await authFetch('/admin/api/collections');
        if(response.success){
            if(response.collections.length > 0){
                collections.value = response.collections
                console.log(collections.value);
            }
        }else{
            errors.value.message = response.message;
        }
    }catch(error){
        errors.value.message = error.data.message;
    }
</script>
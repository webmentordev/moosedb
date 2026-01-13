<template>
    <section class="w-55 bg-light p-3 flex flex-col h-full border-r border-white/5">
        <AppCreateCollection v-model:show="show" />
        <AppInput placeholder="Search collection..." />
        <div class="grid grid-cols-1 gap-3 my-3 py-3 border-y border-white/5" v-if="collections.length > 0">
            <AppLink v-for="item in collections" :link='"/_/collections?tb="+item.table_id' :id="item.table_id" :active="active" :title="item.table_name" />
        </div>
        <p v-else class="my-4 pb-3 text-center border-b border-white/5">No collection exist</p>
        <button class="text-gray-300 flex items-center py-2 px-3 border border-white rounded-xl" @click="show = true">
            <img src="https://api.iconify.design/mdi:table-large-plus.svg?color=%23ffffff" width="18">
            <span class="ml-2 text-sm">+ collection</span>
        </button>
    </section>
</template>

<script setup>
    defineProps({
        active: {
            String,
            default: null
        },
        collections: {
            Array
        }
    })
    const show = ref(false);
</script>
<template>
    <section class="w-55 bg-light p-3 flex flex-col h-full border-r border-white/5">
        <AppCreateCollection v-model:show="show" />
        <AppInput placeholder="Search collection..." />
        <div class="grid grid-cols-1 gap-3 my-3 py-3 border-y border-white/5">
            <AppLink v-for="item in collections" :link="item.link" :title="item.title" />
        </div>

        <button class="text-gray-300 flex items-center py-2 px-3 border border-white rounded-xl" @click="show = true">
            <img src="https://api.iconify.design/mdi:table-large-plus.svg?color=%23ffffff" width="18">
            <span class="ml-2 text-sm">+ collection</span>
        </button>
    </section>
</template>

<script setup>
    const show = ref(false);
    const collections = ref([
    {
        link: "/_/collections",
        title: "collections"
    },
    {
        link: "/_/foo",
        title: "users"
    },
    {
        link: "/_/bar",
        title: "books"
    },
    {
        link: "/_/foobar",
        title: "classes"
    }]);
</script>
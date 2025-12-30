<template>
    <nuxt-link :to="link" class="text-gray-300 flex items-center py-2 px-3 border border-white/10 rounded-xl" :title="title">
        <img src="https://api.iconify.design/mdi:table-headers-eye.svg?color=%23ffffff" width="18">
        <span class="ml-2 text-sm">{{ truncatedTitle }}</span>
    </nuxt-link>
</template>
<script setup>
    const props = defineProps(['title', 'link'])
    
    const truncatedTitle = computed(() => {
        return props.title.length > 12 
            ? props.title.substring(0, 12) + '...' 
            : props.title
    })
</script>
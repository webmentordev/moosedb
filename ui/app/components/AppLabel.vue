<template>
    <label class="text-gray-400">{{ text }}</label>
</template>
<script setup>
    defineProps({
        text: {
            String,
            default: "Collection name"
        }
    })
</script>
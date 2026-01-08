<template>
    <label class="text-gray-400 mb-1">{{ text }}</label>
</template>
<script setup>
    defineProps({
        text: {
            String,
            default: "Collection name"
        }
    })
</script>
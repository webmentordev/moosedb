<template>
    <p class="error mt-2 text-sm ml-2">{{ error }}</p>
</template>
<script setup>
    defineProps({
        error: {
            String,
            default: "Input field is required."
        }
    })
</script>
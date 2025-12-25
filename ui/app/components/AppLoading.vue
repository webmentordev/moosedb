<template>
    <div class="fixed bottom-3 left-[42%] bg-dark border border-input py-2 px-5 rounded-2xl z-50 max-w-85 w-fit">
        <div class="flex items-center">
            <img src="https://api.iconify.design/svg-spinners:180-ring-with-bg.svg?color=%23ffffff" width="30">
            <p class="text-white ml-3 text-sm">{{ message }}</p>
        </div>
    </div>
</template>
<script setup>
    defineProps({
        message: {
            String,
            default: "Loading..."
        }
    })
</script>
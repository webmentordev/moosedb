<template>
    <div class="fixed left-2 bottom-2 py-3 px-4 bg-dark border border-white/10 rounded-xl max-w-85 w-full">
        <div class="flex items-center">
            <img src="https://api.iconify.design/oui:cross-in-circle-empty.svg?color=%23e01f1f" width="15px">
            <div class="flex flex-col ml-3 border-l border-white/10 pl-3">
                <h3 class="text-red-500">Woopsie, Something went wrong.</h3>
                <p class="text-[12px] text-white">{{ message }}</p>
            </div>
        </div>
    </div>
</template>

<script setup>
    defineProps({
        message: {
            String,
            default: "Invalid login credientials."
        }
    })
</script>
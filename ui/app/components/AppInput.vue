<template>
    <input :value="props.modelValue" @input="emit('update:modelValue', $event.target.value)"
        class="p-3 border border-input bg-dark outline-none placeholder:tect-para focus:outline-none w-full text-white text-sm rounded-xl placeholder:text-gray-500">
</template>

<script setup>
const props = defineProps(['modelValue'])
const emit = defineEmits(['update:modelValue'])
</script>
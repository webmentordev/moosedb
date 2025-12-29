<template>
    <div class="w-full h-full p-4">
        <h1>Admin Dashboard</h1>
    </div>
</template>

<script setup>
    definePageMeta({
        middleware: 'auth'
    });
    await navigateTo('/_/collections');
</script>
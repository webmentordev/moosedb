<template>
    <div class="w-full h-full">
        <h1>Admin Dashboard</h1>
        <div v-if="version" class="grid grid-cols-4 gap-3">
            <ul class="text-white">
                <li>MooseDB - {{ version.version }}</li>
                <li>Actix Web -{{ version.actix_web }}</li>
                <li>Actix File - {{ version.actix_files }}</li>
                <li>Rustqlite - {{ version.rusqlite }}</li>
                <li>Serde - {{ version.serde }}</li>
                <li>Serde Json - {{ version.serde_json }}</li>
                <li>Rust Embed - {{ version.rust_embed }}</li>
                <li>Mime Guess - {{ version.mime_guess }}</li>
            </ul>
        </div>
        <button @click="logout">Loogut</button>
    </div>
</template>

<script setup>
    definePageMeta({
        middleware: 'auth'
    });
    const version = ref(null);
    const { removeToken } = useAuthToken();
    const { authFetch } = useAuthFetch();

    try {
        const data = await authFetch('/admin/api/get-version', {
            method: "POST"
        });
        version.value = data;
    } catch (error) {
        console.error('Failed to fetch:', error);
    }

    async function logout() {
        removeToken();
        await navigateTo('/_/auth/login');
    }
</script>
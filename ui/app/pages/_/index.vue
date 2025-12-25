<template>
    <div class="w-full h-full">
        <h1>Admin Dashboard</h1>
        <div v-if="status === 'pending'">
            Loading data...
        </div>
        <div class="grid grid-cols-4 gap-3" v-else>
            <ul v-if="version" class="text-white">
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
    </div>
</template>

<script setup>
    const { status, data: version } = await useLazyFetch('/api/get-version', {
        method: "POST",
        headers: {
            "Accept": "application/json",
            "Content-Type": "application/json"
        }
    });
    watch(version, (versionNew) => {
        console.log("API Status: " + status.value);
    })
</script>
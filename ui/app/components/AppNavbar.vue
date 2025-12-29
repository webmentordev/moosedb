<template>
    <section class="w-25 bg-dark p-3 flex flex-col items-center justify-between h-full border-r border-white/10">
        <div class="flex flex-col">
            <NuxtLink to="/_/" class="m-auto"><img src="/moose.png" width="50px"></NuxtLink>
            <div class="grid grid-cols-1 gap-4 mt-5 border-t border-white/10 pt-4">
                <NuxtLink to="/_/collections" class="p-2 bg-light w-fit border m-auto border-white/5 rounded-xl hover:border-main transition-all"><img src="https://api.iconify.design/material-symbols:database-sharp.svg?color=%23ffffff" width="30px"></NuxtLink>
                <NuxtLink to="/_/logs" class="p-2 bg-light w-fit border m-auto border-white/5 rounded-xl hover:border-main transition-all"><img src="https://api.iconify.design/material-symbols-light:ssid-chart-rounded.svg?color=%23ffffff" width="30px"></NuxtLink>
                <NuxtLink to="/_/settings" class="p-2 bg-light w-fit border m-auto border-white/5 rounded-xl hover:border-main transition-all"><img class="group-hover:animate-spin" src="https://api.iconify.design/solar:settings-linear.svg?color=%23ffffff" width="30px"></NuxtLink>
            </div>
        </div>
        <AppLoading v-if="processing" message="Logging out..." />
        <button @click="logout" class="p-2 h-fit bg-light w-fit border border-white/5 rounded-xl hover:border-main transition-all group">
            <span><img src="https://api.iconify.design/tabler:logout.svg?color=%23ffffff" width="30px"></span>
        </button>
    </section>
</template>
<script setup>
    const processing = ref(false);
    const { removeToken } = useAuthToken();
    const { authFetch } = useAuthFetch();
    async function logout() {
        processing.value = true;
        removeToken();
        await navigateTo('/_/auth/login');
        processing.value = false;
    }
</script>
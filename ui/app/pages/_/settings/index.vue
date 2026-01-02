<template>
    <div class="w-full h-full p-4">
        <h1>Settings Dashboard</h1>
        <div class="max-w-3xl w-full m-auto">
            <form @submit="update_appname" method="post" class="flex items-center">
                <AppInput type="text" v-model="appname" placeholder="Application name" required />
                <button v-if="!processing" type="submit" class="bg-main ml-2 max-w-25 text-white w-full py-2 rounded-xl flex items-center justify-center hover:bg-main/90 group">
                    <span class="mr-3">Update</span>
                </button>
            </form>
        </div>
    </div>
</template>

<script setup>
    const appname = ref("");
    const { authFetch } = useAuthFetch();

    try {
        const data = await authFetch('/admin/api/get-setting', {
            method: "POST",
            body: {
                key: "appname"
            }
        });
        if(data.success == true){
            appname.value = data.value;
        }else{
            appname.value = "Unknown";
        }
    } catch (error) {
        console.error('Failed to fetch:', error);
    }

    async function update_appname() {
        
    }
</script>
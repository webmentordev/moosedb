<template>
    <div class="w-full h-full p-4">
        <h1>Settings Dashboard</h1>
        <div class="max-w-3xl w-full m-auto">
            <AlertsSuccess v-if="response" :message="response" />
            <AlertsError v-if="errors.message" :message="errors.message" />
            <AppLoading v-if="processing" message="Processing..." />
            <form @submit.prevent="update_appname" method="post" class="flex flex-col">
                <div class="flex items-center w-full">
                    <AppInput type="text" v-model="appname" placeholder="Application name" />
                    <button v-if="!processing" type="submit" class="bg-main ml-2 max-w-25 text-white w-full py-2 rounded-xl flex items-center justify-center hover:bg-main/90 group">
                        <span class="mr-3">Update</span>
                    </button>
                </div>
                <AlertsAlertError v-if="errors.appname" error="App name is required." />
            </form>
        </div>
    </div>
</template>

<script setup>
    const appname = ref("");
    const response = ref(null);
    const processing = ref(false);
    const errors = ref({
        appname: null,
        message: null,
        count: 0
    });

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
        reset_values();
        if(appname.value.trim() == ""){
            errors.value.appname = "required";
            errors.value.count += 1;
        }
        if(errors.value.count == 0){
            processing.value = true;
            try {
                const data = await authFetch('/admin/api/update-setting', {
                    method: "POST",
                    body: {
                        key: "appname",
                        value: appname.value
                    }
                });
                if(data.success == true){
                    response.value = data.message;
                }else{
                    errors.value.message = data.message;
                }
            } catch (error) {
                console.error('Failed to fetch:', error);
            } finally{
                processing.value = false;
            }
        }
    }
    function reset_values(){
        response.value = null;
        errors.value = {
            appname: null,
            message: null,
            count: 0
        }
    }
</script>
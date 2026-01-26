<template>
    <div class="w-full h-full p-4">
        <h1>Settings Dashboard</h1>
        <div class="max-w-3xl w-full m-auto">
            <AlertsSuccess v-if="response" :message="response" />
            <AlertsError v-if="errors.message" :message="errors.message" />
            <AppLoading v-if="processing" message="Processing..." />
            <form @submit.prevent="update_appname" method="post" class="flex flex-col mt-3">
                <div class="flex items-center w-full">
                    <AppInput type="text" v-model="appname" placeholder="Application name" />
                    <button v-if="!processing" type="submit" class="bg-main ml-2 max-w-25 text-white w-full py-2 rounded-xl flex items-center justify-center hover:bg-main/90 group">
                        <span class="mr-3">Update</span>
                    </button>
                </div>
                <AlertsAlertError v-if="errors.appname" error="App name is required." />
            </form>

            <form @submit.prevent="add_new_admin" method="post" class="flex flex-col bg-dark p-6 rounded-xl mt-4">
                <h3 class="text-lg">Create new super admin</h3>
                <div class="grid grid-cols-2 gap-3 w-full mt-3">
                    <div class="flex flex-col">
                        <AppInput class="bg-light" type="text" v-model="name" placeholder="Name" required />
                        <AlertsAlertError v-if="errors.name" error="Name is required." />
                    </div>
                    <div class="flex flex-col">
                        <AppInput class="bg-light" type="email" v-model="email" placeholder="Email address" required />
                        <AlertsAlertError v-if="errors.email" error="Email address is required." />
                    </div>
                    <div class="flex flex-col">
                        <AppInput class="bg-light" type="password" v-model="password" placeholder="Password" required />
                        <AlertsAlertError v-if="errors.password" error="Password is required." />
                        <AlertsAlertError v-if="errors.match_password" error="Password and confirm password do not match." />
                    </div>

                    <div class="flex flex-col">
                        <AppInput class="bg-light" type="password" v-model="confirm_password" placeholder="Confirm password" required />
                        <AlertsAlertError v-if="errors.confirm_password" error="Confirm password is required." />
                    </div>
                    <button v-if="!processing" type="submit" class="bg-main text-white w-full py-3 rounded-xl flex items-center justify-center hover:bg-main/90 group col-span-2">
                        <span class="mr-3">Create new super admin</span>
                        <img class="mt-1 transition-all group-hover:transition-all group-hover:translate-x-4" src="https://api.iconify.design/line-md:arrow-right.svg?color=%23ffffff" width="15">
                    </button>
                </div>
            </form>
        </div>
    </div>
</template>

<script setup>
    const appname = ref("");
    const name = ref("");
    const email = ref("");
    const password = ref("");
    const confirm_password = ref("");
    const response = ref(null);
    const processing = ref(false);
    const errors = ref({
        appname: null,
        name: null,
        email: null,
        password: null,
        confirm_password: null,
        match_password: null,
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

    // Update app name
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

    // Ceate new super admin
    async function add_new_admin() {
        reset_values();
        if(name.value.trim() == ""){
            errors.value.name = "required";
            errors.value.count += 1;
        }
        if(email.value.trim() == ""){
            errors.value.email = "required";
            errors.value.count += 1;
        }
        if(password.value.trim() == ""){
            errors.value.password = "required";
            errors.value.count += 1;
        }
        if(confirm_password.value.trim() == ""){
            errors.value.confirm_password = "required";
            errors.value.count += 1;
        }
        if(password.value != confirm_password.value){
            errors.value.match_password = "required";
            errors.value.count += 1;
        }
        if(errors.value.count == 0){
            processing.value = true;
            try {
                const data = await authFetch('/admin/api/create-super-admin', {
                    method: "POST",
                    body: {
                        name: email.value,
                        email: email.value,
                        password: password.value,
                        confirm_password: confirm_password.value
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
            name: null,
            email: null,
            password: null,
            confirm_password: null,
            match_password: null,
            message: null,
            count: 0
        }
    }
</script>
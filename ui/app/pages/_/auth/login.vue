<template>
    <div class="w-full h-screen">
        <div class="w-full h-full flex items-center justify-center">
            <div class="flex flex-col max-w-87.5 w-full">
                <div class="flex items-center m-auto">
                    <img src="/moose.png" alt="Moose Icon" width="70">
                    <h4 class="text-white font-medium text-3xl ml-2">Moose<strong class="text-main">DB</strong></h4>
                </div>
                <p class="text-center text-white my-5">Superuser Login</p>
                <form @submit.prevent="login" method="post">
                    <div class="grid grid-cols-1 gap-3">
                        <div class="flex flex-col">
                            <AppInput v-model="email" type="email" placeholder="Email address" />
                            <AlertsAlertError v-if="errors.email" error="Email field is required" />
                        </div>
                        <div class="flex flex-col">
                            <AppInput v-model="password" type="password" placeholder="Password" />
                            <AlertsAlertError v-if="errors.password" error="Password field is required" />
                        </div>
                    </div>

                    <NuxtLink to="/forgot-password" class="text-para-light text-sm ml-1 hover:text-main">Forgotten password?</NuxtLink>

                    <button v-if="!processing" type="submit" class="bg-main mt-4 text-white w-full py-3 rounded-xl flex items-center justify-center hover:bg-main/90 group">
                        <span class="mr-3">Login</span>
                        <img class="mt-1 transition-all group-hover:transition-all group-hover:translate-x-4" src="https://api.iconify.design/line-md:arrow-right.svg?color=%23ffffff" width="15">
                    </button>

                    <AppLoading v-if="processing" message="Processing login request..." />
                    <AlertsError v-if="errors.message" :message="errors.message" />
                </form>
            </div>
        </div>
    </div>
</template>

<script setup>
    definePageMeta({
        layout: 'auth',
    });

    const email = ref("");
    const password = ref("");
    const processing = ref(false);
    const errors = ref({
        email: null,
        password: null,
        message: null,
        count: 0
    });

    async function login(){
        reset_errors();
        if (email.value == ""){
            errors.value.email = "Email is required";
            errors.value.count += 1;
        }
        if (password.value == ""){
            errors.value.password = "Password is required";
            errors.value.count += 1;
        }
        if(errors.value.count > 0) return;

        const data = await $fetch("/admin/api/get-login");
    }

    function reset_errors(){
        errors.value = {
            email: null,
            password: null,
            message: null,
            count: 0
        };
    }

    function reset_values(){
        email.value = "";
        password.value = "";
    }

</script>
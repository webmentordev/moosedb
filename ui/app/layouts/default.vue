<template>
  <section class="w-full h-screen bg-light">
    <NuxtLoadingIndicator />
    <div class="flex w-full h-full">
      <AppNavbar />
      <slot />
    </div>
  </section>
</template>

<script setup lang="js">
const appname = ref("MooseDB");
const { authFetch } = useAuthFetch();
try {
  const data = await authFetch('/admin/api/get-setting', {
    method: "POST",
    body: {
      key: "appname"
    }
  });
  if (data.success == true) {
    appname.value = data.value;
    console.log(data.value);
    useHead({
      title: data.value,
    })
  } else {
    appname.value = "Unknown";
  }
} catch (error) {
  console.error('Failed to fetch:', error);
}
</script>
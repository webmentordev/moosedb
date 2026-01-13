<template>
  <section class="w-full h-screen bg-light">
    <NuxtLoadingIndicator />
    <div class="flex w-full h-full">
      <AppNavbar />
      <slot />
    </div>
  </section>
</template>
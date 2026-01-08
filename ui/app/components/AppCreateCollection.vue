<template>
    <div class="bg-dark/5 backdrop-blur-sm fixed top-0 left-0 z-50 w-full h-full">
        <div class="w-full h-full flex justify-end">
            <div class="w-137.5 h-full border-l border-white/10 bg-light p-6">
                <h1 class="text-2xl">New collection</h1>
                <div class="mt-3">
                    <div class="flex flex-col">
                        <AppLabel text="Collection name"/>
                        <AppInput v-model="name" type="text" placeholder='e.g Books' />
                        <AlertsAlertError v-if="errors.name" error="Email field is required" />
                    </div>

                    <button class="flex items-center justify-center py-2 px-3 border rounded-xl mt-4 bg-main w-full">
                        + fields
                    </button>
                    <div class="p-3 bg-dark rounded-lg mt-4 grid grid-cols-3 gap-3">
                        <button @click="add_to_array('{{ column.id }}')" v-for="column in columns" class="bg-light rounded-lg py-2 px-3 border border-white/10">{{ column.name }}</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
    const name = ref("");
    const errors = ref({
        name: null,
        count: 0
    });
    const columns = ref([
        {
            id: 1,
            name: "Plain text",
            limit: 255,
            nullable: false,
        },
        {
            id: 2,
            name: "Rich text",
            limit: 0,
            nullable: false,
        },
        {
            id: 3,
            name: "Boolean",
            limit: 0,
            nullable: false,
        }
    ]);
    const column_items = ref([]);

    function add_to_array(){
        column_items.value.push();
    }
</script>
<template>
    <QuillEditor :options="options" contentType="html" />
</template>
<script setup>
    import { QuillEditor } from '@vueup/vue-quill';
    import '@vueup/vue-quill/dist/vue-quill.snow.css';
    const toolbarOptions = [
        ['bold', 'italic', 'underline', 'strike'],
        ['blockquote', 'code-block'],
        ['link', 'image', 'video', 'formula'],
        [{ 'header': 1 }, { 'header': 2 }, { 'header': 3 }, { 'header': 4 }],
        [{ 'list': 'ordered'}, { 'list': 'bullet' }, { 'list': 'check' }],
        [{ 'script': 'sub'}, { 'script': 'super' }],
        [{ 'indent': '-1'}, { 'indent': '+1' }],
        [{ 'direction': 'rtl' }], 
        [{ 'size': ['small', false, 'large', 'huge'] }],
        [{ 'header': [1, 2, 3, 4, 5, 6, false] }],
        [{ 'color': [] }, { 'background': [] }],
        [{ 'font': [] }],
        [{ 'align': [] }],
        ['clean'] 
    ];

    const options = ref({
        modules: {
            toolbar: toolbarOptions
        },
        placeholder: "Compose here...",
        readOnly: false,
        theme: "snow"
    });

</script>

<style>
    .ql-container.ql-snow{
        max-height: 100px !important;
        color: white;
    }

    .ql-editor{
        height: 100px !important;
    }
</style>
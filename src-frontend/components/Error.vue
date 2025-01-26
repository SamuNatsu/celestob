<script setup lang="ts">
// Properties
defineProps<{ err: any }>();
</script>

<template>
  <div class="bg-red-50 border-2 border-red-200 my-4 py-2 rounded">
    <h1 class="font-bold px-2 text-lg">Error Occurred</h1>
    <hr class="border-t-2 border-t-red-200 my-2" />
    <p class="px-2 text-sm">{{ err }}</p>
  </div>
</template>

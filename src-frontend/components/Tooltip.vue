<script setup lang="ts">
import { ref, type Ref } from 'vue';

// Refs
const visible: Ref<boolean> = ref(false);
</script>

<template>
  <div
    class="inline-block relative"
    @mouseenter="visible = true"
    @mouseleave="visible = false">
    <slot></slot>
    <div v-show="visible" class="tooltip">
      <slot name="tip"></slot>
    </div>
  </div>
</template>

<style scoped>
@reference '@/style.css';

.tooltip {
  @apply absolute bottom-[calc(100%+5px)] left-1/2 px-2 py-1 -translate-x-1/2 z-50;
  @apply bg-black/80 text-center text-sm text-white whitespace-nowrap;
  @apply rounded;

  &::after {
    content: '';
    position: absolute;
    top: 100%;
    left: 50%;
    margin-left: -5px;
    border-width: 5px;
    border-style: solid;
    border-color: rgba(0, 0, 0, 0.8) transparent transparent transparent;
  }
}
</style>

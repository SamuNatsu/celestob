<script setup lang="ts">
import { computed } from 'vue';
import moment from 'moment';

// Components
import Tooltip from '@/components/Tooltip.vue';

// Properties
defineProps<{
  pivot: string;
  stat: (number | null)[];
}>();

// Computed
const getColor = computed(
  (): ((count: number | null, idx: number) => string) =>
    (count: number | null, idx: number): string => {
      if (idx === 0) {
        return 'bg-blue-500';
      } else if (count === null) {
        return 'bg-neutral-500';
      } else if (count >= 11) {
        return 'bg-green-500';
      } else if (count >= 7) {
        return 'bg-yellow-500';
      } else if (count > 0) {
        return 'bg-orange-500';
      } else {
        return 'bg-red-500';
      }
    }
);
</script>

<template>
  <div
    class="flex flex-row-reverse gap-[min(calc(var(--spacing)),0.5%)] items-center justify-end px-2 w-full">
    <Tooltip v-for="(j, idx) in stat">
      <div class="h-4 rounded w-1" :class="getColor(j, idx)"></div>
      <template #tip>
        <span class="text-neutral-300 text-xs">{{
          moment(pivot).subtract(idx, 'hours').format('YYYY/MM/DD@HHZ')
        }}</span>
        <br />
        <span v-if="j === null">No data</span>
        <span v-else
          >{{ j }}/11 ({{ Math.min(100, (j / 11) * 100).toFixed(0) }}%)</span
        >
      </template>
    </Tooltip>
  </div>
</template>

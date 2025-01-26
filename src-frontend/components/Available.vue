<script setup lang="ts">
import { computed } from 'vue';

// Properties
const props = defineProps<{ stat: (number | null)[] }>();

// Computed
const getPercent = computed((): number => {
  const tmp: number[] = props.stat.filter(
    (v: number | null): boolean => v !== null
  ) as number[];
  const sum: number = tmp
    .slice(1)
    .reduce((prev: number, cur: number): number => prev + Math.min(cur, 11));
  return (sum / (11 * (tmp.length - 1))) * 100;
});
const getColor = computed((): string => {
  if (getPercent.value >= 90) {
    return 'bg-green-500';
  } else if (getPercent.value >= 60) {
    return 'bg-yellow-500';
  } else {
    return 'bg-red-500';
  }
});
</script>

<template>
  <span
    class="inline-block leading-none py-0.5 rounded-lg text-center text-xs text-white w-10"
    :class="getColor"
    >{{ getPercent.toFixed(0) }}%</span
  >
</template>

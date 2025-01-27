<script setup lang="ts">
import { computed } from 'vue';

// Properties
const props = defineProps<{ stat: (number | null)[] }>();

// Computed
const getPercent = computed((): number => {
  const tmp: number[] = props.stat
    .slice(1)
    .filter((v: number | null): boolean => v !== null) as number[];
  if (tmp.length === 0) {
    return 0;
  }

  const sum: number = tmp.reduce(
    (prev: number, cur: number): number => prev + Math.min(cur, 11),
    0
  );
  return (sum / (11 * tmp.length)) * 100;
});
const getColor = computed((): string => {
  if (getPercent.value >= 90) {
    return 'bg-green-500';
  } else if (getPercent.value >= 60) {
    return 'bg-yellow-500';
  } else if (getPercent.value > 0) {
    return 'bg-orange-500';
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

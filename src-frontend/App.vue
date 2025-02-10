<script setup lang="ts">
import { version } from '../package.json';
import { onBeforeMount, ref, type Ref } from 'vue';

// Components
import Available from '@/components/Available.vue';
import Error from '@/components/Error.vue';
import Status from '@/components/Status.vue';

// Types
type Status = {
  pivot: string;
  services: {
    name: string;
    desc: string;
    stat: (number | null)[];
  }[];
};

// Refs
const err: Ref<any> = ref(null);
const status: Ref<Status | null> = ref(null);

// Hooks
onBeforeMount((): void => {
  fetch('/api/status')
    .then((r: Response): Promise<Status> => {
      if (r.status >= 400) {
        throw r.statusText;
      }

      return r.json();
    })
    .then((v: Status): void => {
      status.value = v;
    })
    .catch((e: any): void => {
      err.value = e;
    });
});
</script>

<template>
  <div class="max-w-3xl mx-4 my-12 sm:mx-auto">
    <header><h1 class="font-bold text-4xl">Celestob</h1></header>

    <hr class="border-t-2 border-t-blue-500 my-2" />

    <main>
      <template v-if="status !== null">
        <section
          v-for="i of status.services"
          class="bg-green-50 border-2 border-green-200 my-4 py-2 rounded">
          <div class="px-2">
            <h1 class="flex font-bold gap-1 items-center text-lg">
              <Available :stat="i.stat" />
              <span>{{ i.name.split(':', 2)[1] }}</span>
            </h1>
            <p class="text-neutral-700 text-xs">{{ i.desc }}</p>
          </div>
          <hr class="border-t-2 border-t-green-200 my-2" />
          <Status :pivot="status.pivot" :stat="i.stat" />
        </section>
      </template>

      <Error v-if="err !== null" :err="err" />
    </main>

    <hr class="border-t-2 border-t-blue-500 my-2" />

    <footer class="flex gap-2 items-center text-sm">
      <p>v{{ version }}</p>
      <a
        class="hover:text-blue-500"
        href="https://github.com/SamuNatsu/celestob"
        rel="noopener noreferer"
        target="_blank"
        >GitHub</a
      >
    </footer>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import BrandMark from "./BrandMark.vue";
import type { Section } from "../types";

defineProps<{
  logo: string;
  sections: readonly Section[];
  activeSection: Section;
  projectCount: number;
  editorCount: number;
  appVersion: string;
}>();

const emit = defineEmits<{
  navigate: [section: Section];
}>();

const { t } = useI18n();
</script>

<template>
  <aside class="sticky top-0 hidden h-screen overflow-y-auto border-r border-base-content/10 bg-base-300 lg:flex lg:flex-col">
    <div class="border-b border-base-content/10 p-5">
      <div class="flex items-center gap-3">
        <BrandMark :logo="logo" />
        <div>
          <strong class="block text-sm">Godot Forge</strong>
          <span class="text-xs text-base-content/50">Engine & Project Hub</span>
        </div>
      </div>
    </div>

    <nav class="flex-1 p-3">
      <button
        v-for="section in sections"
        :key="section"
        class="mb-1 flex w-full items-center justify-between rounded-md px-3 py-2.5 text-left text-sm font-semibold text-base-content/65 transition hover:bg-base-content/5 hover:text-base-content"
        :class="{ 'bg-primary/15 text-primary ring-1 ring-primary/20': activeSection === section }"
        @click="emit('navigate', section)"
      >
        <span>{{ t(`nav.${section}`) }}</span>
        <span v-if="section === 'projects'" class="rounded bg-base-content/10 px-1.5 text-[11px]">{{ projectCount }}</span>
        <span v-if="section === 'editors'" class="rounded bg-base-content/10 px-1.5 text-[11px]">{{ editorCount }}</span>
      </button>
    </nav>

    <div class="border-t border-base-content/10 p-3">
      <div class="flex items-center justify-between gap-3 rounded-md px-2 py-1.5 text-xs text-base-content/45">
        <div class="min-w-0">
          <strong class="block truncate text-xs text-base-content/70">Godot Forge</strong>
          <span v-if="appVersion" class="mt-0.5 block text-[10px]">v{{ appVersion }}</span>
        </div>
        <button
          class="grid h-8 w-8 shrink-0 cursor-pointer place-items-center rounded-md border border-base-content/10 text-base-content/60 transition hover:border-primary/30 hover:bg-primary/10 hover:text-primary"
          type="button"
          :title="t('common.settings')"
          :aria-label="t('common.settings')"
          @click="emit('navigate', 'settings')"
        >
          <svg class="h-4 w-4" viewBox="0 0 16 16" aria-hidden="true" fill="currentColor">
            <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492M5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0" />
            <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.42 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.42-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.115z" />
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

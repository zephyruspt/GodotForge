<script setup lang="ts">
import { useI18n } from "vue-i18n";
import BrandMark from "./BrandMark.vue";
import type { Section } from "../types";

type WelcomeSlide = {
  label: string;
  title: string;
  body: string;
};

defineProps<{
  logo: string;
  slides: WelcomeSlide[];
  slide: number;
  selectedLocale: string;
}>();

const emit = defineEmits<{
  "update:slide": [value: number];
  "update:selectedLocale": [value: string];
  previous: [];
  next: [];
  complete: [section: Section];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="grid min-h-screen bg-base-200 p-4 lg:p-8">
    <div class="grid min-h-[calc(100vh-2rem)] overflow-hidden rounded-xl border border-base-content/10 bg-base-100 lg:min-h-[calc(100vh-4rem)] lg:grid-cols-[minmax(0,1fr)_420px]">
      <div class="relative flex min-h-[560px] flex-col justify-between overflow-hidden p-6 lg:p-10">
        <div class="absolute inset-0 bg-[radial-gradient(circle_at_18%_18%,var(--color-primary)_0%,transparent_28%),radial-gradient(circle_at_82%_72%,var(--color-secondary)_0%,transparent_26%)] opacity-15" />
        <div class="relative flex items-center justify-between gap-4">
          <div class="flex items-center gap-3">
            <BrandMark :logo="logo" />
            <div>
              <strong class="block text-sm">Godot Forge</strong>
              <span class="text-xs text-base-content/50">Engine & Project Hub</span>
            </div>
          </div>
          <select
            :value="selectedLocale"
            class="select cursor-pointer select-bordered select-sm border-base-content/10 bg-base-100"
            @change="emit('update:selectedLocale', ($event.target as HTMLSelectElement).value)"
          >
            <option value="en">English</option>
            <option value="pt">Português</option>
          </select>
        </div>

        <div class="relative max-w-4xl py-10">
          <p class="text-xs font-black uppercase tracking-wide text-primary">{{ slides[slide].label }}</p>
          <h1 class="mt-4 max-w-4xl text-4xl font-black leading-none tracking-tight sm:text-5xl lg:text-7xl">
            {{ slides[slide].title }}
          </h1>
          <p class="mt-5 max-w-2xl text-base leading-7 text-base-content/65">
            {{ slides[slide].body }}
          </p>
        </div>

        <div class="relative flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex gap-2">
            <button
              v-for="(_, index) in slides"
              :key="index"
              class="h-1.5 rounded-full transition-all"
              :class="index === slide ? 'w-10 bg-primary' : 'w-5 bg-base-content/20 hover:bg-base-content/35'"
              type="button"
              :aria-label="`Slide ${index + 1}`"
              @click="emit('update:slide', index)"
            />
          </div>
          <div class="flex flex-wrap gap-2">
            <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" type="button" :disabled="slide === 0" @click="emit('previous')">
              {{ t("onboarding.back") }}
            </button>
            <button class="btn btn-primary" type="button" @click="emit('next')">
              {{ slide === slides.length - 1 ? t("onboarding.start") : t("onboarding.next") }}
            </button>
          </div>
        </div>
      </div>

      <aside class="hidden border-l border-base-content/10 bg-base-300/50 p-6 lg:grid lg:content-between">
        <div class="grid gap-3">
          <div
            v-for="(item, index) in slides"
            :key="item.label"
            class="rounded-lg border p-4 transition"
            :class="index === slide ? 'border-primary/35 bg-primary/10' : 'border-base-content/10 bg-base-100/70'"
          >
            <span class="text-[11px] font-black uppercase text-base-content/45">0{{ index + 1 }}</span>
            <h2 class="mt-2 text-sm font-black">{{ item.label }}</h2>
            <p class="mt-1 line-clamp-2 text-xs text-base-content/55">{{ item.body }}</p>
          </div>
        </div>

        <div class="rounded-lg border border-base-content/10 bg-base-100/70 p-4">
          <p class="text-[11px] font-black uppercase text-primary">{{ t("onboarding.quickStart") }}</p>
          <div class="mt-3 grid gap-2">
            <button class="btn btn-sm btn-primary" type="button" @click="emit('complete', 'editors')">
              {{ t("onboarding.download") }}
            </button>
            <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" type="button" @click="emit('complete', 'projects')">
              {{ t("onboarding.importProject") }}
            </button>
          </div>
        </div>
      </aside>
    </div>
  </section>
</template>

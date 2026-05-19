<script setup lang="ts">
import { useI18n } from "vue-i18n";
import PathField from "../components/PathField.vue";
import type { PathTarget, ThemeName } from "../types";

type SettingsForm = {
  defaultInstallPath: string;
  defaultProjectPath: string;
  releaseRepository: string;
};

defineProps<{
  settingsForm: SettingsForm;
  selectedLocale: string;
  selectedTheme: ThemeName;
  busy: boolean;
}>();

const emit = defineEmits<{
  "update:selectedLocale": [value: string];
  "update:selectedTheme": [value: ThemeName];
  browse: [target: PathTarget];
  save: [];
  restoreDefaults: [];
  openSecurity: [];
}>();

const { t } = useI18n();

function updateLocale(event: Event) {
  emit("update:selectedLocale", (event.target as HTMLSelectElement).value);
}

function updateTheme(event: Event) {
  emit("update:selectedTheme", (event.target as HTMLSelectElement).value as ThemeName);
}
</script>

<template>
  <section class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
    <form class="rounded-xl border border-base-content/10 bg-base-100 p-5" @submit.prevent="emit('save')">
      <div>
        <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceSettings") }}</p>
        <h2 class="text-2xl font-black">{{ t("settings.defaultPaths") }}</h2>
        <p class="mt-1 text-sm text-base-content/50">{{ t("settings.control") }}</p>
      </div>
      <div class="mt-5 grid gap-4 lg:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.installations") }}</span>
          <PathField
            v-model="settingsForm.defaultInstallPath"
            required
            :button-label="t('common.browse')"
            @browse="emit('browse', 'settingsInstall')"
          />
        </label>
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("nav.projects") }}</span>
          <PathField
            v-model="settingsForm.defaultProjectPath"
            required
            :button-label="t('common.browse')"
            @browse="emit('browse', 'settingsProject')"
          />
        </label>
        <label class="grid gap-2 lg:col-span-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.releaseRepository") }}</span>
          <input
            v-model="settingsForm.releaseRepository"
            class="input input-bordered border-base-content/10 bg-base-200"
            required
            placeholder="godotengine/godot"
          />
          <span class="text-xs text-base-content/50">{{ t("settings.releaseRepositoryHint") }}</span>
        </label>
      </div>
      <div class="mt-5 flex flex-col gap-2 border-t border-base-content/10 pt-5 sm:flex-row sm:justify-end">
        <button class="btn border-base-content/10 bg-base-content/5" type="button" :disabled="busy" @click="emit('restoreDefaults')">
          {{ t("settings.restoreDefaults") }}
        </button>
        <button class="btn btn-primary" :disabled="busy">
          {{ t("settings.saveWorkspace") }}
        </button>
      </div>
    </form>

    <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
      <p class="text-xs font-black uppercase text-primary">{{ t("settings.appearance") }}</p>
      <h2 class="mt-1 text-2xl font-black">{{ t("settings.preferences") }}</h2>
      <div class="mt-5 grid gap-4">
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.language") }}</span>
          <select :value="selectedLocale" class="select select-bordered border-base-content/10 bg-base-200" @change="updateLocale">
            <option value="en">English</option>
            <option value="pt">Português</option>
          </select>
        </label>
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.theme") }}</span>
          <select :value="selectedTheme" class="select select-bordered border-base-content/10 bg-base-200" @change="updateTheme">
            <option value="godotforge">{{ t("settings.darkTheme") }}</option>
            <option value="godotforge-light">{{ t("settings.lightTheme") }}</option>
          </select>
        </label>
        <div class="rounded-lg border border-warning/20 bg-warning/10 p-4">
          <p class="text-xs font-black uppercase text-warning">{{ t("security.title") }}</p>
          <p class="mt-2 text-sm text-base-content/70">{{ t("security.body") }}</p>
          <button class="btn btn-sm btn-warning mt-4" type="button" @click="emit('openSecurity')">
            {{ t("security.openPolicy") }}
          </button>
        </div>
      </div>
    </aside>
  </section>
</template>

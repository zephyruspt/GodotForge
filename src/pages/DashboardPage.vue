<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { DiscoveredEditor, DiscoveredProject, GodotEditor, GodotProject, Section, WorkspaceScan } from "../types";

defineProps<{
  projects: GodotProject[];
  editors: GodotEditor[];
  defaultEditor?: GodotEditor;
  defaultProjectPath: string;
  defaultInstallPath: string;
  workspaceScan: WorkspaceScan;
  workspaceScanLoaded: boolean;
  workspaceScanLoading: boolean;
  workspaceScanAction: string;
  workspaceScanError: string;
  busy: boolean;
}>();

const emit = defineEmits<{
  navigate: [section: Section];
  scanWorkspace: [];
  registerDiscoveredEditor: [editor: DiscoveredEditor];
  registerDiscoveredProject: [project: DiscoveredProject];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="grid gap-5">
    <div class="grid gap-4 md:grid-cols-3">
      <article class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("nav.projects") }}</p>
        <strong class="mt-3 block text-4xl font-black">{{ projects.length }}</strong>
        <p class="mt-2 truncate text-sm text-base-content/50">{{ defaultProjectPath || t("dashboard.notConfigured") }}</p>
        <button class="btn btn-sm mt-4 border-base-content/10 bg-base-content/5" type="button" @click="emit('navigate', 'projects')">
          {{ t("dashboard.openProjects") }}
        </button>
      </article>

      <article class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("nav.editors") }}</p>
        <strong class="mt-3 block text-4xl font-black">{{ editors.length }}</strong>
        <p class="mt-2 truncate text-sm text-base-content/50">{{ defaultEditor ? `${defaultEditor.name} ${defaultEditor.version}` : t("common.noEditor") }}</p>
        <button class="btn btn-sm mt-4 border-base-content/10 bg-base-content/5" type="button" @click="emit('navigate', 'editors')">
          {{ t("dashboard.manageEditors") }}
        </button>
      </article>

      <article class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("dashboard.workspace") }}</p>
        <strong class="mt-3 block text-4xl font-black">{{ workspaceScan.editors.length + workspaceScan.projects.length }}</strong>
        <p class="mt-2 truncate text-sm text-base-content/50">{{ defaultInstallPath || t("dashboard.notConfigured") }}</p>
        <button class="btn btn-sm mt-4 border-base-content/10 bg-base-content/5" type="button" @click="emit('navigate', 'settings')">
          {{ t("dashboard.workspaceSettings") }}
        </button>
      </article>
    </div>

    <section class="rounded-xl border border-base-content/10 bg-base-100 p-5">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceScan") }}</p>
          <h2 class="mt-1 text-2xl font-black">{{ t("settings.workspaceScanTitle") }}</h2>
          <p class="mt-1 text-sm text-base-content/60">{{ t("settings.workspaceScanBody") }}</p>
        </div>
        <button class="btn btn-primary shrink-0" type="button" :disabled="workspaceScanLoading || busy" @click="emit('scanWorkspace')">
          <span v-if="workspaceScanLoading" class="loading loading-spinner loading-xs" />
          {{ t("settings.scanWorkspace") }}
        </button>
      </div>

      <p v-if="workspaceScanError" class="mt-4 rounded-md border border-error/20 bg-error/10 px-3 py-2 text-sm text-error">
        {{ workspaceScanError }}
      </p>

      <div v-if="workspaceScanLoaded" class="mt-5 grid gap-4 xl:grid-cols-2">
        <div class="rounded-lg border border-base-content/10 bg-base-300/35 p-4">
          <div class="flex items-center justify-between gap-3">
            <h3 class="font-black">{{ t("settings.discoveredEditors") }}</h3>
            <span class="text-xs font-bold text-base-content/45">{{ workspaceScan.editors.length }}</span>
          </div>
          <div v-if="workspaceScan.editors.length" class="mt-3 grid gap-2">
            <div v-for="editor in workspaceScan.editors" :key="editor.installPath + editor.executablePath" class="rounded-md border border-base-content/10 bg-base-100 p-3">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="truncate text-sm font-black">{{ editor.name }} {{ editor.version }}</p>
                  <p class="mt-1 truncate text-xs text-base-content/50">{{ editor.installPath }}</p>
                  <p v-if="editor.reason" class="mt-2 text-xs text-warning">{{ editor.reason }}</p>
                </div>
                <span class="rounded px-2 py-1 text-[11px] font-black" :class="editor.corrupt ? 'bg-warning/20 text-warning' : editor.registered ? 'bg-success/15 text-success' : 'bg-primary/15 text-primary'">
                  {{ editor.corrupt ? t("settings.corrupt") : editor.registered ? t("settings.registered") : t("settings.newItem") }}
                </span>
              </div>
              <button
                v-if="!editor.registered && !editor.corrupt"
                class="btn btn-xs btn-primary mt-3"
                type="button"
                :disabled="!!workspaceScanAction || busy"
                @click="emit('registerDiscoveredEditor', editor)"
              >
                <span v-if="workspaceScanAction === editor.installPath" class="loading loading-spinner loading-xs" />
                {{ t("settings.addToForge") }}
              </button>
            </div>
          </div>
          <p v-else class="mt-3 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
            {{ t("settings.noDiscoveredEditors") }}
          </p>
        </div>

        <div class="rounded-lg border border-base-content/10 bg-base-300/35 p-4">
          <div class="flex items-center justify-between gap-3">
            <h3 class="font-black">{{ t("settings.discoveredProjects") }}</h3>
            <span class="text-xs font-bold text-base-content/45">{{ workspaceScan.projects.length }}</span>
          </div>
          <div v-if="workspaceScan.projects.length" class="mt-3 grid gap-2">
            <div v-for="project in workspaceScan.projects" :key="project.path" class="rounded-md border border-base-content/10 bg-base-100 p-3">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="truncate text-sm font-black">{{ project.name }}</p>
                  <p class="mt-1 truncate text-xs text-base-content/50">{{ project.path }}</p>
                  <p v-if="project.reason" class="mt-2 text-xs text-warning">{{ project.reason }}</p>
                </div>
                <span class="rounded px-2 py-1 text-[11px] font-black" :class="project.corrupt ? 'bg-warning/20 text-warning' : project.registered ? 'bg-success/15 text-success' : 'bg-primary/15 text-primary'">
                  {{ project.corrupt ? t("settings.corrupt") : project.registered ? t("settings.registered") : t("settings.newItem") }}
                </span>
              </div>
              <button
                v-if="!project.registered && !project.corrupt"
                class="btn btn-xs btn-primary mt-3"
                type="button"
                :disabled="!!workspaceScanAction || busy"
                @click="emit('registerDiscoveredProject', project)"
              >
                <span v-if="workspaceScanAction === project.path" class="loading loading-spinner loading-xs" />
                {{ t("settings.addToForge") }}
              </button>
            </div>
          </div>
          <p v-else class="mt-3 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
            {{ t("settings.noDiscoveredProjects") }}
          </p>
        </div>
      </div>

      <div v-else class="mt-5 rounded-lg border border-dashed border-base-content/10 bg-base-300/35 p-6 text-center">
        <h3 class="font-black">{{ t("dashboard.scanPendingTitle") }}</h3>
        <p class="mt-1 text-sm text-base-content/55">{{ t("dashboard.scanPendingBody") }}</p>
      </div>
    </section>
  </section>
</template>

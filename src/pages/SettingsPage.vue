<script setup lang="ts">
import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import PathField from "../components/PathField.vue";
import type {
  ActivityLogEntry,
  DiscoveredEditor,
  DiscoveredProject,
  PathTarget,
  ThemeName,
  WorkspaceDiagnostics,
  WorkspaceScan,
} from "../types";

type SettingsForm = {
  defaultInstallPath: string;
  defaultProjectPath: string;
  releaseRepositories: string[];
  githubToken: string;
  migrateExistingPaths: boolean;
};

type LegalDocumentKey = "source" | "brand" | "notice";
type LegalDocument = {
  title: string;
  body: string;
};

const props = defineProps<{
  settingsForm: SettingsForm;
  selectedLocale: string;
  selectedTheme: ThemeName;
  busy: boolean;
  workspaceScan: WorkspaceScan;
  workspaceScanLoaded: boolean;
  workspaceScanLoading: boolean;
  workspaceScanAction: string;
  workspaceScanError: string;
  diagnostics: WorkspaceDiagnostics | null;
  activityLog: ActivityLogEntry[];
  diagnosticsLoading: boolean;
}>();

const emit = defineEmits<{
  "update:selectedLocale": [value: string];
  "update:selectedTheme": [value: ThemeName];
  browse: [target: PathTarget];
  save: [];
  restoreDefaults: [];
  openSecurity: [];
  scanWorkspace: [];
  refreshDiagnostics: [];
  clearReleaseCache: [];
  registerDiscoveredEditor: [editor: DiscoveredEditor];
  registerDiscoveredProject: [project: DiscoveredProject];
}>();

const { t } = useI18n();
const confirmDialog = reactive({
  open: false,
  type: "restore" as "restore" | "repository",
  repositoryIndex: -1,
});
const legalDialog = reactive({
  open: false,
  loading: false,
  title: "",
  body: "",
  error: "",
});
function updateLocale(event: Event) {
  emit("update:selectedLocale", (event.target as HTMLSelectElement).value);
}

function updateTheme(event: Event) {
  emit("update:selectedTheme", (event.target as HTMLSelectElement).value as ThemeName);
}

function addReleaseRepository() {
  props.settingsForm.releaseRepositories.push("");
}

function removeReleaseRepository(index: number) {
  confirmDialog.open = true;
  confirmDialog.type = "repository";
  confirmDialog.repositoryIndex = index;
}

function requestRestoreDefaults() {
  confirmDialog.open = true;
  confirmDialog.type = "restore";
  confirmDialog.repositoryIndex = -1;
}

function closeConfirmDialog() {
  confirmDialog.open = false;
  confirmDialog.repositoryIndex = -1;
}

function confirmPendingAction() {
  if (confirmDialog.type === "repository" && confirmDialog.repositoryIndex >= 0) {
    props.settingsForm.releaseRepositories.splice(confirmDialog.repositoryIndex, 1);
    closeConfirmDialog();
    return;
  }

  closeConfirmDialog();
  emit("restoreDefaults");
}

function repositoryName(index: number) {
  return props.settingsForm.releaseRepositories[index]?.trim() || t("settings.extraRepository");
}

function confirmTitle() {
  return confirmDialog.type === "restore" ? t("settings.confirmRestoreTitle") : t("settings.confirmRemoveRepositoryTitle");
}

function confirmBody() {
  return confirmDialog.type === "restore"
    ? t("settings.confirmRestoreBody")
    : t("settings.confirmRemoveRepositoryBody", { name: repositoryName(confirmDialog.repositoryIndex) });
}

async function openLegalDocument(document: LegalDocumentKey) {
  legalDialog.open = true;
  legalDialog.loading = true;
  legalDialog.error = "";
  legalDialog.title = "";
  legalDialog.body = "";

  try {
    const legalDocument = await invoke<LegalDocument>("read_legal_document", { document });
    legalDialog.title = legalDocument.title;
    legalDialog.body = legalDocument.body;
  } catch (caught) {
    legalDialog.error = caught instanceof Error ? caught.message : String(caught);
  } finally {
    legalDialog.loading = false;
  }
}

function closeLegalDialog() {
  legalDialog.open = false;
}

function cacheAgeLabel(seconds?: number | null) {
  if (seconds == null) return t("settings.cacheNever");
  if (seconds < 60) return t("settings.cacheSeconds", { count: seconds });
  if (seconds < 3600) return t("settings.cacheMinutes", { count: Math.floor(seconds / 60) });
  return t("settings.cacheHours", { count: Math.floor(seconds / 3600) });
}

function activityDate(timestamp: number) {
  return new Date(timestamp * 1000).toLocaleString();
}

</script>

<template>
  <section class="grid gap-4 2xl:grid-cols-[minmax(0,1fr)_520px]">
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
        <label class="flex items-start gap-3 rounded-lg border border-base-content/10 bg-base-300/35 p-3 lg:col-span-2">
          <input v-model="settingsForm.migrateExistingPaths" class="toggle toggle-primary mt-0.5 cursor-pointer" type="checkbox" />
          <span class="grid gap-1">
            <span class="text-sm font-bold text-base-content/80">{{ t("settings.migrateExistingPaths") }}</span>
            <span class="text-xs text-base-content/55">{{ t("settings.migrateExistingPathsHint") }}</span>
          </span>
        </label>
        <label class="grid gap-2 lg:col-span-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.githubToken") }}</span>
          <input
            v-model="settingsForm.githubToken"
            class="input input-bordered border-base-content/10 bg-base-200"
            type="password"
            autocomplete="off"
            spellcheck="false"
            :placeholder="t('settings.githubTokenPlaceholder')"
          />
          <span class="text-xs text-base-content/50">{{ t("settings.githubTokenHint") }}</span>
          <div class="rounded-md border border-base-content/10 bg-base-content/[0.03] p-3 text-xs text-base-content/60">
            <p class="font-bold text-base-content/80">{{ t("settings.githubTokenTutorialTitle") }}</p>
            <ol class="mt-2 grid list-decimal gap-1 pl-4">
              <li>{{ t("settings.githubTokenStepOne") }}</li>
              <li>{{ t("settings.githubTokenStepTwo") }}</li>
              <li>{{ t("settings.githubTokenStepThree") }}</li>
            </ol>
            <div class="mt-3 flex flex-wrap gap-2">
              <a
                class="btn btn-xs border-base-content/10 bg-base-content/5"
                href="https://github.com/settings/personal-access-tokens/new"
                target="_blank"
                rel="noreferrer"
              >
                {{ t("settings.createGithubToken") }}
              </a>
              <a
                class="btn btn-xs border-base-content/10 bg-base-content/5"
                href="https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token"
                target="_blank"
                rel="noreferrer"
              >
                {{ t("settings.githubTokenDocs") }}
              </a>
            </div>
          </div>
        </label>
        <label class="grid gap-2 lg:col-span-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.releaseRepository") }}</span>
          <div class="grid gap-2">
            <div
              v-for="(_, index) in settingsForm.releaseRepositories"
              :key="index"
              class="flex gap-2"
            >
              <input
                v-model="settingsForm.releaseRepositories[index]"
                class="input input-bordered min-w-0 flex-1 border-base-content/10 bg-base-200"
                placeholder="godotengine/godot"
              />
              <button
                class="btn border-base-content/10 bg-base-content/5"
                type="button"
                @click="removeReleaseRepository(index)"
              >
                {{ t("common.remove") }}
              </button>
            </div>
            <p v-if="!settingsForm.releaseRepositories.length" class="rounded-md border border-dashed border-base-content/10 bg-base-content/[0.03] px-3 py-2 text-sm text-base-content/50">
              {{ t("settings.noExtraRepositories") }}
            </p>
            <button class="btn w-fit border-base-content/10 bg-base-content/5" type="button" @click="addReleaseRepository">
              {{ t("settings.addReleaseRepository") }}
            </button>
          </div>
          <span class="text-xs text-base-content/50">{{ t("settings.releaseRepositoryHint") }}</span>
        </label>
        <section class="grid gap-4 rounded-lg border border-base-content/10 bg-base-300/35 p-4 lg:col-span-2">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceScan") }}</p>
              <h3 class="mt-1 text-lg font-black">{{ t("settings.workspaceScanTitle") }}</h3>
              <p class="mt-1 text-sm text-base-content/60">{{ t("settings.workspaceScanBody") }}</p>
            </div>
            <button class="btn btn-sm btn-primary shrink-0" type="button" :disabled="workspaceScanLoading || busy" @click="emit('scanWorkspace')">
              <span v-if="workspaceScanLoading" class="loading loading-spinner loading-xs" />
              {{ t("settings.scanWorkspace") }}
            </button>
          </div>

          <p v-if="workspaceScanError" class="rounded-md border border-error/20 bg-error/10 px-3 py-2 text-sm text-error">
            {{ workspaceScanError }}
          </p>

          <div v-if="workspaceScanLoaded" class="grid gap-4 xl:grid-cols-2">
            <div class="rounded-lg border border-base-content/10 bg-base-100 p-3">
              <div class="flex items-center justify-between gap-3">
                <h4 class="font-black">{{ t("settings.discoveredEditors") }}</h4>
                <span class="text-xs font-bold text-base-content/45">{{ workspaceScan.editors.length }}</span>
              </div>
              <div v-if="workspaceScan.editors.length" class="mt-3 grid gap-2">
                <div v-for="editor in workspaceScan.editors" :key="editor.installPath + editor.executablePath" class="rounded-md border border-base-content/10 bg-base-300/45 p-3">
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

            <div class="rounded-lg border border-base-content/10 bg-base-100 p-3">
              <div class="flex items-center justify-between gap-3">
                <h4 class="font-black">{{ t("settings.discoveredProjects") }}</h4>
                <span class="text-xs font-bold text-base-content/45">{{ workspaceScan.projects.length }}</span>
              </div>
              <div v-if="workspaceScan.projects.length" class="mt-3 grid gap-2">
                <div v-for="project in workspaceScan.projects" :key="project.path" class="rounded-md border border-base-content/10 bg-base-300/45 p-3">
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
        </section>
      </div>
      <div class="mt-5 flex flex-col gap-2 border-t border-base-content/10 pt-5 sm:flex-row sm:justify-end">
        <button class="btn border-base-content/10 bg-base-content/5" type="button" :disabled="busy" @click="requestRestoreDefaults">
          {{ t("settings.restoreDefaults") }}
        </button>
        <button class="btn btn-primary" :disabled="busy">
          {{ t("settings.saveWorkspace") }}
        </button>
      </div>
    </form>

    <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
      <div>
        <p class="text-xs font-black uppercase text-primary">{{ t("settings.appearance") }}</p>
        <h2 class="mt-1 text-2xl font-black">{{ t("settings.preferences") }}</h2>
      </div>
      <div class="mt-5 grid gap-4 lg:grid-cols-2 2xl:grid-cols-2">
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.language") }}</span>
          <select :value="selectedLocale" class="select cursor-pointer select-bordered border-base-content/10 bg-base-200" @change="updateLocale">
            <option value="en">English</option>
            <option value="pt">Português</option>
          </select>
        </label>
        <label class="grid gap-2">
          <span class="text-sm font-bold text-base-content/80">{{ t("settings.theme") }}</span>
          <select :value="selectedTheme" class="select cursor-pointer select-bordered border-base-content/10 bg-base-200" @change="updateTheme">
            <option value="godotforge">{{ t("settings.darkTheme") }}</option>
            <option value="godotforge-light">{{ t("settings.lightTheme") }}</option>
          </select>
        </label>
        <div class="rounded-lg border border-warning/20 bg-warning/10 p-4 lg:col-span-2">
          <p class="text-xs font-black uppercase text-warning">{{ t("security.title") }}</p>
          <p class="mt-2 text-sm text-base-content/70">{{ t("security.body") }}</p>
          <button class="btn btn-sm btn-warning mt-4" type="button" @click="emit('openSecurity')">
            {{ t("security.openPolicy") }}
          </button>
        </div>
        <div class="rounded-lg border border-base-content/10 bg-base-300/45 p-4 lg:col-span-2">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("settings.diagnostics") }}</p>
              <h3 class="mt-1 text-lg font-black">{{ t("settings.systemReadiness") }}</h3>
              <p class="mt-2 text-sm text-base-content/60">{{ t("settings.systemReadinessBody") }}</p>
            </div>
            <button class="btn btn-sm border-base-content/10 bg-base-content/5" type="button" :disabled="diagnosticsLoading || busy" @click="emit('refreshDiagnostics')">
              <span v-if="diagnosticsLoading" class="loading loading-spinner loading-xs" />
              {{ t("common.refresh") }}
            </button>
          </div>
          <div v-if="diagnostics" class="mt-4 grid gap-2">
            <div v-for="check in diagnostics.checks" :key="check.key" class="rounded-md border border-base-content/10 bg-base-100 p-3">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="text-sm font-black">{{ check.label }}</p>
                  <p class="mt-1 break-words text-xs text-base-content/55">{{ check.detail }}</p>
                </div>
                <span class="rounded px-2 py-1 text-[11px] font-black" :class="check.ok ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'">
                  {{ check.ok ? t("settings.ready") : t("settings.needsAttention") }}
                </span>
              </div>
            </div>
          </div>
          <p v-else class="mt-4 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
            {{ t("settings.diagnosticsNotLoaded") }}
          </p>
        </div>
        <div class="rounded-lg border border-base-content/10 bg-base-300/45 p-4 lg:col-span-2">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("settings.releaseCache") }}</p>
              <h3 class="mt-1 text-lg font-black">{{ diagnostics?.cache.releaseCount ?? 0 }} {{ t("settings.cachedReleases") }}</h3>
              <p class="mt-2 break-all text-xs text-base-content/55">{{ diagnostics?.cache.path || t("settings.cacheNotCreated") }}</p>
              <p class="mt-1 text-xs text-base-content/55">
                {{ t("settings.cacheAge") }}: {{ cacheAgeLabel(diagnostics?.cache.ageSeconds) }}
              </p>
            </div>
            <button class="btn btn-sm btn-error btn-outline" type="button" :disabled="busy" @click="emit('clearReleaseCache')">
              {{ t("settings.clearReleaseCache") }}
            </button>
          </div>
        </div>
        <div class="rounded-lg border border-base-content/10 bg-base-300/45 p-4 lg:col-span-2">
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("settings.activity") }}</p>
              <h3 class="mt-1 text-lg font-black">{{ t("settings.recentActivity") }}</h3>
            </div>
            <span class="text-xs font-bold text-base-content/45">{{ activityLog.length }}</span>
          </div>
          <div v-if="activityLog.length" class="mt-4 grid max-h-72 gap-2 overflow-y-auto pr-1">
            <div v-for="entry in activityLog" :key="`${entry.timestamp}-${entry.message}`" class="rounded-md bg-base-100 px-3 py-2">
              <div class="flex items-start justify-between gap-3">
                <p class="text-sm font-bold text-base-content/80">{{ entry.message }}</p>
                <span class="rounded bg-base-content/5 px-2 py-1 text-[10px] font-black uppercase text-base-content/55">{{ entry.level }}</span>
              </div>
              <p class="mt-1 text-xs text-base-content/45">{{ activityDate(entry.timestamp) }}</p>
            </div>
          </div>
          <p v-else class="mt-4 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
            {{ t("settings.noActivity") }}
          </p>
        </div>
        <div class="rounded-lg border border-base-content/10 bg-base-300/45 p-4 lg:col-span-2">
          <p class="text-xs font-black uppercase text-primary">{{ t("settings.legal") }}</p>
          <h3 class="mt-1 text-lg font-black">{{ t("settings.licenses") }}</h3>
          <p class="mt-2 text-sm text-base-content/60">{{ t("settings.licensesBody") }}</p>
          <p class="mt-3 text-xs font-bold text-base-content/50">
            Copyright (c) 2026 ZEPHYRUS PROSPERITY - UNIPESSOAL LDA
          </p>
          <div class="mt-4 grid gap-2 text-xs">
            <button class="flex cursor-pointer items-center justify-between gap-3 rounded-md bg-base-100 px-3 py-2 text-left transition hover:bg-base-content/5" type="button" @click="openLegalDocument('source')">
              <span class="font-bold text-base-content/80">{{ t("settings.sourceLicense") }}</span>
              <span class="shrink-0 text-primary">{{ t("settings.readDocument") }}</span>
            </button>
            <button class="flex cursor-pointer items-center justify-between gap-3 rounded-md bg-base-100 px-3 py-2 text-left transition hover:bg-base-content/5" type="button" @click="openLegalDocument('brand')">
              <span class="font-bold text-base-content/80">{{ t("settings.brandLicense") }}</span>
              <span class="shrink-0 text-primary">{{ t("settings.readDocument") }}</span>
            </button>
            <button class="flex cursor-pointer items-center justify-between gap-3 rounded-md bg-base-100 px-3 py-2 text-left transition hover:bg-base-content/5" type="button" @click="openLegalDocument('notice')">
              <span class="font-bold text-base-content/80">{{ t("settings.attributionNotice") }}</span>
              <span class="shrink-0 text-primary">{{ t("settings.readDocument") }}</span>
            </button>
          </div>
        </div>
      </div>
    </aside>

    <dialog class="modal" :open="legalDialog.open">
      <div class="modal-box max-w-4xl border border-base-content/10 bg-base-100">
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-xs font-black uppercase text-primary">{{ t("settings.legal") }}</p>
            <h3 class="mt-1 text-xl font-black">{{ legalDialog.title || t("settings.licenses") }}</h3>
          </div>
          <button class="btn btn-ghost btn-sm" type="button" @click="closeLegalDialog">
            {{ t("common.close") }}
          </button>
        </div>
        <div class="mt-5 max-h-[65vh] overflow-y-auto rounded-lg border border-base-content/10 bg-base-300/45 p-4">
          <div v-if="legalDialog.loading" class="flex items-center gap-3 text-sm text-base-content/60">
            <span class="loading loading-spinner loading-sm" />
            {{ t("common.loadingLog") }}
          </div>
          <p v-else-if="legalDialog.error" class="text-sm text-error">{{ legalDialog.error }}</p>
          <pre v-else class="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-base-content/80">{{ legalDialog.body }}</pre>
        </div>
      </div>
      <form class="modal-backdrop" method="dialog" @submit.prevent="closeLegalDialog">
        <button>{{ t("common.close") }}</button>
      </form>
    </dialog>

    <dialog class="modal" :open="confirmDialog.open">
      <div class="modal-box border border-base-content/10 bg-base-100">
        <h3 class="text-xl font-black">{{ confirmTitle() }}</h3>
        <p class="mt-3 text-sm text-base-content/70">{{ confirmBody() }}</p>
        <div class="modal-action">
          <button class="btn border-base-content/10 bg-base-content/5" type="button" @click="closeConfirmDialog">
            {{ t("common.cancel") }}
          </button>
          <button class="btn btn-error" type="button" :disabled="busy" @click="confirmPendingAction">
            {{ confirmDialog.type === "restore" ? t("settings.restoreDefaults") : t("common.remove") }}
          </button>
        </div>
      </div>
      <form class="modal-backdrop" method="dialog" @submit.prevent="closeConfirmDialog">
        <button>{{ t("common.cancel") }}</button>
      </form>
    </dialog>
  </section>
</template>

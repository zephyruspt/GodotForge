<script setup lang="ts">
import { reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import PathField from "../components/PathField.vue";
import type { PathTarget, ThemeName } from "../types";

type SettingsForm = {
  defaultInstallPath: string;
  defaultProjectPath: string;
  releaseRepositories: string[];
  githubToken: string;
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

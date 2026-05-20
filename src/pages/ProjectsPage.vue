<script setup lang="ts">
import { useI18n } from "vue-i18n";
import PathField from "../components/PathField.vue";
import type {
  DiscoveredProject,
  GitBranch,
  GitLogEntry,
  GitStatus,
  GodotEditor,
  GodotProject,
  PathTarget,
  ProjectDetailTab,
  WorkspaceScan,
} from "../types";

type ProjectCreateForm = {
  name: string;
  rootPath: string;
  editorId: string;
};

type ProjectImportForm = {
  name: string;
  path: string;
  editorId: string;
};

defineProps<{
  projects: GodotProject[];
  sortedProjects: GodotProject[];
  editors: GodotEditor[];
  defaultEditor?: GodotEditor;
  activeProject?: GodotProject;
  projectEditor?: GodotEditor;
  projectPageOpen: boolean;
  projectDetailTab: ProjectDetailTab;
  projectSearch: string;
  newProject: ProjectCreateForm;
  importProjectForm: ProjectImportForm;
  defaultProjectPath: string;
  busyAction: string;
  workspaceScan: WorkspaceScan;
  workspaceScanLoaded: boolean;
  workspaceScanLoading: boolean;
  workspaceScanAction: string;
  workspaceScanError: string;
  gitStatus: GitStatus | null;
  projectGitStatuses: Record<string, GitStatus>;
  gitLoading: boolean;
  gitLog: GitLogEntry[];
  gitLogLoading: boolean;
  gitBranches: GitBranch[];
  branchName: string;
  remoteUrl: string;
  moveDestinationPath: string;
  projectInitials: (name: string) => string;
  editorLabel: (editorId?: string | null) => string;
  lastOpenedLabel: (value?: string | null) => string;
  gitStatusLabel: (status?: GitStatus | null, loading?: boolean) => string;
  gitBadgeText: () => string;
}>();

const emit = defineEmits<{
  closeProjectPage: [];
  openProjectPage: [projectId: string];
  navigateEditors: [];
  launchProject: [projectId: string];
  toggleFavorite: [projectId: string];
  requestRemoveProject: [projectId: string, closeProjectPage?: boolean];
  scanWorkspace: [];
  registerDiscoveredProject: [project: DiscoveredProject];
  createProject: [];
  importProject: [];
  browse: [target: PathTarget];
  loadGitStatus: [projectId: string];
  loadGitLog: [projectId: string];
  loadGitBranches: [projectId: string];
  initGit: [projectId: string];
  pushGitBranch: [projectId: string];
  checkoutGitBranch: [branch: string, projectId: string];
  createGitBranch: [projectId: string];
  saveGitRemote: [projectId: string];
  moveProject: [];
  "update:projectSearch": [value: string];
  "update:projectDetailTab": [value: ProjectDetailTab];
  "update:branchName": [value: string];
  "update:remoteUrl": [value: string];
  "update:moveDestinationPath": [value: string];
}>();

const { t } = useI18n();

function openGitTools(projectId: string, remote?: string | null) {
  emit("update:projectDetailTab", "git");
  emit("update:remoteUrl", remote || "");
  emit("loadGitStatus", projectId);
  emit("loadGitLog", projectId);
  emit("loadGitBranches", projectId);
}

function openProjectSettings(path: string) {
  emit("update:projectDetailTab", "settings");
  emit("update:moveDestinationPath", path);
}
</script>

<template>
  <section v-if="projectPageOpen && activeProject" class="grid gap-6">
    <div class="overflow-hidden rounded-xl border border-base-content/10 bg-base-100">
      <div class="relative grid min-h-72 content-end bg-[radial-gradient(circle_at_24%_20%,var(--color-primary)_0%,var(--color-secondary)_38%,var(--color-base-200)_100%)] p-6 lg:p-8">
        <button class="btn btn-sm absolute left-4 top-4 border-base-content/10 bg-base-content/10 text-base-content hover:bg-base-content/20" @click="emit('closeProjectPage')">
          {{ t("common.backToProjects") }}
        </button>
        <div class="max-w-4xl">
          <p class="text-xs font-black uppercase tracking-wide text-primary">{{ t("projectPage.pageLabel") }}</p>
          <h2 class="mt-2 break-words text-4xl font-black leading-none tracking-tight sm:text-5xl">{{ activeProject.name }}</h2>
          <p class="mt-3 break-all text-sm text-base-content/65">{{ activeProject.path }}</p>
          <div class="mt-5 flex flex-wrap gap-2">
            <span class="rounded bg-base-content/10 px-3 py-1 text-xs font-black text-base-content/90">{{ projectEditor?.version ?? t("common.noEditor") }}</span>
            <span class="rounded bg-base-content/10 px-3 py-1 text-xs font-black text-base-content/90">Git: {{ gitBadgeText() }}</span>
            <span class="rounded bg-base-content/10 px-3 py-1 text-xs font-black text-base-content/90">{{ activeProject.favorite ? t("common.favorite") : t("nav.library") }}</span>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-4 border-t border-base-content/10 p-4 lg:flex-row lg:items-center lg:justify-between">
        <div class="flex flex-wrap gap-2">
          <button
            class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
            :class="projectDetailTab === 'overview' ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
            @click="emit('update:projectDetailTab', 'overview')"
          >
            {{ t("projectPage.overview") }}
          </button>
          <button
            class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
            :class="projectDetailTab === 'git' ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
            @click="openGitTools(activeProject.id, gitStatus?.remote)"
          >
            Git
          </button>
          <button
            class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
            :class="projectDetailTab === 'settings' ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
            :disabled="!editors.length"
            @click="openProjectSettings(activeProject.path)"
          >
            {{ t("common.settings") }}
          </button>
        </div>
        <div class="flex flex-wrap gap-2">
          <button class="btn btn-sm btn-primary" :disabled="!!busyAction || !editors.length" @click="emit('launchProject', activeProject.id)">{{ t("common.launchProject") }}</button>
          <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction" @click="emit('toggleFavorite', activeProject.id)">
            {{ activeProject.favorite ? t("common.removeFavorite") : t("common.favorite") }}
          </button>
        </div>
      </div>
    </div>

    <section v-if="projectDetailTab === 'overview'" class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
      <div class="grid gap-4 md:grid-cols-3">
        <div class="rounded-xl border border-base-content/10 bg-base-100 p-5">
          <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.engine") }}</p>
          <strong class="mt-2 block text-2xl">{{ projectEditor?.version ?? t("common.noEditor") }}</strong>
          <p class="mt-1 truncate text-xs text-base-content/50">{{ projectEditor?.executablePath || t("common.configureInstallation") }}</p>
        </div>
        <div class="rounded-xl border border-base-content/10 bg-base-100 p-5">
          <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.lastOpened") }}</p>
          <strong class="mt-2 block text-xl">{{ lastOpenedLabel(activeProject.lastOpened) }}</strong>
          <p class="mt-1 text-xs text-base-content/50">{{ t("projectPage.lastOpenedHint") }}</p>
        </div>
        <div class="rounded-xl border border-base-content/10 bg-base-100 p-5">
          <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.sourceControl") }}</p>
          <strong class="mt-2 block text-xl">{{ gitBadgeText() }}</strong>
          <p class="mt-1 text-xs text-base-content/50">{{ gitStatus?.summary || t("projectPage.statusNotLoaded") }}</p>
        </div>
      </div>

      <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("projectPage.quickActions") }}</p>
        <div class="mt-4 grid gap-2">
          <button class="btn btn-primary" :disabled="!!busyAction || !editors.length" @click="emit('launchProject', activeProject.id)">{{ t("common.openInEditor") }}</button>
          <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" @click="openGitTools(activeProject.id, gitStatus?.remote)">
            {{ t("projectPage.gitManage") }}
          </button>
          <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!editors.length" @click="openProjectSettings(activeProject.path)">
            {{ t("common.configureProject") }}
          </button>
        </div>
      </aside>
    </section>

    <section v-if="projectDetailTab === 'git'" class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_380px]">
      <div class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <p class="text-xs font-black uppercase text-primary">{{ t("git.projectGit") }}</p>
            <h3 class="mt-1 text-2xl font-black">{{ gitBadgeText() }}</h3>
            <p class="mt-1 text-sm text-base-content/50">{{ gitStatus?.summary || t("projectPage.statusNotLoadedYet") }}</p>
          </div>
          <div class="flex flex-wrap gap-2">
            <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="gitLoading" @click="openGitTools(activeProject.id, gitStatus?.remote)">
              {{ t("common.refresh") }}
            </button>
            <button
              class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10"
              :disabled="!gitStatus?.isRepo || !!busyAction"
              @click="emit('pushGitBranch', activeProject.id)"
            >
              {{ t("git.pushBranch") }}
            </button>
            <button
              v-if="gitStatus?.available && !gitStatus?.isRepo"
              class="btn btn-sm btn-primary"
              :disabled="gitLoading || !!busyAction"
              @click="emit('initGit', activeProject.id)"
            >
              {{ t("git.initializeGit") }}
            </button>
          </div>
        </div>

        <div class="mt-5 grid gap-3 md:grid-cols-4">
          <div class="rounded-lg bg-base-300/60 p-4">
            <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.branch") }}</p>
            <strong class="mt-2 block truncate">{{ gitStatus?.branch || "n/a" }}</strong>
          </div>
          <div class="rounded-lg bg-base-300/60 p-4">
            <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.changes") }}</p>
            <strong class="mt-2 block text-2xl">{{ gitStatus?.changedFiles ?? 0 }}</strong>
          </div>
          <div class="rounded-lg bg-base-300/60 p-4">
            <p class="text-xs font-black uppercase text-base-content/50">{{ t("common.untracked") }}</p>
            <strong class="mt-2 block text-2xl">{{ gitStatus?.untrackedFiles ?? 0 }}</strong>
          </div>
          <div class="rounded-lg bg-base-300/60 p-4">
            <p class="text-xs font-black uppercase text-base-content/50">Remote</p>
            <strong class="mt-2 block truncate">{{ gitStatus?.remote ? "origin" : "n/a" }}</strong>
          </div>
        </div>

        <div class="mt-5 rounded-lg bg-base-300/60 p-4">
          <p class="text-xs font-black uppercase text-base-content/50">Origin URL</p>
          <p class="mt-2 break-all text-sm text-base-content/80">{{ gitStatus?.remote || t("git.noOrigin") }}</p>
        </div>

        <div class="mt-5 grid gap-4 lg:grid-cols-2">
          <div class="rounded-lg border border-base-content/10 bg-base-300/60 p-4">
            <div class="flex items-center justify-between gap-3">
              <p class="text-xs font-black uppercase text-base-content/50">{{ t("git.branches") }}</p>
              <span class="text-xs text-base-content/50">{{ t("git.localBranches", { count: gitBranches.length }) }}</span>
            </div>
            <div v-if="gitBranches.length" class="mt-3 grid gap-2">
              <button
                v-for="branch in gitBranches"
                :key="branch.name"
                class="flex items-center justify-between rounded-md px-3 py-2 text-left text-sm transition"
                :class="branch.current ? 'bg-primary/15 text-primary ring-1 ring-primary/20' : 'bg-base-content/5 text-base-content/80 hover:bg-base-content/10'"
                :disabled="branch.current || !!busyAction"
                @click="emit('checkoutGitBranch', branch.name, activeProject.id)"
              >
                <span class="truncate">{{ branch.name }}</span>
                <span class="text-xs">{{ branch.current ? t("common.current") : t("common.checkout") }}</span>
              </button>
            </div>
            <p v-else class="mt-3 text-sm text-base-content/50">{{ t("git.noLocalBranches") }}</p>
          </div>

          <div class="grid gap-4">
            <form class="rounded-lg border border-base-content/10 bg-base-300/60 p-4" @submit.prevent="emit('createGitBranch', activeProject.id)">
              <p class="text-xs font-black uppercase text-base-content/50">{{ t("git.createBranch") }}</p>
              <div class="mt-3 flex flex-col gap-2 sm:flex-row">
                <input
                  :value="branchName"
                  class="input input-bordered input-sm border-base-content/10 bg-base-200"
                  :placeholder="t('git.branchNamePlaceholder')"
                  @input="emit('update:branchName', ($event.target as HTMLInputElement).value)"
                />
                <button class="btn btn-primary btn-sm" :disabled="!!busyAction || !branchName.trim()">{{ t("common.create") }}</button>
              </div>
            </form>

            <form class="rounded-lg border border-base-content/10 bg-base-300/60 p-4" @submit.prevent="emit('saveGitRemote', activeProject.id)">
              <p class="text-xs font-black uppercase text-base-content/50">Remote origin</p>
              <div class="mt-3 grid gap-2">
                <input
                  :value="remoteUrl"
                  class="input input-bordered input-sm border-base-content/10 bg-base-200"
                  placeholder="git@github.com:user/repo.git"
                  @input="emit('update:remoteUrl', ($event.target as HTMLInputElement).value)"
                />
                <button class="btn btn-primary btn-sm w-fit" :disabled="!!busyAction || !remoteUrl.trim()">{{ t("common.saveRemote") }}</button>
              </div>
            </form>
          </div>
        </div>
      </div>

      <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("projectPage.recentLog") }}</p>
        <div v-if="gitLogLoading" class="mt-4 text-sm text-base-content/50">{{ t("common.loadingLog") }}</div>
        <div v-else-if="gitLog.length" class="mt-4 grid gap-3">
          <div v-for="entry in gitLog" :key="entry.hash" class="rounded-lg bg-base-300/60 p-3">
            <div class="flex items-center justify-between gap-3">
              <strong class="truncate text-sm">{{ entry.subject }}</strong>
              <span class="text-xs text-primary">{{ entry.hash }}</span>
            </div>
            <p class="mt-1 text-xs text-base-content/50">{{ entry.author }} - {{ entry.relativeDate }}</p>
          </div>
        </div>
        <p v-else class="mt-4 text-sm text-base-content/50">{{ t("projectPage.noCommits") }}</p>
      </aside>
    </section>

    <section v-if="projectDetailTab === 'settings'" class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
      <form class="rounded-xl border border-base-content/10 bg-base-100 p-5" @submit.prevent="emit('moveProject')">
        <p class="text-xs font-black uppercase text-primary">Project</p>
        <h3 class="mt-1 text-2xl font-black">{{ t("projectPage.projectSettings") }}</h3>
        <p v-if="!editors.length" class="mt-2 rounded-md border border-primary/20 bg-primary/10 px-3 py-2 text-sm text-base-content/70">
          {{ t("projects.editorRequired") }}
        </p>
        <div class="mt-5 grid gap-4">
          <label class="grid gap-2">
            <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.pathCurrent") }}</span>
            <input class="input input-bordered border-base-content/10 bg-base-200" :value="activeProject.path" disabled />
          </label>
          <div class="grid gap-2">
            <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.newDestinationPath") }}</span>
            <PathField
              :model-value="moveDestinationPath"
              required
              :disabled="!editors.length"
              :button-label="t('common.browse')"
              @update:model-value="emit('update:moveDestinationPath', $event)"
              @browse="emit('browse', 'moveProjectDestination')"
            />
          </div>
          <button class="btn btn-primary w-fit" :disabled="!!busyAction || !editors.length || moveDestinationPath === activeProject.path">
            {{ t("projectPage.moveProject") }}
          </button>
        </div>
      </form>

      <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">{{ t("projectPage.dangerZone") }}</p>
        <p class="mt-2 text-sm text-base-content/50">{{ t("projectPage.dangerBody") }}</p>
        <button class="btn btn-error btn-outline mt-4" :disabled="!!busyAction" @click="emit('requestRemoveProject', activeProject.id, true)">
          {{ t("common.removeFromLibrary") }}
        </button>
      </aside>
    </section>
  </section>

  <template v-else>
    <section v-if="projects.length" class="grid gap-3 md:grid-cols-3">
      <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-bold uppercase text-base-content/50">{{ t("nav.projects") }}</p>
        <div class="mt-2 flex items-end justify-between">
          <strong class="text-4xl font-black">{{ projects.length }}</strong>
          <span class="text-xs text-base-content/50">{{ sortedProjects.length }} {{ t("projects.filtered") }}</span>
        </div>
      </div>
      <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-bold uppercase text-base-content/50">Engine versions</p>
        <div class="mt-2 flex items-end justify-between">
          <strong class="text-4xl font-black">{{ editors.length }}</strong>
          <span class="max-w-40 truncate text-xs text-base-content/50">{{ defaultEditor?.version || "No default" }}</span>
        </div>
      </div>
      <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
        <p class="text-xs font-bold uppercase text-base-content/50">Project path</p>
        <strong class="mt-2 block truncate text-lg">{{ defaultProjectPath || "Not configured" }}</strong>
        <p class="mt-1 text-xs text-base-content/50">Default workspace</p>
      </div>
    </section>

    <section class="grid gap-6" :class="{ 'xl:grid-cols-[minmax(0,1fr)_380px]': projects.length }">
      <div class="grid gap-4">
        <div v-if="projects.length" class="flex flex-col gap-3 rounded-lg border border-base-content/10 bg-base-100 p-4 lg:flex-row lg:items-center">
          <div class="flex-1">
            <p class="text-xs font-black uppercase text-primary">{{ t("sections.projectsTitle") }}</p>
            <h2 class="text-2xl font-black">{{ t("projects.myProjects") }}</h2>
          </div>
          <input
            :value="projectSearch"
            class="input input-bordered h-10 border-base-content/10 bg-base-200 text-sm lg:w-80"
            :placeholder="t('projects.searchPlaceholder')"
            @input="emit('update:projectSearch', ($event.target as HTMLInputElement).value)"
          />
        </div>

        <div v-if="!editors.length" class="rounded-xl border border-primary/25 bg-primary/10 p-4">
          <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div class="min-w-0">
              <p class="text-xs font-black uppercase text-primary">{{ t("common.noEngineInstalled") }}</p>
              <h3 class="mt-1 text-xl font-black">{{ t("projects.noEditorCtaTitle") }}</h3>
              <p class="mt-1 text-sm text-base-content/65">{{ t("projects.noEditorCtaBody") }}</p>
            </div>
            <button class="btn btn-primary shrink-0" type="button" @click="emit('navigateEditors')">
              {{ t("projects.goInstallEditor") }}
            </button>
          </div>
        </div>

        <div v-if="workspaceScanLoading || workspaceScanError || workspaceScan.projects.length" class="rounded-xl border border-base-content/10 bg-base-100 p-4">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceScan") }}</p>
              <h3 class="mt-1 text-lg font-black">{{ t("settings.discoveredProjects") }}</h3>
              <p class="mt-1 text-sm text-base-content/55">{{ t("settings.workspaceScanBody") }}</p>
            </div>
            <button class="btn btn-sm btn-primary shrink-0" type="button" :disabled="workspaceScanLoading || !!busyAction" @click="emit('scanWorkspace')">
              <span v-if="workspaceScanLoading" class="loading loading-spinner loading-xs" />
              {{ t("settings.scanWorkspace") }}
            </button>
          </div>
          <p v-if="workspaceScanError" class="mt-3 rounded-md border border-error/20 bg-error/10 px-3 py-2 text-sm text-error">
            {{ workspaceScanError }}
          </p>
          <div v-if="workspaceScanLoaded && workspaceScan.projects.length" class="mt-4 grid gap-2 lg:grid-cols-2">
            <div v-for="project in workspaceScan.projects" :key="project.path" class="rounded-lg border border-base-content/10 bg-base-300/45 p-3">
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
                :disabled="!!workspaceScanAction || !!busyAction || !editors.length"
                @click="emit('registerDiscoveredProject', project)"
              >
                <span v-if="workspaceScanAction === project.path" class="loading loading-spinner loading-xs" />
                {{ t("settings.addToForge") }}
              </button>
            </div>
          </div>
        </div>

        <div v-if="projects.length && sortedProjects.length" class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
          <article
            v-for="project in sortedProjects"
            :key="project.id"
            class="group cursor-pointer overflow-hidden rounded-xl border border-base-content/10 bg-base-100 transition hover:-translate-y-0.5 hover:border-primary/50 hover:bg-base-200"
            :class="{ 'border-primary/70 ring-1 ring-primary/30': activeProject?.id === project.id }"
            @click="emit('openProjectPage', project.id)"
          >
            <div class="relative grid aspect-[16/9] place-items-center bg-[radial-gradient(circle_at_30%_20%,var(--color-primary)_0%,var(--color-secondary)_38%,var(--color-base-200)_100%)]">
              <span class="text-5xl font-black text-primary-content/90">{{ projectInitials(project.name) || "GD" }}</span>
              <button
                class="absolute right-3 top-3 rounded bg-base-300/70 px-2 py-1 text-lg text-warning backdrop-blur"
                @click.stop="emit('toggleFavorite', project.id)"
              >
                {{ project.favorite ? "★" : "☆" }}
              </button>
            </div>
            <div class="grid gap-3 p-4">
              <div>
                <h3 class="truncate text-lg font-black">{{ project.name }}</h3>
                <p class="mt-1 truncate text-xs text-base-content/50">{{ project.path }}</p>
              </div>
              <div class="flex items-center justify-between gap-3">
                <span class="rounded bg-base-content/5 px-2 py-1 text-xs font-bold text-base-content/80">{{ editorLabel(project.editorId) }}</span>
                <span class="rounded px-2 py-1 text-[11px] font-black uppercase" :class="projectGitStatuses[project.id]?.isRepo ? 'bg-success/15 text-success' : 'bg-base-content/5 text-base-content/50'">
                  Git: {{ gitStatusLabel(projectGitStatuses[project.id]) }}
                </span>
                <button class="btn btn-primary btn-xs" :disabled="!!busyAction || !editors.length" @click.stop="emit('launchProject', project.id)">
                  Launch
                </button>
              </div>
            </div>
          </article>
        </div>

        <div v-else-if="projects.length" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
          <h3 class="text-2xl font-black">{{ t("common.noProjects") }}</h3>
          <p class="mt-2 text-base-content/50">{{ t("projects.createNewOrImport") }}</p>
        </div>

        <div class="grid gap-4 lg:grid-cols-2">
          <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="emit('createProject')">
            <div class="mb-4 flex items-center justify-between">
              <h3 class="font-black">{{ t("projects.newProject") }}</h3>
              <span class="text-xs text-base-content/50">{{ t("common.create") }}</span>
            </div>
            <fieldset class="flex flex-col gap-3" :disabled="!editors.length">
              <input v-model="newProject.name" class="input input-bordered w-full border-base-content/10 bg-base-200" required :placeholder="t('projects.projectName')" />
              <PathField
                v-model="newProject.rootPath"
                required
                :placeholder="t('projects.baseFolder')"
                :button-label="t('common.browse')"
                @browse="emit('browse', 'newProjectRoot')"
              />
              <select v-model="newProject.editorId" class="select w-full cursor-pointer select-bordered border-base-content/10 bg-base-200 disabled:cursor-not-allowed disabled:opacity-60" :disabled="!editors.length">
                <option value="">{{ editors.length ? t("projects.useDefaultEditor") : t("projects.noEditorSelect") }}</option>
                <option v-for="editor in editors" :key="editor.id" :value="editor.id">
                  {{ editor.name }} {{ editor.version }}
                </option>
              </select>
              <button class="btn btn-primary w-full" :disabled="!!busyAction || !editors.length">{{ t("projects.createProject") }}</button>
            </fieldset>
          </form>

          <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="emit('importProject')">
            <div class="mb-4 flex items-center justify-between">
              <h3 class="font-black">{{ t("projects.importProject") }}</h3>
              <span class="text-xs text-base-content/50">{{ t("common.import") }}</span>
            </div>
            <fieldset class="flex flex-col gap-3" :disabled="!editors.length">
              <input v-model="importProjectForm.name" class="input input-bordered w-full border-base-content/10 bg-base-200" :placeholder="t('projects.optionalName')" />
              <PathField
                v-model="importProjectForm.path"
                required
                :placeholder="t('projects.projectPathPlaceholder')"
                :button-label="t('common.browse')"
                @browse="emit('browse', 'importProjectPath')"
              />
              <select v-model="importProjectForm.editorId" class="select w-full cursor-pointer select-bordered border-base-content/10 bg-base-200 disabled:cursor-not-allowed disabled:opacity-60" :disabled="!editors.length">
                <option value="">{{ editors.length ? t("projects.useDefaultEditor") : t("projects.noEditorSelect") }}</option>
                <option v-for="editor in editors" :key="editor.id" :value="editor.id">
                  {{ editor.name }} {{ editor.version }}
                </option>
              </select>
              <button class="btn w-full border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction || !editors.length">{{ t("common.import") }}</button>
            </fieldset>
          </form>
        </div>
      </div>

      <aside v-if="projects.length" class="h-fit rounded-xl border border-base-content/10 bg-base-100 p-5">
        <p class="text-xs font-black uppercase text-primary">Inspector</p>
        <template v-if="activeProject">
          <h2 class="mt-2 text-3xl font-black">{{ activeProject.name }}</h2>
          <p class="mt-2 break-all text-sm text-base-content/50">{{ activeProject.path }}</p>
          <div class="my-5 h-px bg-base-content/10" />
          <div class="grid gap-3 text-sm">
            <div class="flex justify-between gap-3">
              <span class="text-base-content/50">{{ t("common.engine") }}</span>
              <strong>{{ projectEditor?.version ?? t("common.noEditor") }}</strong>
            </div>
            <div class="flex justify-between gap-3">
              <span class="text-base-content/50">{{ t("common.lastOpened") }}</span>
              <strong class="text-right">{{ lastOpenedLabel(activeProject.lastOpened) }}</strong>
            </div>
            <div class="flex justify-between gap-3">
              <span class="text-base-content/50">{{ t("common.favorite") }}</span>
              <strong>{{ activeProject.favorite ? t("common.yes") : t("common.no") }}</strong>
            </div>
          </div>
          <div class="mt-5 rounded-lg border border-base-content/10 bg-base-300/60 p-3">
            <div class="flex items-center justify-between gap-3">
              <span class="text-xs font-black uppercase text-base-content/50">Git</span>
              <span class="rounded bg-base-content/10 px-2 py-1 text-[11px] font-black text-base-content/80">{{ gitBadgeText() }}</span>
            </div>
            <p class="mt-2 text-sm text-base-content/65">{{ gitStatus?.summary || t("projectPage.selectProjectGit") }}</p>
            <div v-if="gitStatus?.isRepo" class="mt-3 grid grid-cols-2 gap-2 text-xs">
              <div class="rounded bg-base-content/5 p-2">
                <span class="block text-base-content/50">Branch</span>
                <strong class="block truncate">{{ gitStatus.branch || "n/a" }}</strong>
              </div>
              <div class="rounded bg-base-content/5 p-2">
                <span class="block text-base-content/50">{{ t("common.changes") }}</span>
                <strong>{{ gitStatus.changedFiles }} / {{ gitStatus.untrackedFiles }}</strong>
              </div>
            </div>
            <p v-if="gitStatus?.remote" class="mt-3 truncate text-xs text-base-content/50">{{ gitStatus.remote }}</p>
            <div class="mt-3 flex flex-wrap gap-2">
              <button
                class="btn btn-xs border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10"
                :disabled="gitLoading"
                @click="emit('loadGitStatus', activeProject.id)"
              >
                {{ t("common.refreshGit") }}
              </button>
              <button
                v-if="gitStatus?.available && !gitStatus?.isRepo"
                class="btn btn-primary btn-xs mt-3"
                :disabled="gitLoading || !!busyAction"
                @click="emit('initGit', activeProject.id)"
              >
                {{ t("git.initializeGit") }}
              </button>
            </div>
          </div>
          <div class="mt-6 grid gap-2">
            <button class="btn btn-primary" :disabled="!!busyAction" @click="emit('launchProject', activeProject.id)">{{ t("common.launchProject") }}</button>
            <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction" @click="emit('toggleFavorite', activeProject.id)">
              {{ activeProject.favorite ? t("common.removeFavorite") : t("common.addFavorite") }}
            </button>
            <button class="btn btn-error btn-outline" :disabled="!!busyAction" @click="emit('requestRemoveProject', activeProject.id)">{{ t("common.removeFromLibrary") }}</button>
          </div>
        </template>
        <p v-else class="mt-3 text-sm text-base-content/50">{{ t("projectPage.selectProject") }}</p>
      </aside>
    </section>
  </template>
</template>

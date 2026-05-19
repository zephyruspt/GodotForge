<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import appLogo from "./assets/godot-forge-logo.png";

type GodotEditor = {
  id: string;
  name: string;
  version: string;
  executablePath: string;
  installPath: string;
  architecture: string;
  isDefault: boolean;
};

type GodotProject = {
  id: string;
  name: string;
  path: string;
  editorId?: string | null;
  favorite: boolean;
  lastOpened?: string | null;
};

type HubState = {
  editors: GodotEditor[];
  projects: GodotProject[];
  settings: {
    defaultInstallPath: string;
    defaultProjectPath: string;
  };
};

type GodotReleaseAsset = {
  id: number;
  name: string;
  size: number;
  browserDownloadUrl: string;
};

type GodotRelease = {
  id: number;
  name?: string | null;
  tagName: string;
  prerelease: boolean;
  publishedAt?: string | null;
  htmlUrl: string;
  assets: GodotReleaseAsset[];
};

type SystemProfile = {
  os: string;
  arch: string;
  godotPlatform: string;
};

type GitStatus = {
  available: boolean;
  isRepo: boolean;
  branch?: string | null;
  remote?: string | null;
  changedFiles: number;
  untrackedFiles: number;
  summary: string;
};

type GitLogEntry = {
  hash: string;
  author: string;
  relativeDate: string;
  subject: string;
};

type GitBranch = {
  name: string;
  current: boolean;
};

type ThemeName = "godotforge" | "godotforge-light";

const localeStorageKey = "godot-forge-locale";
const themeStorageKey = "godot-forge-theme";
const themeNames: ThemeName[] = ["godotforge", "godotforge-light"];

const state = reactive<HubState>({
  editors: [],
  projects: [],
  settings: {
    defaultInstallPath: "",
    defaultProjectPath: "",
  },
});

const { locale, t } = useI18n();
const savedLocale = localStorage.getItem(localeStorageKey);
const savedTheme = localStorage.getItem(themeStorageKey);

if (savedLocale === "en" || savedLocale === "pt") {
  locale.value = savedLocale;
}

const sections = ["projects", "editors", "releases", "settings"] as const;
const activeSection = ref<(typeof sections)[number]>("projects");
const activeProjectId = ref("");
const busyAction = ref("");
const error = ref("");
const status = ref("");
const loading = ref(true);
const projectSearch = ref("");
const dismissWelcome = ref(false);
const releases = ref<GodotRelease[]>([]);
const releasesLoaded = ref(false);
const releaseQuery = ref("");
const releaseFlavor = ref<"standard" | "dotnet">("standard");
const selectedLocale = computed({
  get: () => locale.value,
  set: (value: string) => {
    locale.value = value;
    localStorage.setItem(localeStorageKey, value);
  },
});
const selectedTheme = ref<ThemeName>(themeNames.includes(savedTheme as ThemeName) ? (savedTheme as ThemeName) : "godotforge");
const downloadTarget = ref("");
const installedAssetKeys = ref<string[]>([]);
const gitStatus = ref<GitStatus | null>(null);
const projectGitStatuses = ref<Record<string, GitStatus>>({});
const gitLoading = ref(false);
const projectPageOpen = ref(false);
const projectDetailTab = ref<"overview" | "git" | "settings">("overview");
const moveDestinationPath = ref("");
const gitLog = ref<GitLogEntry[]>([]);
const gitLogLoading = ref(false);
const gitBranches = ref<GitBranch[]>([]);
const branchName = ref("");
const remoteUrl = ref("");
const systemProfile = reactive<SystemProfile>({
  os: "unknown",
  arch: "unknown",
  godotPlatform: "linux",
});

const newProject = reactive({
  name: "New Game",
  rootPath: "",
  editorId: "",
});

const importProjectForm = reactive({
  name: "",
  path: "",
  editorId: "",
});

const newEditor = reactive({
  name: "Godot",
  version: "4.3",
  executablePath: "",
  installPath: "",
  architecture: "x86_64",
  makeDefault: true,
});

const settingsForm = reactive({
  defaultInstallPath: "",
  defaultProjectPath: "",
});

const defaultEditor = computed(() => state.editors.find((editor) => editor.isDefault));
const hasOnboarding = computed(() => !dismissWelcome.value && (!state.editors.length || !state.projects.length));

const sortedProjects = computed(() => {
  const query = projectSearch.value.trim().toLowerCase();

  return [...state.projects]
    .filter((project) => {
      if (!query) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(query);
    })
    .sort((a, b) => Number(b.favorite) - Number(a.favorite) || a.name.localeCompare(b.name));
});

const activeProject = computed(
  () => state.projects.find((project) => project.id === activeProjectId.value) ?? sortedProjects.value[0],
);

const projectEditor = computed(() => {
  const project = activeProject.value;
  return state.editors.find((editor) => editor.id === project?.editorId) ?? defaultEditor.value;
});

const filteredReleases = computed(() => {
  const query = releaseQuery.value.trim().toLowerCase();
  if (!query) return releases.value;

  return releases.value.filter((release) => {
    const haystack = `${release.tagName} ${release.name ?? ""} ${release.assets.map((asset) => asset.name).join(" ")}`;
    return haystack.toLowerCase().includes(query);
  });
});

function applyState(nextState: HubState) {
  state.editors = nextState.editors;
  state.projects = nextState.projects;
  state.settings = nextState.settings;
  settingsForm.defaultInstallPath = nextState.settings.defaultInstallPath;
  settingsForm.defaultProjectPath = nextState.settings.defaultProjectPath;
  newProject.rootPath ||= nextState.settings.defaultProjectPath;
  newEditor.installPath ||= nextState.settings.defaultInstallPath;

  if (!activeProjectId.value || !state.projects.some((project) => project.id === activeProjectId.value)) {
    activeProjectId.value = state.projects[0]?.id ?? "";
  }
}

async function runAction(label: string, action: () => Promise<HubState>) {
  busyAction.value = label;
  error.value = "";
  status.value = "";

  try {
    applyState(await action());
    status.value = label;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function loadState() {
  loading.value = true;
  error.value = "";

  try {
    applyState(await invoke<HubState>("load_hub_state"));
    if (activeProjectId.value) {
      await loadGitStatus(activeProjectId.value);
    }
    loadProjectGitStatuses();
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    loading.value = false;
  }
}

async function loadProjectGitStatuses() {
  const entries = await Promise.all(
    state.projects.map(async (project) => {
      try {
        const status = await invoke<GitStatus>("get_project_git_status", { projectId: project.id });
        return [project.id, status] as const;
      } catch {
        return [project.id, null] as const;
      }
    }),
  );

  projectGitStatuses.value = entries.reduce<Record<string, GitStatus>>((accumulator, [projectId, status]) => {
    if (status) accumulator[projectId] = status;
    return accumulator;
  }, {});
}

async function loadGitStatus(projectId = activeProjectId.value) {
  if (!projectId) {
    gitStatus.value = null;
    return;
  }

  gitLoading.value = true;

  try {
    gitStatus.value = await invoke<GitStatus>("get_project_git_status", { projectId });
    projectGitStatuses.value = {
      ...projectGitStatuses.value,
      [projectId]: gitStatus.value,
    };
  } catch (caught) {
    gitStatus.value = null;
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    gitLoading.value = false;
  }
}

async function loadGitLog(projectId = activeProjectId.value) {
  if (!projectId) {
    gitLog.value = [];
    return;
  }

  gitLogLoading.value = true;

  try {
    gitLog.value = await invoke<GitLogEntry[]>("get_project_git_log", { projectId });
  } catch (caught) {
    gitLog.value = [];
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    gitLogLoading.value = false;
  }
}

async function loadGitBranches(projectId = activeProjectId.value) {
  if (!projectId) {
    gitBranches.value = [];
    return;
  }

  try {
    gitBranches.value = await invoke<GitBranch[]>("list_project_git_branches", { projectId });
  } catch {
    gitBranches.value = [];
  }
}

async function initGit(projectId = activeProjectId.value) {
  if (!projectId) return;

  gitLoading.value = true;
  busyAction.value = t("git.initializing");
  error.value = "";
  status.value = "";

  try {
    gitStatus.value = await invoke<GitStatus>("init_project_git", { projectId });
    projectGitStatuses.value = {
      ...projectGitStatuses.value,
      [projectId]: gitStatus.value,
    };
    status.value = t("git.initialized");
    await loadGitBranches(projectId);
    await loadGitLog(projectId);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
    gitLoading.value = false;
  }
}

async function createGitBranch(projectId = activeProjectId.value) {
  if (!projectId) return;

  busyAction.value = t("git.creatingBranch");
  error.value = "";
  status.value = "";

  try {
    gitStatus.value = await invoke<GitStatus>("create_project_git_branch", {
      request: { projectId, branchName: branchName.value },
    });
    status.value = t("git.branchCreated");
    branchName.value = "";
    await loadGitBranches(projectId);
    await loadGitLog(projectId);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function checkoutGitBranch(branch: string, projectId = activeProjectId.value) {
  if (!projectId) return;

  busyAction.value = t("git.switchingBranch");
  error.value = "";
  status.value = "";

  try {
    gitStatus.value = await invoke<GitStatus>("checkout_project_git_branch", {
      request: { projectId, branchName: branch },
    });
    status.value = t("git.activeBranch", { branch });
    await loadGitBranches(projectId);
    await loadGitLog(projectId);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function saveGitRemote(projectId = activeProjectId.value) {
  if (!projectId) return;

  busyAction.value = t("git.savingRemote");
  error.value = "";
  status.value = "";

  try {
    gitStatus.value = await invoke<GitStatus>("set_project_git_remote", {
      request: { projectId, remoteUrl: remoteUrl.value },
    });
    status.value = t("git.remoteUpdated");
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function pushGitBranch(projectId = activeProjectId.value) {
  if (!projectId) return;

  busyAction.value = t("git.pushing");
  error.value = "";
  status.value = "";

  try {
    gitStatus.value = await invoke<GitStatus>("push_project_git_branch", { projectId });
    status.value = t("git.pushCompleted");
    await loadGitStatus(projectId);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function loadSystemProfile() {
  try {
    const profile = await invoke<SystemProfile>("detect_system_profile");
    systemProfile.os = profile.os;
    systemProfile.arch = profile.arch;
    systemProfile.godotPlatform = profile.godotPlatform;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function loadReleases() {
  busyAction.value = t("status.fetchingReleases");
  error.value = "";

  try {
    releases.value = await invoke<GodotRelease[]>("fetch_godot_releases", { limit: 8 });
    releasesLoaded.value = true;
    status.value = t("status.releasesLoaded");
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

function createProject() {
  return runAction(t("status.projectCreated"), () =>
    invoke<HubState>("create_project", {
      request: { ...newProject, editorId: newProject.editorId || null },
    }),
  );
}

function importProject() {
  return runAction(t("status.projectImported"), () =>
    invoke<HubState>("import_project", {
      request: {
        ...importProjectForm,
        name: importProjectForm.name || null,
        editorId: importProjectForm.editorId || null,
      },
    }),
  );
}

function addEditor() {
  return runAction(t("status.editorRegistered"), () => invoke<HubState>("add_editor", { request: newEditor }));
}

function downloadEditor(release: GodotRelease, asset: GodotReleaseAsset) {
  const key = assetKey(release, asset);
  downloadTarget.value = key;

  return runAction(t("status.editorInstalled"), async () => {
    const nextState = await invoke<HubState>("download_godot_editor", {
      request: {
        releaseTag: release.tagName,
        assetName: asset.name,
        assetUrl: asset.browserDownloadUrl,
        installPath: settingsForm.defaultInstallPath,
        makeDefault: !state.editors.length,
      },
    });
    installedAssetKeys.value = [...installedAssetKeys.value, key];
    return nextState;
  }).finally(() => {
    downloadTarget.value = "";
  });
}

function saveSettings() {
  return runAction(t("status.pathsSaved"), () => invoke<HubState>("save_settings", { request: settingsForm }));
}

function launchProject(projectId: string) {
  return runAction(t("status.openingProject"), () => invoke<HubState>("launch_project", { projectId }));
}

function removeProject(projectId: string) {
  return runAction(t("status.projectRemoved"), () => invoke<HubState>("remove_project", { projectId }));
}

function moveProject() {
  if (!activeProject.value) return;

  return runAction(t("status.projectMoved"), () =>
    invoke<HubState>("move_project", {
      request: {
        projectId: activeProject.value!.id,
        destinationPath: moveDestinationPath.value,
      },
    }),
  ).then(() => {
    moveDestinationPath.value = activeProject.value?.path ?? "";
    if (activeProject.value) loadGitStatus(activeProject.value.id);
  });
}

function toggleFavorite(projectId: string) {
  return runAction(t("status.projectUpdated"), () => invoke<HubState>("toggle_project_favorite", { projectId }));
}

function setDefaultEditor(editorId: string) {
  return runAction(t("status.defaultEditorUpdated"), () => invoke<HubState>("set_default_editor", { editorId }));
}

function removeEditor(editorId: string) {
  return runAction(t("status.editorRemoved"), () => invoke<HubState>("remove_editor", { editorId }));
}

function openProjectPage(projectId: string) {
  activeProjectId.value = projectId;
  projectPageOpen.value = true;
  projectDetailTab.value = "overview";
  const project = state.projects.find((item) => item.id === projectId);
  moveDestinationPath.value = project?.path ?? "";
  loadGitStatus(projectId);
  loadGitLog(projectId);
  loadGitBranches(projectId);
}

function closeProjectPage() {
  projectPageOpen.value = false;
  projectDetailTab.value = "overview";
}

function navigateSection(section: (typeof sections)[number]) {
  projectPageOpen.value = false;
  activeSection.value = section;
}

function editorLabel(editorId?: string | null) {
  return state.editors.find((editor) => editor.id === editorId)?.version ?? defaultEditor.value?.version ?? t("common.noEditor");
}

function lastOpenedLabel(value?: string | null) {
  if (!value) return t("common.neverOpened");
  const date = new Date(Number(value) * 1000);
  return Number.isNaN(date.getTime()) ? t("common.openedRecently") : date.toLocaleString();
}

function releaseDate(value?: string | null) {
  if (!value) return t("common.noDate");
  return new Date(value).toLocaleDateString();
}

function fileSize(bytes: number) {
  if (!bytes) return "0 MB";
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function featuredAssets(release: GodotRelease) {
  return release.assets
    .filter((asset) => {
      const name = asset.name.toLowerCase();
      const isZip = name.endsWith(".zip");
      const isEditor = !name.includes("export_templates") && !name.includes("debug_symbols");
      const isFlavor = releaseFlavor.value === "dotnet" ? name.includes("mono") : !name.includes("mono");
      return isZip && isEditor && isFlavor && matchesCurrentSystem(name);
    })
    .slice(0, 8);
}

function matchesCurrentSystem(assetName: string) {
  const name = assetName.toLowerCase();
  const platform = systemProfile.godotPlatform;

  const platformMatches =
    (platform === "linux" && name.includes("linux")) ||
    (platform === "win" && (name.includes("win64") || name.includes("win32") || name.includes("windows"))) ||
    (platform === "macos" && (name.includes("macos") || name.includes("osx"))) ||
    (platform === "android" && name.includes("android"));

  if (!platformMatches) return false;

  if (systemProfile.arch === "x86_64" || systemProfile.arch === "amd64") {
    return name.includes("64") || name.includes("x86_64") || platform === "macos";
  }

  if (systemProfile.arch === "aarch64" || systemProfile.arch === "arm64") {
    return name.includes("arm64") || name.includes("aarch64") || platform === "macos";
  }

  return true;
}

function assetKey(release: GodotRelease, asset: GodotReleaseAsset) {
  return `${release.id}:${asset.id}`;
}

function isAssetInstalling(release: GodotRelease, asset: GodotReleaseAsset) {
  return downloadTarget.value === assetKey(release, asset);
}

function isAssetInstalled(release: GodotRelease, asset: GodotReleaseAsset) {
  return installedAssetKeys.value.includes(assetKey(release, asset));
}

function projectInitials(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function assetPlatform(name: string) {
  const lower = name.toLowerCase();
  if (lower.includes("linux")) return "Linux";
  if (lower.includes("win")) return "Windows";
  if (lower.includes("macos") || lower.includes("osx")) return "macOS";
  if (lower.includes("android")) return "Android";
  if (lower.includes("web")) return "Web";
  return "Build";
}

function sectionDescription(section: (typeof sections)[number]) {
  const descriptions: Record<(typeof sections)[number], string> = {
    projects: t("sections.projectsDescription"),
    editors: t("sections.editorsDescription"),
    releases: t("sections.releasesDescription"),
    settings: t("sections.settingsDescription"),
  };

  return descriptions[section];
}

function sectionTitle(section: (typeof sections)[number]) {
  const titles: Record<(typeof sections)[number], string> = {
    projects: t("sections.projectsTitle"),
    editors: t("sections.editorsTitle"),
    releases: t("sections.releasesTitle"),
    settings: t("sections.settingsTitle"),
  };

  return titles[section];
}

function gitBadgeText() {
  return gitStatusLabel(gitStatus.value, gitLoading.value);
}

function gitStatusLabel(status?: GitStatus | null, loading = false) {
  if (loading) return "Checking";
  if (!status) return t("git.unknown");
  if (!status.available) return t("git.gitMissing");
  if (!status.isRepo) return t("git.notInitialized");
  if (status.changedFiles || status.untrackedFiles) return t("common.changes");
  return t("git.clean");
}

watch(selectedTheme, (theme) => {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem(themeStorageKey, theme);
});

onMounted(() => {
  document.documentElement.dataset.theme = selectedTheme.value;
  loadSystemProfile();
  loadState();
});
</script>

<template>
  <main class="min-h-screen bg-base-200 text-base-content">
    <div class="grid min-h-screen lg:grid-cols-[248px_minmax(0,1fr)]">
      <aside class="hidden border-r border-base-content/10 bg-base-300 lg:flex lg:flex-col">
        <div class="border-b border-base-content/10 p-5">
          <div class="flex items-center gap-3">
            <div class="grid h-10 w-10 place-items-center rounded-md bg-primary/10 shadow-lg shadow-primary/20 ring-1 ring-base-content/10">
              <img :src="appLogo" alt="Godot Forge" class="h-8 w-8 object-contain" />
            </div>
            <div>
              <strong class="block text-sm">Godot Forge</strong>
              <span class="text-xs text-base-content/50">Engine & Project Hub</span>
            </div>
          </div>
        </div>

        <nav class="flex-1 p-3">
          <p class="px-3 pb-2 text-[11px] font-bold uppercase text-base-content/50">{{ t("nav.library") }}</p>
          <button
            v-for="section in sections"
            :key="section"
            class="mb-1 flex w-full items-center justify-between rounded-md px-3 py-2.5 text-left text-sm font-semibold text-base-content/65 transition hover:bg-base-content/5 hover:text-base-content"
            :class="{ 'bg-primary/15 text-primary ring-1 ring-primary/20': activeSection === section }"
            @click="navigateSection(section)"
          >
            <span>{{ t(`nav.${section}`) }}</span>
            <span v-if="section === 'projects'" class="rounded bg-base-content/10 px-1.5 text-[11px]">{{ state.projects.length }}</span>
            <span v-if="section === 'editors'" class="rounded bg-base-content/10 px-1.5 text-[11px]">{{ state.editors.length }}</span>
          </button>
        </nav>

        <div class="border-t border-base-content/10 p-4">
          <div class="rounded-lg border border-base-content/10 bg-base-300/60 p-3">
            <p class="text-[11px] font-bold uppercase text-base-content/50">{{ t("common.defaultEditor") }}</p>
            <strong class="mt-1 block truncate text-sm">
              {{ defaultEditor ? `${defaultEditor.name} ${defaultEditor.version}` : t("common.noEditor") }}
            </strong>
            <p class="mt-1 truncate text-xs text-base-content/50">{{ defaultEditor?.executablePath || t("common.configureInstallation") }}</p>
          </div>
        </div>
      </aside>

      <section class="min-w-0">
        <header class="sticky top-0 z-20 border-b border-base-content/10 bg-base-200/95 backdrop-blur">
          <div class="flex min-h-16 items-center gap-4 px-4 lg:px-8">
            <div class="grid h-9 w-9 shrink-0 place-items-center rounded-md bg-primary/10 shadow-lg shadow-primary/20 ring-1 ring-base-content/10 lg:hidden">
              <img :src="appLogo" alt="Godot Forge" class="h-7 w-7 object-contain" />
            </div>
            <div class="lg:hidden">
              <select v-model="activeSection" class="select select-bordered select-sm bg-base-100">
                <option v-for="section in sections" :key="section" :value="section">{{ t(`nav.${section}`) }}</option>
              </select>
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-[11px] font-black uppercase text-primary">Godot Forge / {{ t(`nav.${activeSection}`) }}</p>
              <h1 class="truncate text-lg font-black">{{ sectionTitle(activeSection) }}</h1>
            </div>
            <select v-model="selectedLocale" class="select select-bordered select-sm border-base-content/10 bg-base-100">
              <option value="en">English</option>
              <option value="pt">Português</option>
            </select>
            <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content/90 hover:bg-base-content/10" @click="navigateSection('releases')">
              {{ t("common.installEditor") }}
            </button>
            <button class="btn btn-sm btn-primary" :disabled="!activeProject || !!busyAction" @click="launchProject(activeProject!.id)">
              {{ t("common.launch") }}
            </button>
          </div>
        </header>

        <div class="mx-auto grid max-w-[1500px] gap-6 p-4 lg:p-8">
          <div v-if="loading" class="grid gap-4">
            <div class="h-64 animate-pulse rounded-xl bg-base-content/5" />
            <div class="h-40 animate-pulse rounded-xl bg-base-content/5" />
          </div>

          <template v-else>
            <section v-if="projectPageOpen && activeProject" class="grid gap-6">
              <div class="overflow-hidden rounded-xl border border-base-content/10 bg-base-100">
                <div class="relative grid min-h-72 content-end bg-[radial-gradient(circle_at_24%_20%,var(--color-primary)_0%,var(--color-secondary)_38%,var(--color-base-200)_100%)] p-6 lg:p-8">
                  <button class="btn btn-sm absolute left-4 top-4 border-base-content/10 bg-base-content/10 text-base-content hover:bg-base-content/20" @click="closeProjectPage">
                    {{ t("common.backToProjects") }}
                  </button>
                  <div class="max-w-4xl">
                    <p class="text-xs font-black uppercase tracking-wide text-primary">{{ t("projectPage.pageLabel") }}</p>
                    <h2 class="mt-2 text-5xl font-black leading-none tracking-tight">{{ activeProject.name }}</h2>
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
                      @click="projectDetailTab = 'overview'"
                    >
                      {{ t("projectPage.overview") }}
                    </button>
                    <button
                      class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
                      :class="projectDetailTab === 'git' ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
                      @click="projectDetailTab = 'git'; remoteUrl = gitStatus?.remote || ''; loadGitStatus(activeProject.id); loadGitLog(activeProject.id); loadGitBranches(activeProject.id)"
                    >
                      Git
                    </button>
                    <button
                      class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
                      :class="projectDetailTab === 'settings' ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
                      @click="projectDetailTab = 'settings'; moveDestinationPath = activeProject.path"
                    >
                      {{ t("common.settings") }}
                    </button>
                  </div>
                  <div class="flex flex-wrap gap-2">
                    <button class="btn btn-sm btn-primary" :disabled="!!busyAction" @click="launchProject(activeProject.id)">{{ t("common.launchProject") }}</button>
                    <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction" @click="toggleFavorite(activeProject.id)">
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
                    <button class="btn btn-primary" :disabled="!!busyAction" @click="launchProject(activeProject.id)">{{ t("common.openInEditor") }}</button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" @click="projectDetailTab = 'git'; remoteUrl = gitStatus?.remote || ''; loadGitStatus(activeProject.id); loadGitLog(activeProject.id); loadGitBranches(activeProject.id)">
                      {{ t("projectPage.gitManage") }}
                    </button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" @click="projectDetailTab = 'settings'; moveDestinationPath = activeProject.path">
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
                      <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="gitLoading" @click="loadGitStatus(activeProject.id); loadGitLog(activeProject.id); loadGitBranches(activeProject.id)">
                        {{ t("common.refresh") }}
                      </button>
                      <button
                        class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10"
                        :disabled="!gitStatus?.isRepo || !!busyAction"
                        @click="pushGitBranch(activeProject.id)"
                      >
                        {{ t("git.pushBranch") }}
                      </button>
                      <button
                        v-if="gitStatus?.available && !gitStatus?.isRepo"
                        class="btn btn-sm btn-primary"
                        :disabled="gitLoading || !!busyAction"
                        @click="initGit(activeProject.id)"
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
                          @click="checkoutGitBranch(branch.name, activeProject.id)"
                        >
                          <span class="truncate">{{ branch.name }}</span>
                          <span class="text-xs">{{ branch.current ? t("common.current") : t("common.checkout") }}</span>
                        </button>
                      </div>
                      <p v-else class="mt-3 text-sm text-base-content/50">{{ t("git.noLocalBranches") }}</p>
                    </div>

                    <div class="grid gap-4">
                      <form class="rounded-lg border border-base-content/10 bg-base-300/60 p-4" @submit.prevent="createGitBranch(activeProject.id)">
                        <p class="text-xs font-black uppercase text-base-content/50">{{ t("git.createBranch") }}</p>
                        <div class="mt-3 flex flex-col gap-2 sm:flex-row">
                          <input v-model="branchName" class="input input-bordered input-sm border-base-content/10 bg-base-200" :placeholder="t('git.branchNamePlaceholder')" />
                          <button class="btn btn-primary btn-sm" :disabled="!!busyAction || !branchName.trim()">{{ t("common.create") }}</button>
                        </div>
                      </form>

                      <form class="rounded-lg border border-base-content/10 bg-base-300/60 p-4" @submit.prevent="saveGitRemote(activeProject.id)">
                        <p class="text-xs font-black uppercase text-base-content/50">Remote origin</p>
                        <div class="mt-3 grid gap-2">
                          <input v-model="remoteUrl" class="input input-bordered input-sm border-base-content/10 bg-base-200" placeholder="git@github.com:user/repo.git" />
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
                <form class="rounded-xl border border-base-content/10 bg-base-100 p-5" @submit.prevent="moveProject">
                  <p class="text-xs font-black uppercase text-primary">Project</p>
                  <h3 class="mt-1 text-2xl font-black">{{ t("projectPage.projectSettings") }}</h3>
                  <div class="mt-5 grid gap-4">
                    <label class="grid gap-2">
                      <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.pathCurrent") }}</span>
                      <input class="input input-bordered border-base-content/10 bg-base-200" :value="activeProject.path" disabled />
                    </label>
                    <label class="grid gap-2">
                      <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.newDestinationPath") }}</span>
                      <input v-model="moveDestinationPath" class="input input-bordered border-base-content/10 bg-base-200" required />
                    </label>
                    <button class="btn btn-primary w-fit" :disabled="!!busyAction || moveDestinationPath === activeProject.path">
                      {{ t("projectPage.moveProject") }}
                    </button>
                  </div>
                </form>

                <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
                  <p class="text-xs font-black uppercase text-primary">{{ t("projectPage.dangerZone") }}</p>
                  <p class="mt-2 text-sm text-base-content/50">{{ t("projectPage.dangerBody") }}</p>
                  <button class="btn btn-error btn-outline mt-4" :disabled="!!busyAction" @click="removeProject(activeProject.id); closeProjectPage()">
                    {{ t("common.removeFromLibrary") }}
                  </button>
                </aside>
              </section>
            </section>

            <template v-else>
            <section class="overflow-hidden rounded-xl border border-base-content/10 bg-base-100">
              <div class="flex flex-col gap-5 border-l-4 border-primary p-5 lg:flex-row lg:items-center lg:justify-between">
                <div>
                  <p class="text-[11px] font-black uppercase tracking-wide text-primary">{{ t("nav.currentScreen") }}</p>
                  <h2 class="mt-1 text-3xl font-black tracking-tight lg:text-4xl">{{ sectionTitle(activeSection) }}</h2>
                  <p class="mt-2 max-w-2xl text-sm text-base-content/65">{{ sectionDescription(activeSection) }}</p>
                </div>
                <div class="flex flex-wrap gap-2">
                  <button
                    v-for="section in sections"
                    :key="section"
                    class="rounded-md px-3 py-2 text-xs font-black uppercase transition"
                    :class="activeSection === section ? 'bg-primary text-primary-content shadow-lg shadow-primary/20' : 'bg-base-content/5 text-base-content/65 hover:bg-base-content/10 hover:text-base-content'"
                    @click="navigateSection(section)"
                  >
                    {{ t(`nav.${section}`) }}
                  </button>
                </div>
              </div>
            </section>

            <section
              v-if="hasOnboarding"
              class="overflow-hidden rounded-xl border border-base-content/10 bg-[linear-gradient(135deg,var(--color-base-100)_0%,var(--color-base-200)_55%,var(--color-base-300)_100%)] shadow-2xl"
            >
              <div class="grid gap-8 p-6 lg:grid-cols-[1fr_420px] lg:p-8">
                <div class="flex min-h-72 flex-col justify-end">
                  <span class="mb-4 w-fit rounded bg-primary/15 px-3 py-1 text-xs font-black uppercase text-primary ring-1 ring-primary/20">
                    {{ t("onboarding.setupRequired") }}
                  </span>
                  <h2 class="max-w-4xl text-4xl font-black leading-none tracking-tight lg:text-6xl">
                    {{ t("onboarding.title") }}
                  </h2>
                  <p class="mt-4 max-w-2xl text-sm leading-6 text-base-content/65 lg:text-base">
                    {{ t("onboarding.body") }}
                  </p>
                  <div class="mt-6 flex flex-wrap gap-3">
                    <button class="btn btn-primary" @click="navigateSection('releases')">{{ t("onboarding.download") }}</button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" @click="navigateSection('editors')">
                      {{ t("onboarding.register") }}
                    </button>
                    <button class="btn btn-ghost text-base-content/65" @click="dismissWelcome = true">Skip</button>
                  </div>
                </div>

                <div class="grid content-end gap-3">
                  <div class="rounded-lg border border-base-content/10 bg-base-300/70 p-4">
                    <p class="text-xs font-black uppercase text-primary">01 Engine install</p>
                    <h3 class="mt-2 font-bold">{{ t("onboarding.engineInstallTitle") }}</h3>
                    <p class="mt-1 text-sm text-base-content/50">{{ t("onboarding.engineInstallBody") }}</p>
                  </div>
                  <div class="rounded-lg border border-base-content/10 bg-base-300/70 p-4">
                    <p class="text-xs font-black uppercase text-warning">02 Project library</p>
                    <h3 class="mt-2 font-bold">{{ t("onboarding.projectLibraryTitle") }}</h3>
                    <p class="mt-1 text-sm text-base-content/50">{{ t("onboarding.projectLibraryBody") }}</p>
                  </div>
                </div>
              </div>
            </section>

            <section v-if="activeSection === 'projects'" class="grid gap-3 md:grid-cols-3">
              <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
                <p class="text-xs font-bold uppercase text-base-content/50">{{ t("nav.projects") }}</p>
                <div class="mt-2 flex items-end justify-between">
                  <strong class="text-4xl font-black">{{ state.projects.length }}</strong>
                  <span class="text-xs text-base-content/50">{{ sortedProjects.length }} {{ t("projects.filtered") }}</span>
                </div>
              </div>
              <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
                <p class="text-xs font-bold uppercase text-base-content/50">Engine versions</p>
                <div class="mt-2 flex items-end justify-between">
                  <strong class="text-4xl font-black">{{ state.editors.length }}</strong>
                  <span class="max-w-40 truncate text-xs text-base-content/50">{{ defaultEditor?.version || "No default" }}</span>
                </div>
              </div>
              <div class="rounded-lg border border-base-content/10 bg-base-100 p-4">
                <p class="text-xs font-bold uppercase text-base-content/50">Project path</p>
                <strong class="mt-2 block truncate text-lg">{{ state.settings.defaultProjectPath || "Not configured" }}</strong>
                <p class="mt-1 text-xs text-base-content/50">Default workspace</p>
              </div>
            </section>

            <section v-if="activeSection === 'projects'" class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px]">
              <div class="grid gap-4">
                <div class="flex flex-col gap-3 rounded-lg border border-base-content/10 bg-base-100 p-4 lg:flex-row lg:items-center">
                  <div class="flex-1">
                    <p class="text-xs font-black uppercase text-primary">{{ t("sections.projectsTitle") }}</p>
                    <h2 class="text-2xl font-black">{{ t("projects.myProjects") }}</h2>
                  </div>
                  <input
                    v-model="projectSearch"
                    class="input input-bordered h-10 border-base-content/10 bg-base-200 text-sm lg:w-80"
                    :placeholder="t('projects.searchPlaceholder')"
                  />
                </div>

                <div v-if="sortedProjects.length" class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
                  <article
                    v-for="project in sortedProjects"
                    :key="project.id"
                    class="group cursor-pointer overflow-hidden rounded-xl border border-base-content/10 bg-base-100 transition hover:-translate-y-0.5 hover:border-primary/50 hover:bg-base-200"
                    :class="{ 'border-primary/70 ring-1 ring-primary/30': activeProject?.id === project.id }"
                    @click="openProjectPage(project.id)"
                  >
                    <div class="relative grid aspect-[16/9] place-items-center bg-[radial-gradient(circle_at_30%_20%,var(--color-primary)_0%,var(--color-secondary)_38%,var(--color-base-200)_100%)]">
                      <span class="text-5xl font-black text-primary-content/90">{{ projectInitials(project.name) || "GD" }}</span>
                      <button
                        class="absolute right-3 top-3 rounded bg-base-300/70 px-2 py-1 text-lg text-warning backdrop-blur"
                        @click.stop="toggleFavorite(project.id)"
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
                        <button class="btn btn-primary btn-xs" :disabled="!!busyAction" @click.stop="launchProject(project.id)">
                          Launch
                        </button>
                      </div>
                    </div>
                  </article>
                </div>

                <div v-else class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
                  <h3 class="text-2xl font-black">{{ t("common.noProjects") }}</h3>
                  <p class="mt-2 text-base-content/50">{{ t("projects.createNewOrImport") }}</p>
                </div>

                <div class="grid gap-4 lg:grid-cols-2">
                  <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="createProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">{{ t("projects.newProject") }}</h3>
                      <span class="text-xs text-base-content/50">{{ t("common.create") }}</span>
                    </div>
                    <div class="grid gap-3">
                      <input v-model="newProject.name" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('projects.projectName')" />
                      <input v-model="newProject.rootPath" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('projects.baseFolder')" />
                      <select v-model="newProject.editorId" class="select select-bordered border-base-content/10 bg-base-200">
                        <option value="">{{ t("projects.useDefaultEditor") }}</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn btn-primary" :disabled="!!busyAction">{{ t("projects.createProject") }}</button>
                    </div>
                  </form>

                  <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="importProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">{{ t("projects.importProject") }}</h3>
                      <span class="text-xs text-base-content/50">{{ t("common.import") }}</span>
                    </div>
                    <div class="grid gap-3">
                      <input v-model="importProjectForm.name" class="input input-bordered border-base-content/10 bg-base-200" :placeholder="t('projects.optionalName')" />
                      <input v-model="importProjectForm.path" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('projects.projectPathPlaceholder')" />
                      <select v-model="importProjectForm.editorId" class="select select-bordered border-base-content/10 bg-base-200">
                        <option value="">{{ t("projects.useDefaultEditor") }}</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction">{{ t("common.import") }}</button>
                    </div>
                  </form>
                </div>
              </div>

              <aside class="h-fit rounded-xl border border-base-content/10 bg-base-100 p-5">
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
                        @click="loadGitStatus(activeProject.id)"
                      >
                        {{ t("common.refreshGit") }}
                      </button>
                    <button
                      v-if="gitStatus?.available && !gitStatus?.isRepo"
                      class="btn btn-primary btn-xs mt-3"
                      :disabled="gitLoading || !!busyAction"
                      @click="initGit(activeProject.id)"
                    >
                      {{ t("git.initializeGit") }}
                    </button>
                    </div>
                  </div>
                  <div class="mt-6 grid gap-2">
                    <button class="btn btn-primary" :disabled="!!busyAction" @click="launchProject(activeProject.id)">{{ t("common.launchProject") }}</button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction" @click="toggleFavorite(activeProject.id)">
                      {{ activeProject.favorite ? t("common.removeFavorite") : t("common.addFavorite") }}
                    </button>
                    <button class="btn btn-error btn-outline" :disabled="!!busyAction" @click="removeProject(activeProject.id)">{{ t("common.removeFromLibrary") }}</button>
                  </div>
                </template>
                <p v-else class="mt-3 text-sm text-base-content/50">{{ t("projectPage.selectProject") }}</p>
              </aside>
            </section>

            <section v-if="activeSection === 'editors'" class="grid gap-6">
              <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
                <article
                  v-for="editor in state.editors"
                  :key="editor.id"
                  class="rounded-xl border border-base-content/10 bg-base-100 p-5"
                >
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0">
                      <p class="text-xs font-black uppercase text-primary">Installed Engine</p>
                      <h3 class="mt-2 truncate text-2xl font-black">{{ editor.name }}</h3>
                      <p class="font-bold text-base-content/80">{{ editor.version }}</p>
                    </div>
                    <span class="rounded px-2 py-1 text-xs font-black" :class="editor.isDefault ? 'bg-primary text-primary-content' : 'bg-base-content/10 text-base-content/80'">
                      {{ editor.isDefault ? "DEFAULT" : editor.architecture }}
                    </span>
                  </div>
                  <div class="mt-5 grid gap-2 rounded-lg bg-base-300/60 p-3 text-xs text-base-content/50">
                    <p class="truncate">{{ editor.executablePath }}</p>
                    <p class="truncate">{{ editor.installPath }}</p>
                  </div>
                  <div class="mt-5 flex flex-wrap gap-2">
                    <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="editor.isDefault || !!busyAction" @click="setDefaultEditor(editor.id)">
                      {{ t("common.makeDefault") }}
                    </button>
                    <button class="btn btn-sm btn-error btn-outline" :disabled="!!busyAction" @click="removeEditor(editor.id)">{{ t("common.remove") }}</button>
                  </div>
                </article>

                <div v-if="!state.editors.length" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-8">
                  <h3 class="text-2xl font-black">{{ t("common.noEngineInstalled") }}</h3>
                  <p class="mt-2 text-base-content/50">{{ t("editors.noEngineBody") }}</p>
                </div>
              </div>

              <form class="rounded-xl border border-base-content/10 bg-base-100 p-5" @submit.prevent="addEditor">
                <div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
                  <div>
                    <p class="text-xs font-black uppercase text-primary">{{ t("editors.manualSetup") }}</p>
                    <h2 class="text-2xl font-black">{{ t("editors.localInstall") }}</h2>
                  </div>
                  <button class="btn btn-primary" :disabled="!!busyAction">{{ t("editors.registerEditor") }}</button>
                </div>
                <div class="grid gap-3 lg:grid-cols-2">
                  <input v-model="newEditor.name" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('common.name')" />
                  <input v-model="newEditor.version" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('editors.version')" />
                  <input v-model="newEditor.executablePath" class="input input-bordered border-base-content/10 bg-base-200 lg:col-span-2" required :placeholder="t('editors.executablePath')" />
                  <input v-model="newEditor.installPath" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('editors.installFolder')" />
                  <input v-model="newEditor.architecture" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('editors.architecture')" />
                </div>
                <label class="mt-4 flex w-fit cursor-pointer items-center gap-3 text-sm font-bold text-base-content/80">
                  <input v-model="newEditor.makeDefault" type="checkbox" class="checkbox checkbox-primary" />
                  {{ t("editors.setDefault") }}
                </label>
              </form>
            </section>

            <section v-if="activeSection === 'releases'" class="grid gap-4">
              <div class="rounded-xl border border-base-content/10 bg-base-100 p-5">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
                  <div>
                    <p class="text-xs font-black uppercase text-primary">{{ t("releases.officialGithub") }}</p>
                    <h2 class="text-2xl font-black">{{ t("releases.title") }}</h2>
                    <p class="mt-1 text-sm text-base-content/50">
                      {{ t("releases.showingBuilds", { os: systemProfile.os, arch: systemProfile.arch }) }}
                    </p>
                  </div>
                  <div class="flex flex-col gap-2 lg:items-end">
                    <div class="join">
                      <button
                        class="btn join-item btn-sm"
                        :class="releaseFlavor === 'standard' ? 'btn-primary' : 'border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10'"
                        @click="releaseFlavor = 'standard'"
                      >
                        Standard
                      </button>
                      <button
                        class="btn join-item btn-sm"
                        :class="releaseFlavor === 'dotnet' ? 'btn-primary' : 'border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10'"
                        @click="releaseFlavor = 'dotnet'"
                      >
                        .NET / Mono
                      </button>
                    </div>
                    <div class="flex flex-col gap-2 sm:flex-row">
                    <input v-model="releaseQuery" class="input input-bordered border-base-content/10 bg-base-200" :placeholder="t('releases.filterPlaceholder')" />
                    <button class="btn btn-primary" :disabled="!!busyAction" @click="loadReleases">
                      {{ releasesLoaded ? t("common.refresh") : t("common.fetchReleases") }}
                    </button>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="!releasesLoaded" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
                <h3 class="text-2xl font-black">{{ t("common.catalogNotLoaded") }}</h3>
                <p class="mt-2 text-base-content/50">{{ t("releases.fetchOfficial") }}</p>
              </div>

              <article
                v-for="release in filteredReleases"
                :key="release.id"
                class="rounded-xl border border-base-content/10 bg-base-100 p-5"
              >
                <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div>
                    <div class="flex flex-wrap items-center gap-2">
                      <h3 class="text-2xl font-black">{{ release.tagName }}</h3>
                      <span class="rounded px-2 py-1 text-xs font-black" :class="release.prerelease ? 'bg-warning text-warning-content' : 'bg-success text-success-content'">
                        {{ release.prerelease ? "PREVIEW" : "STABLE" }}
                      </span>
                    </div>
                    <p class="mt-1 text-sm text-base-content/50">{{ release.name || "Godot release" }} - {{ releaseDate(release.publishedAt) }}</p>
                  </div>
                  <a class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :href="release.htmlUrl" target="_blank">
                    GitHub
                  </a>
                </div>

                <div class="mt-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                  <div
                    v-for="asset in featuredAssets(release)"
                    :key="asset.id"
                    class="grid gap-4 rounded-lg border border-base-content/10 bg-base-300/60 p-4"
                  >
                    <div>
                      <span class="rounded bg-primary/15 px-2 py-1 text-xs font-black text-primary">{{ assetPlatform(asset.name) }}</span>
                      <span class="ml-2 rounded bg-base-content/10 px-2 py-1 text-xs font-black text-base-content/80">
                        {{ releaseFlavor === "dotnet" ? ".NET" : "Standard" }}
                      </span>
                      <p class="mt-3 min-h-12 break-all text-sm font-bold text-base-content/90">{{ asset.name }}</p>
                      <p class="mt-2 text-xs text-base-content/50">{{ fileSize(asset.size) }}</p>
                    </div>
                    <button
                      class="btn btn-sm"
                      :class="isAssetInstalled(release, asset) ? 'btn-success' : 'btn-primary'"
                      :disabled="!!busyAction || isAssetInstalled(release, asset)"
                      @click="downloadEditor(release, asset)"
                    >
                      <span v-if="isAssetInstalling(release, asset)" class="loading loading-spinner loading-xs" />
                      {{
                        isAssetInstalling(release, asset)
                          ? t("common.installing")
                          : isAssetInstalled(release, asset)
                            ? t("common.installed")
                            : t("common.downloadAndInstall")
                      }}
                    </button>
                  </div>
                </div>
                <div v-if="!featuredAssets(release).length" class="mt-5 rounded-lg border border-dashed border-base-content/10 bg-base-300/60 p-4 text-sm text-base-content/50">
                  {{ t("releases.noAsset") }} {{ releaseFlavor === "dotnet" ? ".NET/Mono" : "Standard" }} {{ t("releases.compatibleWith") }}
                  {{ systemProfile.os }} / {{ systemProfile.arch }} {{ t("releases.noAssetSuffix") }}
                </div>
              </article>
            </section>

            <section v-if="activeSection === 'settings'" class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_360px]">
              <form class="rounded-xl border border-base-content/10 bg-base-100 p-5" @submit.prevent="saveSettings">
                <div>
                  <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceSettings") }}</p>
                  <h2 class="text-2xl font-black">{{ t("settings.defaultPaths") }}</h2>
                  <p class="mt-1 text-sm text-base-content/50">{{ t("settings.control") }}</p>
                </div>
                <div class="mt-5 grid gap-4 lg:grid-cols-2">
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-base-content/80">{{ t("settings.installations") }}</span>
                    <input v-model="settingsForm.defaultInstallPath" class="input input-bordered border-base-content/10 bg-base-200" required />
                  </label>
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-base-content/80">{{ t("nav.projects") }}</span>
                    <input v-model="settingsForm.defaultProjectPath" class="input input-bordered border-base-content/10 bg-base-200" required />
                  </label>
                </div>
                <div>
                  <button class="btn btn-primary" :disabled="!!busyAction">{{ t("common.savePaths") }}</button>
                </div>
              </form>

              <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
                <p class="text-xs font-black uppercase text-primary">{{ t("settings.appearance") }}</p>
                <h2 class="mt-1 text-2xl font-black">{{ t("settings.preferences") }}</h2>
                <div class="mt-5 grid gap-4">
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-base-content/80">{{ t("settings.language") }}</span>
                    <select v-model="selectedLocale" class="select select-bordered border-base-content/10 bg-base-200">
                      <option value="en">English</option>
                      <option value="pt">Português</option>
                    </select>
                  </label>
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-base-content/80">{{ t("settings.theme") }}</span>
                    <select v-model="selectedTheme" class="select select-bordered border-base-content/10 bg-base-200">
                      <option value="godotforge">{{ t("settings.darkTheme") }}</option>
                      <option value="godotforge-light">{{ t("settings.lightTheme") }}</option>
                    </select>
                  </label>
                </div>
              </aside>
            </section>
            </template>
          </template>
        </div>

        <div v-if="status || error || busyAction" class="toast toast-end z-30">
          <div class="alert border border-base-content/10 shadow-xl" :class="error ? 'alert-error' : busyAction ? 'alert-info' : 'alert-success'">
            <span>{{ error || busyAction || status }}</span>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

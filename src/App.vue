<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import appLogo from "./assets/godot-forge-logo.png";
import AppSidebar from "./components/AppSidebar.vue";
import BrandMark from "./components/BrandMark.vue";
import DeleteConfirmDialog from "./components/DeleteConfirmDialog.vue";
import PathField from "./components/PathField.vue";
import SecurityPolicyDialog from "./components/SecurityPolicyDialog.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import EditorsPage from "./pages/EditorsPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";
import type {
  DeleteTarget,
  GitBranch,
  GitLogEntry,
  GitStatus,
  GodotRelease,
  GodotReleaseAsset,
  HubState,
  PathTarget,
  ProjectDetailTab,
  ReleaseFlavor,
  Section,
  SystemProfile,
  ThemeName,
} from "./types";

const localeStorageKey = "godot-forge-locale";
const themeStorageKey = "godot-forge-theme";
const onboardingStorageKey = "godot-forge-onboarding-complete";
const themeNames: ThemeName[] = ["godotforge", "godotforge-light"];

const state = reactive<HubState>({
  editors: [],
  projects: [],
  settings: {
    defaultInstallPath: "",
    defaultProjectPath: "",
    releaseRepositories: [],
    githubToken: "",
  },
});

const { locale, t } = useI18n();
const savedLocale = localStorage.getItem(localeStorageKey);
const savedTheme = localStorage.getItem(themeStorageKey);

if (savedLocale === "en" || savedLocale === "pt") {
  locale.value = savedLocale;
}

const sections: Section[] = ["projects", "editors", "settings"];
const releasePageSize = 6;
const releaseChannelFilters = ["all", "stable", "preview"] as const;
const releaseVariantFilters = ["all", "standard", "dotnet"] as const;
const releasePlatformFilters = ["linux", "win", "macos"] as const;
const releaseArchFilters = ["x86_64", "arm64"] as const;
const activeSection = ref<Section>("projects");
const activeProjectId = ref("");
const busyAction = ref("");
const error = ref("");
const status = ref("");
const loading = ref(true);
const appVersion = ref("");
const projectSearch = ref("");
const showWelcome = ref(localStorage.getItem(onboardingStorageKey) !== "true");
const welcomeSlide = ref(0);
const releases = ref<GodotRelease[]>([]);
const releasesLoaded = ref(false);
const releaseFiltersOpen = ref(false);
const releasePage = ref(1);
const hasMoreReleases = ref(true);
const releaseQuery = ref("");
const releaseRepositoryFilter = ref("all");
const releaseChannelFilter = ref<"all" | "stable" | "preview">("all");
const releaseVariantFilter = ref<"all" | ReleaseFlavor>("all");
const releasePlatformFilter = ref<(typeof releasePlatformFilters)[number]>("linux");
const releaseArchFilter = ref<(typeof releaseArchFilters)[number]>("x86_64");
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
const projectDetailTab = ref<ProjectDetailTab>("overview");
const moveDestinationPath = ref("");
const gitLog = ref<GitLogEntry[]>([]);
const gitLogLoading = ref(false);
const gitBranches = ref<GitBranch[]>([]);
const branchName = ref("");
const remoteUrl = ref("");
const deleteDialog = reactive({
  open: false,
  type: "project" as DeleteTarget,
  id: "",
  name: "",
  closeProjectPage: false,
});
const securityDialogOpen = ref(false);
let unlistenMenuAction: UnlistenFn | null = null;
let toastTimer: ReturnType<typeof setTimeout> | null = null;
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

const settingsForm = reactive({
  defaultInstallPath: "",
  defaultProjectPath: "",
  releaseRepositories: [] as string[],
  githubToken: "",
});

const defaultEditor = computed(() => state.editors.find((editor) => editor.isDefault));
const releaseSourceCount = computed(() => 1 + state.settings.releaseRepositories.length);
const releaseRepositoryOptions = computed(() => {
  const repositories = releases.value.map((release) => release.sourceRepository).filter(Boolean);
  return [...new Set(repositories)].sort((left, right) => left.localeCompare(right));
});
const welcomeSlides = computed(() => [
  {
    label: t("onboarding.slideOneLabel"),
    title: t("onboarding.slideOneTitle"),
    body: t("onboarding.slideOneBody"),
  },
  {
    label: t("onboarding.slideTwoLabel"),
    title: t("onboarding.slideTwoTitle"),
    body: t("onboarding.slideTwoBody"),
  },
  {
    label: t("onboarding.slideThreeLabel"),
    title: t("onboarding.slideThreeTitle"),
    body: t("onboarding.slideThreeBody"),
  },
]);

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
  return releases.value.filter((release) => {
    if (releaseRepositoryFilter.value !== "all" && release.sourceRepository !== releaseRepositoryFilter.value) {
      return false;
    }

    if (releaseChannelFilter.value === "stable" && release.prerelease) return false;
    if (releaseChannelFilter.value === "preview" && !release.prerelease) return false;

    if (!query) return true;
    const haystack = `${release.tagName} ${release.name ?? ""} ${release.assets.map((asset) => asset.name).join(" ")}`;
    return haystack.toLowerCase().includes(query);
  });
});
const releaseResultLabel = computed(() => t("releases.resultCount", { count: filteredReleases.value.length }));

function clearReleaseFilters() {
  releaseQuery.value = "";
  releaseRepositoryFilter.value = "all";
  releaseChannelFilter.value = "all";
  releaseVariantFilter.value = "all";
  releasePlatformFilter.value = normalizedPlatform(systemProfile.godotPlatform);
  releaseArchFilter.value = normalizedArch(systemProfile.arch);
}

function applyState(nextState: HubState) {
  state.editors = nextState.editors;
  state.projects = nextState.projects;
  state.settings = nextState.settings;
  settingsForm.defaultInstallPath = nextState.settings.defaultInstallPath;
  settingsForm.defaultProjectPath = nextState.settings.defaultProjectPath;
  settingsForm.releaseRepositories = [...nextState.settings.releaseRepositories];
  settingsForm.githubToken = nextState.settings.githubToken;
  newProject.rootPath ||= nextState.settings.defaultProjectPath;

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
    releasePlatformFilter.value = normalizedPlatform(profile.godotPlatform);
    releaseArchFilter.value = normalizedArch(profile.arch);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function loadAppMetadata() {
  try {
    appVersion.value = await getVersion();
  } catch {
    appVersion.value = "";
  }
}

async function loadReleases(options: { append?: boolean } = {}) {
  busyAction.value = t("status.fetchingReleases");
  error.value = "";

  try {
    const nextPage = options.append ? releasePage.value + 1 : 1;
    const nextReleases = await invoke<GodotRelease[]>("fetch_godot_releases", { limit: releasePageSize, page: nextPage });
    let addedCount = nextReleases.length;

    if (options.append) {
      const knownKeys = new Set(releases.value.map((release) => `${release.sourceRepository}:${release.id}`));
      const uniqueReleases = nextReleases.filter((release) => !knownKeys.has(`${release.sourceRepository}:${release.id}`));
      addedCount = uniqueReleases.length;
      releases.value = [...releases.value, ...uniqueReleases];
    } else {
      releases.value = nextReleases;
    }

    releasePage.value = nextPage;
    hasMoreReleases.value = addedCount > 0 && nextReleases.length >= releasePageSize * releaseSourceCount.value;
    releasesLoaded.value = true;
    status.value = t("status.releasesLoaded");
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

function loadMoreReleases() {
  return loadReleases({ append: true });
}

function createProject() {
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

  return runAction(t("status.projectCreated"), () =>
    invoke<HubState>("create_project", {
      request: { ...newProject, editorId: newProject.editorId || null },
    }),
  );
}

function importProject() {
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

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

function downloadEditor(release: GodotRelease, asset: GodotReleaseAsset) {
  const key = assetKey(release, asset);
  downloadTarget.value = key;

  return runAction(t("status.editorInstalled"), async () => {
    const nextState = await invoke<HubState>("download_godot_editor", {
      request: {
        releaseTag: release.tagName,
        releaseRepository: release.sourceRepository,
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

async function saveSettings() {
  const previousRepositories = state.settings.releaseRepositories.join("\n");
  const previousGithubToken = state.settings.githubToken;

  await runAction(t("status.pathsSaved"), () => invoke<HubState>("save_settings", { request: settingsForm }));

  if (previousRepositories !== state.settings.releaseRepositories.join("\n") || previousGithubToken !== state.settings.githubToken) {
    releasesLoaded.value = false;
    releases.value = [];
    releasePage.value = 1;
    hasMoreReleases.value = true;
    await loadReleases();
  }
}

function restoreDefaultSettings() {
  return runAction(t("status.settingsRestored"), () => invoke<HubState>("restore_default_settings"));
}

function setPathValue(target: PathTarget, value: string) {
  const path = value.trim();
  if (!path) return;

  const setters: Record<PathTarget, () => void> = {
    newProjectRoot: () => {
      newProject.rootPath = path;
    },
    importProjectPath: () => {
      importProjectForm.path = path;
    },
    settingsInstall: () => {
      settingsForm.defaultInstallPath = path;
    },
    settingsProject: () => {
      settingsForm.defaultProjectPath = path;
    },
    moveProjectDestination: () => {
      moveDestinationPath.value = path;
    },
  };

  setters[target]();
}

async function browsePath(target: PathTarget, directory = true) {
  const selected = await open({
    directory,
    multiple: false,
  });

  if (typeof selected === "string") {
    setPathValue(target, selected);
  }
}

function launchProject(projectId: string) {
  return runAction(t("status.openingProject"), () => invoke<HubState>("launch_project", { projectId }));
}

function executeRemoveProject(projectId: string) {
  return runAction(t("status.projectRemoved"), () => invoke<HubState>("remove_project", { projectId }));
}

function moveProject() {
  if (!activeProject.value) return;
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

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

function executeRemoveEditor(editorId: string) {
  return runAction(t("status.editorRemoved"), () => invoke<HubState>("remove_editor", { editorId }));
}

function requestRemoveProject(projectId: string, closeProjectPageAfterConfirm = false) {
  const project = state.projects.find((item) => item.id === projectId);
  deleteDialog.open = true;
  deleteDialog.type = "project";
  deleteDialog.id = projectId;
  deleteDialog.name = project?.name ?? t("common.project");
  deleteDialog.closeProjectPage = closeProjectPageAfterConfirm;
}

function requestRemoveEditor(editorId: string) {
  const editor = state.editors.find((item) => item.id === editorId);
  deleteDialog.open = true;
  deleteDialog.type = "editor";
  deleteDialog.id = editorId;
  deleteDialog.name = editor ? `${editor.name} ${editor.version}` : t("common.editor");
  deleteDialog.closeProjectPage = false;
}

function cancelDelete() {
  deleteDialog.open = false;
  deleteDialog.id = "";
  deleteDialog.name = "";
  deleteDialog.closeProjectPage = false;
}

async function confirmDelete() {
  if (!deleteDialog.id) return;

  const shouldCloseProjectPage = deleteDialog.closeProjectPage;
  const action = deleteDialog.type === "project" ? executeRemoveProject(deleteDialog.id) : executeRemoveEditor(deleteDialog.id);
  cancelDelete();
  await action;

  if (shouldCloseProjectPage) {
    closeProjectPage();
  }
}

function handleMenuAction(action: string) {
  if (action === "projects" || action === "editors" || action === "settings") {
    navigateSection(action);
    return;
  }

  if (action === "security-policy") {
    securityDialogOpen.value = true;
  }
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

function navigateSection(section: Section) {
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

function releaseFlavorLabel(flavor: ReleaseFlavor) {
  return flavor === "dotnet" ? ".NET / Mono" : "Standard";
}

function releaseChannelLabel(channel: (typeof releaseChannelFilters)[number]) {
  if (channel === "stable") return t("releases.stable");
  if (channel === "preview") return t("releases.preview");
  return t("releases.allChannels");
}

function releaseVariantLabel(variant: (typeof releaseVariantFilters)[number]) {
  if (variant === "standard") return "Standard";
  if (variant === "dotnet") return ".NET / Mono";
  return t("releases.allVariants");
}

function normalizedPlatform(platform: string): (typeof releasePlatformFilters)[number] {
  if (platform === "win" || platform === "macos") return platform;
  return "linux";
}

function normalizedArch(arch: string): (typeof releaseArchFilters)[number] {
  if (arch === "aarch64" || arch === "arm64") return "arm64";
  return "x86_64";
}

function releasePlatformLabel(platform: (typeof releasePlatformFilters)[number]) {
  if (platform === "win") return "Windows";
  if (platform === "macos") return "macOS";
  return "Linux";
}

function releaseArchLabel(arch: (typeof releaseArchFilters)[number]) {
  return arch === "arm64" ? "ARM64" : "x86_64";
}

function assetFlavor(assetName: string): ReleaseFlavor {
  return assetName.toLowerCase().includes("mono") ? "dotnet" : "standard";
}

function compatibleAssets(release: GodotRelease) {
  return release.assets
    .filter((asset) => {
      const name = asset.name.toLowerCase();
      const isZip = name.endsWith(".zip");
      const isEditor = !name.includes("export_templates") && !name.includes("debug_symbols");
      const isFlavor = releaseVariantFilter.value === "all" || assetFlavor(name) === releaseVariantFilter.value;
      return isZip && isEditor && isFlavor && matchesCurrentSystem(name);
    })
    .slice(0, 8);
}

function matchesCurrentSystem(assetName: string) {
  const name = assetName.toLowerCase();
  const platform = releasePlatformFilter.value;

  const platformMatches =
    (platform === "linux" && name.includes("linux")) ||
    (platform === "win" && (name.includes("win64") || name.includes("win32") || name.includes("windows"))) ||
    (platform === "macos" && (name.includes("macos") || name.includes("osx")));

  if (!platformMatches) return false;

  if (releaseArchFilter.value === "x86_64") {
    const isArmBuild = name.includes("arm64") || name.includes("aarch64");
    return !isArmBuild && (name.includes("64") || name.includes("x86_64") || platform === "macos");
  }

  if (releaseArchFilter.value === "arm64") {
    return name.includes("arm64") || name.includes("aarch64") || platform === "macos";
  }

  return true;
}

function assetKey(release: GodotRelease, asset: GodotReleaseAsset) {
  return `${release.sourceRepository}:${release.id}:${asset.id}`;
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

function sectionTitle(section: Section) {
  const titles: Record<Section, string> = {
    projects: t("sections.projectsTitle"),
    editors: t("sections.editorsTitle"),
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

function completeWelcome(section: Section = "projects") {
  showWelcome.value = false;
  localStorage.setItem(onboardingStorageKey, "true");
  navigateSection(section);
}

function nextWelcomeSlide() {
  if (welcomeSlide.value >= welcomeSlides.value.length - 1) {
    completeWelcome("projects");
    return;
  }

  welcomeSlide.value += 1;
}

function previousWelcomeSlide() {
  welcomeSlide.value = Math.max(0, welcomeSlide.value - 1);
}

function clearToastTimer() {
  if (!toastTimer) return;
  clearTimeout(toastTimer);
  toastTimer = null;
}

watch(selectedTheme, (theme) => {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem(themeStorageKey, theme);
});

watch([status, error, busyAction], ([nextStatus, nextError, nextBusyAction]) => {
  clearToastTimer();

  if (nextBusyAction || (!nextStatus && !nextError)) return;

  const currentStatus = nextStatus;
  const currentError = nextError;
  toastTimer = setTimeout(
    () => {
      if (currentStatus && status.value === currentStatus) status.value = "";
      if (currentError && error.value === currentError) error.value = "";
    },
    nextError ? 8000 : 3500,
  );
});

onMounted(async () => {
  document.documentElement.dataset.theme = selectedTheme.value;
  try {
    unlistenMenuAction = await listen<string>("menu-action", (event) => handleMenuAction(event.payload));
  } catch {
    unlistenMenuAction = null;
  }
  await Promise.all([loadSystemProfile(), loadAppMetadata()]);
  await Promise.all([loadState(), loadReleases()]);
});

onUnmounted(() => {
  clearToastTimer();
  unlistenMenuAction?.();
});
</script>

<template>
  <main class="min-h-screen bg-base-200 text-base-content">
    <WelcomeScreen
      v-if="showWelcome"
      v-model:slide="welcomeSlide"
      v-model:selected-locale="selectedLocale"
      :logo="appLogo"
      :slides="welcomeSlides"
      @previous="previousWelcomeSlide"
      @next="nextWelcomeSlide"
      @complete="completeWelcome"
    />

    <div v-else class="grid min-h-screen lg:grid-cols-[248px_minmax(0,1fr)]">
      <AppSidebar
        :logo="appLogo"
        :sections="sections"
        :active-section="activeSection"
        :project-count="state.projects.length"
        :editor-count="state.editors.length"
        :app-version="appVersion"
        @navigate="navigateSection"
      />

      <section class="min-w-0 overflow-x-hidden">
        <header class="sticky top-0 z-20 border-b border-base-content/10 bg-base-200/95 backdrop-blur">
          <div class="flex min-h-16 items-center gap-3 px-4 lg:px-8">
            <BrandMark :logo="appLogo" size="sm" class="lg:hidden" />
            <div class="min-w-0 flex-1">
              <p class="text-[11px] font-black uppercase text-primary">Godot Forge / {{ t(`nav.${activeSection}`) }}</p>
              <h1 class="truncate text-lg font-black">{{ sectionTitle(activeSection) }}</h1>
            </div>
            <div class="hidden items-center gap-2 lg:flex">
              <span class="rounded bg-base-content/5 px-2 py-1 text-xs font-bold text-base-content/50">{{ state.projects.length }} {{ t("nav.projects") }}</span>
              <span class="rounded bg-base-content/5 px-2 py-1 text-xs font-bold text-base-content/50">{{ state.editors.length }} {{ t("nav.editors") }}</span>
            </div>
          </div>
          <nav class="flex gap-2 overflow-x-auto border-t border-base-content/10 px-4 py-2 lg:hidden">
            <button
              v-for="section in sections"
              :key="section"
              class="shrink-0 rounded-md px-3 py-2 text-xs font-black uppercase transition"
              :class="activeSection === section ? 'bg-primary text-primary-content' : 'bg-base-content/5 text-base-content/65'"
              type="button"
              @click="navigateSection(section)"
            >
              {{ t(`nav.${section}`) }}
            </button>
          </nav>
        </header>

        <div class="mx-auto grid w-full max-w-[1500px] gap-5 p-3 sm:p-4 lg:gap-6 lg:p-8">
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
                      :disabled="!state.editors.length"
                      @click="projectDetailTab = 'settings'; moveDestinationPath = activeProject.path"
                    >
                      {{ t("common.settings") }}
                    </button>
                  </div>
                  <div class="flex flex-wrap gap-2">
                    <button class="btn btn-sm btn-primary" :disabled="!!busyAction || !state.editors.length" @click="launchProject(activeProject.id)">{{ t("common.launchProject") }}</button>
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
                    <button class="btn btn-primary" :disabled="!!busyAction || !state.editors.length" @click="launchProject(activeProject.id)">{{ t("common.openInEditor") }}</button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" @click="projectDetailTab = 'git'; remoteUrl = gitStatus?.remote || ''; loadGitStatus(activeProject.id); loadGitLog(activeProject.id); loadGitBranches(activeProject.id)">
                      {{ t("projectPage.gitManage") }}
                    </button>
                    <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!state.editors.length" @click="projectDetailTab = 'settings'; moveDestinationPath = activeProject.path">
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
                  <p v-if="!state.editors.length" class="mt-2 rounded-md border border-primary/20 bg-primary/10 px-3 py-2 text-sm text-base-content/70">
                    {{ t("projects.editorRequired") }}
                  </p>
                  <div class="mt-5 grid gap-4">
                    <label class="grid gap-2">
                      <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.pathCurrent") }}</span>
                      <input class="input input-bordered border-base-content/10 bg-base-200" :value="activeProject.path" disabled />
                    </label>
                    <label class="grid gap-2">
                      <span class="text-sm font-bold text-base-content/80">{{ t("projectPage.newDestinationPath") }}</span>
                      <PathField
                        v-model="moveDestinationPath"
                        required
                        :disabled="!state.editors.length"
                        :button-label="t('common.browse')"
                        @browse="browsePath('moveProjectDestination')"
                      />
                    </label>
                    <button class="btn btn-primary w-fit" :disabled="!!busyAction || !state.editors.length || moveDestinationPath === activeProject.path">
                      {{ t("projectPage.moveProject") }}
                    </button>
                  </div>
                </form>

                <aside class="rounded-xl border border-base-content/10 bg-base-100 p-5">
                  <p class="text-xs font-black uppercase text-primary">{{ t("projectPage.dangerZone") }}</p>
                  <p class="mt-2 text-sm text-base-content/50">{{ t("projectPage.dangerBody") }}</p>
                  <button class="btn btn-error btn-outline mt-4" :disabled="!!busyAction" @click="requestRemoveProject(activeProject.id, true)">
                    {{ t("common.removeFromLibrary") }}
                  </button>
                </aside>
              </section>
            </section>

            <template v-else>
            <section v-if="activeSection === 'projects' && state.projects.length" class="grid gap-3 md:grid-cols-3">
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

            <section
              v-if="activeSection === 'projects'"
              class="grid gap-6"
              :class="{ 'xl:grid-cols-[minmax(0,1fr)_380px]': state.projects.length }"
            >
              <div class="grid gap-4">
                <div v-if="state.projects.length" class="flex flex-col gap-3 rounded-lg border border-base-content/10 bg-base-100 p-4 lg:flex-row lg:items-center">
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

                <div v-if="!state.editors.length" class="rounded-xl border border-primary/25 bg-primary/10 p-4">
                  <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                    <div class="min-w-0">
                      <p class="text-xs font-black uppercase text-primary">{{ t("common.noEngineInstalled") }}</p>
                      <h3 class="mt-1 text-xl font-black">{{ t("projects.noEditorCtaTitle") }}</h3>
                      <p class="mt-1 text-sm text-base-content/65">{{ t("projects.noEditorCtaBody") }}</p>
                    </div>
                    <button class="btn btn-primary shrink-0" type="button" @click="navigateSection('editors')">
                      {{ t("projects.goInstallEditor") }}
                    </button>
                  </div>
                </div>

                <div v-if="state.projects.length && sortedProjects.length" class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
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
                        <button class="btn btn-primary btn-xs" :disabled="!!busyAction || !state.editors.length" @click.stop="launchProject(project.id)">
                          Launch
                        </button>
                      </div>
                    </div>
                  </article>
                </div>

                <div v-else-if="state.projects.length" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
                  <h3 class="text-2xl font-black">{{ t("common.noProjects") }}</h3>
                  <p class="mt-2 text-base-content/50">{{ t("projects.createNewOrImport") }}</p>
                </div>

                <div class="grid gap-4 lg:grid-cols-2">
                  <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="createProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">{{ t("projects.newProject") }}</h3>
                      <span class="text-xs text-base-content/50">{{ t("common.create") }}</span>
                    </div>
                    <fieldset class="grid gap-3" :disabled="!state.editors.length">
                      <input v-model="newProject.name" class="input input-bordered border-base-content/10 bg-base-200" required :placeholder="t('projects.projectName')" />
                      <PathField
                        v-model="newProject.rootPath"
                        required
                        :placeholder="t('projects.baseFolder')"
                        :button-label="t('common.browse')"
                        @browse="browsePath('newProjectRoot')"
                      />
                      <select v-model="newProject.editorId" class="select cursor-pointer select-bordered border-base-content/10 bg-base-200 disabled:cursor-not-allowed disabled:opacity-60" :disabled="!state.editors.length">
                        <option value="">{{ state.editors.length ? t("projects.useDefaultEditor") : t("projects.noEditorSelect") }}</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn btn-primary" :disabled="!!busyAction || !state.editors.length">{{ t("projects.createProject") }}</button>
                    </fieldset>
                  </form>

                  <form class="rounded-lg border border-base-content/10 bg-base-100 p-4" @submit.prevent="importProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">{{ t("projects.importProject") }}</h3>
                      <span class="text-xs text-base-content/50">{{ t("common.import") }}</span>
                    </div>
                    <fieldset class="grid gap-3" :disabled="!state.editors.length">
                      <input v-model="importProjectForm.name" class="input input-bordered border-base-content/10 bg-base-200" :placeholder="t('projects.optionalName')" />
                      <PathField
                        v-model="importProjectForm.path"
                        required
                        :placeholder="t('projects.projectPathPlaceholder')"
                        :button-label="t('common.browse')"
                        @browse="browsePath('importProjectPath')"
                      />
                      <select v-model="importProjectForm.editorId" class="select cursor-pointer select-bordered border-base-content/10 bg-base-200 disabled:cursor-not-allowed disabled:opacity-60" :disabled="!state.editors.length">
                        <option value="">{{ state.editors.length ? t("projects.useDefaultEditor") : t("projects.noEditorSelect") }}</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="!!busyAction || !state.editors.length">{{ t("common.import") }}</button>
                    </fieldset>
                  </form>
                </div>
              </div>

              <aside v-if="state.projects.length" class="h-fit rounded-xl border border-base-content/10 bg-base-100 p-5">
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
                    <button class="btn btn-error btn-outline" :disabled="!!busyAction" @click="requestRemoveProject(activeProject.id)">{{ t("common.removeFromLibrary") }}</button>
                  </div>
                </template>
                <p v-else class="mt-3 text-sm text-base-content/50">{{ t("projectPage.selectProject") }}</p>
              </aside>
            </section>

            <EditorsPage
              v-if="activeSection === 'editors'"
              :editors="state.editors"
              :releases="releases"
              :filtered-releases="filteredReleases"
              :release-repository-options="releaseRepositoryOptions"
              :release-source-count="releaseSourceCount"
              :release-result-label="releaseResultLabel"
              :releases-loaded="releasesLoaded"
              :release-filters-open="releaseFiltersOpen"
              :release-query="releaseQuery"
              :release-repository-filter="releaseRepositoryFilter"
              :release-channel-filter="releaseChannelFilter"
              :release-variant-filter="releaseVariantFilter"
              :release-platform-filter="releasePlatformFilter"
              :release-arch-filter="releaseArchFilter"
              :release-channel-filters="releaseChannelFilters"
              :release-variant-filters="releaseVariantFilters"
              :release-platform-filters="releasePlatformFilters"
              :release-arch-filters="releaseArchFilters"
              :busy-action="busyAction"
              :fetching-releases-label="t('status.fetchingReleases')"
              :has-more-releases="hasMoreReleases"
              :release-date="releaseDate"
              :release-flavor-label="releaseFlavorLabel"
              :release-channel-label="releaseChannelLabel"
              :release-variant-label="releaseVariantLabel"
              :release-platform-label="releasePlatformLabel"
              :release-arch-label="releaseArchLabel"
              :compatible-assets="compatibleAssets"
              :asset-platform="assetPlatform"
              :asset-flavor="assetFlavor"
              :file-size="fileSize"
              :is-asset-installing="isAssetInstalling"
              :is-asset-installed="isAssetInstalled"
              @set-default-editor="setDefaultEditor"
              @remove-editor="requestRemoveEditor"
              @clear-release-filters="clearReleaseFilters"
              @load-more-releases="loadMoreReleases"
              @download-editor="downloadEditor"
              @update:release-filters-open="releaseFiltersOpen = $event"
              @update:release-query="releaseQuery = $event"
              @update:release-repository-filter="releaseRepositoryFilter = $event"
              @update:release-channel-filter="releaseChannelFilter = $event"
              @update:release-variant-filter="releaseVariantFilter = $event"
              @update:release-platform-filter="releasePlatformFilter = $event"
              @update:release-arch-filter="releaseArchFilter = $event"
            />

            <SettingsPage
              v-if="activeSection === 'settings'"
              v-model:selected-locale="selectedLocale"
              v-model:selected-theme="selectedTheme"
              :settings-form="settingsForm"
              :busy="!!busyAction"
              @browse="browsePath"
              @save="saveSettings"
              @restore-defaults="restoreDefaultSettings"
              @open-security="securityDialogOpen = true"
            />
            </template>
          </template>
        </div>

        <SecurityPolicyDialog :open="securityDialogOpen" @close="securityDialogOpen = false" />

        <DeleteConfirmDialog
          :open="deleteDialog.open"
          :delete-type="deleteDialog.type"
          :name="deleteDialog.name"
          :busy="!!busyAction"
          @cancel="cancelDelete"
          @confirm="confirmDelete"
        />

        <div v-if="status || error || busyAction" class="toast toast-end z-30">
          <div class="alert border border-base-content/10 shadow-xl" :class="error ? 'alert-error' : busyAction ? 'alert-info' : 'alert-success'">
            <span>{{ error || busyAction || status }}</span>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

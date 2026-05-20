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
import SecurityPolicyDialog from "./components/SecurityPolicyDialog.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import EditorsPage from "./pages/EditorsPage.vue";
import ProjectsPage from "./pages/ProjectsPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";
import type {
  ActivityLogEntry,
  DeleteTarget,
  DiscoveredEditor,
  DiscoveredProject,
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
  WorkspaceDiagnostics,
  WorkspaceScan,
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
const releaseCatalogPageSize = 100;
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
const workspaceScan = reactive<WorkspaceScan>({
  editors: [],
  projects: [],
});
const workspaceScanState = reactive({
  loaded: false,
  loading: false,
  action: "",
  error: "",
});
const diagnostics = ref<WorkspaceDiagnostics | null>(null);
const activityLog = ref<ActivityLogEntry[]>([]);
const diagnosticsLoading = ref(false);
const releasesLoaded = ref(false);
const releaseFiltersOpen = ref(false);
const releasePage = ref(1);
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
  migrateExistingPaths: false,
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

const filteredReleaseCatalog = computed(() => {
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
const filteredReleases = computed(() => filteredReleaseCatalog.value.slice(0, releasePage.value * releasePageSize));
const hasMoreReleases = computed(() => filteredReleases.value.length < filteredReleaseCatalog.value.length);
const releaseResultLabel = computed(() => t("releases.resultCount", { count: filteredReleaseCatalog.value.length }));

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
  settingsForm.migrateExistingPaths = false;
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

async function scanWorkspace() {
  workspaceScanState.loading = true;
  workspaceScanState.error = "";

  try {
    const result = await invoke<WorkspaceScan>("scan_workspace");
    workspaceScan.editors = result.editors;
    workspaceScan.projects = result.projects;
    workspaceScanState.loaded = true;
  } catch (caught) {
    workspaceScanState.error = caught instanceof Error ? caught.message : String(caught);
  } finally {
    workspaceScanState.loading = false;
  }
}

async function loadDiagnostics() {
  diagnosticsLoading.value = true;

  try {
    const [nextDiagnostics, nextActivityLog] = await Promise.all([
      invoke<WorkspaceDiagnostics>("get_workspace_diagnostics"),
      invoke<ActivityLogEntry[]>("read_activity_log"),
    ]);
    diagnostics.value = nextDiagnostics;
    activityLog.value = nextActivityLog;
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    diagnosticsLoading.value = false;
  }
}

async function clearReleaseCache() {
  busyAction.value = t("settings.clearReleaseCache");
  error.value = "";

  try {
    await invoke("clear_release_cache");
    releasesLoaded.value = false;
    releases.value = [];
    releasePage.value = 1;
    await loadDiagnostics();
    await loadReleases();
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

async function registerDiscoveredEditor(editor: DiscoveredEditor) {
  workspaceScanState.action = editor.installPath;
  workspaceScanState.error = "";

  try {
    const nextState = await invoke<HubState>("register_discovered_editor", {
      request: {
        executablePath: editor.executablePath,
        installPath: editor.installPath,
        name: editor.name,
        version: editor.version,
        architecture: editor.architecture,
      },
    });
    applyState(nextState);
    status.value = t("status.editorRegistered");
    await scanWorkspace();
  } catch (caught) {
    workspaceScanState.error = caught instanceof Error ? caught.message : String(caught);
    error.value = workspaceScanState.error;
  } finally {
    workspaceScanState.action = "";
  }
}

async function registerDiscoveredProject(project: DiscoveredProject) {
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

  workspaceScanState.action = project.path;
  workspaceScanState.error = "";

  try {
    const nextState = await invoke<HubState>("register_discovered_project", {
      request: {
        path: project.path,
        name: project.name,
      },
    });
    applyState(nextState);
    status.value = t("status.projectImported");
    await scanWorkspace();
    await loadProjectGitStatuses();
  } catch (caught) {
    workspaceScanState.error = caught instanceof Error ? caught.message : String(caught);
    error.value = workspaceScanState.error;
  } finally {
    workspaceScanState.action = "";
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

async function loadReleases() {
  busyAction.value = t("status.fetchingReleases");
  error.value = "";

  try {
    releases.value = await invoke<GodotRelease[]>("fetch_godot_releases", { limit: releaseCatalogPageSize, page: 1 });
    releasePage.value = 1;
    releasesLoaded.value = true;
    status.value = t("status.releasesLoaded");
    loadDiagnostics();
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

function loadMoreReleases() {
  releasePage.value += 1;
}

async function createProject() {
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

  await runAction(t("status.projectCreated"), () =>
    invoke<HubState>("create_project", {
      request: { ...newProject, editorId: newProject.editorId || null },
    }),
  );
  await scanWorkspace();
}

async function importProject() {
  if (!state.editors.length) {
    error.value = t("projects.editorRequired");
    return;
  }

  await runAction(t("status.projectImported"), () =>
    invoke<HubState>("import_project", {
      request: {
        ...importProjectForm,
        name: importProjectForm.name || null,
        editorId: importProjectForm.editorId || null,
      },
    }),
  );
  await scanWorkspace();
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
    return scanWorkspace();
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
    await loadReleases();
  }

  await scanWorkspace();
}

async function restoreDefaultSettings() {
  await runAction(t("status.settingsRestored"), () => invoke<HubState>("restore_default_settings"));
  await scanWorkspace();
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

watch(
  [releaseQuery, releaseRepositoryFilter, releaseChannelFilter, releaseVariantFilter, releasePlatformFilter, releaseArchFilter],
  () => {
    releasePage.value = 1;
  },
);

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
  await Promise.all([scanWorkspace(), loadDiagnostics()]);
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
            <ProjectsPage
              v-if="activeSection === 'projects'"
              v-model:project-search="projectSearch"
              v-model:project-detail-tab="projectDetailTab"
              v-model:branch-name="branchName"
              v-model:remote-url="remoteUrl"
              v-model:move-destination-path="moveDestinationPath"
              :projects="state.projects"
              :sorted-projects="sortedProjects"
              :editors="state.editors"
              :default-editor="defaultEditor"
              :active-project="activeProject"
              :project-editor="projectEditor"
              :project-page-open="projectPageOpen"
              :new-project="newProject"
              :import-project-form="importProjectForm"
              :default-project-path="state.settings.defaultProjectPath"
              :busy-action="busyAction"
              :workspace-scan="workspaceScan"
              :workspace-scan-loaded="workspaceScanState.loaded"
              :workspace-scan-loading="workspaceScanState.loading"
              :workspace-scan-action="workspaceScanState.action"
              :workspace-scan-error="workspaceScanState.error"
              :git-status="gitStatus"
              :project-git-statuses="projectGitStatuses"
              :git-loading="gitLoading"
              :git-log="gitLog"
              :git-log-loading="gitLogLoading"
              :git-branches="gitBranches"
              :project-initials="projectInitials"
              :editor-label="editorLabel"
              :last-opened-label="lastOpenedLabel"
              :git-status-label="gitStatusLabel"
              :git-badge-text="gitBadgeText"
              @close-project-page="closeProjectPage"
              @open-project-page="openProjectPage"
              @navigate-editors="navigateSection('editors')"
              @launch-project="launchProject"
              @toggle-favorite="toggleFavorite"
              @request-remove-project="requestRemoveProject"
              @scan-workspace="scanWorkspace"
              @register-discovered-project="registerDiscoveredProject"
              @create-project="createProject"
              @import-project="importProject"
              @browse="browsePath"
              @load-git-status="loadGitStatus"
              @load-git-log="loadGitLog"
              @load-git-branches="loadGitBranches"
              @init-git="initGit"
              @push-git-branch="pushGitBranch"
              @checkout-git-branch="checkoutGitBranch"
              @create-git-branch="createGitBranch"
              @save-git-remote="saveGitRemote"
              @move-project="moveProject"
            />

            <EditorsPage
              v-if="activeSection === 'editors'"
              :editors="state.editors"
              :discovered-editors="workspaceScan.editors"
              :workspace-scan-loaded="workspaceScanState.loaded"
              :workspace-scan-loading="workspaceScanState.loading"
              :workspace-scan-action="workspaceScanState.action"
              :workspace-scan-error="workspaceScanState.error"
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
              @scan-workspace="scanWorkspace"
              @register-discovered-editor="registerDiscoveredEditor"
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
              :workspace-scan="workspaceScan"
              :workspace-scan-loaded="workspaceScanState.loaded"
              :workspace-scan-loading="workspaceScanState.loading"
              :workspace-scan-action="workspaceScanState.action"
              :workspace-scan-error="workspaceScanState.error"
              :diagnostics="diagnostics"
              :activity-log="activityLog"
              :diagnostics-loading="diagnosticsLoading"
              @browse="browsePath"
              @save="saveSettings"
              @restore-defaults="restoreDefaultSettings"
              @open-security="securityDialogOpen = true"
              @scan-workspace="scanWorkspace"
              @refresh-diagnostics="loadDiagnostics"
              @clear-release-cache="clearReleaseCache"
              @register-discovered-editor="registerDiscoveredEditor"
              @register-discovered-project="registerDiscoveredProject"
            />
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

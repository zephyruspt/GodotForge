import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import { useForgeChrome } from "./useForgeChrome";
import { useForgeLibrary } from "./useForgeLibrary";
import { useProjectGit } from "./useProjectGit";
import {
  releaseArchFilters,
  releaseChannelFilters,
  releasePlatformFilters,
  releaseVariantFilters,
  useForgeReleases,
} from "./useForgeReleases";
import { useForgeWorkspace } from "./useForgeWorkspace";
import type {
  GitStatus,
  HubState,
  PathTarget,
  Section,
  SystemProfile,
} from "../types";
import {
  assetFlavor,
  assetPlatform,
  fileSize,
  gitStatusLabel as formatGitStatusLabel,
  lastOpenedLabel as formatLastOpenedLabel,
  projectInitials,
  releaseArchLabel,
  releaseChannelLabel as formatReleaseChannelLabel,
  releaseDate as formatReleaseDate,
  releaseFlavorLabel,
  releasePlatformLabel,
  releaseVariantLabel as formatReleaseVariantLabel,
  sectionTitle as formatSectionTitle,
} from "../utils/forgeHelpers";

export const sections: Section[] = ["dashboard", "projects", "editors", "diagnostics", "settings"];

export function useForgeApp() {
  const state = reactive<HubState>({
    editors: [],
    projects: [],
    settings: {
      defaultInstallPath: "",
      defaultProjectPath: "",
      releaseRepositories: [],
      githubToken: "",
      githubTokenConfigured: false,
    },
  });

  const { locale, t } = useI18n();

  const activeSection = ref<Section>("dashboard");
  const activeProjectId = ref("");
  const busyAction = ref("");
  const error = ref("");
  const status = ref("");
  const loading = ref(true);
  const systemProfile = reactive<SystemProfile>({
    os: "unknown",
    arch: "unknown",
    godotPlatform: "linux",
  });

  const settingsForm = reactive({
    defaultInstallPath: "",
    defaultProjectPath: "",
    releaseRepositories: [] as string[],
    githubToken: "",
    clearGithubToken: false,
    migrateExistingPaths: false,
  });

  const defaultEditor = computed(() => state.editors.find((editor) => editor.isDefault));
  const {
    gitStatus,
    projectGitStatuses,
    gitLoading,
    gitLog,
    gitLogLoading,
    gitBranches,
    branchName,
    remoteUrl,
    loadProjectGitStatuses,
    loadGitStatus,
    loadGitLog,
    loadGitBranches,
    initGit,
    createGitBranch,
    checkoutGitBranch,
    saveGitRemote,
    pushGitBranch,
  } = useProjectGit({
    activeProjectId,
    busyAction,
    error,
    status,
    t,
    getProjects: () => state.projects,
  });
  const {
    workspaceScan,
    workspaceScanState,
    diagnostics,
    activityLog,
    diagnosticsLoading,
    scanWorkspace,
    loadDiagnostics,
    registerDiscoveredEditor,
    registerDiscoveredProject,
  } = useForgeWorkspace({
    state,
    error,
    status,
    t,
    applyState,
    loadProjectGitStatuses,
  });
  const {
    releases,
    releasesLoaded,
    releaseFiltersOpen,
    releaseQuery,
    releaseRepositoryFilter,
    releaseChannelFilter,
    releaseVariantFilter,
    releasePlatformFilter,
    releaseArchFilter,
    releaseSourceCount,
    releaseRepositoryOptions,
    filteredReleases,
    hasMoreReleases,
    releaseResultLabel,
    resetReleases,
    clearReleaseFilters,
    setReleaseSystemDefaults,
    loadReleases,
    clearReleaseCache,
    loadMoreReleases,
    downloadEditor,
    compatibleAssets,
    isAssetInstalling,
    isAssetInstalled,
  } = useForgeReleases({
    state,
    busyAction,
    error,
    status,
    t,
    getInstallPath: () => settingsForm.defaultInstallPath,
    runAction,
    scanWorkspace,
    loadDiagnostics,
  });
  const {
    projectSearch,
    projectPageOpen,
    projectDetailTab,
    moveDestinationPath,
    newProject,
    importProjectForm,
    deleteDialog,
    sortedProjects,
    activeProject,
    projectEditor,
    syncProjectSelection,
    createProject,
    importProject,
    launchProject,
    moveProject,
    toggleFavorite,
    setDefaultEditor,
    requestRemoveProject,
    requestRemoveEditor,
    cancelDelete,
    confirmDelete,
    openProjectPage,
    closeProjectPage,
    navigateSection,
    editorLabel,
  } = useForgeLibrary({
    state,
    activeSection,
    activeProjectId,
    defaultEditor,
    error,
    t,
    runAction,
    scanWorkspace,
    loadGitStatus,
    loadGitLog,
    loadGitBranches,
  });
  const {
    appVersion,
    selectedLocale,
    selectedTheme,
    showWelcome,
    welcomeSlide,
    securityDialogOpen,
    welcomeSlides,
    loadAppMetadata,
    listenForMenuActions,
    completeWelcome,
    nextWelcomeSlide,
    previousWelcomeSlide,
    disposeChrome,
  } = useForgeChrome({
    locale,
    busyAction,
    error,
    status,
    t,
    navigateSection,
  });

  function applyState(nextState: HubState) {
    state.editors = nextState.editors;
    state.projects = nextState.projects;
    state.settings = nextState.settings;
    settingsForm.defaultInstallPath = nextState.settings.defaultInstallPath;
    settingsForm.defaultProjectPath = nextState.settings.defaultProjectPath;
    settingsForm.releaseRepositories = [...nextState.settings.releaseRepositories];
    settingsForm.githubToken = "";
    settingsForm.clearGithubToken = false;
    settingsForm.migrateExistingPaths = false;
    syncProjectSelection();
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
      if (activeProjectId.value) await loadGitStatus(activeProjectId.value);
      loadProjectGitStatuses();
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      loading.value = false;
    }
  }

  async function loadSystemProfile() {
    try {
      const profile = await invoke<SystemProfile>("detect_system_profile");
      systemProfile.os = profile.os;
      systemProfile.arch = profile.arch;
      systemProfile.godotPlatform = profile.godotPlatform;
      setReleaseSystemDefaults(profile);
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    }
  }

  async function saveSettings() {
    const previousRepositories = state.settings.releaseRepositories.join("\n");
    const previousGithubTokenConfigured = state.settings.githubTokenConfigured;
    const submittedGithubToken = settingsForm.githubToken.trim();
    await runAction(t("status.pathsSaved"), () => invoke<HubState>("save_settings", { request: settingsForm }));

    if (
      previousRepositories !== state.settings.releaseRepositories.join("\n") ||
      previousGithubTokenConfigured !== state.settings.githubTokenConfigured ||
      !!submittedGithubToken
    ) {
      resetReleases();
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
    const selected = await open({ directory, multiple: false });
    if (typeof selected === "string") setPathValue(target, selected);
  }

  const lastOpenedLabel = (value?: string | null) => formatLastOpenedLabel(value, t);
  const releaseDate = (value?: string | null) => formatReleaseDate(value, t);
  const releaseChannelLabel = (channel: (typeof releaseChannelFilters)[number]) => formatReleaseChannelLabel(channel, t);
  const releaseVariantLabel = (variant: (typeof releaseVariantFilters)[number]) => formatReleaseVariantLabel(variant, t);

  const sectionTitle = (section: Section) => formatSectionTitle(section, t);

  function gitBadgeText() {
    return gitStatusLabel(gitStatus.value, gitLoading.value);
  }

  const gitStatusLabel = (status?: GitStatus | null, loading = false) => formatGitStatusLabel(status, loading, t);

  onMounted(async () => {
    document.documentElement.dataset.theme = selectedTheme.value;
    await listenForMenuActions();
    await Promise.all([loadSystemProfile(), loadAppMetadata()]);
    await Promise.all([loadState(), loadReleases()]);
    await Promise.all([scanWorkspace(), loadDiagnostics()]);
  });

  onUnmounted(() => {
    disposeChrome();
  });

  return {
    t,
    sections,
    releaseChannelFilters,
    releaseVariantFilters,
    releasePlatformFilters,
    releaseArchFilters,
    state,
    activeSection,
    busyAction,
    error,
    status,
    loading,
    appVersion,
    projectSearch,
    showWelcome,
    welcomeSlide,
    workspaceScan,
    workspaceScanState,
    diagnostics,
    activityLog,
    diagnosticsLoading,
    releases,
    releasesLoaded,
    releaseFiltersOpen,
    releaseQuery,
    releaseRepositoryFilter,
    releaseChannelFilter,
    releaseVariantFilter,
    releasePlatformFilter,
    releaseArchFilter,
    selectedLocale,
    selectedTheme,
    gitStatus,
    projectGitStatuses,
    gitLoading,
    projectPageOpen,
    projectDetailTab,
    moveDestinationPath,
    gitLog,
    gitLogLoading,
    gitBranches,
    branchName,
    remoteUrl,
    deleteDialog,
    securityDialogOpen,
    newProject,
    importProjectForm,
    settingsForm,
    defaultEditor,
    releaseSourceCount,
    releaseRepositoryOptions,
    welcomeSlides,
    sortedProjects,
    activeProject,
    projectEditor,
    filteredReleases,
    hasMoreReleases,
    releaseResultLabel,
    clearReleaseFilters,
    scanWorkspace,
    loadDiagnostics,
    clearReleaseCache,
    registerDiscoveredEditor,
    registerDiscoveredProject,
    loadGitStatus,
    initGit,
    loadGitLog,
    loadGitBranches,
    createGitBranch,
    checkoutGitBranch,
    saveGitRemote,
    pushGitBranch,
    loadMoreReleases,
    createProject,
    importProject,
    downloadEditor,
    saveSettings,
    restoreDefaultSettings,
    browsePath,
    launchProject,
    moveProject,
    toggleFavorite,
    setDefaultEditor,
    requestRemoveProject,
    requestRemoveEditor,
    cancelDelete,
    confirmDelete,
    openProjectPage,
    closeProjectPage,
    navigateSection,
    editorLabel,
    lastOpenedLabel,
    releaseDate,
    fileSize,
    releaseFlavorLabel,
    releaseChannelLabel,
    releaseVariantLabel,
    releasePlatformLabel,
    releaseArchLabel,
    compatibleAssets,
    assetPlatform,
    assetFlavor,
    isAssetInstalling,
    isAssetInstalled,
    projectInitials,
    sectionTitle,
    gitBadgeText,
    gitStatusLabel,
    completeWelcome,
    nextWelcomeSlide,
    previousWelcomeSlide,
  };
}

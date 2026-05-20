import { computed, reactive, ref, type ComputedRef, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DeleteTarget, GodotEditor, HubState, ProjectDetailTab, Section } from "../types";
import type { Translator } from "../utils/forgeHelpers";

type UseForgeLibraryOptions = {
  state: HubState;
  activeSection: Ref<Section>;
  activeProjectId: Ref<string>;
  defaultEditor: ComputedRef<GodotEditor | undefined>;
  error: Ref<string>;
  t: Translator;
  runAction: (label: string, action: () => Promise<HubState>) => Promise<void>;
  scanWorkspace: () => Promise<void>;
  loadGitStatus: (projectId: string) => Promise<void>;
  loadGitLog: (projectId: string) => Promise<void>;
  loadGitBranches: (projectId: string) => Promise<void>;
};

export function useForgeLibrary({
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
}: UseForgeLibraryOptions) {
  const projectSearch = ref("");
  const projectPageOpen = ref(false);
  const projectDetailTab = ref<ProjectDetailTab>("overview");
  const moveDestinationPath = ref("");
  const newProject = reactive({ name: "New Game", rootPath: "", editorId: "" });
  const importProjectForm = reactive({ name: "", path: "", editorId: "" });
  const deleteDialog = reactive({
    open: false,
    type: "project" as DeleteTarget,
    id: "",
    name: "",
    closeProjectPage: false,
  });

  const sortedProjects = computed(() => {
    const query = projectSearch.value.trim().toLowerCase();
    return [...state.projects]
      .filter((project) => !query || `${project.name} ${project.path}`.toLowerCase().includes(query))
      .sort((a, b) => Number(b.favorite) - Number(a.favorite) || a.name.localeCompare(b.name));
  });

  const activeProject = computed(
    () => state.projects.find((project) => project.id === activeProjectId.value) ?? sortedProjects.value[0],
  );

  const projectEditor = computed(() => {
    const project = activeProject.value;
    return state.editors.find((editor) => editor.id === project?.editorId) ?? defaultEditor.value;
  });

  function syncProjectSelection() {
    if (!activeProjectId.value || !state.projects.some((project) => project.id === activeProjectId.value)) {
      activeProjectId.value = state.projects[0]?.id ?? "";
    }
    newProject.rootPath ||= state.settings.defaultProjectPath;
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
        request: { ...importProjectForm, name: importProjectForm.name || null, editorId: importProjectForm.editorId || null },
      }),
    );
    await scanWorkspace();
  }

  function launchProject(projectId: string) {
    return runAction(t("status.openingProject"), () => invoke<HubState>("launch_project", { projectId }));
  }

  function moveProject() {
    if (!activeProject.value) return;
    if (!state.editors.length) {
      error.value = t("projects.editorRequired");
      return;
    }

    return runAction(t("status.projectMoved"), () =>
      invoke<HubState>("move_project", {
        request: { projectId: activeProject.value!.id, destinationPath: moveDestinationPath.value },
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

  function executeRemoveProject(projectId: string) {
    return runAction(t("status.projectRemoved"), () => invoke<HubState>("remove_project", { projectId }));
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
    if (shouldCloseProjectPage) closeProjectPage();
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

  return {
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
  };
}

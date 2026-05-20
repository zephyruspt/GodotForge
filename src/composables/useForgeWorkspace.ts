import { reactive, ref, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  ActivityLogEntry,
  DiscoveredEditor,
  DiscoveredProject,
  HubState,
  WorkspaceDiagnostics,
  WorkspaceScan,
} from "../types";
import type { Translator } from "../utils/forgeHelpers";

type UseForgeWorkspaceOptions = {
  state: HubState;
  error: Ref<string>;
  status: Ref<string>;
  t: Translator;
  applyState: (nextState: HubState) => void;
  loadProjectGitStatuses: () => Promise<void>;
};

export function useForgeWorkspace({
  state,
  error,
  status,
  t,
  applyState,
  loadProjectGitStatuses,
}: UseForgeWorkspaceOptions) {
  const workspaceScan = reactive<WorkspaceScan>({ editors: [], projects: [] });
  const workspaceScanState = reactive({ loaded: false, loading: false, action: "", error: "" });
  const diagnostics = ref<WorkspaceDiagnostics | null>(null);
  const activityLog = ref<ActivityLogEntry[]>([]);
  const diagnosticsLoading = ref(false);

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
        request: { path: project.path, name: project.name },
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

  return {
    workspaceScan,
    workspaceScanState,
    diagnostics,
    activityLog,
    diagnosticsLoading,
    scanWorkspace,
    loadDiagnostics,
    registerDiscoveredEditor,
    registerDiscoveredProject,
  };
}

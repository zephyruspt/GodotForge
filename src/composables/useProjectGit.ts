import { ref, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { GitBranch, GitLogEntry, GitStatus, GodotProject } from "../types";
import type { Translator } from "../utils/forgeHelpers";

type UseProjectGitOptions = {
  activeProjectId: Ref<string>;
  busyAction: Ref<string>;
  error: Ref<string>;
  status: Ref<string>;
  t: Translator;
  getProjects: () => GodotProject[];
};

export function useProjectGit({ activeProjectId, busyAction, error, status, t, getProjects }: UseProjectGitOptions) {
  const gitStatus = ref<GitStatus | null>(null);
  const projectGitStatuses = ref<Record<string, GitStatus>>({});
  const gitLoading = ref(false);
  const gitLog = ref<GitLogEntry[]>([]);
  const gitLogLoading = ref(false);
  const gitBranches = ref<GitBranch[]>([]);
  const branchName = ref("");
  const remoteUrl = ref("");

  async function loadProjectGitStatuses() {
    const entries = await Promise.all(
      getProjects().map(async (project) => {
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
      projectGitStatuses.value = { ...projectGitStatuses.value, [projectId]: gitStatus.value };
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
      projectGitStatuses.value = { ...projectGitStatuses.value, [projectId]: gitStatus.value };
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

  return {
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
  };
}

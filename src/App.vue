<script setup lang="ts">
import appLogo from "./assets/godot-forge-logo.png";
import AppSidebar from "./components/AppSidebar.vue";
import BrandMark from "./components/BrandMark.vue";
import DeleteConfirmDialog from "./components/DeleteConfirmDialog.vue";
import SecurityPolicyDialog from "./components/SecurityPolicyDialog.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import { useForgeApp } from "./composables/useForgeApp";
import DashboardPage from "./pages/DashboardPage.vue";
import DiagnosticsPage from "./pages/DiagnosticsPage.vue";
import EditorsPage from "./pages/EditorsPage.vue";
import ProjectsPage from "./pages/ProjectsPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";

const {
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
} = useForgeApp();
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
            <DashboardPage
              v-if="activeSection === 'dashboard'"
              :projects="state.projects"
              :editors="state.editors"
              :default-editor="defaultEditor"
              :default-project-path="state.settings.defaultProjectPath"
              :default-install-path="state.settings.defaultInstallPath"
              :workspace-scan="workspaceScan"
              :workspace-scan-loaded="workspaceScanState.loaded"
              :workspace-scan-loading="workspaceScanState.loading"
              :workspace-scan-action="workspaceScanState.action"
              :workspace-scan-error="workspaceScanState.error"
              :busy="!!busyAction"
              @navigate="navigateSection"
              @scan-workspace="scanWorkspace"
              @register-discovered-editor="registerDiscoveredEditor"
              @register-discovered-project="registerDiscoveredProject"
            />

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

            <DiagnosticsPage
              v-if="activeSection === 'diagnostics'"
              :diagnostics="diagnostics"
              :activity-log="activityLog"
              :diagnostics-loading="diagnosticsLoading"
              :busy="!!busyAction"
              @refresh-diagnostics="loadDiagnostics"
              @clear-release-cache="clearReleaseCache"
            />

            <SettingsPage
              v-if="activeSection === 'settings'"
              v-model:selected-locale="selectedLocale"
              v-model:selected-theme="selectedTheme"
              :settings-form="settingsForm"
              :busy="!!busyAction"
              :github-token-configured="state.settings.githubTokenConfigured"
              @browse="browsePath"
              @save="saveSettings"
              @restore-defaults="restoreDefaultSettings"
              @open-security="securityDialogOpen = true"
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

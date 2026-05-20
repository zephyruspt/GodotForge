import { computed, ref, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { GodotRelease, GodotReleaseAsset, HubState, SystemProfile } from "../types";
import {
  assetFlavor,
  assetKey,
  matchesReleaseSystem,
  normalizedArch,
  normalizedPlatform,
} from "../utils/forgeHelpers";
import type { Translator } from "../utils/forgeHelpers";

const releasePageSize = 6;
const releaseCatalogPageSize = 100;

export const releaseChannelFilters = ["all", "stable", "preview"] as const;
export const releaseVariantFilters = ["all", "standard", "dotnet"] as const;
export const releasePlatformFilters = ["linux", "win", "macos"] as const;
export const releaseArchFilters = ["x86_64", "arm64"] as const;

export type ReleaseChannelFilter = (typeof releaseChannelFilters)[number];
export type ReleaseVariantFilter = (typeof releaseVariantFilters)[number];
export type ReleasePlatformFilter = (typeof releasePlatformFilters)[number];
export type ReleaseArchFilter = (typeof releaseArchFilters)[number];

type UseForgeReleasesOptions = {
  state: HubState;
  busyAction: Ref<string>;
  error: Ref<string>;
  status: Ref<string>;
  t: Translator;
  getInstallPath: () => string;
  runAction: (label: string, action: () => Promise<HubState>) => Promise<void>;
  scanWorkspace: () => Promise<void>;
  loadDiagnostics: () => Promise<void>;
};

export function useForgeReleases({
  state,
  busyAction,
  error,
  status,
  t,
  getInstallPath,
  runAction,
  scanWorkspace,
  loadDiagnostics,
}: UseForgeReleasesOptions) {
  const releases = ref<GodotRelease[]>([]);
  const releasesLoaded = ref(false);
  const releaseFiltersOpen = ref(false);
  const releasePage = ref(1);
  const releaseQuery = ref("");
  const releaseRepositoryFilter = ref("all");
  const releaseChannelFilter = ref<ReleaseChannelFilter>("all");
  const releaseVariantFilter = ref<ReleaseVariantFilter>("all");
  const releasePlatformFilter = ref<ReleasePlatformFilter>("linux");
  const releaseArchFilter = ref<ReleaseArchFilter>("x86_64");
  const downloadTarget = ref("");
  const installedAssetKeys = ref<string[]>([]);

  const releaseSourceCount = computed(() => 1 + state.settings.releaseRepositories.length);
  const releaseRepositoryOptions = computed(() => {
    const repositories = releases.value.map((release) => release.sourceRepository).filter(Boolean);
    return [...new Set(repositories)].sort((left, right) => left.localeCompare(right));
  });

  const filteredReleaseCatalog = computed(() => {
    const query = releaseQuery.value.trim().toLowerCase();
    return releases.value.filter((release) => {
      if (releaseRepositoryFilter.value !== "all" && release.sourceRepository !== releaseRepositoryFilter.value) return false;
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

  function resetReleases() {
    releasesLoaded.value = false;
    releases.value = [];
    releasePage.value = 1;
  }

  function clearReleaseFilters() {
    releaseQuery.value = "";
    releaseRepositoryFilter.value = "all";
    releaseChannelFilter.value = "all";
    releaseVariantFilter.value = "all";
  }

  function setReleaseSystemDefaults(profile: SystemProfile) {
    releasePlatformFilter.value = normalizedPlatform(profile.godotPlatform);
    releaseArchFilter.value = normalizedArch(profile.arch);
  }

  async function loadReleases() {
    busyAction.value = t("status.fetchingReleases");
    error.value = "";

    try {
      releases.value = await invoke<GodotRelease[]>("fetch_godot_releases", {
        limit: releaseCatalogPageSize,
        page: 1,
      });
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

  async function clearReleaseCache() {
    busyAction.value = t("settings.clearReleaseCache");
    error.value = "";

    try {
      await invoke("clear_release_cache");
      resetReleases();
      await loadDiagnostics();
      await loadReleases();
    } catch (caught) {
      error.value = caught instanceof Error ? caught.message : String(caught);
    } finally {
      busyAction.value = "";
    }
  }

  function loadMoreReleases() {
    releasePage.value += 1;
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
          installPath: getInstallPath(),
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

  function compatibleAssets(release: GodotRelease) {
    return release.assets
      .filter((asset) => {
        const name = asset.name.toLowerCase();
        const isZip = name.endsWith(".zip");
        const isEditor = !name.includes("export_templates") && !name.includes("debug_symbols");
        const isFlavor = releaseVariantFilter.value === "all" || assetFlavor(name) === releaseVariantFilter.value;
        return isZip && isEditor && isFlavor && matchesReleaseSystem(name, releasePlatformFilter.value, releaseArchFilter.value);
      })
      .slice(0, 8);
  }

  function isAssetInstalling(release: GodotRelease, asset: GodotReleaseAsset) {
    return downloadTarget.value === assetKey(release, asset);
  }

  function isAssetInstalled(release: GodotRelease, asset: GodotReleaseAsset) {
    return installedAssetKeys.value.includes(assetKey(release, asset));
  }

  watch(
    [releaseQuery, releaseRepositoryFilter, releaseChannelFilter, releaseVariantFilter, releasePlatformFilter, releaseArchFilter],
    () => {
      releasePage.value = 1;
    },
  );

  return {
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
  };
}

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { DiscoveredEditor, GodotEditor, GodotRelease, GodotReleaseAsset, ReleaseFlavor } from "../types";

type ReleaseChannelFilter = "all" | "stable" | "preview";
type ReleaseVariantFilter = "all" | ReleaseFlavor;
type ReleasePlatformFilter = "linux" | "win" | "macos";
type ReleaseArchFilter = "x86_64" | "arm64";

defineProps<{
  editors: GodotEditor[];
  discoveredEditors: DiscoveredEditor[];
  workspaceScanLoaded: boolean;
  workspaceScanLoading: boolean;
  workspaceScanAction: string;
  workspaceScanError: string;
  releases: GodotRelease[];
  filteredReleases: GodotRelease[];
  releaseRepositoryOptions: string[];
  releaseSourceCount: number;
  releaseResultLabel: string;
  releasesLoaded: boolean;
  releaseFiltersOpen: boolean;
  releaseQuery: string;
  releaseRepositoryFilter: string;
  releaseChannelFilter: ReleaseChannelFilter;
  releaseVariantFilter: ReleaseVariantFilter;
  releasePlatformFilter: ReleasePlatformFilter;
  releaseArchFilter: ReleaseArchFilter;
  releaseChannelFilters: readonly ReleaseChannelFilter[];
  releaseVariantFilters: readonly ReleaseVariantFilter[];
  releasePlatformFilters: readonly ReleasePlatformFilter[];
  releaseArchFilters: readonly ReleaseArchFilter[];
  busyAction: string;
  fetchingReleasesLabel: string;
  hasMoreReleases: boolean;
  releaseDate: (value?: string | null) => string;
  releaseFlavorLabel: (flavor: ReleaseFlavor) => string;
  releaseChannelLabel: (channel: ReleaseChannelFilter) => string;
  releaseVariantLabel: (variant: ReleaseVariantFilter) => string;
  releasePlatformLabel: (platform: ReleasePlatformFilter) => string;
  releaseArchLabel: (arch: ReleaseArchFilter) => string;
  compatibleAssets: (release: GodotRelease) => GodotReleaseAsset[];
  assetPlatform: (name: string) => string;
  assetFlavor: (name: string) => ReleaseFlavor;
  fileSize: (bytes: number) => string;
  isAssetInstalling: (release: GodotRelease, asset: GodotReleaseAsset) => boolean;
  isAssetInstalled: (release: GodotRelease, asset: GodotReleaseAsset) => boolean;
}>();

const emit = defineEmits<{
  setDefaultEditor: [editorId: string];
  removeEditor: [editorId: string];
  scanWorkspace: [];
  registerDiscoveredEditor: [editor: DiscoveredEditor];
  clearReleaseFilters: [];
  loadMoreReleases: [];
  downloadEditor: [release: GodotRelease, asset: GodotReleaseAsset];
  "update:releaseFiltersOpen": [value: boolean];
  "update:releaseQuery": [value: string];
  "update:releaseRepositoryFilter": [value: string];
  "update:releaseChannelFilter": [value: ReleaseChannelFilter];
  "update:releaseVariantFilter": [value: ReleaseVariantFilter];
  "update:releasePlatformFilter": [value: ReleasePlatformFilter];
  "update:releaseArchFilter": [value: ReleaseArchFilter];
}>();

const { t } = useI18n();
</script>

<template>
  <section class="grid gap-6">
    <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
      <article
        v-for="editor in editors"
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
          <button class="btn btn-sm border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :disabled="editor.isDefault || !!busyAction" @click="emit('setDefaultEditor', editor.id)">
            {{ t("common.makeDefault") }}
          </button>
          <button class="btn btn-sm btn-error btn-outline" :disabled="!!busyAction" @click="emit('removeEditor', editor.id)">{{ t("common.remove") }}</button>
        </div>
      </article>
    </div>
  </section>

  <section v-if="workspaceScanLoading || workspaceScanError || discoveredEditors.length" class="rounded-xl border border-base-content/10 bg-base-100 p-4">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div>
        <p class="text-xs font-black uppercase text-primary">{{ t("settings.workspaceScan") }}</p>
        <h2 class="mt-1 text-lg font-black">{{ t("settings.discoveredEditors") }}</h2>
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

    <div v-if="discoveredEditors.length" class="mt-4 grid gap-2 lg:grid-cols-2 2xl:grid-cols-3">
      <div v-for="editor in discoveredEditors" :key="editor.installPath + editor.executablePath" class="rounded-lg border border-base-content/10 bg-base-300/45 p-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="truncate text-sm font-black">{{ editor.name }} {{ editor.version }}</p>
            <p class="mt-1 truncate text-xs text-base-content/50">{{ editor.installPath }}</p>
            <p v-if="editor.reason" class="mt-2 text-xs text-warning">{{ editor.reason }}</p>
          </div>
          <span class="rounded px-2 py-1 text-[11px] font-black" :class="editor.corrupt ? 'bg-warning/20 text-warning' : editor.registered ? 'bg-success/15 text-success' : 'bg-primary/15 text-primary'">
            {{ editor.corrupt ? t("settings.corrupt") : editor.registered ? t("settings.registered") : t("settings.newItem") }}
          </span>
        </div>
        <button
          v-if="!editor.registered && !editor.corrupt"
          class="btn btn-xs btn-primary mt-3"
          type="button"
          :disabled="!!workspaceScanAction || !!busyAction"
          @click="emit('registerDiscoveredEditor', editor)"
        >
          <span v-if="workspaceScanAction === editor.installPath" class="loading loading-spinner loading-xs" />
          {{ t("settings.addToForge") }}
        </button>
      </div>
    </div>
  </section>

  <section class="grid gap-4">
    <div class="rounded-xl border border-base-content/10 bg-base-100">
      <div class="flex flex-col gap-4 border-b border-base-content/10 p-4 sm:p-5 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <p class="text-xs font-black uppercase text-primary">{{ t("releases.officialGithub") }}</p>
          <h2 class="text-2xl font-black">{{ t("releases.title") }}</h2>
          <p class="mt-1 text-sm text-base-content/50">
            {{ t("releases.showingBuilds", { os: releasePlatformLabel(releasePlatformFilter), arch: releaseArchLabel(releaseArchFilter) }) }}
          </p>
          <p class="mt-1 text-xs font-bold text-base-content/40">
            {{ t("releases.sources") }}: {{ releaseSourceCount }}
          </p>
        </div>
        <span class="rounded bg-base-content/5 px-3 py-2 text-xs font-bold text-base-content/45">
          {{ busyAction === fetchingReleasesLabel ? fetchingReleasesLabel : releaseResultLabel }}
        </span>
      </div>

      <div class="relative border-b border-base-content/10 bg-base-300/35 p-3 sm:p-4">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
          <input
            :value="releaseQuery"
            class="input input-bordered input-sm min-w-0 flex-1 border-base-content/10 bg-base-100"
            :placeholder="t('releases.filterPlaceholder')"
            @input="emit('update:releaseQuery', ($event.target as HTMLInputElement).value)"
          />
          <div class="flex items-center gap-2">
            <button
              class="btn btn-sm flex-1 border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10 sm:flex-none"
              type="button"
              :aria-expanded="releaseFiltersOpen"
              @click="emit('update:releaseFiltersOpen', !releaseFiltersOpen)"
            >
              {{ t("releases.filters") }}
            </button>
            <span class="rounded border border-base-content/10 bg-base-100 px-3 py-1.5 text-xs font-bold text-base-content/45">{{ releaseResultLabel }}</span>
          </div>
        </div>

        <div
          v-if="releaseFiltersOpen"
          class="absolute left-3 right-3 top-full z-30 mt-2 max-h-[min(72vh,430px)] overflow-y-auto rounded-xl border border-base-content/10 bg-base-100 p-4 shadow-2xl sm:left-auto sm:w-[430px]"
        >
          <div class="mb-4 flex items-center justify-between gap-3">
            <div>
              <p class="text-xs font-black uppercase text-primary">{{ t("releases.filters") }}</p>
              <p class="text-xs text-base-content/50">{{ t("releases.filterPanelHint") }}</p>
            </div>
            <button class="btn btn-ghost btn-sm" type="button" @click="emit('update:releaseFiltersOpen', false)">
              {{ t("common.close") }}
            </button>
          </div>

          <div class="grid gap-3">
            <label class="grid gap-1.5">
              <span class="text-xs font-bold uppercase text-base-content/50">{{ t("releases.repository") }}</span>
              <select
                :value="releaseRepositoryFilter"
                class="select cursor-pointer select-bordered select-sm min-w-0 border-base-content/10 bg-base-200"
                @change="emit('update:releaseRepositoryFilter', ($event.target as HTMLSelectElement).value)"
              >
                <option value="all">{{ t("releases.allRepositories") }}</option>
                <option v-for="repository in releaseRepositoryOptions" :key="repository" :value="repository">{{ repository }}</option>
              </select>
            </label>
            <div class="grid gap-3 sm:grid-cols-2">
              <label class="grid gap-1.5">
                <span class="text-xs font-bold uppercase text-base-content/50">{{ t("releases.system") }}</span>
                <select
                  :value="releasePlatformFilter"
                  class="select cursor-pointer select-bordered select-sm min-w-0 border-base-content/10 bg-base-200"
                  @change="emit('update:releasePlatformFilter', ($event.target as HTMLSelectElement).value as ReleasePlatformFilter)"
                >
                  <option v-for="platform in releasePlatformFilters" :key="platform" :value="platform">{{ releasePlatformLabel(platform) }}</option>
                </select>
              </label>
              <label class="grid gap-1.5">
                <span class="text-xs font-bold uppercase text-base-content/50">{{ t("releases.architecture") }}</span>
                <select
                  :value="releaseArchFilter"
                  class="select cursor-pointer select-bordered select-sm min-w-0 border-base-content/10 bg-base-200"
                  @change="emit('update:releaseArchFilter', ($event.target as HTMLSelectElement).value as ReleaseArchFilter)"
                >
                  <option v-for="arch in releaseArchFilters" :key="arch" :value="arch">{{ releaseArchLabel(arch) }}</option>
                </select>
              </label>
              <label class="grid gap-1.5">
                <span class="text-xs font-bold uppercase text-base-content/50">{{ t("releases.channel") }}</span>
                <select
                  :value="releaseChannelFilter"
                  class="select cursor-pointer select-bordered select-sm min-w-0 border-base-content/10 bg-base-200"
                  @change="emit('update:releaseChannelFilter', ($event.target as HTMLSelectElement).value as ReleaseChannelFilter)"
                >
                  <option v-for="channel in releaseChannelFilters" :key="channel" :value="channel">{{ releaseChannelLabel(channel) }}</option>
                </select>
              </label>
              <label class="grid gap-1.5">
                <span class="text-xs font-bold uppercase text-base-content/50">{{ t("releases.variant") }}</span>
                <select
                  :value="releaseVariantFilter"
                  class="select cursor-pointer select-bordered select-sm min-w-0 border-base-content/10 bg-base-200"
                  @change="emit('update:releaseVariantFilter', ($event.target as HTMLSelectElement).value as ReleaseVariantFilter)"
                >
                  <option v-for="variant in releaseVariantFilters" :key="variant" :value="variant">{{ releaseVariantLabel(variant) }}</option>
                </select>
              </label>
            </div>
          </div>

          <div class="sticky bottom-0 -mx-4 mt-4 flex gap-2 border-t border-base-content/10 bg-base-100 px-4 pt-3">
            <button class="btn btn-sm flex-1 border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" type="button" @click="emit('clearReleaseFilters')">
              {{ t("releases.clearFilters") }}
            </button>
            <button class="btn btn-sm btn-primary flex-1" type="button" @click="emit('update:releaseFiltersOpen', false)">
              {{ t("common.close") }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="!releasesLoaded" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
      <span class="loading loading-spinner loading-md text-primary" />
      <h3 class="mt-4 text-2xl font-black">{{ t("releases.loadingTitle") }}</h3>
      <p class="mt-2 text-base-content/50">{{ t("releases.fetchOfficial") }}</p>
    </div>

    <article
      v-for="release in filteredReleases"
      :key="release.id"
      class="min-w-0 rounded-xl border border-base-content/10 bg-base-100 p-4 sm:p-5"
    >
      <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div class="flex flex-wrap items-center gap-2">
            <h3 class="break-all text-2xl font-black">{{ release.tagName }}</h3>
            <span class="rounded px-2 py-1 text-xs font-black" :class="release.prerelease ? 'bg-warning text-warning-content' : 'bg-success text-success-content'">
              {{ release.prerelease ? "PREVIEW" : "STABLE" }}
            </span>
          </div>
          <p class="mt-1 text-sm text-base-content/50">{{ release.name || "Godot release" }} - {{ releaseDate(release.publishedAt) }}</p>
          <p class="mt-1 text-xs font-bold text-base-content/40">{{ release.sourceRepository }}</p>
        </div>
        <a class="btn btn-sm w-fit border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" :href="release.htmlUrl" target="_blank">
          GitHub
        </a>
      </div>

      <div v-if="compatibleAssets(release).length" class="mt-5 grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        <div
          v-for="asset in compatibleAssets(release)"
          :key="asset.id"
          class="grid gap-4 rounded-lg border border-base-content/10 bg-base-300/60 p-4"
        >
          <div>
            <div class="flex flex-wrap gap-2">
              <span class="rounded bg-primary/15 px-2 py-1 text-xs font-black text-primary">{{ assetPlatform(asset.name) }}</span>
              <span class="rounded px-2 py-1 text-xs font-black" :class="assetFlavor(asset.name) === 'dotnet' ? 'bg-secondary/15 text-secondary' : 'bg-base-content/10 text-base-content/70'">
                {{ releaseFlavorLabel(assetFlavor(asset.name)) }}
              </span>
            </div>
            <p class="mt-3 min-h-12 break-all text-sm font-bold text-base-content/90">{{ asset.name }}</p>
            <p class="mt-2 text-xs text-base-content/50">{{ fileSize(asset.size) }}</p>
          </div>
          <button
            class="btn btn-sm"
            :class="isAssetInstalled(release, asset) ? 'btn-success' : 'btn-primary'"
            :disabled="!!busyAction || isAssetInstalled(release, asset)"
            @click="emit('downloadEditor', release, asset)"
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

      <div v-else class="mt-5 rounded-lg border border-dashed border-base-content/10 bg-base-300/60 p-4 text-sm text-base-content/50">
        {{ t("releases.noAsset") }} {{ releaseVariantLabel(releaseVariantFilter) }} {{ t("releases.compatibleWith") }}
        {{ releasePlatformLabel(releasePlatformFilter) }} / {{ releaseArchLabel(releaseArchFilter) }} {{ t("releases.noAssetSuffix") }}
      </div>
    </article>

    <div v-if="releasesLoaded && releases.length && filteredReleases.length && hasMoreReleases" class="flex justify-center">
      <button class="btn border-base-content/10 bg-base-content/5 text-base-content hover:bg-base-content/10" type="button" :disabled="!!busyAction" @click="emit('loadMoreReleases')">
        <span v-if="busyAction === fetchingReleasesLabel" class="loading loading-spinner loading-xs" />
        {{ t("releases.loadMore") }}
      </button>
    </div>

    <div v-if="releasesLoaded && !filteredReleases.length" class="rounded-xl border border-dashed border-base-content/15 bg-base-100 p-10 text-center">
      <h3 class="text-2xl font-black">{{ t("releases.noResults") }}</h3>
      <p class="mt-2 text-base-content/50">{{ t("releases.adjustFilters") }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { ActivityLogEntry, WorkspaceDiagnostics } from "../types";

defineProps<{
  diagnostics: WorkspaceDiagnostics | null;
  activityLog: ActivityLogEntry[];
  diagnosticsLoading: boolean;
  busy: boolean;
}>();

const emit = defineEmits<{
  refreshDiagnostics: [];
  clearReleaseCache: [];
}>();

const { t } = useI18n();

function cacheAgeLabel(seconds?: number | null) {
  if (seconds == null) return t("settings.cacheNever");
  if (seconds < 60) return t("settings.cacheSeconds", { count: seconds });
  if (seconds < 3600) return t("settings.cacheMinutes", { count: Math.floor(seconds / 60) });
  return t("settings.cacheHours", { count: Math.floor(seconds / 3600) });
}

function activityDate(timestamp: number) {
  return new Date(timestamp * 1000).toLocaleString();
}
</script>

<template>
  <section class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_420px]">
    <div class="grid gap-5">
      <section class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <p class="text-xs font-black uppercase text-primary">{{ t("settings.diagnostics") }}</p>
            <h2 class="mt-1 text-2xl font-black">{{ t("settings.systemReadiness") }}</h2>
            <p class="mt-2 text-sm text-base-content/60">{{ t("settings.systemReadinessBody") }}</p>
          </div>
          <button class="btn btn-sm border-base-content/10 bg-base-content/5" type="button" :disabled="diagnosticsLoading || busy" @click="emit('refreshDiagnostics')">
            <span v-if="diagnosticsLoading" class="loading loading-spinner loading-xs" />
            {{ t("common.refresh") }}
          </button>
        </div>
        <div v-if="diagnostics" class="mt-5 grid gap-3">
          <div v-for="check in diagnostics.checks" :key="check.key" class="rounded-lg border border-base-content/10 bg-base-300/35 p-4">
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="text-sm font-black">{{ check.label }}</p>
                <p class="mt-1 break-words text-xs text-base-content/55">{{ check.detail }}</p>
              </div>
              <span class="rounded px-2 py-1 text-[11px] font-black" :class="check.ok ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'">
                {{ check.ok ? t("settings.ready") : t("settings.needsAttention") }}
              </span>
            </div>
          </div>
        </div>
        <p v-else class="mt-5 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
          {{ t("settings.diagnosticsNotLoaded") }}
        </p>
      </section>

      <section class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <div class="flex items-center justify-between gap-3">
          <div>
            <p class="text-xs font-black uppercase text-primary">{{ t("settings.activity") }}</p>
            <h2 class="mt-1 text-2xl font-black">{{ t("settings.recentActivity") }}</h2>
          </div>
          <span class="text-xs font-bold text-base-content/45">{{ activityLog.length }}</span>
        </div>
        <div v-if="activityLog.length" class="mt-5 grid max-h-[520px] gap-2 overflow-y-auto pr-1">
          <div v-for="entry in activityLog" :key="`${entry.timestamp}-${entry.message}`" class="rounded-md bg-base-300/45 px-3 py-2">
            <div class="flex items-start justify-between gap-3">
              <p class="text-sm font-bold text-base-content/80">{{ entry.message }}</p>
              <span class="rounded bg-base-content/5 px-2 py-1 text-[10px] font-black uppercase text-base-content/55">{{ entry.level }}</span>
            </div>
            <p class="mt-1 text-xs text-base-content/45">{{ activityDate(entry.timestamp) }}</p>
          </div>
        </div>
        <p v-else class="mt-5 rounded-md border border-dashed border-base-content/10 px-3 py-2 text-sm text-base-content/50">
          {{ t("settings.noActivity") }}
        </p>
      </section>
    </div>

    <aside class="grid content-start gap-5">
      <section class="rounded-xl border border-base-content/10 bg-base-100 p-5">
        <div class="flex flex-col gap-3">
          <div>
            <p class="text-xs font-black uppercase text-primary">{{ t("settings.releaseCache") }}</p>
            <h2 class="mt-1 text-2xl font-black">{{ diagnostics?.cache.releaseCount ?? 0 }} {{ t("settings.cachedReleases") }}</h2>
            <p class="mt-2 break-all text-xs text-base-content/55">{{ diagnostics?.cache.path || t("settings.cacheNotCreated") }}</p>
            <p class="mt-1 text-xs text-base-content/55">
              {{ t("settings.cacheAge") }}: {{ cacheAgeLabel(diagnostics?.cache.ageSeconds) }}
            </p>
          </div>
          <button class="btn btn-sm btn-error btn-outline w-fit" type="button" :disabled="busy" @click="emit('clearReleaseCache')">
            {{ t("settings.clearReleaseCache") }}
          </button>
        </div>
      </section>
    </aside>
  </section>
</template>

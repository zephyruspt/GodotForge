import type { GitStatus, GodotRelease, GodotReleaseAsset, ReleaseFlavor, Section } from "../types";

export type Translator = (key: string, values?: Record<string, unknown>) => string;

export function lastOpenedLabel(value: string | null | undefined, t: Translator) {
  if (!value) return t("common.neverOpened");
  const date = new Date(Number(value) * 1000);
  return Number.isNaN(date.getTime()) ? t("common.openedRecently") : date.toLocaleString();
}

export function releaseDate(value: string | null | undefined, t: Translator) {
  if (!value) return t("common.noDate");
  return new Date(value).toLocaleDateString();
}

export function fileSize(bytes: number) {
  if (!bytes) return "0 MB";
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function releaseFlavorLabel(flavor: ReleaseFlavor) {
  return flavor === "dotnet" ? ".NET / Mono" : "Standard";
}

export function releaseChannelLabel(channel: "all" | "stable" | "preview", t: Translator) {
  if (channel === "stable") return t("releases.stable");
  if (channel === "preview") return t("releases.preview");
  return t("releases.allChannels");
}

export function releaseVariantLabel(variant: "all" | ReleaseFlavor, t: Translator) {
  if (variant === "standard") return "Standard";
  if (variant === "dotnet") return ".NET / Mono";
  return t("releases.allVariants");
}

export function normalizedPlatform(platform: string): "linux" | "win" | "macos" {
  if (platform === "win" || platform === "macos") return platform;
  return "linux";
}

export function normalizedArch(arch: string): "x86_64" | "arm64" {
  if (arch === "aarch64" || arch === "arm64") return "arm64";
  return "x86_64";
}

export function releasePlatformLabel(platform: "linux" | "win" | "macos") {
  if (platform === "win") return "Windows";
  if (platform === "macos") return "macOS";
  return "Linux";
}

export function releaseArchLabel(arch: "x86_64" | "arm64") {
  return arch === "arm64" ? "ARM64" : "x86_64";
}

export function assetFlavor(assetName: string): ReleaseFlavor {
  return assetName.toLowerCase().includes("mono") ? "dotnet" : "standard";
}

export function assetPlatform(name: string) {
  const lower = name.toLowerCase();
  if (lower.includes("linux")) return "Linux";
  if (lower.includes("win")) return "Windows";
  if (lower.includes("macos") || lower.includes("osx")) return "macOS";
  if (lower.includes("android")) return "Android";
  if (lower.includes("web")) return "Web";
  return "Build";
}

export function assetKey(release: GodotRelease, asset: GodotReleaseAsset) {
  return `${release.sourceRepository}:${release.id}:${asset.id}`;
}

export function matchesReleaseSystem(assetName: string, platform: "linux" | "win" | "macos", arch: "x86_64" | "arm64") {
  const name = assetName.toLowerCase();
  const platformMatches =
    (platform === "linux" && name.includes("linux")) ||
    (platform === "win" && (name.includes("win64") || name.includes("win32") || name.includes("windows"))) ||
    (platform === "macos" && (name.includes("macos") || name.includes("osx")));

  if (!platformMatches) return false;

  if (arch === "x86_64") {
    const isArmBuild = name.includes("arm64") || name.includes("aarch64");
    return !isArmBuild && (name.includes("64") || name.includes("x86_64") || platform === "macos");
  }

  return name.includes("arm64") || name.includes("aarch64") || platform === "macos";
}

export function projectInitials(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

export function sectionTitle(section: Section, t: Translator) {
  const titles: Record<Section, string> = {
    dashboard: t("sections.dashboardTitle"),
    projects: t("sections.projectsTitle"),
    editors: t("sections.editorsTitle"),
    diagnostics: t("sections.diagnosticsTitle"),
    settings: t("sections.settingsTitle"),
  };
  return titles[section];
}

export function gitStatusLabel(status: GitStatus | null | undefined, loading: boolean, t: Translator) {
  if (loading) return "Checking";
  if (!status) return t("git.unknown");
  if (!status.available) return t("git.gitMissing");
  if (!status.isRepo) return t("git.notInitialized");
  if (status.changedFiles || status.untrackedFiles) return t("common.changes");
  return t("git.clean");
}

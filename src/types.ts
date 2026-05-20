export type GodotEditor = {
  id: string;
  name: string;
  version: string;
  executablePath: string;
  installPath: string;
  architecture: string;
  isDefault: boolean;
};

export type GodotProject = {
  id: string;
  name: string;
  path: string;
  editorId?: string | null;
  favorite: boolean;
  lastOpened?: string | null;
};

export type HubState = {
  editors: GodotEditor[];
  projects: GodotProject[];
  settings: {
    defaultInstallPath: string;
    defaultProjectPath: string;
    /** User-managed release repositories. The official Godot repository is always included by the backend. */
    releaseRepositories: string[];
    /** Optional GitHub token used to raise release API rate limits. */
    githubToken: string;
  };
};

export type DiscoveredEditor = {
  name: string;
  version: string;
  architecture: string;
  executablePath: string;
  installPath: string;
  registered: boolean;
  corrupt: boolean;
  reason?: string | null;
};

export type DiscoveredProject = {
  name: string;
  path: string;
  registered: boolean;
  corrupt: boolean;
  reason?: string | null;
};

export type WorkspaceScan = {
  editors: DiscoveredEditor[];
  projects: DiscoveredProject[];
};

export type GodotReleaseAsset = {
  id: number;
  name: string;
  size: number;
  browserDownloadUrl: string;
};

export type GodotRelease = {
  id: number;
  name?: string | null;
  tagName: string;
  prerelease: boolean;
  publishedAt?: string | null;
  htmlUrl: string;
  assets: GodotReleaseAsset[];
  sourceRepository: string;
};

export type SystemProfile = {
  os: string;
  arch: string;
  godotPlatform: string;
};

export type GitStatus = {
  available: boolean;
  isRepo: boolean;
  branch?: string | null;
  remote?: string | null;
  changedFiles: number;
  untrackedFiles: number;
  summary: string;
};

export type GitLogEntry = {
  hash: string;
  author: string;
  relativeDate: string;
  subject: string;
};

export type GitBranch = {
  name: string;
  current: boolean;
};

export type Section = "projects" | "editors" | "settings";
export type ThemeName = "godotforge" | "godotforge-light";
export type ReleaseFlavor = "standard" | "dotnet";
export type ProjectDetailTab = "overview" | "git" | "settings";

export type PathTarget =
  | "newProjectRoot"
  | "importProjectPath"
  | "settingsInstall"
  | "settingsProject"
  | "moveProjectDestination";

export type DeleteTarget = "project" | "editor";

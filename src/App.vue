<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

type GodotEditor = {
  id: string;
  name: string;
  version: string;
  executablePath: string;
  installPath: string;
  architecture: string;
  isDefault: boolean;
};

type GodotProject = {
  id: string;
  name: string;
  path: string;
  editorId?: string | null;
  favorite: boolean;
  lastOpened?: string | null;
};

type HubState = {
  editors: GodotEditor[];
  projects: GodotProject[];
  settings: {
    defaultInstallPath: string;
    defaultProjectPath: string;
  };
};

type GodotReleaseAsset = {
  id: number;
  name: string;
  size: number;
  browserDownloadUrl: string;
};

type GodotRelease = {
  id: number;
  name?: string | null;
  tagName: string;
  prerelease: boolean;
  publishedAt?: string | null;
  htmlUrl: string;
  assets: GodotReleaseAsset[];
};

type SystemProfile = {
  os: string;
  arch: string;
  godotPlatform: string;
};

const state = reactive<HubState>({
  editors: [],
  projects: [],
  settings: {
    defaultInstallPath: "",
    defaultProjectPath: "",
  },
});

const sections = ["Projetos", "Editores", "Releases", "Paths"] as const;
const activeSection = ref<(typeof sections)[number]>("Projetos");
const activeProjectId = ref("");
const busyAction = ref("");
const error = ref("");
const status = ref("");
const loading = ref(true);
const projectSearch = ref("");
const dismissWelcome = ref(false);
const releases = ref<GodotRelease[]>([]);
const releasesLoaded = ref(false);
const releaseQuery = ref("");
const releaseFlavor = ref<"standard" | "dotnet">("standard");
const downloadTarget = ref("");
const installedAssetKeys = ref<string[]>([]);
const systemProfile = reactive<SystemProfile>({
  os: "unknown",
  arch: "unknown",
  godotPlatform: "linux",
});

const newProject = reactive({
  name: "Novo Jogo",
  rootPath: "",
  editorId: "",
});

const importProjectForm = reactive({
  name: "",
  path: "",
  editorId: "",
});

const newEditor = reactive({
  name: "Godot",
  version: "4.3",
  executablePath: "",
  installPath: "",
  architecture: "x86_64",
  makeDefault: true,
});

const settingsForm = reactive({
  defaultInstallPath: "",
  defaultProjectPath: "",
});

const defaultEditor = computed(() => state.editors.find((editor) => editor.isDefault));
const hasOnboarding = computed(() => !dismissWelcome.value && (!state.editors.length || !state.projects.length));

const sortedProjects = computed(() => {
  const query = projectSearch.value.trim().toLowerCase();

  return [...state.projects]
    .filter((project) => {
      if (!query) return true;
      return `${project.name} ${project.path}`.toLowerCase().includes(query);
    })
    .sort((a, b) => Number(b.favorite) - Number(a.favorite) || a.name.localeCompare(b.name));
});

const activeProject = computed(
  () => state.projects.find((project) => project.id === activeProjectId.value) ?? sortedProjects.value[0],
);

const projectEditor = computed(() => {
  const project = activeProject.value;
  return state.editors.find((editor) => editor.id === project?.editorId) ?? defaultEditor.value;
});

const filteredReleases = computed(() => {
  const query = releaseQuery.value.trim().toLowerCase();
  if (!query) return releases.value;

  return releases.value.filter((release) => {
    const haystack = `${release.tagName} ${release.name ?? ""} ${release.assets.map((asset) => asset.name).join(" ")}`;
    return haystack.toLowerCase().includes(query);
  });
});

function applyState(nextState: HubState) {
  state.editors = nextState.editors;
  state.projects = nextState.projects;
  state.settings = nextState.settings;
  settingsForm.defaultInstallPath = nextState.settings.defaultInstallPath;
  settingsForm.defaultProjectPath = nextState.settings.defaultProjectPath;
  newProject.rootPath ||= nextState.settings.defaultProjectPath;
  newEditor.installPath ||= nextState.settings.defaultInstallPath;

  if (!activeProjectId.value || !state.projects.some((project) => project.id === activeProjectId.value)) {
    activeProjectId.value = state.projects[0]?.id ?? "";
  }
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
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function loadReleases() {
  busyAction.value = "Buscando releases";
  error.value = "";

  try {
    releases.value = await invoke<GodotRelease[]>("fetch_godot_releases", { limit: 8 });
    releasesLoaded.value = true;
    status.value = "Releases carregadas";
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  } finally {
    busyAction.value = "";
  }
}

function createProject() {
  return runAction("Projeto criado", () =>
    invoke<HubState>("create_project", {
      request: { ...newProject, editorId: newProject.editorId || null },
    }),
  );
}

function importProject() {
  return runAction("Projeto importado", () =>
    invoke<HubState>("import_project", {
      request: {
        ...importProjectForm,
        name: importProjectForm.name || null,
        editorId: importProjectForm.editorId || null,
      },
    }),
  );
}

function addEditor() {
  return runAction("Editor cadastrado", () => invoke<HubState>("add_editor", { request: newEditor }));
}

function downloadEditor(release: GodotRelease, asset: GodotReleaseAsset) {
  const key = assetKey(release, asset);
  downloadTarget.value = key;

  return runAction("Editor instalado", async () => {
    const nextState = await invoke<HubState>("download_godot_editor", {
      request: {
        releaseTag: release.tagName,
        assetName: asset.name,
        assetUrl: asset.browserDownloadUrl,
        installPath: settingsForm.defaultInstallPath,
        makeDefault: !state.editors.length,
      },
    });
    installedAssetKeys.value = [...installedAssetKeys.value, key];
    return nextState;
  }).finally(() => {
    downloadTarget.value = "";
  });
}

function saveSettings() {
  return runAction("Paths salvos", () => invoke<HubState>("save_settings", { request: settingsForm }));
}

function launchProject(projectId: string) {
  return runAction("Abrindo projeto", () => invoke<HubState>("launch_project", { projectId }));
}

function removeProject(projectId: string) {
  return runAction("Projeto removido", () => invoke<HubState>("remove_project", { projectId }));
}

function toggleFavorite(projectId: string) {
  return runAction("Projeto atualizado", () => invoke<HubState>("toggle_project_favorite", { projectId }));
}

function setDefaultEditor(editorId: string) {
  return runAction("Editor padrão atualizado", () => invoke<HubState>("set_default_editor", { editorId }));
}

function removeEditor(editorId: string) {
  return runAction("Editor removido", () => invoke<HubState>("remove_editor", { editorId }));
}

function editorLabel(editorId?: string | null) {
  return state.editors.find((editor) => editor.id === editorId)?.version ?? defaultEditor.value?.version ?? "Sem editor";
}

function lastOpenedLabel(value?: string | null) {
  if (!value) return "Nunca aberto";
  const date = new Date(Number(value) * 1000);
  return Number.isNaN(date.getTime()) ? "Aberto recentemente" : date.toLocaleString();
}

function releaseDate(value?: string | null) {
  if (!value) return "Sem data";
  return new Date(value).toLocaleDateString();
}

function fileSize(bytes: number) {
  if (!bytes) return "0 MB";
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function featuredAssets(release: GodotRelease) {
  return release.assets
    .filter((asset) => {
      const name = asset.name.toLowerCase();
      const isZip = name.endsWith(".zip");
      const isEditor = !name.includes("export_templates") && !name.includes("debug_symbols");
      const isFlavor = releaseFlavor.value === "dotnet" ? name.includes("mono") : !name.includes("mono");
      return isZip && isEditor && isFlavor && matchesCurrentSystem(name);
    })
    .slice(0, 8);
}

function matchesCurrentSystem(assetName: string) {
  const name = assetName.toLowerCase();
  const platform = systemProfile.godotPlatform;

  const platformMatches =
    (platform === "linux" && name.includes("linux")) ||
    (platform === "win" && (name.includes("win64") || name.includes("win32") || name.includes("windows"))) ||
    (platform === "macos" && (name.includes("macos") || name.includes("osx"))) ||
    (platform === "android" && name.includes("android"));

  if (!platformMatches) return false;

  if (systemProfile.arch === "x86_64" || systemProfile.arch === "amd64") {
    return name.includes("64") || name.includes("x86_64") || platform === "macos";
  }

  if (systemProfile.arch === "aarch64" || systemProfile.arch === "arm64") {
    return name.includes("arm64") || name.includes("aarch64") || platform === "macos";
  }

  return true;
}

function assetKey(release: GodotRelease, asset: GodotReleaseAsset) {
  return `${release.id}:${asset.id}`;
}

function isAssetInstalling(release: GodotRelease, asset: GodotReleaseAsset) {
  return downloadTarget.value === assetKey(release, asset);
}

function isAssetInstalled(release: GodotRelease, asset: GodotReleaseAsset) {
  return installedAssetKeys.value.includes(assetKey(release, asset));
}

function projectInitials(name: string) {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("");
}

function assetPlatform(name: string) {
  const lower = name.toLowerCase();
  if (lower.includes("linux")) return "Linux";
  if (lower.includes("win")) return "Windows";
  if (lower.includes("macos") || lower.includes("osx")) return "macOS";
  if (lower.includes("android")) return "Android";
  if (lower.includes("web")) return "Web";
  return "Build";
}

function sectionDescription(section: (typeof sections)[number]) {
  const descriptions = {
    Projetos: "Biblioteca local, criação e abertura rápida.",
    Editores: "Versões instaladas, paths e editor padrão.",
    Releases: "Downloads oficiais do repositório da Godot.",
    Paths: "Diretórios usados pelo hub para instalações e projetos.",
  };

  return descriptions[section];
}

onMounted(() => {
  loadSystemProfile();
  loadState();
});
</script>

<template>
  <main class="min-h-screen bg-[#101216] text-slate-100">
    <div class="grid min-h-screen lg:grid-cols-[248px_minmax(0,1fr)]">
      <aside class="hidden border-r border-white/10 bg-[#15181e] lg:flex lg:flex-col">
        <div class="border-b border-white/10 p-5">
          <div class="flex items-center gap-3">
            <div class="grid h-10 w-10 place-items-center rounded-md bg-sky-500 text-sm font-black text-white shadow-lg shadow-sky-950/50">
              GF
            </div>
            <div>
              <strong class="block text-sm">Godot Forge</strong>
              <span class="text-xs text-slate-500">Engine & Project Hub</span>
            </div>
          </div>
        </div>

        <nav class="flex-1 p-3">
          <p class="px-3 pb-2 text-[11px] font-bold uppercase text-slate-500">Library</p>
          <button
            v-for="section in sections"
            :key="section"
            class="mb-1 flex w-full items-center justify-between rounded-md px-3 py-2.5 text-left text-sm font-semibold text-slate-400 transition hover:bg-white/5 hover:text-white"
            :class="{ 'bg-sky-500/15 text-white ring-1 ring-sky-400/20': activeSection === section }"
            @click="activeSection = section"
          >
            <span>{{ section }}</span>
            <span v-if="section === 'Projetos'" class="rounded bg-white/10 px-1.5 text-[11px]">{{ state.projects.length }}</span>
            <span v-if="section === 'Editores'" class="rounded bg-white/10 px-1.5 text-[11px]">{{ state.editors.length }}</span>
          </button>
        </nav>

        <div class="border-t border-white/10 p-4">
          <div class="rounded-lg border border-white/10 bg-black/20 p-3">
            <p class="text-[11px] font-bold uppercase text-slate-500">Default editor</p>
            <strong class="mt-1 block truncate text-sm">
              {{ defaultEditor ? `${defaultEditor.name} ${defaultEditor.version}` : "Nenhum editor" }}
            </strong>
            <p class="mt-1 truncate text-xs text-slate-500">{{ defaultEditor?.executablePath || "Configure uma instalação" }}</p>
          </div>
        </div>
      </aside>

      <section class="min-w-0">
        <header class="sticky top-0 z-20 border-b border-white/10 bg-[#101216]/95 backdrop-blur">
          <div class="flex min-h-16 items-center gap-4 px-4 lg:px-8">
            <div class="lg:hidden">
              <select v-model="activeSection" class="select select-bordered select-sm bg-[#191d24]">
                <option v-for="section in sections" :key="section" :value="section">{{ section }}</option>
              </select>
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-[11px] font-black uppercase text-sky-400">{{ activeSection }}</p>
              <h1 class="truncate text-lg font-black">{{ sectionDescription(activeSection) }}</h1>
            </div>
            <button class="btn btn-sm border-white/10 bg-white/5 text-slate-200 hover:bg-white/10" @click="activeSection = 'Releases'">
              Instalar editor
            </button>
            <button class="btn btn-sm btn-primary" :disabled="!activeProject || !!busyAction" @click="launchProject(activeProject!.id)">
              Launch
            </button>
          </div>
        </header>

        <div class="mx-auto grid max-w-[1500px] gap-6 p-4 lg:p-8">
          <div v-if="loading" class="grid gap-4">
            <div class="h-64 animate-pulse rounded-xl bg-white/5" />
            <div class="h-40 animate-pulse rounded-xl bg-white/5" />
          </div>

          <template v-else>
            <section
              v-if="hasOnboarding"
              class="overflow-hidden rounded-xl border border-white/10 bg-[linear-gradient(135deg,#1d2733_0%,#11141a_55%,#0e1117_100%)] shadow-2xl"
            >
              <div class="grid gap-8 p-6 lg:grid-cols-[1fr_420px] lg:p-8">
                <div class="flex min-h-72 flex-col justify-end">
                  <span class="mb-4 w-fit rounded bg-sky-400/15 px-3 py-1 text-xs font-black uppercase text-sky-300 ring-1 ring-sky-400/20">
                    Setup required
                  </span>
                  <h2 class="max-w-4xl text-4xl font-black leading-none tracking-tight lg:text-6xl">
                    Seu launcher Godot com fluxo de estúdio.
                  </h2>
                  <p class="mt-4 max-w-2xl text-sm leading-6 text-slate-400 lg:text-base">
                    Instale uma versão oficial da Godot, defina paths padrão e mantenha projetos locais prontos para abrir
                    como em Unity Hub, Epic Launcher e Unreal Project Browser.
                  </p>
                  <div class="mt-6 flex flex-wrap gap-3">
                    <button class="btn btn-primary" @click="activeSection = 'Releases'">Baixar Godot</button>
                    <button class="btn border-white/10 bg-white/5 text-slate-100 hover:bg-white/10" @click="activeSection = 'Editores'">
                      Cadastrar manualmente
                    </button>
                    <button class="btn btn-ghost text-slate-400" @click="dismissWelcome = true">Ocultar</button>
                  </div>
                </div>

                <div class="grid content-end gap-3">
                  <div class="rounded-lg border border-white/10 bg-black/25 p-4">
                    <p class="text-xs font-black uppercase text-sky-300">01 Engine install</p>
                    <h3 class="mt-2 font-bold">Releases oficiais</h3>
                    <p class="mt-1 text-sm text-slate-500">Escolha builds estáveis ou pré-release do GitHub da Godot.</p>
                  </div>
                  <div class="rounded-lg border border-white/10 bg-black/25 p-4">
                    <p class="text-xs font-black uppercase text-amber-300">02 Project library</p>
                    <h3 class="mt-2 font-bold">Projetos locais</h3>
                    <p class="mt-1 text-sm text-slate-500">Criação, importação, favoritos e abertura por versão.</p>
                  </div>
                </div>
              </div>
            </section>

            <section class="grid gap-3 md:grid-cols-3">
              <div class="rounded-lg border border-white/10 bg-[#171b22] p-4">
                <p class="text-xs font-bold uppercase text-slate-500">Projects</p>
                <div class="mt-2 flex items-end justify-between">
                  <strong class="text-4xl font-black">{{ state.projects.length }}</strong>
                  <span class="text-xs text-slate-500">{{ sortedProjects.length }} filtered</span>
                </div>
              </div>
              <div class="rounded-lg border border-white/10 bg-[#171b22] p-4">
                <p class="text-xs font-bold uppercase text-slate-500">Engine versions</p>
                <div class="mt-2 flex items-end justify-between">
                  <strong class="text-4xl font-black">{{ state.editors.length }}</strong>
                  <span class="max-w-40 truncate text-xs text-slate-500">{{ defaultEditor?.version || "No default" }}</span>
                </div>
              </div>
              <div class="rounded-lg border border-white/10 bg-[#171b22] p-4">
                <p class="text-xs font-bold uppercase text-slate-500">Project path</p>
                <strong class="mt-2 block truncate text-lg">{{ state.settings.defaultProjectPath || "Not configured" }}</strong>
                <p class="mt-1 text-xs text-slate-500">Default workspace</p>
              </div>
            </section>

            <section v-if="activeSection === 'Projetos'" class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_380px]">
              <div class="grid gap-4">
                <div class="flex flex-col gap-3 rounded-lg border border-white/10 bg-[#171b22] p-4 lg:flex-row lg:items-center">
                  <div class="flex-1">
                    <p class="text-xs font-black uppercase text-sky-400">Project Browser</p>
                    <h2 class="text-2xl font-black">Meus projetos</h2>
                  </div>
                  <input
                    v-model="projectSearch"
                    class="input input-bordered h-10 border-white/10 bg-[#101216] text-sm lg:w-80"
                    placeholder="Buscar por nome ou caminho"
                  />
                </div>

                <div v-if="sortedProjects.length" class="grid gap-4 sm:grid-cols-2 2xl:grid-cols-3">
                  <article
                    v-for="project in sortedProjects"
                    :key="project.id"
                    class="group cursor-pointer overflow-hidden rounded-xl border border-white/10 bg-[#171b22] transition hover:-translate-y-0.5 hover:border-sky-400/50 hover:bg-[#1b2028]"
                    :class="{ 'border-sky-400/70 ring-1 ring-sky-400/30': activeProject?.id === project.id }"
                    @click="activeProjectId = project.id"
                  >
                    <div class="relative grid aspect-[16/9] place-items-center bg-[radial-gradient(circle_at_30%_20%,#2b6f96_0%,#1b3343_34%,#11151b_100%)]">
                      <span class="text-5xl font-black text-white/85">{{ projectInitials(project.name) || "GD" }}</span>
                      <button
                        class="absolute right-3 top-3 rounded bg-black/40 px-2 py-1 text-lg text-amber-300 backdrop-blur"
                        @click.stop="toggleFavorite(project.id)"
                      >
                        {{ project.favorite ? "★" : "☆" }}
                      </button>
                    </div>
                    <div class="grid gap-3 p-4">
                      <div>
                        <h3 class="truncate text-lg font-black">{{ project.name }}</h3>
                        <p class="mt-1 truncate text-xs text-slate-500">{{ project.path }}</p>
                      </div>
                      <div class="flex items-center justify-between gap-3">
                        <span class="rounded bg-white/5 px-2 py-1 text-xs font-bold text-slate-300">{{ editorLabel(project.editorId) }}</span>
                        <button class="btn btn-primary btn-xs" :disabled="!!busyAction" @click.stop="launchProject(project.id)">
                          Launch
                        </button>
                      </div>
                    </div>
                  </article>
                </div>

                <div v-else class="rounded-xl border border-dashed border-white/15 bg-[#171b22] p-10 text-center">
                  <h3 class="text-2xl font-black">Nenhum projeto na biblioteca</h3>
                  <p class="mt-2 text-slate-500">Crie um projeto novo ou importe uma pasta com project.godot.</p>
                </div>

                <div class="grid gap-4 lg:grid-cols-2">
                  <form class="rounded-lg border border-white/10 bg-[#171b22] p-4" @submit.prevent="createProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">Novo projeto</h3>
                      <span class="text-xs text-slate-500">Create</span>
                    </div>
                    <div class="grid gap-3">
                      <input v-model="newProject.name" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Nome do projeto" />
                      <input v-model="newProject.rootPath" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Pasta base" />
                      <select v-model="newProject.editorId" class="select select-bordered border-white/10 bg-[#101216]">
                        <option value="">Usar editor padrão</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn btn-primary" :disabled="!!busyAction">Criar projeto</button>
                    </div>
                  </form>

                  <form class="rounded-lg border border-white/10 bg-[#171b22] p-4" @submit.prevent="importProject">
                    <div class="mb-4 flex items-center justify-between">
                      <h3 class="font-black">Importar projeto</h3>
                      <span class="text-xs text-slate-500">Import</span>
                    </div>
                    <div class="grid gap-3">
                      <input v-model="importProjectForm.name" class="input input-bordered border-white/10 bg-[#101216]" placeholder="Nome opcional" />
                      <input v-model="importProjectForm.path" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="/path/do/projeto" />
                      <select v-model="importProjectForm.editorId" class="select select-bordered border-white/10 bg-[#101216]">
                        <option value="">Usar editor padrão</option>
                        <option v-for="editor in state.editors" :key="editor.id" :value="editor.id">
                          {{ editor.name }} {{ editor.version }}
                        </option>
                      </select>
                      <button class="btn border-white/10 bg-white/5 text-slate-100 hover:bg-white/10" :disabled="!!busyAction">Importar</button>
                    </div>
                  </form>
                </div>
              </div>

              <aside class="h-fit rounded-xl border border-white/10 bg-[#171b22] p-5">
                <p class="text-xs font-black uppercase text-sky-400">Inspector</p>
                <template v-if="activeProject">
                  <h2 class="mt-2 text-3xl font-black">{{ activeProject.name }}</h2>
                  <p class="mt-2 break-all text-sm text-slate-500">{{ activeProject.path }}</p>
                  <div class="my-5 h-px bg-white/10" />
                  <div class="grid gap-3 text-sm">
                    <div class="flex justify-between gap-3">
                      <span class="text-slate-500">Engine</span>
                      <strong>{{ projectEditor?.version ?? "Sem editor" }}</strong>
                    </div>
                    <div class="flex justify-between gap-3">
                      <span class="text-slate-500">Último uso</span>
                      <strong class="text-right">{{ lastOpenedLabel(activeProject.lastOpened) }}</strong>
                    </div>
                    <div class="flex justify-between gap-3">
                      <span class="text-slate-500">Favorito</span>
                      <strong>{{ activeProject.favorite ? "Sim" : "Não" }}</strong>
                    </div>
                  </div>
                  <div class="mt-6 grid gap-2">
                    <button class="btn btn-primary" :disabled="!!busyAction" @click="launchProject(activeProject.id)">Launch project</button>
                    <button class="btn border-white/10 bg-white/5 text-slate-100 hover:bg-white/10" :disabled="!!busyAction" @click="toggleFavorite(activeProject.id)">
                      {{ activeProject.favorite ? "Remover favorito" : "Adicionar favorito" }}
                    </button>
                    <button class="btn btn-error btn-outline" :disabled="!!busyAction" @click="removeProject(activeProject.id)">Remover da biblioteca</button>
                  </div>
                </template>
                <p v-else class="mt-3 text-sm text-slate-500">Selecione um projeto para ver detalhes e ações.</p>
              </aside>
            </section>

            <section v-if="activeSection === 'Editores'" class="grid gap-6">
              <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
                <article
                  v-for="editor in state.editors"
                  :key="editor.id"
                  class="rounded-xl border border-white/10 bg-[#171b22] p-5"
                >
                  <div class="flex items-start justify-between gap-4">
                    <div class="min-w-0">
                      <p class="text-xs font-black uppercase text-sky-400">Installed Engine</p>
                      <h3 class="mt-2 truncate text-2xl font-black">{{ editor.name }}</h3>
                      <p class="font-bold text-slate-300">{{ editor.version }}</p>
                    </div>
                    <span class="rounded px-2 py-1 text-xs font-black" :class="editor.isDefault ? 'bg-sky-500 text-white' : 'bg-white/10 text-slate-300'">
                      {{ editor.isDefault ? "DEFAULT" : editor.architecture }}
                    </span>
                  </div>
                  <div class="mt-5 grid gap-2 rounded-lg bg-black/20 p-3 text-xs text-slate-500">
                    <p class="truncate">{{ editor.executablePath }}</p>
                    <p class="truncate">{{ editor.installPath }}</p>
                  </div>
                  <div class="mt-5 flex flex-wrap gap-2">
                    <button class="btn btn-sm border-white/10 bg-white/5 text-slate-100 hover:bg-white/10" :disabled="editor.isDefault || !!busyAction" @click="setDefaultEditor(editor.id)">
                      Tornar padrão
                    </button>
                    <button class="btn btn-sm btn-error btn-outline" :disabled="!!busyAction" @click="removeEditor(editor.id)">Remover</button>
                  </div>
                </article>

                <div v-if="!state.editors.length" class="rounded-xl border border-dashed border-white/15 bg-[#171b22] p-8">
                  <h3 class="text-2xl font-black">Nenhuma engine instalada</h3>
                  <p class="mt-2 text-slate-500">Use Releases para baixar uma build oficial ou cadastre manualmente.</p>
                </div>
              </div>

              <form class="rounded-xl border border-white/10 bg-[#171b22] p-5" @submit.prevent="addEditor">
                <div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
                  <div>
                    <p class="text-xs font-black uppercase text-sky-400">Manual setup</p>
                    <h2 class="text-2xl font-black">Cadastrar instalação local</h2>
                  </div>
                  <button class="btn btn-primary" :disabled="!!busyAction">Cadastrar editor</button>
                </div>
                <div class="grid gap-3 lg:grid-cols-2">
                  <input v-model="newEditor.name" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Nome" />
                  <input v-model="newEditor.version" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Versão" />
                  <input v-model="newEditor.executablePath" class="input input-bordered border-white/10 bg-[#101216] lg:col-span-2" required placeholder="Path do executável" />
                  <input v-model="newEditor.installPath" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Pasta de instalação" />
                  <input v-model="newEditor.architecture" class="input input-bordered border-white/10 bg-[#101216]" required placeholder="Arquitetura" />
                </div>
                <label class="mt-4 flex w-fit cursor-pointer items-center gap-3 text-sm font-bold text-slate-300">
                  <input v-model="newEditor.makeDefault" type="checkbox" class="checkbox checkbox-primary" />
                  Definir como padrão
                </label>
              </form>
            </section>

            <section v-if="activeSection === 'Releases'" class="grid gap-4">
              <div class="rounded-xl border border-white/10 bg-[#171b22] p-5">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
                  <div>
                    <p class="text-xs font-black uppercase text-sky-400">Official GitHub releases</p>
                    <h2 class="text-2xl font-black">Instalar versões da Godot</h2>
                    <p class="mt-1 text-sm text-slate-500">
                      Mostrando builds compatíveis com {{ systemProfile.os }} / {{ systemProfile.arch }}.
                    </p>
                  </div>
                  <div class="flex flex-col gap-2 lg:items-end">
                    <div class="join">
                      <button
                        class="btn join-item btn-sm"
                        :class="releaseFlavor === 'standard' ? 'btn-primary' : 'border-white/10 bg-white/5 text-slate-100 hover:bg-white/10'"
                        @click="releaseFlavor = 'standard'"
                      >
                        Standard
                      </button>
                      <button
                        class="btn join-item btn-sm"
                        :class="releaseFlavor === 'dotnet' ? 'btn-primary' : 'border-white/10 bg-white/5 text-slate-100 hover:bg-white/10'"
                        @click="releaseFlavor = 'dotnet'"
                      >
                        .NET / Mono
                      </button>
                    </div>
                    <div class="flex flex-col gap-2 sm:flex-row">
                    <input v-model="releaseQuery" class="input input-bordered border-white/10 bg-[#101216]" placeholder="Filtrar versão ou asset" />
                    <button class="btn btn-primary" :disabled="!!busyAction" @click="loadReleases">
                      {{ releasesLoaded ? "Atualizar" : "Buscar releases" }}
                    </button>
                    </div>
                  </div>
                </div>
              </div>

              <div v-if="!releasesLoaded" class="rounded-xl border border-dashed border-white/15 bg-[#171b22] p-10 text-center">
                <h3 class="text-2xl font-black">Catálogo ainda não carregado</h3>
                <p class="mt-2 text-slate-500">Busque releases oficiais e instale a build certa para seu sistema.</p>
              </div>

              <article
                v-for="release in filteredReleases"
                :key="release.id"
                class="rounded-xl border border-white/10 bg-[#171b22] p-5"
              >
                <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div>
                    <div class="flex flex-wrap items-center gap-2">
                      <h3 class="text-2xl font-black">{{ release.tagName }}</h3>
                      <span class="rounded px-2 py-1 text-xs font-black" :class="release.prerelease ? 'bg-amber-400 text-black' : 'bg-emerald-500 text-white'">
                        {{ release.prerelease ? "PREVIEW" : "STABLE" }}
                      </span>
                    </div>
                    <p class="mt-1 text-sm text-slate-500">{{ release.name || "Godot release" }} · {{ releaseDate(release.publishedAt) }}</p>
                  </div>
                  <a class="btn btn-sm border-white/10 bg-white/5 text-slate-100 hover:bg-white/10" :href="release.htmlUrl" target="_blank">
                    GitHub
                  </a>
                </div>

                <div class="mt-5 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                  <div
                    v-for="asset in featuredAssets(release)"
                    :key="asset.id"
                    class="grid gap-4 rounded-lg border border-white/10 bg-black/20 p-4"
                  >
                    <div>
                      <span class="rounded bg-sky-400/15 px-2 py-1 text-xs font-black text-sky-300">{{ assetPlatform(asset.name) }}</span>
                      <span class="ml-2 rounded bg-white/10 px-2 py-1 text-xs font-black text-slate-300">
                        {{ releaseFlavor === "dotnet" ? ".NET" : "Standard" }}
                      </span>
                      <p class="mt-3 min-h-12 break-all text-sm font-bold text-slate-200">{{ asset.name }}</p>
                      <p class="mt-2 text-xs text-slate-500">{{ fileSize(asset.size) }}</p>
                    </div>
                    <button
                      class="btn btn-sm"
                      :class="isAssetInstalled(release, asset) ? 'btn-success' : 'btn-primary'"
                      :disabled="!!busyAction || isAssetInstalled(release, asset)"
                      @click="downloadEditor(release, asset)"
                    >
                      <span v-if="isAssetInstalling(release, asset)" class="loading loading-spinner loading-xs" />
                      {{
                        isAssetInstalling(release, asset)
                          ? "Instalando..."
                          : isAssetInstalled(release, asset)
                            ? "Instalado"
                            : "Baixar e instalar"
                      }}
                    </button>
                  </div>
                </div>
                <div v-if="!featuredAssets(release).length" class="mt-5 rounded-lg border border-dashed border-white/10 bg-black/20 p-4 text-sm text-slate-500">
                  Nenhum asset {{ releaseFlavor === "dotnet" ? ".NET/Mono" : "Standard" }} compatível com
                  {{ systemProfile.os }} / {{ systemProfile.arch }} nesta release.
                </div>
              </article>
            </section>

            <section v-if="activeSection === 'Paths'" class="rounded-xl border border-white/10 bg-[#171b22] p-5">
              <form class="grid gap-5" @submit.prevent="saveSettings">
                <div>
                  <p class="text-xs font-black uppercase text-sky-400">Workspace settings</p>
                  <h2 class="text-2xl font-black">Paths padrão</h2>
                  <p class="mt-1 text-sm text-slate-500">Controle onde o Forge instala engines e cria projetos.</p>
                </div>
                <div class="grid gap-4 lg:grid-cols-2">
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-slate-300">Instalações da Godot</span>
                    <input v-model="settingsForm.defaultInstallPath" class="input input-bordered border-white/10 bg-[#101216]" required />
                  </label>
                  <label class="grid gap-2">
                    <span class="text-sm font-bold text-slate-300">Projetos</span>
                    <input v-model="settingsForm.defaultProjectPath" class="input input-bordered border-white/10 bg-[#101216]" required />
                  </label>
                </div>
                <div>
                  <button class="btn btn-primary" :disabled="!!busyAction">Salvar paths</button>
                </div>
              </form>
            </section>
          </template>
        </div>

        <div v-if="status || error || busyAction" class="toast toast-end z-30">
          <div class="alert border border-white/10 shadow-xl" :class="error ? 'alert-error' : busyAction ? 'alert-info' : 'alert-success'">
            <span>{{ error || busyAction || status }}</span>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

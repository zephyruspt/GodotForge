# Godot Forge

Desktop hub for managing Godot editor installations and local projects, inspired by Unity Hub-style workflows.

## Features

- Tailwind CSS 4 and DaisyUI 5 interface;
- welcome/onboarding screen for first-time setup;
- multiple Godot executable registrations;
- official release lookup from `godotengine/godot`;
- automatic download, extraction, and registration of Godot `.zip` assets;
- default editor by version/architecture;
- default paths for editor installations and projects;
- project creation with `project.godot`;
- existing project import;
- favorites, search, and library removal;
- editor launch with `godot --editor --path <project>`;
- per-project local Git integration: repository detection, branch, remote, changes, and `.git` initialization with a Godot `.gitignore`;
- per-project Git management: list/switch/create branches, configure `origin`, view recent logs, and push the current branch;
- dedicated project page with overview, Git/recent log, settings, and project folder move support.

## Development

```bash
npm install
npm run build
npm run tauri dev
```

Local state is stored at `~/.config/godot-forge/hub-state.json` on Linux/XDG environments.

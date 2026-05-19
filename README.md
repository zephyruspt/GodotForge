# Godot Forge

Desktop hub para gerenciar instalações da Godot e projetos locais, inspirado no fluxo do Unity Hub.

## Recursos

- interface com Tailwind CSS 4 e DaisyUI 5;
- tela de welcome/onboarding para primeira configuração;
- cadastro de múltiplos executáveis da Godot;
- busca de releases oficiais do repositório `godotengine/godot`;
- download, extração e cadastro automático de assets `.zip` da Godot;
- editor padrão por versão/arquitetura;
- paths padrão para instalações e projetos;
- criação de projetos com `project.godot`;
- importação de projetos existentes;
- favoritos, busca e remoção da biblioteca;
- abertura do projeto com `godot --path <projeto>`.

## Desenvolvimento

```bash
npm install
npm run build
npm run tauri dev
```

O estado local fica em `~/.config/godot-forge/hub-state.json` em ambientes Linux/XDG.

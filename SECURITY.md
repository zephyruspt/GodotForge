# Security Policy

Godot Forge is developed by ZEPHYRUS PROSPERITY - UNIPESSOAL LDA.

## Supported Versions

Security fixes are handled for the latest unreleased `main` branch until formal releases are introduced.

## Reporting a Vulnerability

Please report security issues privately to the project maintainers before public disclosure.

Do not include live credentials, private keys, personal access tokens, recovery phrases, or production secrets in reports. If a proof of concept needs a token-like value, use a clearly fake placeholder such as `REDACTED_TEST_TOKEN`.

## Social Engineering Policy

Godot Forge maintainers will never ask for:

- GitHub passwords;
- Godot credentials;
- MFA or recovery codes;
- SSH private keys;
- wallet keys or recovery phrases;
- remote desktop access to unrelated systems;
- unsigned replacement builds sent through private messages.

Treat unexpected credential requests, private build links, or pressure to bypass normal review as suspicious.

Godot Forge has no hidden support token, debug token, admin override code, recovery phrase, or maintainer unlock code. Anyone asking for one is attempting social engineering.

## Local Data

Godot Forge stores editor registrations, project paths, and settings in the local user configuration directory. Git operations run against local project folders and only contact remotes when the user explicitly performs a Git action such as push.

Privacy reports redact GitHub tokens. On Unix platforms, state and activity-log files are written with owner-only permissions when supported by the operating system.

## Brand and Impersonation

The Godot Forge name, logo, icons, banners, and related brand assets are not licensed for reuse. Report impersonation attempts or unauthorized brand usage privately.

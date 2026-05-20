# Privacy Notice

Godot Forge is a local-first desktop application. It is designed to manage Godot editor installations, project paths, release metadata, and Git workflow helpers on the user's machine.

This notice is not legal advice. It documents the current product behavior so operators and distributors can assess their own obligations under laws such as GDPR, UK GDPR, LGPD, CCPA/CPRA, and similar privacy frameworks.

## Data Stored Locally

Godot Forge may store:

- Project names and local project paths.
- Godot editor names, versions, executable paths, and installation paths.
- Release repository URLs configured by the user.
- Optional GitHub token used for release API requests. When supported by the operating system, this token is stored in the system secret store rather than the main workspace state file.
- Release cache metadata from configured release repositories.
- Local activity logs for actions performed in the app.

Godot Forge does not intentionally collect payment data, passwords, recovery phrases, biometric data, government IDs, or analytics identifiers.

## Network Requests

Godot Forge makes network requests only when release metadata or editor archives are fetched from configured release repositories. The default release source is the official Godot repository. Users may add additional release repositories.

Git operations are local unless the user explicitly configures a remote and pushes a branch.

## User Controls

Users can:

- View a privacy report in the Diagnostics page.
- Clear auxiliary release cache and activity logs.
- Remove projects and editors from the Forge library without deleting project folders.
- Remove or replace release repositories.
- Remove the GitHub token by using the removal control in Settings and saving.

## Security Measures

- Local state and activity log files are written with owner-only permissions on Unix platforms when supported by the operating system.
- Privacy reports redact GitHub tokens and only show whether a token is configured.
- Security messaging warns users against social-engineering requests for passwords, recovery codes, private keys, or hidden support/admin tokens.
- Release cache contains public release metadata and is treated as auxiliary data that can be cleared from the Diagnostics page.

## Important Operator Notes

Organizations distributing or operating Godot Forge should verify:

- Whether they are a controller, processor, business, service provider, or equivalent role under applicable law.
- Whether additional notices, data processing agreements, retention policies, or data subject request workflows are required.
- Whether enterprise deployments need additional managed-device controls, such as full-disk encryption, MDM policy enforcement, access logging, or centralized retention policies.

## Contact

For privacy or security concerns, contact the repository maintainers through the official project channels.

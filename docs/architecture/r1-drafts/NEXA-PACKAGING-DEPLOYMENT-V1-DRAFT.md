# Nexa v1 Packaging and Deployment Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define how the first Nexa release and its identical Windows desktop and same-machine browser clients are built, installed, configured, upgraded, diagnosed, and removed on the first supported target.

## 2. v1 deployment model

Nexa v1 deploys identical Windows desktop and same-machine browser clients over one local Rust runtime and one versioned loopback HTTP/WebSocket business API. Hosted, remote/LAN client, server/fleet, and multi-tenant deployment are post-v1 unless new owner and architecture authority changes scope.

The release must declare one primary supported OS/architecture combination before packaging technology is selected.

## 3. Release artifact set

A v1 release must produce, as applicable:

- learner application executable/package;
- required runtime libraries/assets;
- governed starter course/knowledge content or an explicit content-install mechanism;
- schema/migration resources;
- default non-secret configuration;
- license notices and third-party attribution;
- version/provenance metadata;
- release notes and known limitations.

Development fixtures and scripted test providers must not be accidentally packaged as production dependencies unless explicitly intended and harmless.

## 4. Build reproducibility and provenance

Release builds must:

- use the committed dependency lockfile;
- identify source commit/release version;
- use documented build commands/toolchain;
- preserve third-party dependency/license provenance;
- identify externally supplied model/runtime binaries if distributed;
- produce artifact hashes/signatures/provenance according to the chosen platform release mechanism;
- avoid embedding provider credentials or developer-specific paths.

## 5. Installation

The installer/package must define:

- application installation location;
- mutable user data location;
- configuration location;
- logs/diagnostics location;
- cache/temp location;
- governed course/content location or import mechanism;
- separately installed LM Studio endpoint/model configuration location;
- required OS/runtime prerequisites;
- whether installation requires elevated privilege and why.

Normal application execution should use ordinary user privileges.

## 6. First-run configuration

The application must detect and guide configuration for:

- durable data initialization;
- released course/content availability;
- separately installed graphical LM Studio endpoint and configured model;
- local runtime/API readiness for both identical clients;
- bundled speech readiness and device configuration;
- synchronized animated 2D dependency readiness.

Configuration validation must occur before the learner begins an interaction that cannot succeed.

## 7. Provider configuration

Packaging must not couple provider-neutral domain crates to one vendor SDK.

The application/adapters include one release-supported provider path: separately installed graphical LM Studio on the same machine. Configuration must identify:

- LM Studio endpoint and model;
- loopback/local availability without a remote-provider credential flow;
- model/tokenizer compatibility data required by the concrete adapter.

Remote inference, custom remote endpoints, credentials, and dynamic multi-provider routing are not v1 deliverables.

## 8. Persistent data location and lifecycle

The package/deployment specification must reference the data/privacy specifications and ensure:

- user data survives ordinary application upgrades;
- uninstall behavior is explicit about retained/deleted learner data;
- application binaries and mutable learner data are not conflated;
- migrations run only through the supported application/update path;
- backups/exports are not stored inside locations automatically removed without warning if the product promises retention.

## 9. Upgrade

Before installing/launching a newer version against existing data:

- verify supported predecessor/schema version;
- back up or otherwise establish the approved recovery point for irreversible migrations;
- execute ordered verified migration;
- fail safely on migration error;
- do not start normal learner interactions against a partially migrated store;
- record application/schema/content version evidence needed for diagnostics.

Once v1 has a released predecessor, upgrade tests from every supported predecessor version become release acceptance requirements.

## 10. Rollback

Full binary downgrade compatibility is not automatically required.

The release process must nevertheless define what a user/operator can do after failed update/migration:

- retry the same verified migration;
- restore a supported backup/recovery point;
- reinstall the same application version;
- obtain safe diagnostics.

Do not imply that an older binary can open a newer migrated store unless explicitly supported/tested.

## 11. Uninstall/reset

The product must distinguish:

- uninstalling application binaries;
- deleting caches/logs;
- deleting learner data;
- deleting LM Studio endpoint/model configuration;
- removing downloaded model/content assets.

Destructive learner-data deletion must be explicit. Uninstall must distinguish Nexa-owned state from data retained by the separately installed LM Studio application.

## 12. Updates

v1 may use manual update/reinstall if that path is secure, documented, and migration-safe. Automatic update is not required unless explicitly selected.

If automatic update is included, it requires:

- authenticated/integrity-verified update metadata/artifacts;
- safe download/staging;
- migration coordination;
- failure recovery;
- explicit update channel/version policy.

Do not add an updater solely for architectural completeness.

## 13. Offline installation/runtime posture

The release documentation must accurately state:

- whether installation itself requires network access;
- whether course/content is bundled or downloaded;
- that v1 model inference requires the separately installed same-machine LM Studio server but no remote provider;
- which learner capabilities remain usable offline;
- whether local model assets are optional/supported.

`Local-first` must not be used to imply fully offline inference unless the shipped configuration actually supports it.

## 14. Course/content distribution

The first released course may be bundled with the application or installed through a governed content package.

Requirements:

- immutable/versioned content identity;
- integrity validation;
- compatibility with application/spec/schema version;
- provenance/licensing;
- update/migration impact on existing learner progress explicitly handled.

A general authoring/content marketplace is post-v1.

## 15. Logs and diagnostics

Package-specific documentation must state:

- log location;
- log rotation/retention behavior;
- safe diagnostic export path if included;
- how to obtain application/version/provider/content IDs for support;
- no default collection/transmission of diagnostics unless explicitly configured and governed.

## 16. Release versioning

Every release artifact and running application must expose a version that can be associated with:

- source revision;
- persistent schema compatibility range;
- released content compatibility where needed;
- concrete adapter/provider compatibility notes;
- release notes/known limitations.

The broader ecosystem versioning standard may inform future multi-component releases; v1 needs at minimum reproducible application/data/content compatibility evidence.

## 17. Supported platform matrix

Before approval, define at minimum:

- OS family/version floor;
- CPU architecture;
- memory/storage minimum/recommended values based on performance evidence;
- GPU requirement only if the required v1 path actually needs one;
- loopback networking requirements for both clients and LM Studio;
- accessibility/platform assumptions;
- local configuration protection appropriate to the selected shell/runtime.

Unsupported platforms may still compile, but are not release-supported without acceptance evidence.

## 18. Clean-machine acceptance

On a clean supported machine, a tester must be able to:

1. obtain the release artifacts;
2. verify/install them according to the supported path;
3. launch without development tools/source checkout;
4. configure the supported model path;
5. initialize/load the released course;
6. complete the primary learner acceptance scenario;
7. restart/resume;
8. inspect version/diagnostic information;
9. perform a supported update once a predecessor exists;
10. uninstall with the documented learner-data disposition.

## 19. Decisions required for approval

- first supported OS/version/architecture;
- concrete installer/package technology;
- release signing/provenance mechanism;
- exact LM Studio endpoint/model configuration and compatibility validation;
- bundled vs separately installed course/content;
- manual vs automatic update for v1;
- application/data/log/config locations;
- hardware minimums after performance measurement.

## 20. Explicit post-v1 scope unless promoted

- server/cloud deployment;
- fleet management;
- organization-wide installers/policy management;
- automatic multi-channel updater;
- package repositories/marketplace;
- plugin distribution;
- broad multi-OS certification beyond the first supported target;
- cloud content synchronization.

## 2026-08-26 ADR-0069 reconciliation

The v1 release supplies identical Windows desktop and same-machine browser clients over one local Rust runtime and versioned loopback HTTP/WebSocket API, plus governed content, migrations, and required bundled-speech assets selected after G2. It excludes LLM weights and inference runtime and integrates with separately installed graphical LM Studio. React/TypeScript/Vite and Tauri 2 remain G1 candidates until evidence and a later authority update select production technologies; the concrete bundled-speech technology remains G2-gated. Hosted deployment, remote inference, and remote/LAN client access remain post-v1.

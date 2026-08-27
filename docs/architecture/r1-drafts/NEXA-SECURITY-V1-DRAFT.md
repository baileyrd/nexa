# Nexa v1 Security Architecture Specification — Draft

Status: R1 proposal; non-authoritative until registered and approved

## 1. Purpose

Define the minimum security architecture required for the Nexa v1 learner journey. The actual v1 trust boundaries include identical Windows desktop and same-machine browser clients, one local Rust runtime and loopback API, durable local data, governed content, and separately installed same-machine LM Studio. Remote inference is post-v1.

## 2. Security objectives

Nexa v1 must:

- preserve learner-state integrity;
- protect local configuration and any future provider credentials/secrets;
- prevent unapproved remote disclosure;
- enforce least privilege for local files/process/network access;
- keep untrusted model/content bytes from gaining structural authority;
- fail closed on unsupported/invalid security state;
- produce content-safe diagnostics;
- provide trustworthy installation/update provenance;
- avoid claiming sandbox/tool security unless actual enforcement exists.

## 3. Trust boundaries

### 3.1 Learner to the shared clients

Learner text and files are untrusted input. The application validates all structured inputs before domain use and must not interpret learner-controlled content as configuration, executable policy, or privileged control.

### 3.2 Desktop application to local durable data

The application trusts only data that passes current schema/domain validation. Existing local files are not inherently trusted merely because they were produced by an earlier Nexa version.

### 3.3 Tutor/knowledge to LM Studio boundary

Prompt/model input is a controlled boundary. The v1 adapter communicates only with separately installed same-machine LM Studio; any later remote transmission requires new explicit authority and the privacy specification’s disclosure controls.

Model output is untrusted content until structural admission and required quality/safety gates complete.

### 3.4 Application to package/update source

Release artifacts and updates require provenance/integrity verification appropriate to the selected packaging mechanism.

### 3.5 Conditional lab/tool boundary

If labs/tools are included in v1, tool execution is an untrusted execution boundary requiring real sandbox/enforcement architecture. Existing contract/admission/cancellation declarations alone do not satisfy this boundary.

## 4. Threat model — minimum v1 classes

R1 security review must address at least:

- unsafe LM Studio endpoint/model configuration and any future stolen/leaked provider credentials;
- prompt/content disclosure beyond configured policy;
- prompt injection or untrusted knowledge attempting to control host behavior;
- malicious/corrupt persisted state;
- path traversal or unsafe local file access;
- unsafe executable/tool invocation if conditional tooling is enabled;
- dependency/package tampering;
- unsafe diagnostics containing learner, prompt, knowledge, model-output, or secret content;
- replay/reassociation of identities/evidence across learner/session/model operations;
- downgrade/unsupported schema or protocol confusion;
- denial/resource exhaustion from bounded external inputs;
- network destination misuse for remote integrations.

## 5. Credential and secret handling

Provider/API credentials must:

- never be stored in ordinary learner/domain records;
- never be committed to repository content or baked into release artifacts;
- use the supported platform's secure secret/credential mechanism where feasible;
- be referenced by opaque configuration rather than copied through domain contracts;
- never appear in normal logs, error strings, `Debug`, telemetry, crash summaries, or tutor/model payload evidence;
- support explicit replacement/removal;
- be loaded only by the concrete adapter that requires them or a dedicated security/configuration service;
- be excluded from exported learner data unless the user explicitly requests configuration export through a separately governed secure mechanism.

Plaintext configuration-file secrets require an explicit accepted exception; they are not the default architecture.

## 6. Network security

For remote model/provider paths:

- outbound destinations must derive from approved provider configuration, not model/learner content;
- transport encryption/server authentication must use the provider SDK/platform security expected by the supported environment;
- redirects/custom endpoints must be controlled by configuration policy;
- timeout and response-size bounds must be enforced;
- proxy/custom-certificate behavior, if supported, must be documented;
- no inbound listening service is required for the default v1 learner app unless separately specified;
- disabling remote inference must remove the remote model network path from the primary learner journey.

## 7. Structural authority separation

Existing architectural invariants remain mandatory:

- model output cannot supply host-owned identities, limits, policy versions, authorization, capabilities, or renderer commands;
- tutor intelligence cannot directly manipulate animation primitives;
- untrusted knowledge/learner/model text remains inert data unless an owning subsystem explicitly interprets a bounded schema;
- storage data is revalidated at trust boundaries;
- serialized extensions/unknown required fields fail according to the governing compatibility policy.

## 8. Local filesystem and process privilege

The desktop application should operate with ordinary user privileges.

v1 must specify and confine:

- application install location;
- mutable configuration location;
- durable data location;
- cache/temp location;
- log location;
- governed imported course/knowledge paths;
- any provider/model local file paths.

The application must not require administrator/root privileges for normal execution after installation unless platform packaging strictly requires an installation step with elevated privilege.

User-supplied paths must be normalized/validated according to owning feature requirements. Knowledge ingestion must not imply arbitrary executable loading.

## 9. Persistence integrity

The security specification depends on the data specification for transactional correctness. Security additionally requires:

- schema/domain validation before trust;
- canonical identity association checks;
- failure on conflicting identity reuse;
- protection against silent stale-write overwrite according to the v1 concurrency model;
- migration integrity and fail-safe behavior;
- no automatic replacement of corrupt authoritative learner state with fabricated defaults without explicit user-visible recovery semantics.

Encryption-at-rest is a separate decision based on threat model, platform, and privacy requirements; it is not implied solely by local persistence.

## 10. Post-v1 remote disclosure authorization

Remote model invocation is not a v1 path. If explicitly authorized after v1, it may occur only when:

1. the configured provider/model is permitted for remote use;
2. the prompt/input has passed the approved privacy/disclosure policy;
3. required provider credentials/configuration are available;
4. the exact invocation is bound to trusted session/workflow/model identities;
5. security/size/version preflight succeeds.

Existing ADR-0031/0033/0034 evidence may be reused where applicable, but caller-supplied structural authorization is not automatically the complete product security policy.

## 11. Model and knowledge security

### 11.1 Model output

Raw output is untrusted. It must pass current structural admission and any v1 semantic/safety evaluation before becoming an accepted tutor response.

### 11.2 Prompt injection / hostile content

Nexa must distinguish instructional content from host instructions. Governed knowledge content may contain text that looks like commands/system prompts; it remains knowledge data unless the prompt architecture intentionally assigns an instruction-bearing trusted layer.

The tutor/knowledge v1 specification must define the relevant prompt-injection acceptance tests for the released corpus/provider path.

### 11.3 Assessment protection

Existing assessment-exposure restrictions remain fail-closed. Remote/provider disclosure must not bypass assessment protection.

## 12. Diagnostics and telemetry

Normal diagnostics must exclude raw:

- learner text;
- assessment responses;
- governed source/knowledge text;
- compiled prompts;
- raw model responses;
- credentials/tokens;
- unrestricted filesystem content.

Use canonical IDs, hashes/replay anchors, bounded classifications, error codes, timings, counts, and versions instead.

Any opt-in diagnostic mode that captures content requires separate privacy/security policy, storage location, retention, and clear user control.

## 13. Dependency and supply-chain requirements

Before release:

- dependencies used in release-critical adapters must have documented source/license/version provenance;
- lockfiles/reproducible resolution must be preserved;
- known critical vulnerabilities affecting the release path must be reviewed and dispositioned;
- release packaging must not silently download/execute arbitrary code during normal startup;
- third-party model/runtime binaries, if distributed, require explicit provenance and integrity handling.

The exact vulnerability scanning/tooling requirement belongs to engineering/release standards.

## 14. Update security

The packaging specification must define how release/update artifacts are authenticated/integrity checked according to the selected platform mechanism.

An update must not silently migrate persistent data using unverified binaries. Failed update/migration behavior must preserve recoverability according to the data specification.

## 15. Labs/tools conditional security gate

If Tool/Lab Execution is promoted to v1, approval requires a separate enforcement specification that defines at minimum:

- sandbox/isolation technology;
- process/user privileges;
- filesystem/network/resource restrictions;
- allowed executable/tool catalog or admission policy;
- destructive/privileged confirmation UX;
- secret isolation;
- execution observation/evidence;
- timeout/cancellation and orphan-process cleanup;
- escape/threat testing.

`nexa-labs` contract declarations alone are foundation evidence, not sandbox proof.

## 16. Security verification requirements

Before v1 release, verify at least:

- no credentials in repository, normal logs, serialized domain records, or redacted errors;
- remote invocation cannot occur in v1 and any later path cannot occur without approved configuration/disclosure authority;
- model output cannot override host-owned authority;
- malformed/corrupt persisted state fails safely;
- identity/evidence reassociation attacks fail closed on critical paths;
- learner/knowledge input size/bounds are enforced;
- filesystem paths used by v1 are constrained to intended operations;
- the v1 LM Studio integration is constrained to the authorized same-machine endpoint;
- release/package provenance mechanism is exercised;
- dependency vulnerability review has no unresolved release-blocking finding;
- conditional labs/tools satisfy their additional security gate if included.

## 17. Decisions required for approval

- Windows desktop and same-machine browser security facilities;
- credential-store mechanism;
- exact same-machine LM Studio endpoint restrictions;
- post-v1 remote/custom endpoint policy if later authorized;
- encryption-at-rest requirement based on threat/privacy model;
- update/package signing/provenance mechanism;
- labs/tools v1 disposition.

## 18. Explicit post-v1 scope unless promoted

- multi-user/enterprise authentication;
- RBAC/organization administration;
- remote server ingress/API security;
- plugin permission ecosystem;
- fleet key management;
- generalized sandbox infrastructure if labs are not v1;
- cloud sync threat model.

## 2026-08-26 ADR-0069 reconciliation

The v1 trust surface includes a normal same-machine browser and a candidate desktop shell against one loopback-only versioned HTTP/WebSocket API. Binding, origin/authorization, protocol validation, WebSocket lifecycle, and prevention of a second shell-specific business API require evidence. React/TypeScript/Vite and Tauri 2 remain G1 candidates; LM Studio is separately installed/local; hosted, remote/LAN, labs/tools, and cloud sync are deferred. Bundled speech and the 2D renderer require dependency/package/device safety review while their concrete technologies remain G2/G3-gated.

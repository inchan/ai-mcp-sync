# Technical Requirements Document (TRD)

## 1. Executive Technical Summary
- **Project Overview:** MCP Sync Service automates detection and synchronization of Model Context Protocol (MCP) configurations across AI-oriented CLI tools. A desktop shell (Tauri/Electron) hosts a React/Svelte UI, while a Rust core performs file system orchestration and SQLite persistence.
- **Core Technology Stack:** React or Svelte (frontend), Tauri/Electron (shell), Rust (backend), SQLite (local data store).
- **Key Technical Objectives:**
  - **Performance:** Deliver fast detection and synchronization with minimal system overhead.
  - **Reliability:** Guarantee data integrity, conflict handling, and recoverability of MCP configurations.
  - **Observability:** Instrument telemetry to support BMAD metrics (drift, adoption, resolution time).
  - **Security:** Protect sensitive secrets during storage and sync operations.
- **Critical Technical Assumptions:**
  - Supported AI tools expose file-based configuration in predictable locations.
  - Users grant necessary filesystem and keychain permissions.
  - Local-first approach; cloud connectivity is optional for catalog updates and health checks.

## 2. BMAD Alignment for Engineering
The engineering roadmap inherits the **BMAD (Business Outcomes, Metrics, Activities, Deliverables)** framing from the PRD.

| Pillar | Technical Interpretation | Implementation Notes |
| ------ | ----------------------- | -------------------- |
| **Business Outcomes** | Eliminate configuration drift, accelerate onboarding, increase trust. | Build deterministic sync pipeline, provide explainable diffs, guarantee safe rollback. |
| **Metrics** | Drift incidents detected/resolved, time-to-sync, adoption of adapters, user-reported confidence. | Instrument structured logs, aggregate telemetry locally, expose optional anonymized metrics export. |
| **Activities** | Adapter development, sync engine hardening, UX feedback loops, observability enablement. | Create adapter SDK, implement file watchers, build conflict resolver, set up OpenTelemetry exporters. |
| **Deliverables** | Desktop tray app, CLI, history viewer, documentation/tooling. | Ship cross-platform binaries, CLI bootstrap, migration scripts, knowledge base with templates. |

BMAD checkpoints must be reviewed at the end of each milestone to ensure technical work moves leading indicators for the business outcomes.

## 3. System Architecture Overview
### 3.1 Component Diagram
```mermaid
flowchart TD
  UI[Frontend (React/Svelte)] -->|IPC| Shell[Tauri/Electron Shell]
  Shell -->|Command| Core[Sync Core (Rust)]
  Core -->|SQL| DB[(SQLite)]
  Core -->|FS| Tools{AI CLI Tool Configs}
  Core -->|Events| Tray[System Tray Service]
  Core -->|Telemetry| Logs[(Structured Logs/OTLP)]
```

### 3.2 Responsibilities
- **Frontend:** Renders dashboards, diff viewers, wizards, and provides CLI onboarding hints. Communicates with Rust core via IPC/HTTP bridge exposed by shell.
- **Shell:** Provides cross-platform window management, system tray integration, auto-update hooks, secure storage bridging (Keychain/DPAPI/libsecret).
- **Sync Core (Rust):** Implements detection, normalization, diffing, merging, encryption, and telemetry. Houses domain logic using modular crates (adapters, engine, storage, telemetry, cli).
- **SQLite:** Persists master configuration metadata, sync history, audit log, adapter registry state.
- **System Tray Service:** Presents current status, last sync result, quick actions. Optionally pushes notifications.

## 4. Detailed Component Design
### 4.1 Adapter Layer
- Each supported AI CLI tool implements `Adapter` trait: `scan()`, `normalize()`, `apply()`, `diff()`.
- Adapter metadata stored in `adapters` table: id, name, version, file_path_patterns, format, capabilities.
- Provide scaffolding CLI command `mcp-sync adapter create <tool>` to bootstrap new adapters.

### 4.2 Synchronization Engine
1. **Discovery:** Scan file system using adapter-provided glob patterns and OS registries.
2. **Normalization:** Parse tool-specific format into canonical `MasterConfig` schema (servers, rules, credentials references).
3. **Diff & Conflict Detection:** Compare canonical schema to desired master state, produce change set. If conflicting manual edits exist, mark conflict with severity.
4. **Apply & Verify:** Write updates back via adapter; perform post-write validation (checksum, schema validation).
5. **History Logging:** Persist before/after snapshots and metadata (timestamp, adapter, user) for rollback.
6. **Telemetry:** Emit events (`sync_started`, `sync_success`, `sync_conflict`, `sync_failed`) with contextual fields.

### 4.3 Configuration CRUD
- Frontend forms validated against JSON schema served by core.
- Secrets stored using OS keychain; fallback to SQLite encrypted blob if unavailable.
- Provide CLI parity for headless environments.

### 4.4 Conflict Resolution Workflow
- Present diff viewer with `ours`, `theirs`, `merged` states.
- Support merge strategies: accept master, accept local, selective merge (field-level), postpone.
- After resolution, sync engine re-validates and updates history.

## 5. Data Model & Storage Strategy
### 5.1 Schema Outline
- `master_configs` (id, version, created_at, updated_at, notes)
- `servers` (id, master_config_id, name, endpoint, auth_type, token_ref, rule_set_id)
- `rule_sets` (id, name, description, file_path, checksum)
- `sync_runs` (id, triggered_by, started_at, completed_at, status, summary, metrics)
- `sync_diffs` (id, sync_run_id, tool, diff_blob, severity)
- `audit_log` (id, actor, action, entity, entity_id, payload, created_at)
- `adapters` (id, name, version, file_patterns, capabilities, last_success_at)

### 5.2 Storage Policies
- Default retention 90 days; configurable via settings.
- Vacuum/compaction scheduled weekly.
- Sensitive fields encrypted using libsodium or OS keystore integration.

## 6. External Interfaces
### 6.1 CLI
- Commands: `mcp-sync detect`, `mcp-sync sync [--dry-run]`, `mcp-sync history`, `mcp-sync rollback <id>`, `mcp-sync adapter list`.
- Output JSON by default with `--human` option for formatted tables.

### 6.2 IPC/HTTP Bridge
- Shell exposes `/api/sync`, `/api/adapters`, `/api/history` endpoints secured with session tokens.
- Rate limiting: 10 requests per second per session to prevent UI loops.

### 6.3 Optional Telemetry Export
- Local OTLP endpoint; user opt-in to forward aggregated metrics to remote collector.
- Provide redaction hooks to avoid secrets in payloads.

## 7. Synchronization Workflows
1. **Scheduled Background Sync:** Cron-like scheduler triggers `scan -> diff -> apply`. Backoff strategy for repeated failures.
2. **User-Initiated Sync:** Manual invocation from UI/CLI triggers immediate run with more verbose logging.
3. **Onboarding Wizard:** Step-based flow populates initial master configuration from templates or imports existing files.
4. **Profile Import/Export:** Serialize master config (sans secrets) to signed archive; verify signature when importing.

## 8. Observability & Quality Gates
- Implement `tracing` crate with JSON formatter; integrate with OpenTelemetry exporter for metrics (`sync_duration_ms`, `drift_count`, `conflict_rate`, `rollback_events`).
- Add health check endpoint providing adapter status and last sync result.
- Set up automated testing tiers:
  - **Unit Tests:** Adapter parsing, diffing logic, encryption helpers.
  - **Integration Tests:** Full sync pipeline using temp directories per tool.
  - **End-to-End Tests:** UI-driven flows executed via Playwright.
- Quality bar: >90% coverage on critical engine modules, zero panics in release builds.

## 9. Security, Privacy & Compliance
- Secrets stored in OS keychain if possible; else encrypted with Argon2-derived key.
- Signed binary distribution; verify signature on auto-updates.
- Provide SBOM for dependencies; run cargo audit and npm audit in CI.
- Implement principle of least privilege: run file operations under user's context, avoid elevated privileges.
- Document data handling for GDPR/CCPA (local-only storage, explicit opt-in telemetry).

## 10. Deployment & DevOps
- **Build Pipeline:** GitHub Actions running lint (ESLint, Rust fmt/clippy), tests, packaging (Tauri bundler, cargo dist).
- **Release Workflow:** Tag release, build installers (.dmg, .msi, .AppImage), publish checksums.
- **Update Channel:** Optional background updater leveraging Tauri's updater API with release notes feed.
- **Configuration Management:** Provide `configs/` templates and environment-specific overrides for dev/staging.

## 11. Roadmap & Milestones (Technical)
| Phase | Duration | Technical Deliverables | BMAD Impact |
| ----- | -------- | --------------------- | ----------- |
| **Foundation** | 2 weeks | Project scaffolding, adapter trait, SQLite schema migration scripts, telemetry baseline. | Establish measurement layer; unblock Activities. |
| **Core Sync MVP** | 6 weeks | Implement detection engine, diff/apply pipeline, CLI interface, minimal UI. | Delivers core Business outcome of consistency; metrics for drift/time-to-sync. |
| **UX & Reliability Hardening** | 4 weeks | Conflict resolver UI, history viewer, rollback, automated tests, packaging. | Improves adoption metrics and reduces conflict resolution time. |
| **Ecosystem Expansion** | Ongoing | Additional adapters, template catalog, plugin SDK, telemetry dashboards. | Extends deliverables, keeps metrics trending positively. |

## 12. Risk Assessment & Mitigation
| Risk | Probability | Impact | Mitigation |
| ---- | ----------- | ------ | ---------- |
| Inconsistent configuration formats across tools | Medium | High | Maintain adapter test fixtures, introduce schema validation per adapter, collaborate with community for specs. |
| File permission issues on user machines | Medium | Medium | Provide troubleshooting guide, request permissions with OS dialogs, retry with exponential backoff. |
| Secret leakage in logs | Low | High | Centralized logging middleware that redacts tokens; automated tests to detect accidental logging. |
| Performance degradation with large configs | Low | Medium | Stream-based parsing, diff chunking, caching canonical representation. |
| Limited visibility into adoption metrics | Medium | Medium | Encourage optional telemetry opt-in, provide manual export of anonymized metrics aligned to BMAD. |

## 13. Open Questions
- Should we ship a headless daemon for server environments or stay desktop-focused?
- What minimum OS versions are officially supported, especially for keychain APIs?
- How do we govern community-contributed adapters (review process, signing, compatibility testing)?

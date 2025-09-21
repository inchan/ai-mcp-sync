# MCP Sync Service Overview

## 1. Purpose & Goals
- **Purpose**: Deliver a reliable way to keep Multiple Control Protocol (MCP) configurations aligned across every AI CLI, desktop, and IDE agent installed on a developer's machine.
- **Primary Goals**:
  - Eliminate manual editing of agent configuration files by centralising them in a master MCP profile.
  - Provide fast rescan, comparison, and synchronisation actions that work the same way for every supported agent.
  - Offer an approachable experience for both CLI-first users and those who prefer a graphical dashboard.

## 2. Target Users & Use Cases
- Developers and power users who manage several AI assistants (Claude, Cursor, Code, 등).
- Data scientists and engineers who experiment with different MCP-capable tools and need parity across environments.
- Teams that want a transparent, auditable record of configuration changes stored locally.

**Representative use cases**
1. Bring a newly installed AI CLI tool in line with the organisation's approved MCP server catalogue in one click.
2. Periodically rescan installed tools to detect drift, review the diff, and apply only the changes that matter.
3. Back up the master configuration before synchronising and restore a previous snapshot if something goes wrong.

## 3. Core Capabilities
### 3.1 Configuration Inventory
- Discover installed agents by scanning known installation paths and manifests under the `tool_configs` directory.
- Record agent metadata (kind, version, activation status, locale) inside SQLite for quick lookups.
- Surface agent-specific options such as available commands or sub-agents so they can be toggled centrally.

### 3.2 MCP Synchronisation Engine
- Maintain a **Master MCP configuration** that acts as the source of truth for all connected agents.
- Compare master data against each agent's configuration, highlighting additions, removals, and edits.
- Synchronise changes automatically or let users apply updates per agent.
- Persist a synchronisation history so that recent activity can be reviewed from the dashboard or CLI.

### 3.3 Rules & Catalogues
- Store reusable MCP rule definitions (YAML/JSON) that describe prompts, permissions, and behaviours.
- Recommend rules based on usage tags or popularity scores.
- Allow agents to be linked to rule entries so that recommended presets remain consistent.

### 3.4 Interfaces
- **CLI (`agentctl`)**: complete coverage of scan, list, diff, apply, and rule management flows.
- **REST API** (served by the backend): powers the React dashboard and any external automation.
- **React Dashboard**: manage agents, trigger sync actions, inspect history, and edit the master profile visually.

## 4. System Architecture
### 4.1 Technology Stack
| Layer      | Technology | Rationale |
| ---------- | ---------- | --------- |
| Frontend   | React 18 + Vite | Component-driven UI with fast dev experience and proxy support for the backend. |
| Backend    | Rust (Axum, Tokio) | High-performance async services for file scanning and sync operations. |
| CLI        | Rust binary (`agentctl`) | Shares backend logic and database access for parity with the UI. |
| Database   | SQLite (rusqlite) | Local, zero-dependency storage for configurations and audit logs. |

### 4.2 Component Responsibilities
- **Frontend**: Fetches agent data, renders configuration editors, and exposes sync controls.
- **Backend Service**: Scans file system locations, loads/stores configurations, applies sync actions, and exposes REST endpoints.
- **Sync Engine Modules**: Handle parsing, diffing, and writing configuration files per agent type.
- **Scheduler Utilities**: Optional background jobs (via Tokio) for periodic rescans or rule updates.

### 4.3 Data Model
```sql
CREATE TABLE agents (
  id TEXT PRIMARY KEY,
  name TEXT,
  kind TEXT CHECK(kind IN ('cli','desktop','ide')),
  version TEXT,
  installed INTEGER NOT NULL,
  active INTEGER NOT NULL,
  language TEXT,
  path TEXT,
  meta JSON
);

CREATE TABLE agent_features (
  agent_id TEXT,
  type TEXT,
  key TEXT,
  enabled INTEGER NOT NULL,
  meta JSON,
  PRIMARY KEY(agent_id, type, key)
);

CREATE TABLE mcp_rules (
  id TEXT PRIMARY KEY,
  title TEXT,
  tags TEXT,
  spec JSON,
  popular_score REAL DEFAULT 0,
  updated_at TEXT
);

CREATE TABLE agent_mcp (
  agent_id TEXT,
  rule_id TEXT,
  status TEXT CHECK(status IN ('applied','pending','removed')),
  meta JSON,
  PRIMARY KEY(agent_id, rule_id)
);
```

## 5. Interfaces & Workflows
### 5.1 REST API Surface
| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| GET    | `/api/tools` | Retrieve registered agents and their sync state. |
| POST   | `/api/tools/rescan` | Trigger a filesystem rescan to detect new or removed tools. |
| GET    | `/api/config/master` | Fetch the master MCP profile. |
| POST   | `/api/config/master` | Update the master MCP profile. |
| POST   | `/api/sync` | Run a sync for all agents or a specified agent. |
| GET    | `/api/sync/history` | Return recent synchronisation activity. |

### 5.2 CLI Highlights
```bash
# Rescan installed tools
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- scan

# Summarise agent status
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- list

# Inspect recommended MCP rules
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- rules list

# Apply a rule to a specific agent
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- apply --rule anthropic --agent claude

# Synchronise every agent with the master configuration
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- sync

# Toggle an agent-specific capability
cargo run --manifest-path backend/Cargo.toml --bin agentctl -- feature toggle --agent claude --key anthropic --on
```

## 6. User Experience Flow
1. **Discover & Install**: Users obtain MCP Sync Service from GitHub and install it locally.
2. **Initial Setup**: On first launch, a guided flow collects preferred MCP servers and optional rule catalogues.
3. **Auto-Detection**: The system scans for supported agents and records their current configuration state.
4. **Review & Configure**: Users inspect detected agents, adjust mappings to rules, and refine the master profile.
5. **Synchronise**: A sync run updates agent configurations and logs results for auditing.
6. **Ongoing Management**: Periodic rescans or scheduled syncs keep every agent aligned; history is reviewable at any time.

## 7. Repository Structure & Development Notes
```
/
├── backend/          # Rust backend service & CLI binaries
├── frontend/         # React + Vite frontend
├── database/         # SQLite database location (created at runtime)
├── tool_configs/     # Sample agent configuration fixtures
├── scripts/          # Build / release automation
├── docs/             # Project documentation (this overview)
└── README.md         # Quickstart and contributor guide
```

- Rust 1.75+ and Node.js 18+ are recommended for local development.
- Backend and CLI share the same SQLite database file to avoid divergent state.
- The frontend development server proxies `/api` to the backend, so run both for end-to-end flows.

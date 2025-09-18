use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};

use crate::config::{
    default_recommended_servers, MasterConfigResponse, McpSettings, RecommendedServer, SyncStatus,
    SyncSummary,
};
use crate::error::{BackendError, BackendResult};

const DB_FILE: &str = "../database/mcp_sync.db";

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn initialize() -> BackendResult<Self> {
        let path = Path::new(DB_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.setup()?;
        db.seed_recommended_servers()?;
        Ok(db)
    }

    fn setup(&self) -> BackendResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS master_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tools (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                config_path TEXT NOT NULL,
                last_detected_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tool_name TEXT NOT NULL,
                status TEXT NOT NULL,
                message TEXT NOT NULL,
                synced_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS recommended_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                endpoint TEXT NOT NULL,
                homepage TEXT,
                category TEXT,
                api_key_required INTEGER NOT NULL DEFAULT 0,
                default_enabled INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        Ok(())
    }

    fn seed_recommended_servers(&self) -> BackendResult<()> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM recommended_servers", [], |row| {
            row.get(0)
        })?;
        if count == 0 {
            for server in default_recommended_servers() {
                conn.execute(
                    "INSERT OR IGNORE INTO recommended_servers (
                        id, name, description, endpoint, homepage, category, api_key_required, default_enabled
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        server.id,
                        server.name,
                        server.description,
                        server.endpoint,
                        server.homepage,
                        server.category,
                        if server.api_key_required { 1 } else { 0 },
                        if server.default_enabled { 1 } else { 0 },
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn upsert_master_config(&self, settings: &McpSettings) -> BackendResult<()> {
        let conn = self.conn.lock();
        let now = Utc::now();
        let content = serde_json::to_string(settings)?;
        conn.execute(
            "INSERT INTO master_config (id, content, updated_at) VALUES (1, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
            params![content, now.to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_master_config(&self) -> BackendResult<MasterConfigResponse> {
        let conn = self.conn.lock();
        let row = conn
            .query_row(
                "SELECT content, updated_at FROM master_config WHERE id = 1",
                [],
                |row| {
                    let content: String = row.get(0)?;
                    let updated_at: String = row.get(1)?;
                    Ok((content, updated_at))
                },
            )
            .optional()?;

        if let Some((content, updated_at)) = row {
            let settings: McpSettings = serde_json::from_str(&content)?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|err| BackendError::Other(err.to_string()))?
                .with_timezone(&Utc);
            Ok(MasterConfigResponse {
                settings,
                updated_at,
            })
        } else {
            Err(BackendError::ConfigNotFound)
        }
    }

    pub fn ensure_master_config(&self) -> BackendResult<MasterConfigResponse> {
        match self.get_master_config() {
            Ok(cfg) => Ok(cfg),
            Err(BackendError::ConfigNotFound) => {
                let default = McpSettings {
                    servers: vec![Default::default()],
                    project_overrides: vec![],
                };
                self.upsert_master_config(&default)?;
                self.get_master_config()
            }
            Err(err) => Err(err),
        }
    }

    pub fn record_tool_detection(&self, name: &str, config_path: &Path) -> BackendResult<()> {
        let conn = self.conn.lock();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO tools (name, config_path, last_detected_at) VALUES (?, ?, ?) \
             ON CONFLICT(name) DO UPDATE SET config_path = excluded.config_path, last_detected_at = excluded.last_detected_at",
            params![name, config_path.to_string_lossy(), now.to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn list_tools(&self) -> BackendResult<Vec<(String, PathBuf)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT name, config_path FROM tools ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let path: String = row.get(1)?;
            Ok((name, PathBuf::from(path)))
        })?;
        let mut tools = Vec::new();
        for row in rows {
            tools.push(row?);
        }
        Ok(tools)
    }

    pub fn record_sync(&self, summary: &SyncSummary) -> BackendResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO sync_history (tool_name, status, message, synced_at) VALUES (?, ?, ?, ?)",
            params![
                summary.tool,
                format!("{:?}", summary.status).to_lowercase(),
                summary.message,
                summary.synced_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn recent_sync_history(&self, limit: usize) -> BackendResult<Vec<SyncSummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT tool_name, status, message, synced_at FROM sync_history ORDER BY synced_at DESC LIMIT ?",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let tool: String = row.get(0)?;
            let status: String = row.get(1)?;
            let message: String = row.get(2)?;
            let synced_at: String = row.get(3)?;
            Ok((tool, status, message, synced_at))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (tool, status, message, synced_at) = row?;
            let status = match status.as_str() {
                "updated" => SyncStatus::Updated,
                "skipped" => SyncStatus::Skipped,
                _ => SyncStatus::Failed,
            };
            let synced_at = DateTime::parse_from_rfc3339(&synced_at)
                .map_err(|err| BackendError::Other(err.to_string()))?
                .with_timezone(&Utc);
            items.push(SyncSummary {
                tool,
                status,
                message,
                synced_at,
            });
        }
        Ok(items)
    }

    pub fn list_recommended_servers(&self) -> BackendResult<Vec<RecommendedServer>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, endpoint, homepage, category, api_key_required, default_enabled \
             FROM recommended_servers ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(RecommendedServer {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                endpoint: row.get(3)?,
                homepage: row.get(4)?,
                category: row.get(5)?,
                api_key_required: {
                    let value: i64 = row.get(6)?;
                    value != 0
                },
                default_enabled: {
                    let value: i64 = row.get(7)?;
                    value != 0
                },
            })
        })?;
        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }
        Ok(servers)
    }

    pub fn get_recommended_server(
        &self,
        server_id: &str,
    ) -> BackendResult<Option<RecommendedServer>> {
        let conn = self.conn.lock();
        let row = conn
            .query_row(
                "SELECT id, name, description, endpoint, homepage, category, api_key_required, default_enabled \
                 FROM recommended_servers WHERE id = ?",
                params![server_id],
                |row| {
                    Ok(RecommendedServer {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        endpoint: row.get(3)?,
                        homepage: row.get(4)?,
                        category: row.get(5)?,
                        api_key_required: {
                            let value: i64 = row.get(6)?;
                            value != 0
                        },
                        default_enabled: {
                            let value: i64 = row.get(7)?;
                            value != 0
                        },
                    })
                },
            )
            .optional()?;
        Ok(row)
    }
}

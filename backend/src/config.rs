use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub api_key: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

impl Default for McpServer {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default MCP Server".to_string(),
            description: Some("Fallback local MCP server".to_string()),
            endpoint: "http://localhost:3001".to_string(),
            api_key: None,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct McpSettings {
    #[serde(default)]
    pub servers: Vec<McpServer>,
    #[serde(default)]
    pub project_overrides: Vec<ProjectOverride>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectOverride {
    pub project: String,
    pub server_id: String,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfiguration {
    pub name: String,
    pub version: Option<String>,
    pub config_path: String,
    pub settings: McpSettings,
}

impl ToolConfiguration {
    pub fn new(
        name: impl Into<String>,
        config_path: impl Into<String>,
        settings: McpSettings,
    ) -> Self {
        Self {
            name: name.into(),
            version: None,
            config_path: config_path.into(),
            settings,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSummary {
    pub tool: String,
    pub status: SyncStatus,
    pub message: String,
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Updated,
    Skipped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub tool: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterRequest {
    pub settings: McpSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterConfigResponse {
    pub settings: McpSettings,
    pub updated_at: DateTime<Utc>,
}

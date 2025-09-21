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

impl McpSettings {
    pub fn apply_recommended_server(&mut self, server: &RecommendedServer, enabled: bool) {
        if let Some(existing) = self.servers.iter_mut().find(|item| item.id == server.id) {
            let existing_api_key = existing.api_key.clone();
            *existing = server.to_mcp_server(enabled);
            existing.api_key = existing_api_key;
        } else {
            self.servers.push(server.to_mcp_server(enabled));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub homepage: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub api_key_required: bool,
    #[serde(default)]
    pub default_enabled: bool,
}

impl RecommendedServer {
    pub fn to_mcp_server(&self, enabled: bool) -> McpServer {
        McpServer {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            endpoint: self.endpoint.clone(),
            api_key: None,
            enabled,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecommendedRequest {
    pub server_id: String,
    #[serde(default)]
    pub enabled: Option<bool>,
}

pub fn default_recommended_servers() -> Vec<RecommendedServer> {
    vec![
        RecommendedServer {
            id: "default".to_string(),
            name: "Default MCP Server".to_string(),
            description: Some("로컬 개발용 기본 MCP 서버".to_string()),
            endpoint: "http://localhost:3001".to_string(),
            homepage: None,
            category: Some("로컬".to_string()),
            api_key_required: false,
            default_enabled: true,
        },
        RecommendedServer {
            id: "anthropic".to_string(),
            name: "Anthropic MCP".to_string(),
            description: Some("Claude 및 엔터프라이즈 워크플로우용 공식 MCP".to_string()),
            endpoint: "https://api.anthropic.com/mcp".to_string(),
            homepage: Some("https://docs.anthropic.com".to_string()),
            category: Some("클라우드".to_string()),
            api_key_required: true,
            default_enabled: false,
        },
        RecommendedServer {
            id: "openai".to_string(),
            name: "OpenAI MCP".to_string(),
            description: Some("GPT 기반 자동화 및 도구 연동에 적합".to_string()),
            endpoint: "https://api.openai.com/v1/mcp".to_string(),
            homepage: Some("https://platform.openai.com/docs".to_string()),
            category: Some("클라우드".to_string()),
            api_key_required: true,
            default_enabled: false,
        },
        RecommendedServer {
            id: "openrouter".to_string(),
            name: "OpenRouter Community MCP".to_string(),
            description: Some("여러 LLM 공급자를 하나로 통합한 커뮤니티 MCP".to_string()),
            endpoint: "https://api.openrouter.ai/mcp".to_string(),
            homepage: Some("https://openrouter.ai".to_string()),
            category: Some("커뮤니티".to_string()),
            api_key_required: true,
            default_enabled: false,
        },
    ]
}

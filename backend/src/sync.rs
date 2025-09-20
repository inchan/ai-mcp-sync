use std::collections::HashSet;
use std::fs;
use std::path::Path;

use chrono::Utc;
use walkdir::WalkDir;

use crate::config::{McpSettings, ProjectOverride, SyncStatus, SyncSummary, ToolConfiguration};
use crate::db::Database;
use crate::error::BackendResult;

const TOOL_CONFIG_DIR: &str = "../tool_configs";
const MCP_FILE_NAME: &str = "mcp.json";

pub fn discover_tools(db: &Database) -> BackendResult<Vec<ToolConfiguration>> {
    let mut tools = Vec::new();
    let root = Path::new(TOOL_CONFIG_DIR);
    if !root.exists() {
        return Ok(Vec::new());
    }
    for entry in WalkDir::new(root)
        .max_depth(2)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && entry.file_name() == MCP_FILE_NAME {
            let path = entry.into_path();
            let tool_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|os| os.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let settings = read_settings_from_file(&path)?;
            let config =
                ToolConfiguration::new(tool_name.clone(), path.to_string_lossy(), settings);
            db.record_tool_detection(&tool_name, &path)?;
            tools.push(config);
        }
    }
    Ok(tools)
}

pub fn read_settings_from_file(path: &Path) -> BackendResult<McpSettings> {
    let content = fs::read_to_string(path)?;
    let settings: McpSettings = serde_json::from_str(&content)?;
    Ok(settings)
}

pub fn write_settings_to_file(path: &Path, settings: &McpSettings) -> BackendResult<()> {
    let json = serde_json::to_string_pretty(settings)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

pub fn sync_tool(
    tool: &ToolConfiguration,
    master: &McpSettings,
    db: &Database,
) -> BackendResult<SyncSummary> {
    let current = &tool.settings;
    if current == master {
        let summary = SyncSummary {
            tool: tool.name.clone(),
            status: SyncStatus::Skipped,
            message: "Already up-to-date".to_string(),
            synced_at: Utc::now(),
        };
        db.record_sync(&summary)?;
        return Ok(summary);
    }

    let normalized = merge_settings(master, current);
    write_settings_to_file(Path::new(&tool.config_path), &normalized)?;

    let summary = SyncSummary {
        tool: tool.name.clone(),
        status: SyncStatus::Updated,
        message: "Configuration updated".to_string(),
        synced_at: Utc::now(),
    };
    db.record_sync(&summary)?;
    Ok(summary)
}

pub fn merge_settings(master: &McpSettings, tool: &McpSettings) -> McpSettings {
    if tool == master {
        return tool.clone();
    }

    let mut merged = master.clone();
    let mut known_servers: HashSet<_> = merged
        .servers
        .iter()
        .map(|server| server.id.clone())
        .collect();

    for server in &tool.servers {
        if known_servers.insert(server.id.clone()) {
            merged.servers.push(server.clone());
        }
    }

    merged.project_overrides =
        merge_project_overrides(&master.project_overrides, &tool.project_overrides);
    merged
}

fn merge_project_overrides(
    master_overrides: &[ProjectOverride],
    overrides: &[ProjectOverride],
) -> Vec<ProjectOverride> {
    let mut merged = master_overrides.to_vec();
    let mut seen: HashSet<_> = merged.iter().map(|item| item.project.clone()).collect();

    for override_entry in overrides {
        if seen.insert(override_entry.project.clone()) {
            merged.push(override_entry.clone());
        }
    }

    merged
}

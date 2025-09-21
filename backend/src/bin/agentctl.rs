use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use backend::config::{
    MasterConfigResponse, McpSettings, RecommendedServer, SyncStatus, SyncSummary,
    ToolConfiguration,
};
use backend::db::Database;
use backend::sync;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "agentctl", about = "AI MCP 동기화 CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 로컬에 설치된 도구를 다시 검색합니다.
    Scan,
    /// 데이터베이스에 기록된 도구와 마스터 구성 상태를 출력합니다.
    List,
    /// 마스터 MCP 구성을 확인하거나 갱신합니다.
    #[command(subcommand)]
    Master(MasterCommand),
    /// 추천 MCP 서버(룰) 목록을 조회합니다.
    #[command(subcommand)]
    Rules(RulesCommand),
    /// 추천 MCP 서버를 마스터 구성에 적용하고 지정한 에이전트에 동기화합니다.
    Apply(ApplyArgs),
    /// 마스터 구성으로 도구를 동기화합니다.
    Sync(SyncArgs),
    /// 최근 동기화 기록을 확인합니다.
    History(HistoryArgs),
    /// 에이전트별 기능(현재는 MCP 서버 활성화) 토글
    #[command(subcommand)]
    Feature(FeatureCommand),
}

#[derive(Subcommand)]
enum MasterCommand {
    /// 현재 마스터 MCP 구성을 출력합니다.
    Show,
    /// JSON 파일에서 마스터 MCP 구성을 갱신합니다.
    Set(MasterSetArgs),
}

#[derive(Args)]
struct MasterSetArgs {
    /// JSON 파일 경로 ("-" 입력 시 STDIN 사용)
    #[arg(value_name = "PATH")]
    path: PathBuf,
}

#[derive(Subcommand)]
enum RulesCommand {
    /// 추천 MCP 서버 목록을 표 형태로 출력합니다.
    List,
}

#[derive(Args)]
struct ApplyArgs {
    /// 룰/서버 ID
    #[arg(long, value_name = "RULE_ID")]
    rule: String,
    /// 적용할 에이전트(도구) 이름
    #[arg(long, value_name = "AGENT")]
    agent: String,
    /// 마스터 구성에 추가할 때 사용할 enabled 값
    #[arg(long)]
    enabled: Option<bool>,
}

#[derive(Args)]
struct SyncArgs {
    /// 특정 도구만 동기화하려면 지정합니다.
    #[arg(long, value_name = "AGENT")]
    agent: Option<String>,
}

#[derive(Args)]
struct HistoryArgs {
    /// 출력할 기록 개수 (기본: 10)
    #[arg(long, default_value_t = 10)]
    limit: usize,
}

#[derive(Subcommand)]
enum FeatureCommand {
    /// MCP 서버의 활성화 여부를 토글합니다.
    Toggle(FeatureToggleArgs),
}

#[derive(Clone, ValueEnum)]
enum FeatureType {
    /// MCP 서버 활성화 토글
    Server,
}

#[derive(Args)]
struct FeatureToggleArgs {
    /// 에이전트 이름
    #[arg(long, value_name = "AGENT")]
    agent: String,
    /// 기능 유형 (현재 server만 지원)
    #[arg(long, value_enum, default_value = "server")]
    feature_type: FeatureType,
    /// 서버 ID
    #[arg(long, value_name = "KEY")]
    key: String,
    /// 활성화
    #[arg(long, action = ArgAction::SetTrue)]
    on: bool,
    /// 비활성화
    #[arg(long, action = ArgAction::SetTrue)]
    off: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = Database::initialize()?;

    match cli.command {
        Command::Scan => handle_scan(&db),
        Command::List => handle_list(&db),
        Command::Master(cmd) => handle_master(&db, cmd),
        Command::Rules(cmd) => handle_rules(&db, cmd),
        Command::Apply(args) => handle_apply(&db, args),
        Command::Sync(args) => handle_sync(&db, args),
        Command::History(args) => handle_history(&db, args),
        Command::Feature(cmd) => handle_feature(&db, cmd),
    }
}

fn handle_scan(db: &Database) -> Result<()> {
    let discovered = sync::discover_tools(db)?;
    println!("{}개의 도구 구성을 검색했습니다.", discovered.len());
    for tool in discovered {
        println!("- {} ({})", tool.name, tool.config_path);
    }
    Ok(())
}

fn handle_list(db: &Database) -> Result<()> {
    let master = db.ensure_master_config()?;
    let tools = load_tool_configs(db)?;

    if tools.is_empty() {
        println!("등록된 도구가 없습니다. 먼저 'agentctl scan'을 실행하세요.");
        return Ok(());
    }

    println!("마스터 구성 최신화: {}", master.updated_at.to_rfc3339());
    println!("\n도구 목록:");
    for tool in tools {
        println!("\n■ {}", tool.name);
        println!("  경로: {}", tool.config_path);
        if tool.settings == master.settings {
            println!("  상태: ✅ 마스터와 동기화됨");
        } else {
            let diff = describe_diff(&master.settings, &tool.settings);
            println!("  상태: ⚠️ 동기화 필요");
            if let Some(diff) = diff {
                println!("    차이: {}", diff);
            }
        }
    }
    Ok(())
}

fn handle_master(db: &Database, cmd: MasterCommand) -> Result<()> {
    match cmd {
        MasterCommand::Show => {
            let config = db.ensure_master_config()?;
            let rendered = serde_json::to_string_pretty(&config.settings)?;
            println!("{}", rendered);
            Ok(())
        }
        MasterCommand::Set(args) => {
            let content = read_from_path_or_stdin(&args.path)?;
            let parsed: McpSettings = serde_json::from_str(&content)
                .context("JSON 형식의 MCP 설정을 읽는 데 실패했습니다")?;
            db.upsert_master_config(&parsed)?;
            let updated = db.ensure_master_config()?;
            println!(
                "마스터 구성을 갱신했습니다. ({} 서버)",
                updated.settings.servers.len()
            );
            Ok(())
        }
    }
}

fn handle_rules(db: &Database, cmd: RulesCommand) -> Result<()> {
    match cmd {
        RulesCommand::List => {
            let rules = db.list_recommended_servers()?;
            if rules.is_empty() {
                println!("추천 MCP 서버가 없습니다.");
            } else {
                println!("추천 MCP 서버 목록:");
                for rule in rules {
                    print_recommended(&rule);
                }
            }
            Ok(())
        }
    }
}

fn handle_apply(db: &Database, args: ApplyArgs) -> Result<()> {
    let server = db
        .get_recommended_server(&args.rule)?
        .ok_or_else(|| anyhow!("추천 서버 '{}'를 찾을 수 없습니다.", args.rule))?;

    let mut master = db.ensure_master_config()?.settings;
    let enabled = args.enabled.unwrap_or(server.default_enabled);
    master.apply_recommended_server(&server, enabled);
    db.upsert_master_config(&master)?;
    println!("'{}' 서버를 마스터 구성에 적용했습니다.", server.name);

    let refreshed = db.ensure_master_config()?;
    let summary = sync_tool_for_agent(db, &args.agent, &refreshed)?;
    print_sync_summary(&summary, true);
    Ok(())
}

fn handle_sync(db: &Database, args: SyncArgs) -> Result<()> {
    let SyncArgs { agent } = args;
    let master = db.ensure_master_config()?;
    let mut entries = db.list_tools()?;

    if let Some(ref target) = agent {
        entries.retain(|(name, _)| name == target);
    }

    if entries.is_empty() {
        if let Some(agent) = agent {
            println!("'{}' 이름의 도구가 없습니다.", agent);
        } else {
            println!("동기화할 도구가 없습니다. 먼저 'agentctl scan'을 실행하세요.");
        }
        return Ok(());
    }

    for (name, path) in entries {
        let config = build_tool_configuration(name, path)?;
        let summary = sync::sync_tool(&config, &master.settings, db)?;
        print_sync_summary(&summary, false);
    }

    Ok(())
}

fn handle_history(db: &Database, args: HistoryArgs) -> Result<()> {
    let history = db.recent_sync_history(args.limit)?;
    if history.is_empty() {
        println!("동기화 기록이 없습니다.");
    } else {
        println!("최근 동기화 기록:");
        for item in history {
            println!(
                "- [{}] {} :: {}",
                item.synced_at.to_rfc3339(),
                item.tool,
                format_status(&item)
            );
        }
    }
    Ok(())
}

fn handle_feature(db: &Database, cmd: FeatureCommand) -> Result<()> {
    match cmd {
        FeatureCommand::Toggle(args) => handle_feature_toggle(db, args),
    }
}

fn handle_feature_toggle(db: &Database, args: FeatureToggleArgs) -> Result<()> {
    if !matches!(args.feature_type, FeatureType::Server) {
        return Err(anyhow!("현재는 server 유형만 지원합니다."));
    }

    let desired = match (args.on, args.off) {
        (true, false) => true,
        (false, true) => false,
        (true, true) => {
            return Err(anyhow!("--on 과 --off 를 동시에 사용할 수 없습니다."));
        }
        (false, false) => {
            return Err(anyhow!("--on 또는 --off 중 하나를 지정하세요."));
        }
    };

    let mut master = db.ensure_master_config()?.settings;
    let server = master
        .servers
        .iter_mut()
        .find(|item| item.id == args.key)
        .ok_or_else(|| anyhow!("마스터 구성에서 '{}' 서버를 찾을 수 없습니다.", args.key))?;
    server.enabled = desired;
    db.upsert_master_config(&master)?;

    println!(
        "'{}' 서버를 {}했습니다.",
        args.key,
        if desired { "활성화" } else { "비활성화" }
    );

    let refreshed = db.ensure_master_config()?;
    let summary = sync_tool_for_agent(db, &args.agent, &refreshed)?;
    print_sync_summary(&summary, true);
    Ok(())
}

fn load_tool_configs(db: &Database) -> Result<Vec<ToolConfiguration>> {
    let mut tools = Vec::new();
    for (name, path) in db.list_tools()? {
        match build_tool_configuration(name.clone(), path) {
            Ok(tool) => tools.push(tool),
            Err(err) => {
                eprintln!("⚠️  {} 의 설정을 불러오는 데 실패했습니다: {}", name, err);
            }
        }
    }
    Ok(tools)
}

fn build_tool_configuration(name: String, path: PathBuf) -> Result<ToolConfiguration> {
    let settings = sync::read_settings_from_file(&path)?;
    Ok(ToolConfiguration::new(
        name,
        path.to_string_lossy(),
        settings,
    ))
}

fn read_from_path_or_stdin(path: &PathBuf) -> Result<String> {
    if path.as_os_str() == "-" {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("STDIN 입력을 읽는데 실패했습니다")?;
        Ok(buffer)
    } else {
        fs::read_to_string(path)
            .with_context(|| format!("{} 파일을 읽을 수 없습니다", path.display()))
    }
}

fn print_recommended(rule: &RecommendedServer) {
    println!("\n- {} ({})", rule.name, rule.id);
    if let Some(desc) = &rule.description {
        println!("  설명: {}", desc);
    }
    println!("  엔드포인트: {}", rule.endpoint);
    if let Some(category) = &rule.category {
        println!("  카테고리: {}", category);
    }
    if let Some(homepage) = &rule.homepage {
        println!("  문서: {}", homepage);
    }
    println!(
        "  기본 상태: {} / API 키: {}",
        if rule.default_enabled {
            "활성화"
        } else {
            "비활성화"
        },
        if rule.api_key_required {
            "필요"
        } else {
            "불필요"
        }
    );
}

fn sync_tool_for_agent(
    db: &Database,
    agent: &str,
    master: &MasterConfigResponse,
) -> Result<SyncSummary> {
    let (name, path) = db
        .list_tools()?
        .into_iter()
        .find(|(name, _)| name == agent)
        .ok_or_else(|| anyhow!("'{}' 이름의 도구를 찾을 수 없습니다.", agent))?;

    let config = build_tool_configuration(name, path)?;
    let summary = sync::sync_tool(&config, &master.settings, db)?;
    Ok(summary)
}

fn describe_diff(master: &McpSettings, tool: &McpSettings) -> Option<String> {
    if master == tool {
        return None;
    }
    let master_ids: Vec<_> = master.servers.iter().map(|s| s.id.clone()).collect();
    let tool_ids: Vec<_> = tool.servers.iter().map(|s| s.id.clone()).collect();

    let missing: Vec<_> = master_ids
        .iter()
        .filter(|id| !tool_ids.contains(id))
        .cloned()
        .collect();
    let extra: Vec<_> = tool_ids
        .iter()
        .filter(|id| !master_ids.contains(id))
        .cloned()
        .collect();

    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!("{} 추가 필요", missing.join(", ")));
    }
    if !extra.is_empty() {
        parts.push(format!("{} 제거 확인", extra.join(", ")));
    }

    if parts.is_empty() {
        Some("서버 구성 차이 존재".to_string())
    } else {
        Some(parts.join(" / "))
    }
}

fn print_sync_summary(summary: &SyncSummary, include_timestamp: bool) {
    if include_timestamp {
        println!(
            "[{}] {} :: {}",
            summary.synced_at.to_rfc3339(),
            summary.tool,
            summary.message
        );
    } else {
        println!("{} :: {}", summary.tool, summary.message);
    }
    println!("  상태: {}", status_label(&summary.status));
}

fn format_status(summary: &SyncSummary) -> String {
    format!("{} - {}", status_label(&summary.status), summary.message)
}

fn status_label(status: &SyncStatus) -> &'static str {
    match status {
        SyncStatus::Updated => "업데이트",
        SyncStatus::Skipped => "동일",
        SyncStatus::Failed => "실패",
    }
}

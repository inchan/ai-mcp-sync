# AI Agent 통합 관리 시스템 설계 문서

## 목차
1. [개요](#1-개요)  
2. [핵심 기능](#2-핵심-기능)  
   - [에이전트 탐색 & 등록](#21-에이전트-탐색--등록)  
   - [MCP 동기화](#22-mcp-동기화)  
   - [특화 기능 관리](#23-특화-기능-관리)  
   - [추천 & 검색](#24-추천--검색)  
   - [업데이트 대응](#25-업데이트-대응)  
3. [기술 스택](#3-기술-스택)  
4. [데이터 모델](#4-데이터-모델)  
5. [MCP 규칙 포맷 (YAML 예시)](#5-mcp-규칙-포맷-yaml-예시)  
6. [Tauri 커맨드 예시](#6-tauri-커맨드-예시)  
7. [CLI 예시](#7-cli-예시)  
8. [업데이트 & 변경 감지 전략](#8-업데이트--변경-감지-전략)  
9. [배포 전략](#9-배포-전략)  
10. [로드맵](#10-로드맵)  

---

## 1. 개요
로컬에 설치된 **AI Agent (CLI, IDE 플러그인, 데스크탑 앱)** 들을 하나의 시스템에서 관리하기 위한 MCP 기반 동기화 툴을 설계한다.  

- **목표**:  
  - 설치된 에이전트 자동 탐색  
  - MCP 규칙(rule) 관리 및 동기화  
  - 에이전트별 특화 기능(커맨드, 서브에이전트) 관리  
  - 업데이트 시 자동 변경 감지 및 동기화 유지  
  - 웹/데스크탑(Tauri) 크로스플랫폼 배포  

---

## 2. 핵심 기능

### 2.1 에이전트 탐색 & 등록
- 설치된 에이전트 자동 탐색  
- 확인 항목:
  - 설치 여부  
  - 버전  
  - 활성화 여부  
  - 언어/로케일 설정  
  - 특화 기능(예: Claude Code → commands, sub-agents)

### 2.2 MCP 동기화
- "내 MCP 목록" ↔ "에이전트 MCP 목록" 비교  
- 기능:
  - 추가 / 삭제 / 수정  
  - 일괄 동기화  
  - 충돌 해결  

### 2.3 특화 기능 관리
- 에이전트별 제공 기능 탐색 및 관리  
  - 명령어(commands)  
  - 서브에이전트(sub-agents)  
  - 기능별 활성화 여부  

### 2.4 추천 & 검색
- 로컬 사용 빈도 기반 인기 규칙 추천  
- 온라인 카탈로그 기반 추천 가능  
- 태그/키워드 기반 검색 기능  

### 2.5 업데이트 대응
- 에이전트 업데이트 시 MCP 변경(추가/삭제/수정) 감지  
- 변경 사항을 DB 및 GUI에 반영  
- 필요 시 사용자 알림 및 재동기화  

---

## 3. 기술 스택

- **프론트엔드**: React + TypeScript (웹/데스크탑 공용)  
- **데스크탑 프레임워크**: Tauri (Rust + WebView)  
- **로컬 백엔드**: Rust 모듈 (에이전트 스캐너, MCP 동기화 엔진)  
- **데이터 저장소**: SQLite + YAML (규칙 저장)  
- **CLI 툴**: `agentctl` (Rust 기반, GUI와 DB 공유)  

---

## 4. 데이터 모델

### agents
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
```

### agent_features
```sql
CREATE TABLE agent_features (
  agent_id TEXT,
  type TEXT,
  key TEXT,
  enabled INTEGER NOT NULL,
  meta JSON,
  PRIMARY KEY(agent_id, type, key)
);
```

### mcp_rules
```sql
CREATE TABLE mcp_rules (
  id TEXT PRIMARY KEY,
  title TEXT,
  tags TEXT,
  spec JSON,
  popular_score REAL DEFAULT 0,
  updated_at TEXT
);
```

### agent_mcp
```sql
CREATE TABLE agent_mcp (
  agent_id TEXT,
  rule_id TEXT,
  status TEXT CHECK(status IN ('applied','pending','removed')),
  meta JSON,
  PRIMARY KEY(agent_id, rule_id)
);
```

---

## 5. MCP 규칙 포맷 (YAML 예시)
```yaml
id: auto_format
title: Auto Formatter
tags: [lint, format, ide]
applies_to:
  - type: ide
    products: [vscode, jetbrains]
spec:
  actions:
    - hook: on_save
      run: ["prettier", "--write", "{file}"]
version: 2
```

---

## 6. Tauri 커맨드 예시
```rust
#[tauri::command]
pub async fn scan_agents() -> Result<Vec<Agent>, String> { ... }

#[tauri::command]
pub async fn list_mcp_rules(q: Option<String>) -> Result<Vec<serde_json::Value>, String> { ... }

#[tauri::command]
pub async fn apply_mcp_rules(payload: ApplyPayload) -> Result<(), String> { ... }

#[tauri::command]
pub async fn toggle_feature(agent_id:String, feature_type:String, key:String, enabled:bool) -> Result<(), String> { ... }
```

---

## 7. CLI 예시
```bash
agentctl scan
agentctl list
agentctl rules list --tag ide --q "format"
agentctl apply --agent claude_code --rule auto_format
agentctl feature toggle --agent claude_code --type command --key explain_code --on
```

---

## 8. 업데이트 & 변경 감지 전략
1. 에이전트 업데이트 주기적으로 스캔  
   - CLI → `--version` 체크  
   - IDE → `package.json`/manifest 비교  
   - 데스크탑 → 버전/설정 파일 비교  

2. MCP 규칙 변화 감지  
   - 추가: 새로운 커맨드/서브에이전트 발견  
   - 삭제: 이전 존재하던 기능 제거됨  
   - 수정: 기존 규칙 스펙 변경  

3. 변경 발생 시 처리  
   - 사용자 알림 (GUI/CLI)  
   - DB 업데이트  
   - 자동 재동기화 옵션 제공  

---

## 9. 배포 전략
- **웹**: 제한적 (규칙 관리, 내보내기)  
- **데스크탑**: 풀 기능 (macOS, Windows, Tauri)  
- **업데이트**: Tauri Auto Updater + 코드사인  

---

## 10. 로드맵
1. **MVP**: CLI 스캐너 + MCP 카탈로그 + 수동 동기화  
2. **Beta**: macOS/Windows 네이티브 스캐너, 롤백 기능, 추천 점수  
3. **1.0**: IDE 플러그인 지원 확대, 자동 동기화 스케줄러, 프리셋 공유  

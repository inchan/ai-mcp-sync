# MCP Sync Service

MCP Sync Service는 다양한 AI CLI 도구의 MCP 구성을 일관되게 유지하기 위한 오픈소스 동기화 도구입니다. Rust로 작성된 백엔드와 React 기반 프런트엔드, SQLite 로컬 데이터베이스를 사용하여 설정을 관리합니다.

## 프로젝트 구조

```
/
├── backend/          # Rust 기반 백엔드 (Axum)
├── frontend/         # React + Vite 프런트엔드
├── database/         # SQLite 데이터베이스 파일 위치
├── tool_configs/     # 샘플 AI CLI 도구 구성 파일
├── scripts/          # 빌드/배포 스크립트
├── prd.md            # Product Requirements Document
├── trd.md            # Technical Requirements Document
└── user-journey.md   # 사용자 여정 문서
```

## 주요 기능

- AI CLI 도구 구성 자동 탐지 (`tool_configs` 폴더 기준)
- 마스터 MCP 구성 관리 및 저장 (SQLite)
- 개별 도구 구성과의 비교 및 동기화
- 동기화 이력 저장 및 조회
- React 기반 대시보드에서 설정, 동기화, 이력 관리

## 빠른 시작

### 1. 백엔드 실행

```bash
cargo run --manifest-path backend/Cargo.toml
```

서버는 기본적으로 `http://localhost:8080`에서 실행됩니다.

### 2. 프런트엔드 실행

```bash
cd frontend
npm install
npm run dev
```

Vite 개발 서버는 `http://localhost:5173`에서 실행되며, `/api` 요청은 백엔드로 프록시됩니다.

## API 개요

| Method | Endpoint              | 설명 |
| ------ | --------------------- | ---- |
| GET    | `/api/tools`          | 등록된 도구 목록 조회 |
| POST   | `/api/tools/rescan`   | 파일 시스템에서 도구 구성 재검색 |
| GET    | `/api/config/master`  | 마스터 MCP 구성 조회 |
| POST   | `/api/config/master`  | 마스터 MCP 구성 업데이트 |
| POST   | `/api/sync`           | 전체 혹은 단일 도구 동기화 수행 |
| GET    | `/api/sync/history`   | 최근 동기화 이력 조회 |

## 개발 가이드

- 백엔드: Rust 1.75+, Axum, Tokio, rusqlite, serde
- 프런트엔드: React 18, Vite, Fetch API
- 데이터베이스: SQLite (자동 생성)

자세한 요구사항은 [`prd.md`](./prd.md), [`trd.md`](./trd.md), [`user-journey.md`](./user-journey.md) 문서를 참고하세요.

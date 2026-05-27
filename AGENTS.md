# AGENTS.md

## 项目概述

melonOS 是一个 local-first 的 agent-native application runtime，不是传统 OS 替代品。当前 Alpha 目标是把 Scenario Pack 作为应用契约，通过 melon Runtime、melon Studio、Tool / Knowledge / Governance / Eval / UI Panel 闭环验证，再进入 Product Bundle 和 Corgi 场景产品。

核心能力：

- Scenario Pack：用 `manifest.yaml`、workflow、tools、permissions、knowledge、ui、evals 描述一个可运行场景。
- melon Runtime：Axum HTTP daemon，负责 pack discovery、task lifecycle、trace、audit、eval、SQLite persistence。
- melon Studio：React + Vite + Monaco 的本地 pack editor 和 Run / Debug UI。
- Tool Layer：`ToolRegistry` + adapter 模型，当前 Demo Ops 使用 `MockToolAdapter`。
- Knowledge Layer：读取 `knowledge/sources.yaml`，导入本地 Markdown / txt，使用 SQLite FTS / fallback search 返回 source citation。
- Governance：policy、approval、audit 的最小闭环。
- UI Panel：从 `ui/layout.yaml` 和 trace / audit / approval / eval 数据派生结构化 panels。

**关键**：需求、设计和路线图的权威来源在用户 Obsidian 知识库 `Melon OS/` 下，尤其是 `02 melonOS MVP 需求与路线图.md`。仓库内已迁移的 `doc/` 规划文档不再维护。

## 开发命令

### 安装依赖

Rust workspace 依赖由 Cargo 管理：

```bash
cargo fetch
```

Studio 前端依赖：

```bash
cd apps/studio
npm install
```

### 启动开发环境

启动 melon Runtime，默认监听 `127.0.0.1:8080`：

```bash
cargo run -p melon-runtime
```

启动 melon Studio，默认监听 `http://localhost:3000`，并通过 Vite proxy 转发 `/api` 到 runtime：

```bash
cd apps/studio
npm run dev
```

指定 runtime 端口：

```bash
MELON_BIND=127.0.0.1:18090 cargo run -p melon-runtime
```

### 测试

运行 Rust 全量测试：

```bash
cargo test
```

运行单个 Rust crate 测试：

```bash
cargo test -p melon-runtime
cargo test -p melon-kb
cargo test -p melon-agent
```

运行 Studio 单测：

```bash
cd apps/studio
npm run test
```

构建 Studio：

```bash
cd apps/studio
npm run build
```

### 黑盒验收

健康检查：

```bash
curl -s http://127.0.0.1:8080/api/health
```

运行 Demo Ops：

```bash
curl -s -X POST http://127.0.0.1:8080/api/packs/demo.ops/run \
  -H 'content-type: application/json' \
  -d '{"user_goal":"Execute workflow"}'
```

查询 task：

```bash
curl -s http://127.0.0.1:8080/api/tasks/<task_id>
```

查询 trace：

```bash
curl -s http://127.0.0.1:8080/api/tasks/<task_id>/traces
```

查询 pending approvals：

```bash
curl -s http://127.0.0.1:8080/api/tasks/<task_id>/approvals
```

批准 approval：

```bash
curl -s -X POST http://127.0.0.1:8080/api/tasks/<task_id>/approvals/<approval_id>/action \
  -H 'content-type: application/json' \
  -d '{"action":"approve"}'
```

运行 eval：

```bash
curl -s -X POST http://127.0.0.1:8080/api/tasks/<task_id>/eval
```

M2 当前验收底线：

```bash
cargo test
cd apps/studio && npm run test
cd apps/studio && npm run build
```

## 项目结构

```text
melon-os/
├── apps/
│   └── studio/                 # React + Vite + Monaco Studio
├── crates/
│   ├── melon-runtime/          # Axum daemon, API routes, SQLite persistence
│   ├── melon-agent/            # Workflow executor, task execution logic
│   ├── melon-tools/            # ToolRegistry, adapter interface, mock adapter
│   ├── melon-kb/               # knowledge/sources.yaml ingestion and search
│   ├── melon-permission/       # governance primitives
│   ├── melon-scenario/         # Scenario Pack schema and validation
│   ├── melon-mcp/              # MCP integration surface
│   └── melon-ui-protocol/      # UI panel protocol types
├── scenarios/
│   ├── demo-ops/               # Alpha no-hardware validation pack
│   └── melon-home/             # Deferred Home Assistant beta scenario
├── docs/
│   └── decisions/              # ADR placeholder
├── Cargo.toml                  # Rust workspace
└── README.md / README.zh-CN.md # GitHub-facing project docs
```

Scenario Pack 标准结构：

```text
scenarios/<pack>/
├── manifest.yaml
├── role.md
├── workflows/*.yaml
├── tools/*.yaml
├── permissions/policy.yaml
├── knowledge/sources.yaml
├── knowledge/fixtures/*.md
├── ui/layout.yaml
└── evals/cases.yaml
```

## 代码规范

### Rust

- crate 命名使用 `melon-*`，模块名使用 snake_case。
- Runtime API 路由放在 `crates/melon-runtime/src/routes/`，每个资源独立文件。
- 数据库初始化集中在 `crates/melon-runtime/src/db.rs`。
- Scenario schema 放在 `crates/melon-scenario/src/`，不要在 Studio 前端重复定义权威 schema。
- Workflow executor 逻辑放在 `crates/melon-agent/src/executor.rs`，不要把 pack 专用逻辑写进 runtime route。
- Tool 调用必须走 `melon-tools::ToolRegistry`，不要在 executor 里新增硬编码 tool response。
- Knowledge 检索必须通过 `melon-kb`，trace 中必须保留 `source_id`、`path`、`title` 或等价 citation。

### TypeScript / React

- Studio 使用 React function components 和 hooks。
- API 类型集中在 `apps/studio/src/lib/api.ts`。
- Panel 派生和 routing 逻辑集中在 `apps/studio/src/lib/panels.ts`。
- Panel 展示组件集中在 `apps/studio/src/components/Panels.tsx`。
- `ui/layout.yaml` 必须通过统一 parser / router 进入 UI，不要在页面组件里写场景专用 layout 分支。
- Panel action 必须通过 `PanelMessage.actions` 和 `onPanelAction` 闭环到 runtime API，不能只显示按钮外观。

### Scenario Pack

- `demo-ops` 是当前 Alpha 验收场景，不能引入硬件依赖。
- `melon-home` 是后续设备场景 Beta，不要提前抢占 M2 / M3 主线。
- 新增 pack 必须能通过 pack validation。
- eval case 必须基于 trace / audit / output marker 验证真实行为，不要只为了通过测试写无意义 marker。

### 文档约定

- **关键**：需求与设计更新写入 Obsidian `Melon OS/`，不要恢复仓库 `doc/` 规划文档。
- README 只保留面向 GitHub 的项目说明、快速启动、架构图和命令。
- 如果需求与旧 Markdown 冲突，以 Obsidian `02 melonOS MVP 需求与路线图.md` 为准。

## 测试策略

### 必跑测试

任何 runtime、agent、tool、knowledge、scenario schema 变更后运行：

```bash
cargo test
```

任何 Studio UI、panel、API client、layout routing 变更后运行：

```bash
cd apps/studio
npm run test
npm run build
```

### M2 Demo Ops 验收

M2 相关改动必须保证：

- `demo.ops` task 能从 `created` / `running` 到 `awaiting_approval`，approval 后到 `completed`。
- trace 包含 service、storage、network、inspection summary、cleanup result、knowledge source reference。
- audit 记录 tool call 和 approval result。
- eval `ops-inspection-001`、`ops-approval-001`、`ops-knowledge-001` 全部通过。
- Panels tab 能展示 Inspection Report、Knowledge Sources、Trace Timeline；approval panel action 能触发 approve / reject。

### 覆盖要求

- 新增 Rust 行为必须有 unit test 或 integration test。
- 新增 Studio panel derivation / routing 行为必须在 `apps/studio/src/lib/*.test.ts` 覆盖。
- 修复 bug 时先补能复现问题的测试，再改实现。
- 黑盒验收结果要能用 curl 或 Studio UI 复现，不接受只靠代码阅读判断完成。

## 安全与边界

⚠️ 不要执行会破坏用户工作区的命令，例如：

```bash
git reset --hard
git checkout -- .
rm -rf *
```

⚠️ 不要把 API key、token、cookie、password、secret 写入 trace、audit、fixtures、README 或测试快照。

⚠️ 不要把 Home Assistant、真实设备控制、向量数据库、网页抓取、复杂 RAG、云同步提前放进当前 M2 主线。

⚠️ 不要把 Corgi 写成绕过 melonOS Runtime 的独立应用。Corgi 必须作为 Scenario Product 验证 Product Bundle、Tool / Skill / UI / History / Mock 能力。

## 当前推进顺序

以 Obsidian 需求知识库为准，当前路线是：

```text
M2-2 Tool Layer 收敛               done
M2-3 Knowledge Layer 最小实现       done
M2-4 UI Panel / Trace Inspector     next
M2-5 Governance hardening
M2-6 Agent Runtime Durability
M2-7 Context Assembly + Observability Config
M3-1 Product Bundle Contract
M3-2 Corgi Scenario Product Alpha
M3-3 Bundle Packaging Smoke Test
M4-1 melon Home Beta
```

**关键**：每次开始新任务前，先读取 Obsidian `Melon OS/02 melonOS MVP 需求与路线图.md`，确认当前任务卡、验收标准和不得做事项。

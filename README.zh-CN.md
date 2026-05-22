# melonOS

> 面向 Scenario Pack 的 agent-native application substrate，用于构建、运行、调试和审计 AI 原生应用。

[English](README.md) | **简体中文**

[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-19-61dafb?logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue?logo=typescript)](https://www.typescriptlang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-planned-24c8db?logo=tauri)](https://tauri.app/)
[![Status](https://img.shields.io/badge/status-alpha-yellow)](#项目状态)

melonOS 不是传统操作系统的替代品，而是一套 local-first 的 runtime 和 studio，用来运行 **agent-native applications**：以版本化文件声明角色、工作流、工具、权限、知识源、UI 面板和 eval 用例的 scenario pack。

```text
melonOS = melon Runtime + Scenario Pack + melon Studio + Governance + Knowledge + Eval
```

当前仓库聚焦 Alpha 闭环：加载 scenario pack，运行 agent workflow，对副作用动作发起审批，记录 trace/audit，渲染 debug panels，并对已完成任务执行 eval。

## 为什么是 melonOS？

很多 agent demo 第一次能跑，但很难持续运营：tool call 不透明、权限散落在代码里、UI 是定制页面、正确性靠人工判断。melonOS 把这些问题变成平台的一等能力。

- **Scenario pack 即应用**：pack 是一个可移植目录，包含 `manifest.yaml`、workflow YAML、permission policy、tool declarations、knowledge sources、UI layout 和 eval cases。
- **可审计 runtime**：每个 task 都会产生 trace events、approval records、audit logs 和 eval results。
- **受治理的动作**：有副作用的步骤可以在执行前暂停并等待用户审批。
- **带引用的知识层**：本地知识源会被加载，并通过 source reference 暴露出处。
- **Studio-first 调试体验**：melon Studio 提供 pack 编辑、validation、Run/Debug、Trace、Audit、Eval 和 layout-driven Panels。

## 项目状态

melonOS 目前处于 Alpha 阶段。当前已验证的场景是 `demo.ops`，这是一个不依赖硬件的运维 pack，用来先证明平台闭环，再进入真实设备控制。

| 模块 | 状态 | 说明 |
|---|---:|---|
| Runtime daemon | Alpha | Axum HTTP API、SQLite persistence、task lifecycle、trace、audit、eval |
| Scenario pack loading | Alpha | Pack discovery、file read/write、schema-backed validation |
| melon Studio | Alpha | React/Vite editor、validation、Run/Debug、approval、trace/audit/eval、panels |
| Tool layer | Alpha | `ToolRegistry` 与 mock adapter 路径已用于 Demo Ops |
| Knowledge layer | Alpha | Demo Ops knowledge report 已具备本地 sources 和 search 路径 |
| Governance | Alpha | Approval/audit 路径已存在；policy engine 仍需 hardening |
| melon Home / device control | Planned | 等 Alpha Demo Ops 加固后进入 |

详细需求和优先级见 [doc/requirements.md](doc/requirements.md)。

## 快速开始

### 前置条件

- Rust toolchain
- Node.js 20+ 和 npm
- macOS、Linux 或 Windows 本地 shell

### 1. 启动 runtime

```bash
cargo run -p melon-runtime
```

runtime 默认监听 `127.0.0.1:8080`。

常用环境变量：

```bash
MELON_BIND=127.0.0.1:18080 cargo run -p melon-runtime
MELON_DB_PATH=/tmp/melon.db cargo run -p melon-runtime
MELON_SCENARIOS_DIR=/path/to/scenarios cargo run -p melon-runtime
```

### 2. 启动 melon Studio

```bash
cd apps/studio
npm install
npm run dev
```

打开命令输出的 Vite URL。Studio dev server 会把 `/api/*` 代理到 runtime。

### 3. 验证 runtime

```bash
curl -s http://127.0.0.1:8080/api/health
curl -s http://127.0.0.1:8080/api/packs
```

## Demo Ops 流程

`scenarios/demo-ops` 是当前 Alpha validation pack，不需要硬件。

1. 在 melon Studio 中打开 `Demo Ops`。
2. 点击 `Run`。
3. 在 Run/Debug panel 查看 trace events。
4. 审批 pending cleanup action。
5. 等待 task 进入 `completed`。
6. 运行 Eval，并确认 Demo Ops cases 通过。

预期闭环如下：

```text
run demo.ops
  -> task awaiting_approval
  -> user approves cleanup
  -> task completed
  -> trace contains status, cleanup, knowledge, and report markers
  -> audit contains the approved side-effect path
  -> eval passes for inspection, approval, and knowledge cases
```

## 仓库结构

```text
melon-os/
├── apps/
│   └── studio/                 # melon Studio, React + TypeScript + Vite
├── crates/
│   ├── melon-runtime/          # HTTP daemon, routing, persistence integration
│   ├── melon-agent/            # Workflow executor and task execution logic
│   ├── melon-tools/            # Tool registry and adapters
│   ├── melon-kb/               # Local knowledge loading and search
│   ├── melon-permission/       # Governance primitives
│   ├── melon-scenario/         # Scenario pack schema and validation
│   ├── melon-mcp/              # MCP integration surface
│   └── melon-ui-protocol/      # Adaptive UI protocol types
├── scenarios/
│   ├── demo-ops/               # Alpha no-hardware validation pack
│   └── melon-home/             # Planned Home Assistant beta pack
├── doc/
│   ├── requirements.md         # MVP requirements, status, and priority map
│   └── phase0-code-review.md   # Phase 0 review notes
└── docs/
    └── decisions/              # Architecture decision records
```

## Scenario Pack 结构

scenario pack 是一个基于文件系统的应用契约：

```text
scenarios/demo-ops/
├── manifest.yaml               # Pack identity, runtime requirement, entrypoint
├── role.md                     # Agent role and operating constraints
├── workflows/default.yaml      # Workflow steps
├── tools/tools.yaml            # Tool declarations
├── permissions/policy.yaml     # Approval and risk policy
├── knowledge/sources.yaml      # Knowledge source declarations
├── knowledge/fixtures/*.md     # Local knowledge content
├── ui/layout.yaml              # Runtime-driven Studio panel layout
└── evals/cases.yaml            # Task-level eval cases
```

这个结构是刻意设计的：pack 应该能用普通开发工作流完成 review、迁移、测试和审计。

## 开发

### 常用命令

```bash
# Rust checks and tests
cargo check
cargo test

# Studio tests and production build
cd apps/studio
npm test
npm run build

# Runtime
cargo run -p melon-runtime
```

### API Surface

Alpha runtime 暴露本地 HTTP endpoints，供 Studio 和黑盒验证使用：

| Endpoint | 用途 |
|---|---|
| `GET /api/health` | Runtime health |
| `GET /api/packs` | List scenario packs |
| `POST /api/packs/{pack_id}/validate` | Validate a pack |
| `GET /api/packs/{pack_id}/files` | List pack files |
| `GET /api/packs/{pack_id}/files/{path}` | Read a pack file |
| `PUT /api/packs/{pack_id}/files/{path}` | Write a pack file |
| `POST /api/packs/{pack_id}/run` | Run a pack workflow |
| `GET /api/tasks/{task_id}` | Read task status |
| `GET /api/tasks/{task_id}/traces` | Read trace events |
| `GET /api/tasks/{task_id}/approvals` | List pending approvals |
| `POST /api/tasks/{task_id}/approvals/{approval_id}/action` | Approve or reject an action |
| `GET /api/tasks/{task_id}/audit` | Read audit logs |
| `POST /api/tasks/{task_id}/eval` | Run eval cases for a completed task |
| `GET /api/packs/{pack_id}/knowledge/sources` | List pack knowledge sources |
| `GET /api/packs/{pack_id}/knowledge/search?q=...` | Search pack knowledge |

## 路线图

### Milestone 1: Pack Loading and Studio Foundation

- Scenario pack schema and validation
- Pack list and editor
- Runtime-backed file read/write
- SQLite-backed task、trace 和基础 metadata persistence

### Milestone 2: Demo Ops Alpha

- Tool registry and mock adapter path
- Knowledge retrieval with source references
- Run/Debug trace inspector
- Approval and audit loop
- Layout-driven Studio panels
- `demo.ops` task-level eval closure

### Milestone 3: melon Home Beta

- Home Assistant adapter
- 低风险 device control path
- Device operation trace and audit
- 中高风险 home actions 的 approval gates

## 设计原则

- **Local first**：开发和 Alpha 验证应在本地机器完成，不依赖云基础设施。
- **Files are the contract**：scenario 行为应该能通过 pack files 被检查，而不是隐藏在 Studio-only state 里。
- **Trace everything important**：tool calls、approvals、knowledge references 和 task state changes 都必须可观察。
- **Govern side effects**：潜在破坏性动作必须显式、可 review、可审计。
- **Keep the MVP narrow**：先证明平台闭环，再扩展设备覆盖和可视化构建器。

## 文档

| 文档 | 说明 |
|---|---|
| [Requirements](doc/requirements.md) | MVP requirements、优先级和当前进度 |
| [Technical Plan](melonOS%20技术方案.md) | 架构、产品分层和技术方向 |
| [MVP Plan](melonOS%20MVP%20开发计划.md) | 里程碑、时间线和验收标准 |
| [melon Home Plan](melon%20Home%20全屋智能技术方案.md) | Home Assistant beta scenario design |
| [Agents OS Feasibility](Agents%20OS%20可行性与产品方案.md) | 产品定位和阶段策略 |

## 贡献

这个仓库仍在快速迭代。大改动开始前，先对齐 [doc/requirements.md](doc/requirements.md) 中的当前里程碑。

提交前建议执行：

```bash
cargo test
cd apps/studio && npm test && npm run build
```

推荐使用小提交和语义化 commit message，例如：

```text
feat: add task eval endpoint
fix: clear stale eval result after approval
docs: refresh runtime quick start
```

## License

当前尚未声明 license。

# melonOS 技术方案

创建日期：2026-05-18

## 1. 项目定义

melonOS 是一个面向 AI 原生应用的 agent runtime 与场景操作系统。

它的核心目标不是第一阶段替代传统桌面 OS，而是提供一套可运行、可组合、可审计、可部署的 agent-native application substrate。

一句话定义：

> melonOS = melon Runtime + Scenario Pack + melon Studio + melon Edge。

其中 melon Runtime 是核心运行时容器，内部包含 Agent、Tool/MCP、Skill、Knowledge、Adaptive UI、Governance 等 layers。

melonOS 要解决的问题：

- agent 如何获得场景上下文；
- agent 如何调用工具和外部系统；
- agent 如何使用知识库和长期记忆；
- agent 如何将任务过程以结构化 UI 展示；
- agent 如何在用户可控、可审计的边界内执行动作；
- 不同业务场景如何被打包、安装、复用和部署；
- 同一套 agent 应用如何运行在 Web、Desktop、Linux、边缘设备和嵌入式设备上。

## 2. 产品分层

melonOS 第一阶段应拆成三个产品层：

```text
melon Studio
用于创建、配置、测试、调试、发布 scenario pack。

melon Runtime
用于运行 agent app，负责任务执行、工具调用、知识检索、权限确认、审计和 UI 编排。

melon Edge
运行在 Linux / x86 工控机 / RK3588 / Jetson / Yocto 设备上的边缘 agent 节点。melon Home Node 只是第一批边缘部署验证形态之一。
```

三者关系：

```text
开发者 / 集成商
      ↓
melon Studio
      ↓ 发布 Scenario Pack
melon Runtime
      ↓ 部署
Web / Desktop / Linux / Edge Device / Private Server
```

## 3. 总体架构

```text
┌───────────────────────────────────────────────┐
│ melon Studio                                  │
│ 场景创建 / 工具配置 / 知识接入 / 权限策略 / 评测调试 │
└───────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────┐
│ Scenario Pack Registry                        │
│ 场景包元数据 / 版本 / 依赖 / 安装 / 更新 / 签名      │
└───────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────┐
│ melon Runtime                                 │
├───────────────────────────────────────────────┤
│ Runtime Kernel                                │
│ Task Lifecycle / Event Bus / State Store / Scheduler │
├───────────────────────────────────────────────┤
│ Agent Layer                                   │
│ Task Graph / Planner / Executor              │
├───────────────────────────────────────────────┤
│ Tool Layer                                    │
│ MCP Client / MCP Server / Local Tools / APIs  │
├───────────────────────────────────────────────┤
│ Skill Layer                                   │
│ Skills / Prompts / Procedures / Validators    │
├───────────────────────────────────────────────┤
│ Knowledge Layer                               │
│ Files / Web / DB / Memory / Search / Embedding│
├───────────────────────────────────────────────┤
│ Adaptive UI Layer                             │
│ Chat / Docs / Tables / Kanban / Graph / Panel │
├───────────────────────────────────────────────┤
│ Governance Layer                              │
│ Policy / Approval / Sandbox / Logs / Rollback │
└───────────────────────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────┐
│ Deployment Targets                            │
│ Desktop / Browser / Server / Linux / Edge / Yocto │
└───────────────────────────────────────────────┘
```

说明：

- `melon Runtime` 是整体运行时，不是某一个单独模块；
- `Runtime Kernel` 负责任务生命周期、事件总线、调度和状态存储；
- `Agent / Tool / Skill / Knowledge / Adaptive UI` 是 runtime 内部的能力层；
- `Governance Layer` 是最特殊的一层，它既是底层能力，也是横切能力，会贯穿 agent 规划、工具调用、知识读取、UI 操作和外部副作用动作。

## 4. 核心设计原则

### 4.1 本地优先

melonOS 应默认支持本地运行：

- 本地知识库；
- 本地任务日志；
- 本地权限策略；
- 本地工具调用；
- 本地模型可选；
- 云模型可选；
- 企业私有部署可选。

### 4.2 场景优先

melonOS 不是泛聊天框，而是通过 scenario pack 交付场景能力。

一个有效的场景包必须包含：

- 目标；
- 知识；
- 工具；
- UI；
- 工作流；
- 权限；
- 评测。

### 4.3 工具显式化

agent 所有外部动作都必须显式声明：

- 能读什么；
- 能写什么；
- 能调用什么；
- 能发到哪里；
- 是否需要用户确认；
- 是否需要沙箱隔离；
- 是否记录审计日志。

### 4.4 UI 结构化

melonOS 的核心差异不是让 agent 只输出文本，而是让 agent 根据任务生成或选择合适的工作界面。

例如：

- research 任务使用文档 + 引用 + 时间线；
- coding 任务使用 diff + 测试结果 + 文件树；
- ops 任务使用设备面板 + 告警列表 + 工单；
- planning 任务使用看板 + 依赖图；
- data 任务使用表格 + 图表 + SQL/查询记录。

### 4.5 可审计、可回滚

任何有副作用的行为都应该被记录：

- 读取了什么；
- 修改了什么；
- 调用了什么工具；
- 使用了什么模型；
- 输入输出是什么摘要；
- 谁批准了动作；
- 是否可以回滚。

## 5. Scenario Pack 规范

Scenario Pack 是 melonOS 的核心应用单元。

推荐目录结构：

```text
my-scenario/
├── manifest.yaml
├── README.md
├── role.md
├── workflows/
│   ├── default.yaml
│   └── diagnostics.yaml
├── tools/
│   ├── tools.yaml
│   └── mcp.yaml
├── knowledge/
│   ├── schema.yaml
│   └── sources.yaml
├── ui/
│   ├── layout.yaml
│   ├── views/
│   └── components/
├── permissions/
│   └── policy.yaml
├── evals/
│   ├── cases.yaml
│   └── fixtures/
└── hooks/
    ├── before_task.js
    └── after_task.js
```

### 5.1 manifest.yaml

```yaml
id: melon.research
name: Research Assistant
version: 0.1.0
description: Research workflow with citations, outlines, reports, and timeline views.
author: melonOS
runtime: ">=0.1.0"
entry: workflows/default.yaml
permissions:
  - read_files
  - write_workspace
  - network_search
dependencies:
  mcp:
    - filesystem
    - browser
  skills:
    - citation_builder
    - report_writer
```

### 5.2 role.md

定义 agent 的角色、目标、边界和输出规范。

示例：

```markdown
You are a research agent.
Your job is to collect evidence, compare viewpoints, preserve citations, and produce structured reports.
Do not fabricate sources.
Ask for confirmation before writing external files.
```

### 5.3 workflows/default.yaml

```yaml
name: default_research_workflow
steps:
  - id: clarify
    type: agent.ask_or_infer
  - id: collect
    type: tool.search
  - id: organize
    type: agent.synthesize
  - id: render
    type: ui.document
  - id: export
    type: tool.write_file
    approval: required
```

### 5.4 permissions/policy.yaml

```yaml
policies:
  read_files:
    default: ask
    scopes:
      - workspace
  write_files:
    default: ask
    scopes:
      - workspace
  network:
    default: ask
  shell:
    default: deny
  external_send:
    default: ask
audit:
  enabled: true
  retain_days: 90
```

## 6. Runtime Kernel 与 Agent Layer

melon Runtime 的执行核心由 Runtime Kernel 与 Agent Layer 共同组成。

Runtime Kernel 负责稳定、确定性的系统职责：

- 任务生命周期；
- 事件总线；
- 状态存储；
- 调度；
- 恢复点；
- trace 写入；
- approval 等待与恢复。

Agent Layer 负责 agentic 行为：

- 任务理解；
- 计划生成；
- 步骤执行；
- 上下文选择；
- 工具调用决策；
- 结果综合；
- 与 UI layer 协作呈现任务状态。

核心模块：

```text
Task Manager
负责创建、暂停、恢复、取消任务。

Planner
负责把用户目标拆解成步骤。

Executor
负责执行工具调用、模型调用、知识检索和 UI 更新。

State Store
保存任务状态、上下文、执行记录、恢复点。

Approval Manager
处理用户确认、人类介入和权限升级。

Trace Manager
记录每一步执行过程，供调试、审计和回放。
```

任务状态机：

```text
created
  ↓
planning
  ↓
awaiting_approval
  ↓
running
  ↓
paused / failed / completed / cancelled
```

任务数据结构：

```ts
type Task = {
  id: string;
  scenarioId: string;
  userGoal: string;
  status: TaskStatus;
  plan: TaskStep[];
  contextRefs: ContextRef[];
  approvals: ApprovalRequest[];
  traces: TraceEvent[];
  createdAt: string;
  updatedAt: string;
};
```

TraceEvent 示例：

```ts
type TraceEvent = {
  id: string;
  taskId: string;
  type: "model" | "tool" | "knowledge" | "ui" | "approval" | "system";
  summary: string;
  inputRef?: string;
  outputRef?: string;
  timestamp: string;
};
```

## 7. Tool / MCP Layer

melonOS 应把工具调用作为一等公民。

Tool / MCP Layer 是 melon Runtime 内部的工具能力层。它不和 melon Runtime 平级，而是为 Agent Layer 提供可控、可审计的外部能力调用。

工具类型：

- MCP Server；
- 本地 CLI；
- HTTP API；
- 数据库连接；
- 浏览器自动化；
- 文件系统；
- 设备接口；
- Python/Node/Rust 插件；
- 企业内部系统连接器。

Tool Layer 需要提供：

- 安装；
- 配置；
- 权限声明；
- 健康检查；
- 调用日志；
- 错误恢复；
- 版本管理；
- 沙箱运行；
- 按需启动和停止。

工具注册示例：

```yaml
id: local.filesystem
type: mcp
command: melon-mcp-filesystem
permissions:
  - read_files
  - write_files
healthcheck:
  command: ping
startup:
  mode: on_demand
```

## 8. Skill Layer

Skill Layer 是 melon Runtime 内部负责可复用任务方法的能力层。

它和 Tool Layer 的边界是：

- Tool Layer 负责「能调用什么外部能力」；
- Skill Layer 负责「如何完成某类任务」；
- Tool 更像能力接口，Skill 更像方法、流程、约束和验证器。

Skill 类型：

- prompt 模板；
- system instruction；
- procedure；
- checklist；
- validator；
- output schema；
- few-shot examples；
- domain playbook；
- task-specific eval。

Skill Layer 需要提供：

- skill 安装与版本管理；
- skill 与 scenario pack 绑定；
- skill 依赖声明；
- skill 输入输出 schema；
- skill 执行前后 hooks；
- skill 质量评测；
- skill 权限声明；
- skill trace 记录。

示例：

```yaml
id: citation_builder
type: procedure
description: Build citation-backed research notes from retrieved sources.
inputs:
  - retrieved_sources
outputs:
  - citation_table
  - evidence_notes
validators:
  - no_missing_source_url
  - no_uncited_claims
```

## 9. Knowledge Layer

Knowledge Layer 是 melon Runtime 内部负责上下文与知识管理的能力层。

它负责把文件、网页、数据库、会话、设备数据等变成 agent 可用上下文，并将检索结果、来源引用、长期记忆提供给 Agent Layer 和 Adaptive UI Layer。

第一阶段建议：

```text
Metadata Store: SQLite
Full-text Search: Tantivy / Meilisearch
Vector Index: 可选，先做插件化
File Parser: Markdown / PDF / HTML / DOCX / CSV
Source Tracking: 必须做
Memory Store: 项目记忆 + 用户偏好 + 场景记忆
```

知识对象模型：

```ts
type KnowledgeItem = {
  id: string;
  sourceId: string;
  uri: string;
  title: string;
  contentType: string;
  hash: string;
  metadata: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
};
```

Chunk 模型：

```ts
type KnowledgeChunk = {
  id: string;
  itemId: string;
  text: string;
  startOffset?: number;
  endOffset?: number;
  embeddingRef?: string;
  citations: CitationRef[];
};
```

设计要求：

- 所有回答尽量能追溯到 source；
- 知识库支持增量索引；
- 文件变更后自动更新；
- 支持 per-project knowledge；
- 支持 per-scenario knowledge；
- 支持全局 memory 与项目 memory 分离；
- 支持本地存储和私有部署。

## 10. Adaptive UI Layer

Adaptive UI Layer 是 melon Runtime 内部负责结构化呈现和交互动作的能力层。

它负责把 agent 的任务状态和结果呈现为结构化界面，并把用户操作、审批、编辑、选择等事件回传给 Runtime Kernel 和 Agent Layer。

第一阶段内置视图：

- Chat View；
- Document View；
- Table View；
- Kanban View；
- Task Graph View；
- Diff View；
- Timeline View；
- Device Panel View；
- Approval Panel；
- Trace Inspector。

UI 由 scenario pack 声明：

```yaml
layout:
  default: research_workspace
views:
  - id: chat
    type: chat
    region: left
  - id: report
    type: document
    region: main
  - id: sources
    type: table
    region: right
  - id: trace
    type: task_graph
    region: bottom
```

Adaptive UI Layer 原则：

- agent 不能随意生成不可控 UI；
- scenario pack 声明可用 UI 类型；
- agent 可以选择视图、更新数据、请求用户确认；
- UI 组件必须能展示来源、状态和风险；
- 所有有副作用动作通过 Approval Panel 完成。

## 11. Governance Layer

melonOS 的可信度取决于 Governance Layer。

Governance Layer 包含权限、审批、审计、沙箱、trace、回滚等能力。它不是单纯位于底部的模块，而是横切所有 runtime layers。

例如：

- Agent Layer 规划任务时，需要检查场景权限和用户策略；
- Tool Layer 调用 MCP、API、shell、设备接口时，需要检查授权和记录日志；
- Knowledge Layer 读取文件、数据库、记忆时，需要检查 scope；
- Adaptive UI Layer 展示按钮或执行用户动作时，需要通过 approval；
- Runtime Kernel 需要把所有关键行为写入 trace，并在失败时支持恢复。

权限维度：

- 文件读取；
- 文件写入；
- 外部网络；
- shell 命令；
- API 调用；
- 数据库读取；
- 数据库写入；
- 设备控制；
- 消息发送；
- 任务自动执行；
- 长期记忆写入。

权限结果：

```text
allow
ask
deny
allow_once
allow_session
allow_scope
```

审计日志需要记录：

- task id；
- user id；
- scenario id；
- tool id；
- action；
- input summary；
- output summary；
- approval status；
- timestamp；
- rollback hint。

有副作用操作的标准流程：

```text
Agent proposes action
      ↓
Governance Layer evaluates policy
      ↓
Approval Panel shows intent, scope, risk
      ↓
User approves / rejects / modifies
      ↓
Executor runs action
      ↓
Audit log records result
```

## 12. 部署形态

### 12.1 本地桌面版

适合开发者、个人知识工作者、小团队。

推荐技术：

- Tauri；
- React；
- Rust daemon；
- SQLite；
- local MCP runtime。

### 12.2 Web / Server 版

适合团队协作、企业部署。

推荐技术：

- Web frontend；
- API server；
- Postgres；
- object storage；
- worker queue；
- containerized tool runtime。

### 12.3 Linux Service Deployment

适合 Linux 工作站、私有服务器、x86 工控机。

推荐组件：

- systemd services；
- melon-agentd；
- melon-kbd；
- melon-mcpd；
- melon-permissiond；
- melon-ui kiosk。

### 12.4 Edge Node

适合工控、门店、机器人、智能屏、边缘 AI 盒子。

第一批可以用 `melon Home Node` 作为 runtime 的边缘部署验证形态：

- 运行 melon Runtime；
- 承载 melon Home Pack；
- 接入 Home Assistant、Matter、Yeelight、小米生态、MQTT 等设备层；
- 提供家庭知识库、房间/设备模型、场景自动化、权限审计和本地 UI；
- 可运行在 x86 mini PC、Raspberry Pi、RK3588 或后续自有参考硬件上。

推荐硬件顺序：

1. x86 mini PC / 工控机；
2. RK3588；
3. Jetson；
4. Raspberry Pi 5；
5. NXP i.MX。

### 12.5 Yocto Edition

适合产品化和量产阶段。

推荐拆分：

```text
meta-melon-core
meta-melon-mcp
meta-melon-kb
meta-melon-ui
meta-melon-device
meta-melon-ota
```

## 13. melon Home 场景概览

melon Home 是 melonOS 的第一批验证场景之一。

它不是 melonOS 的产品重心，也不是要把 melonOS 变成智能家居平台。它的作用是作为一个由 Studio 创建和维护的具象 scenario pack，用来验证 melon Runtime 的通用能力：工具接入、知识建模、动态 UI、权限审计和本地/边缘部署。

melon Home 的差异化不是“也能控制灯”。Home Assistant 更像规则引擎和控制面板，melon Home 要验证的是：

> 用户说出模糊意图，scenario pack 将其转成可解释的任务步骤；Runtime 执行每一步，并保留 trace、approval、audit 和 source/context。

核心原则：

- runtime-first：所有 melon Home 能力都必须沉淀为可复用的 melon Runtime 能力；
- 第一阶段把 Home Assistant 作为设备抽象层；
- 优先支持灯光、插座、传感器、房间视图和场景模式；
- 小米灯接入优先走 Home Assistant，其次 Yeelight LAN，长期支持 Matter；
- 短期不自研灯泡、插座、门锁、摄像头等设备；
- 中期做 `melon Home Node` 参考中枢；
- 门锁、摄像头、安防、强电、燃气等高风险动作必须确认和审计。

完整方案见：[[melon Home 全屋智能技术方案]]

MVP 开发计划见：[[melonOS MVP 开发计划]]

## 14. 推荐技术选型

### 14.1 MVP 技术栈

```text
Frontend: React + TypeScript
Desktop Shell: Tauri
Backend Daemon: Rust
Plugin Runtime: Node.js / Python subprocess
Metadata DB: SQLite
Search: SQLite FTS first, Tantivy later
Vector: optional after MVP
MCP: Standard MCP client/server
Model Provider: OpenAI-compatible API + local model adapter
Config: YAML + JSON Schema
Sandbox: systemd / process user / bubblewrap later
```

MVP 技术决策：

- Runtime 第一版直接使用 Rust，不再预留 Go/Node.js 替代路线；
- Studio 第一版直接使用 React + Tauri，开发期可用 Web dev server 提升调试效率，但交付形态仍以 Tauri 为准；
- Node/Python 只作为插件、adapter、脚本或 MCP server 子进程，不作为 Runtime Kernel 实现；
- 搜索第一版使用 SQLite FTS，Tantivy 后置；
- 向量索引不进 MVP。

原则：

> 交付速度靠收窄 MVP 范围解决，不靠切换 Runtime 技术栈解决。Rust + Tauri 是 melonOS 本地优先、权限审计、daemon、桌面和边缘部署路线的一部分。

### 14.2 为什么后端优先 Rust

Rust 适合 melonOS 的原因：

- 适合写 daemon；
- 性能和内存控制好；
- 适合长期运行；
- 适合权限和沙箱边界；
- 适合跨平台；
- 能和 Tauri 生态配合；
- 适合未来 Linux/嵌入式部署。

但插件层不应该只用 Rust。Node/Python 对 MCP、AI 工具、数据处理更友好。

### 14.3 不建议第一阶段使用的复杂组件

第一阶段尽量避免：

- Kubernetes；
- 复杂 service mesh；
- 多租户企业权限；
- 自研向量数据库；
- 完整 Linux 发行版；
- Yocto；
- 大模型本地推理作为默认依赖；
- 全动态 UI 生成器。

## 15. MVP 里程碑

### Milestone 1：Studio 创建 Pack，Runtime 加载 Pack

目标：先证明“场景可被创建、保存、校验、加载”。

交付：

- Scenario Pack schema；
- melon Studio 最小版；
- Pack List / Pack Editor；
- manifest / role / tools / permissions / knowledge / ui / eval 配置；
- validation panel；
- pack import/export；
- melon Runtime 最小 daemon；
- pack loader；
- SQLite 基础表。

验收：

- 用户能在 Studio 创建一个 scenario pack；
- schema 错误能被明确提示；
- pack 能保存到 `scenarios/`；
- Runtime 能发现并加载 Studio 创建的 pack。

### Milestone 2：Tool + Governance + Knowledge 跑通，Demo Ops Pack 无硬件验证

目标：用无硬件依赖的 Demo Ops Pack 跑通平台闭环。

交付：

- Runtime task manager；
- run/debug panel；
- tool registry；
- mock tool adapter；
- policy engine；
- approval panel；
- audit log；
- trace inspector；
- Knowledge Layer 最小实现；
- eval runner；
- Demo Ops Pack；
- 通用 UI panel 协议。

验收：

- Demo Ops Pack 能执行巡检、审批、知识报告 3 个 demo flow；
- mock tool call 能写入 trace；
- mock approval action 能触发 approval；
- knowledge retrieval 能显示 source；
- eval runner 能输出通过/失败。

### Milestone 3：melon Home Pack + 真实设备控制

目标：在 Alpha 闭环跑通后，用 melon Home 接入第一个真实 tool adapter。

交付：

- melon Home Pack；
- Home Assistant Adapter；
- home model；
- room/device/scene panels；
- light/switch/sensor 控制；
- scene execution；
- 真实设备 trace/audit。

验收：

- Studio 能配置 Home Assistant Adapter；
- Runtime 能读取 Home Assistant 设备列表；
- Runtime 能控制灯和插座；
- 中风险设备动作触发 approval；
- 每次真实设备调用都有 trace 和 audit。

MVP 后第一个迭代：

- Agent intent routing；
- MVP 集成试用；
- Yeelight LAN Adapter；
- Event Timeline；
- x86 mini PC / Raspberry Pi 部署；
- 设备异常诊断。

## 16. 第一版目录结构建议

```text
melon-os/
├── apps/
│   ├── studio/
│   └── desktop/
├── crates/
│   ├── melon-runtime/
│   ├── melon-agent/
│   ├── melon-tools/
│   ├── melon-mcp/
│   ├── melon-kb/
│   ├── melon-permission/
│   ├── melon-scenario/
│   └── melon-ui-protocol/
├── packages/
│   ├── ui/
│   ├── scenario-schema/
│   └── sdk-js/
├── scenarios/
│   ├── demo-ops/
│   ├── melon-home/
│   ├── research/
│   ├── coding/
│   └── edge-ops/
├── docs/
├── examples/
└── scripts/
```

## 17. 数据存储建议

本地 SQLite 表：

```text
tasks
task_steps
trace_events
approval_requests
tools
tool_calls
scenario_packs
knowledge_sources
knowledge_items
knowledge_chunks
memories
permissions
audit_logs
ui_sessions
eval_cases
eval_runs
homes
rooms
home_devices
home_scenes
home_events
```

关键原则：

- task 和 trace 是一等实体；
- tool call 必须可追踪；
- knowledge item 必须保留 source；
- permission decision 必须可审计；
- UI session 应能恢复；
- pack version 必须记录。
- eval case 和 eval run 必须可追踪；
- home/device/scene/event 需要作为全屋智能场景的一等实体。

## 18. Edge / Embedded 设计

melon Edge 第一阶段基于 Ubuntu/Debian minimal，不急着上 Yocto。

推荐服务：

```text
melon-agentd.service
melon-mcpd.service
melon-kbd.service
melon-permissiond.service
melon-ui.service
melon-update.service
```

x86 工控机推荐最小配置：

- 4C CPU；
- 8GB RAM 起步，16GB 更好；
- 128GB SSD 起步；
- 双网口可选；
- Ubuntu Server / Debian minimal。

RK3588 推荐：

- 8GB RAM 起步；
- eMMC/SSD；
- Debian/Ubuntu minimal；
- 不把 NPU 作为第一阶段核心依赖。

设备端能力抽象：

```text
Device Adapter
├── camera
├── serial
├── gpio
├── modbus
├── can
├── plc
├── sensor
├── actuator
├── matter
├── zigbee
├── thread
├── infrared
├── home_assistant
├── yeelight
├── xiaomi_miot
├── xiaomi_home
├── mqtt
└── homekit
```

设备控制必须经过权限系统。

## 19. 安全设计

第一阶段必须做：

- tool allowlist；
- shell 默认禁用；
- 文件写入需要确认；
- 外部网络需要确认；
- 所有 tool call 记录 trace；
- 有副作用动作进入 approval panel；
- scenario pack 需要 manifest validation；
- pack 安装前展示权限；
- 本地 secrets 加密存储；
- 日志避免保存完整敏感内容，优先保存摘要和引用。
- 家庭设备按风险分级；
- 门锁、摄像头、安防、强电、燃气等动作默认不自动执行；
- 家庭成员、房间、设备、场景变更需要审计。

第二阶段再做：

- pack 签名；
- tool sandbox；
- per-scenario user；
- Linux capabilities 限制；
- seccomp/bubblewrap；
- remote attestation；
- device fleet policy；
- OTA 签名和回滚。

## 20. 评测体系

每个 scenario pack 都应该带 eval。

MVP 必须包含一个最小 eval runner，否则 scenario pack 的“验证”会变成主观判断。

MVP eval runner 只做：

- eval case YAML；
- expected checks；
- pass/fail 输出；
- eval run 记录；
- Studio 中显示 eval 结果。

不做：

- 复杂自动评分；
- 大规模 benchmark；
- 多模型对比；
- 成本/延迟长期报表。

评测维度：

- 任务完成率；
- 工具调用正确率；
- 权限触发正确率；
- 引用准确率；
- 幻觉率；
- UI 渲染正确性；
- 失败恢复能力；
- 用户介入次数；
- 成本；
- 延迟。

eval case 示例：

```yaml
id: research-001
goal: "整理某个主题的可行性分析"
expected:
  must_include:
    - citations
    - risk analysis
    - phased roadmap
  must_not:
    - fabricate sources
    - write files without approval
```

melon Home eval case 示例：

```yaml
id: home-lighting-001
goal: "把客厅切换到观影模式"
expected:
  must_include:
    - dim_living_room_lights
    - set_warm_color_temperature
    - record_audit_log
  must_not:
    - unlock_door
    - turn_off_security_devices
    - modify_automation_without_approval
```

## 21. 开源与商业化

建议开源：

- melon runtime core；
- scenario pack spec；
- basic MCP manager；
- local knowledge runtime；
- basic UI components；
- SDK；
- example packs。

商业化：

- enterprise console；
- team collaboration；
- advanced audit；
- permission policy center；
- private deployment support；
- device fleet management；
- OTA；
- premium scenario packs；
- industry solutions。
- melon Home Node 参考硬件；
- melon Home 高级家庭自动化包；
- 本地语音/多模态家庭控制套件。

## 22. 风险与对策

### 风险 1：做成普通聊天框

对策：第一版必须有 structured UI、trace、approval、scenario pack。

### 风险 2：场景过宽

对策：MVP 先用 Demo Ops Pack 无硬件验证平台骨架，再用 melon Home 作为真实设备验证场景。研发重心必须放在 melon Studio + melon Runtime；全屋智能只承担 scenario pack、tool adapter、UI panel、permission/audit 的真实场景验证任务。

### 风险 3：知识库弱

对策：MVP 必须做文件导入、简单全文检索、per-scenario knowledge 和 source tracking，但不做复杂 RAG、embedding pipeline 或知识图谱。

### 风险 4：工具调用不可靠

对策：tool healthcheck、trace、重试、fallback、permission prompt 必须是基础设施。

### 风险 5：权限系统后补

对策：从第一版就把 permission/audit 作为核心模块。

### 风险 6：过早做 OS

对策：先 runtime，再 Linux service，再 edge image，最后 Yocto。

### 风险 7：智能家居设备生态碎片化

对策：先接 Home Assistant，再做 Yeelight/Matter/MQTT 等重点 adapter；不要一开始直接适配所有品牌私有协议。

### 风险 8：物理世界误操作

对策：设备动作按风险分级；低风险灯光可自动执行，中高风险动作必须确认，高风险动作默认拒绝或只允许 owner 审批。

### 风险 9：过早自研硬件

对策：第一阶段不做灯泡、插座、门锁、摄像头；先做 melon Home Node 参考中枢，复用现有智能家居设备生态。

### 风险 10：场景反客为主

对策：melon Home 只能作为第一个 scenario pack。核心里程碑、目录结构、SDK、schema、tool registry、governance、trace、UI protocol 都必须保持场景无关，避免把 runtime 写死成智能家居系统。

### 风险 11：评测体系缺席

对策：MVP 必须包含最小 eval runner，支持 YAML case、expected checks、pass/fail 输出和 eval run 记录。

## 23. 推荐推进顺序

```text
1. Milestone 1: Studio 创建 pack，Runtime 加载 pack
2. Milestone 2: Tool + Governance + Knowledge 跑通，Demo Ops Pack 无硬件验证
3. Milestone 3: melon Home Pack + Home Assistant Adapter，真实设备控制
4. MVP 后迭代: Agent intent routing、真实家庭试用、Yeelight、边缘部署
5. 横向验证: Research Pack 或 Coding Pack
6. 再考虑 RK3588 / Jetson / Yocto
```

## 24. 最终判断

melonOS 最合理的技术路线是：

> 先做 melon Studio + melon Runtime 的场景创建/运行/调试闭环，再用 Demo Ops Pack 无硬件验证平台骨架，然后用 melon Home 作为第一个真实 scenario pack 验证 runtime 的通用能力，最后把同一套 runtime 下沉到 Linux、边缘节点和嵌入式设备。

第一阶段的成功标准不是「像一个完整 OS」，而是：

- 一个场景能被快速装配；
- 一个 agent 能安全调用工具；
- 一个任务能被结构化展示；
- 一个知识库能可靠引用来源；
- 一个有副作用的动作能被确认和审计；
- 一个 scenario pack 能安装、运行、评测和复用。

如果这些成立，melonOS 就拥有了从 agent platform 演进为 agent operating environment 的基础。

全屋智能是一个很适合展示 melonOS OS 感的场景，但它只是场景，不是内核。第一版的边界必须克制：用它验证 runtime，不做完整硬件生态；先编排现有设备，不替代 Home Assistant；先控制低风险设备，不让 agent 无确认地操作高风险物理设备。

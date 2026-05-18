# melonOS

melonOS 是一个面向 AI 原生应用的 agent runtime 与场景操作系统。

> melonOS = melon Runtime + Scenario Pack + melon Studio + melon Edge

## 项目定位

melonOS 不是传统操作系统的替代品，而是一套可运行、可组合、可审计、可部署的 **agent-native application substrate**。它通过 scenario pack 机制，让不同业务场景快速装配成可用的 agent app。

## 文档索引

| 文档 | 说明 |
|---|---|
| [需求文档](doc/requirements.md) | MVP 需求拆分为 3 个里程碑、42 个 backlog 需求项，并标注 P0/P1 边界 |
| [技术方案](melonOS%20技术方案.md) | 总体架构、产品分层、技术选型、设计原则 |
| [MVP 开发计划](melonOS%20MVP%20开发计划.md) | 3 个里程碑、时间线、验收标准、风险对策 |
| [Home 全屋智能方案](melon%20Home%20全屋智能技术方案.md) | melon Home 作为第一个真实 scenario pack 的技术方案 |
| [Agents OS 可行性方案](Agents%20OS%20可行性与产品方案.md) | 产品定位、阶段规划、商业模式、差异化分析 |

## MVP 三个里程碑

| 里程碑 | 周期 | 目标 |
|---|---|---|
| **M1** | 第 1-2 周 | Studio 创建 Pack，Runtime 加载 Pack |
| **M2** | 第 3-5 周 | Tool + Governance + Knowledge 跑通，Demo Ops Pack 无硬件验证 |
| **M3** | 第 6-8 周 | melon Home Pack + Home Assistant Adapter + 单设备真实控制 |

详细需求见 [doc/requirements.md](doc/requirements.md)。

**MVP 边界**：

- Studio P0 先做通用 pack 文件树、YAML / Markdown 编辑、JSON Schema 校验、Run / Debug，不优先做完整可视化编排器。
- Scenario Pack P0 必须跑通 `manifest / role / tools / permissions / knowledge / ui / evals` 的最小闭环。
- Demo Ops Pack 是 Alpha 验证场景，不依赖硬件。
- melon Home Pack 是 Beta 验证场景，只验证 Home Assistant 下的一盏灯或等价低风险设备；完整全屋智能面板、房间管理、多设备联动放到 P1。
- Agent intent routing、持续家庭试用、Yeelight 直连、边缘镜像部署不进入 MVP。

## 技术栈

```text
Frontend:       React + TypeScript
Desktop Shell:  Tauri
Backend Daemon: Rust
Plugin Runtime: Node.js / Python subprocess
Metadata DB:    SQLite
Search:         SQLite FTS first, Tantivy later
MCP:            Standard MCP client/server
Model Provider: OpenAI-compatible API + local model adapter
Config:         YAML + JSON Schema
```

**原则**：交付速度靠收窄 MVP 范围解决，不靠切换 Runtime 技术栈解决。Rust + Tauri 是 melonOS 本地优先、权限审计、daemon、桌面和边缘部署路线的一部分。

## 目录结构

```text
melon-os/
├── apps/
│   ├── studio/              # melon Studio（React + TypeScript）
│   └── desktop/             # Tauri 桌面壳
├── crates/
│   ├── melon-runtime/       # Runtime Kernel
│   ├── melon-agent/         # Agent Layer
│   ├── melon-tools/         # Tool / MCP Layer
│   ├── melon-mcp/           # MCP client/server
│   ├── melon-kb/            # Knowledge Layer
│   ├── melon-permission/    # Governance Layer
│   ├── melon-scenario/      # Scenario Pack schema & loader
│   └── melon-ui-protocol/   # Adaptive UI Protocol
├── packages/
│   ├── ui/                  # 共享 UI 组件库
│   ├── scenario-schema/     # Scenario Pack JSON Schema
│   └── sdk-js/              # JavaScript SDK
├── scenarios/
│   ├── demo-ops/            # Demo Ops Pack（Alpha 验证）
│   └── melon-home/          # melon Home Pack（Beta 验证）
├── doc/
│   └── requirements.md      # 需求文档
└── docs/                    # 项目文档
```

## 开发命令

以下是目标命令，项目脚手架落地后以实际 workspace scripts 为准。

```bash
# 启动 Studio 开发服务器
npm run dev

# 启动 Runtime daemon
cargo run -p melon-runtime

# 启动 Tauri 桌面壳
npm run tauri dev

# 运行测试
cargo test
npm test

# 构建
cargo build --release
npm run build
```

## 开发规范

### Git

- 使用语义化的 commit message（如 `feat: add pack validation`、`fix: resolve tool registry crash`）
- 每个 commit 对应一个独立的变更，避免大杂烩
- 提交前确保代码能通过 lint 和编译检查
- 功能分支命名：`feat/xxx`、`fix/xxx`、`refactor/xxx`、`docs/xxx`

### Rust

- 遵循 `clippy` 和 `rustfmt` 规则
- 错误处理使用 `thiserror` / `anyhow`，不在库代码中使用 `.unwrap()`
- 公共 API 必须有文档注释 `///`
- crate 间依赖通过 workspace 管理
- 新增 crate 需要在根 `Cargo.toml` workspace members 中注册

### TypeScript / React

- 严格模式（`strict: true`）
- 组件优先使用函数式 + hooks
- 状态管理优先使用 React Context / Zustand，避免过早引入 Redux
- 组件文件使用 `.tsx` 扩展名
- 导出的组件、跨模块 API、非显而易见的业务逻辑需要注释；普通函数不强制写空泛注释

### Scenario Pack

- 所有 YAML 文件必须通过 JSON Schema 校验
- pack 安装前必须展示其声明的权限
- scenario-specific 模型不得污染 Runtime Kernel，必须放在 scenario state model 层

### 数据存储

- 所有 migration 通过版本化 SQL 文件管理
- task 和 trace 是一等实体，必须可查询
- tool call 必须可追踪，写入 trace_events
- knowledge item 必须保留 source
- permission decision 必须可审计
- 有副作用的操作必须写入 audit_logs

### 安全

- tool allowlist 默认开启
- shell 命令默认禁用
- 文件写入需要确认
- 网络访问按目标和风险评估：localhost / LAN 内已授权 adapter 可会话级放行，未知公网目标默认 ask
- 本地 secrets 加密存储
- token / api key 只能以 secret ref 形式被 scenario pack 引用，不能明文写入 pack YAML，也不能随 pack export 导出
- 日志优先保存摘要，避免完整敏感内容
- 设备动作按风险分级：低风险可自动执行，中风险必须审批，高风险默认拒绝

### 测试

- Rust crate 必须包含单元测试
- 公共 API 必须有集成测试
- 前端组件必须有 snapshot 测试或行为测试
- Scenario Pack 必须有 eval case（RQ026）

### 文档

- 重要设计决策记录在 `docs/decisions/` 目录（ADR 格式）
- API 变更同步更新文档
- 需求变更同步更新 `doc/requirements.md` 状态

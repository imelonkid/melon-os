# melonOS MVP 开发计划

创建日期：2026-05-18

关联文档：

- [[melonOS 技术方案]]
- [[melon Home 全屋智能技术方案]]

## 1. MVP 定位

melonOS MVP 的核心不是全屋智能，也不是完整 OS。

MVP 要验证的主闭环是：

```text
melon Studio 创建 Scenario Pack
      ↓
melon Runtime 加载 Scenario Pack
      ↓
Runtime 执行任务、调用工具、检索知识、更新 UI、触发审批、记录 trace/audit
      ↓
Studio 查看运行结果、eval 结果，并迭代 Scenario Pack
```

MVP 分两层验证：

```text
Alpha: Demo Ops Pack
无硬件依赖，验证 Studio + Runtime + Tool + Knowledge + UI + Governance + Eval 的平台骨架。

Beta: melon Home Pack
真实设备验证，证明同一套 Runtime 可以接入 Home Assistant 和物理设备。
```

melon Home 是第一个真实世界样板 scenario pack，不是第一个平台闭环验证。所有实现必须沉淀为通用能力，避免把 melon Runtime 写成智能家居专用系统。

## 2. MVP 成功标准

MVP 成功不是“能控制一盏灯”，而是证明 melonOS 的平台骨架成立。

必须证明：

- Studio 能创建、编辑、校验一个 scenario pack；
- Runtime 能加载并运行这个 scenario pack；
- Tool Layer 能注册并调用 mock adapter 和至少一个真实 adapter；
- Knowledge Layer 能导入文件、索引、检索，并保留 source tracking；
- Governance Layer 能对有副作用动作做 approval 和 audit；
- Adaptive UI Layer 能根据 pack 配置渲染结构化界面；
- Trace 能记录任务、知识检索、工具调用、审批、UI 更新全过程；
- Eval runner 能执行 pack 自带 eval case 并输出通过/失败；
- Demo Ops Pack 能在无硬件环境下跑通平台闭环；
- melon Home Pack 能在 Beta 阶段完成真实家庭设备控制。

## 3. 三个核心里程碑

## Milestone 1：Studio 创建 Pack，Runtime 加载 Pack

目标：先证明“场景可被创建、保存、校验、加载”。

交付：

- Scenario Pack schema；
- melon Studio 最小版；
- Pack List；
- Pack Editor；
- Manifest Editor；
- Role Editor；
- Tools Config；
- Permissions Config；
- Knowledge Sources Config；
- UI Layout Config；
- Eval Cases Config；
- Validation Panel；
- pack import/export；
- melon Runtime 最小 daemon；
- pack loader；
- SQLite 基础表。

最小数据表：

```text
scenario_packs
tasks
trace_events
approval_requests
tools
permissions
knowledge_sources
knowledge_items
knowledge_chunks
eval_cases
eval_runs
audit_logs
ui_sessions
```

验收：

- 用户能在 Studio 创建一个空 scenario pack；
- 用户能编辑 manifest、role、tools、permissions、knowledge、ui layout、eval cases；
- schema 错误能被明确提示；
- pack 能保存到 `scenarios/`；
- Runtime 能发现并加载 Studio 创建的 pack；
- Studio 能显示当前 pack 的 validation 状态。

## Milestone 2：Tool + Governance + Knowledge 跑通，Demo Ops Pack 无硬件验证

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

Knowledge Layer MVP：

- 文件导入；
- Markdown / txt 解析；
- metadata 存储；
- 简单全文检索；
- per-scenario knowledge source；
- source tracking 数据结构；
- knowledge retrieval trace。

Eval MVP：

- eval case YAML；
- eval runner；
- expected checks；
- pass/fail 输出；
- eval run 记录；
- Studio 中显示 eval 结果。

Demo Ops Pack 结构：

```text
scenarios/demo-ops/
├── manifest.yaml
├── role.md
├── workflows/
│   ├── inspection.yaml
│   ├── approval.yaml
│   └── report.yaml
├── tools/
│   └── mock_tools.yaml
├── knowledge/
│   ├── sources.yaml
│   └── fixtures/
├── ui/
│   └── layout.yaml
├── permissions/
│   └── policy.yaml
└── evals/
    └── cases.yaml
```

Demo Ops 必须包含 3 个可演示流程：

### Demo 1：巡检任务

```text
用户发起“执行今日系统巡检”
      ↓
Runtime 创建 task
      ↓
Agent/Workflow 生成检查项
      ↓
mock_check_service 检查服务状态
      ↓
mock_check_storage 检查磁盘状态
      ↓
mock_check_network 检查网络状态
      ↓
发现 storage warning
      ↓
UI 显示异常卡片和 trace
      ↓
生成巡检摘要
```

验证点：

- task lifecycle；
- mock tool call；
- trace；
- entity/status panel；
- knowledge retrieval；
- report panel。

### Demo 2：审批动作

```text
用户发起“清理临时文件”
      ↓
Runtime 判断这是有副作用动作
      ↓
Governance 触发 approval
      ↓
Approval Panel 展示动作、范围、风险
      ↓
用户批准
      ↓
mock_cleanup_temp 执行
      ↓
audit log 记录结果
```

验证点：

- permission policy；
- approval；
- tool execution after approval；
- audit log；
- rollback hint。

### Demo 3：基于知识的报告

```text
用户发起“根据巡检手册生成处理建议”
      ↓
Knowledge Layer 检索 docs/inspection_runbook.md
      ↓
Runtime 记录 source ref
      ↓
生成处理建议
      ↓
UI 展示来源引用
      ↓
eval runner 检查必须包含 source ref
```

验证点：

- 文件导入；
- per-scenario knowledge；
- source tracking；
- citation/source ref；
- eval runner。

验收：

- Demo Ops Pack 由 Studio 创建或导入；
- Runtime 能加载 Demo Ops Pack；
- 三个 demo flow 都能执行；
- mock tool call 能写入 trace；
- mock approval action 能触发 approval；
- knowledge retrieval 能显示 source；
- eval runner 能输出通过/失败；
- Demo model 不污染 Runtime Kernel，只作为 scenario state model 存在。

## Milestone 3：melon Home Pack + 真实设备控制

目标：在 Alpha 闭环跑通后，用 melon Home 接入第一个真实 tool adapter。

交付：

- melon Home Pack；
- Home Assistant Adapter；
- home model schema；
- room schema；
- device schema；
- scene schema；
- device risk level；
- home event model；
- Home Dashboard；
- Room View；
- Device Panel；
- Scene Panel；
- Approval Panel 复用；
- Trace/Audit 复用。

Home Assistant Adapter MVP：

- 配置 Home Assistant URL/token；
- 获取 entities；
- 映射 light/switch/sensor；
- 读取状态；
- 调用 service；
- 写入 tool call trace；
- 写入 audit log；
- 错误展示。

最小 API：

```text
GET /api/states
POST /api/services/light/turn_on
POST /api/services/light/turn_off
POST /api/services/switch/turn_on
POST /api/services/switch/turn_off
```

验收：

- melon Home Pack 由 Studio 创建或导入；
- Runtime 能加载 melon Home Pack；
- home model 不污染 Runtime Kernel，只作为 scenario state model 存在；
- Studio 能配置 Home Assistant Adapter；
- Runtime 能读取 Home Assistant 设备列表；
- Runtime 能控制一个灯开关；
- Runtime 能调亮度；
- Runtime 能控制一个插座；
- Runtime 能读取一个传感器状态；
- 中风险设备动作触发 approval；
- 每次真实设备调用都有 trace 和 audit。

## 4. MVP 范围

### P0 必须支持

- Scenario Pack schema；
- melon Studio 最小编辑器；
- pack validation；
- pack run/debug；
- melon Runtime 最小任务系统；
- task / trace / approval / audit；
- Tool Adapter 接口；
- Permission Policy 接口；
- Adaptive UI Panel 接口；
- Knowledge Layer 最小实现；
- Eval runner 最小实现；
- Demo Ops Pack；
- 本地 SQLite 存储。

### P1 支持

- melon Home Pack；
- Home Assistant Adapter；
- Home model；
- Home Dashboard；
- Device control；
- Approval Panel；
- Scene Panel。

### MVP 后第一个迭代

- Agent intent routing；
- MVP 集成试用；
- Yeelight LAN Adapter；
- Event Timeline；
- x86 mini PC / Raspberry Pi 部署；
- 设备异常诊断。

### P2 支持

- Matter Adapter；
- MQTT Adapter；
- 本地语音输入；
- Home Node 镜像；
- 多场景包模板；
- Research Pack 作为第二个横向验证包。

### 第一版不做

- 完整插件市场；
- 多租户企业后台；
- 复杂团队协作；
- 完整 Linux 发行版；
- Yocto；
- RK3588/Jetson 深度适配；
- 自研灯泡/插座/传感器；
- 门锁自动开门；
- 摄像头人脸识别；
- 燃气/热水器控制；
- 强电自动控制；
- 完全自学习的无人确认自动化。

## 5. 技术栈决策

MVP 直接使用 Rust + Tauri，不做 Go/Node.js Runtime fallback。

```text
Frontend: React + TypeScript
Desktop Shell: Tauri
Backend Daemon: Rust
Plugin Runtime: Node.js / Python subprocess
Metadata DB: SQLite
Search: SQLite FTS first, Tantivy later
Model Provider: OpenAI-compatible API + local model adapter
Config: YAML + JSON Schema
```

决策：

- Runtime Kernel 第一版就用 Rust；
- Studio 第一版就用 React + Tauri；
- 开发期可以用 Web dev server 调试 Studio，但交付形态仍以 Tauri 为准；
- Node/Python 只作为插件、adapter、脚本或 MCP server 子进程；
- 搜索第一版用 SQLite FTS，Tantivy 后置；
- 向量索引不进 MVP。

原则：

> 交付速度靠收窄 MVP 范围解决，不靠切换 Runtime 技术栈解决。Rust + Tauri 是 melonOS 本地优先、权限审计、daemon、桌面和边缘部署路线的一部分。

## 6. 推荐时间线

### 第 1-2 周：Milestone 1

- Studio skeleton；
- Runtime daemon；
- scenario schema；
- pack editor；
- validation panel；
- pack loader；
- SQLite 基础表。

### 第 3-5 周：Milestone 2

- task/trace/approval；
- tool registry；
- mock tool；
- Knowledge Layer 最小实现；
- eval runner；
- Demo Ops Pack；
- run/debug panel；
- audit log。

### 第 6-8 周：Milestone 3

- melon Home Pack；
- home model；
- Home Assistant Adapter；
- home dashboard；
- room/device/scene panels；
- light/switch/sensor 控制；
- 真实设备 trace/audit。

### 第 9-10 周：MVP 后第一个迭代

- intent routing；
- entity resolver；
- action planner；
- x86/Raspberry Pi 部署；
- 真实家庭试用。

## 7. 主要风险

### 7.1 Studio 做得太重

对策：Studio MVP 只做 pack 创建、schema 编辑、validation、run/debug，不做完整低代码平台。

### 7.2 Runtime 被 melon Home 绑死

对策：所有 Home 相关模型都放在 scenario state model 或 adapter 层，Runtime Kernel、Tool Layer、Governance、UI Panel 协议保持场景无关。

### 7.3 Knowledge Layer 被做成完整 RAG 大工程

对策：MVP 只做文件导入、简单全文检索、source tracking，不做复杂 embedding pipeline 和知识图谱。

### 7.4 过早进入真实设备复杂度

对策：Alpha 先用 Demo Ops Pack 和 mock adapter 跑通平台闭环，Beta 再接 Home Assistant 和真实设备。

### 7.5 Home Assistant 依赖过重

对策：第一版接受 HA 作为设备抽象层，但 adapter interface 要独立，后续可接 Yeelight/Matter/MQTT。

### 7.6 评测体系过重

对策：MVP eval runner 只做 YAML case、expected checks、pass/fail 输出，不做复杂自动评分。

### 7.7 MVP 范围膨胀

对策：MVP 只做 3 个里程碑；Agent intent routing、集成试用、Yeelight、边缘部署全部放到 MVP 后第一个迭代。

## 8. 第一版完成定义

MVP 完成时，melonOS 应该能做到：

- 在 Studio 中创建和编辑 scenario pack；
- 校验 scenario pack；
- 运行 scenario pack；
- 在 Runtime 中创建 task；
- 调用 mock tool adapter；
- 调用 Home Assistant Adapter；
- 展示 trace；
- 触发 approval；
- 记录 audit；
- 导入文件并检索 per-scenario knowledge；
- 展示 source tracking；
- 运行 eval case 并输出 pass/fail；
- 通过 UI panels 展示结构化状态；
- 用 Demo Ops Pack 无硬件跑通 Alpha 闭环；
- 用 melon Home Pack 连接 Home Assistant；
- 发现并映射家庭设备；
- 控制灯光和插座；
- 读取传感器状态。

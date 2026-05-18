# melonOS MVP 需求文档

> 基于 melonOS MVP 开发计划（2026-05-18）
> 拆分为 3 个里程碑，共 42 个 backlog 需求项；P0 是 MVP 必须交付，P1 是 MVP 后第一轮增强

## 需求总览

### MVP 裁剪原则

- P0 优先验证 `Studio 创建 pack -> Runtime 加载 pack -> Tool / Governance / Knowledge / Eval 闭环 -> 一个真实设备动作`。
- Studio P0 不做完整低代码可视化编排器，只做文件树、YAML / Markdown 编辑、JSON Schema 校验、Run / Debug。专用可视化编辑器归 P1。
- Knowledge P0 只支持本地 Markdown / txt 文件导入、SQLite FTS、source tracking、retrieval trace。目录监听、网页抓取、向量检索、embedding 不进 MVP。
- Demo Ops Pack 是 Alpha 验证场景，必须有明确 demo script 和 eval case，但不依赖硬件。
- melon Home Pack 是 Beta 验证场景，P0 只要求通过 Home Assistant 控制一盏灯或等价低风险设备。完整房间视图、多设备面板、插座、传感器、场景面板归 P1。
- Agent intent routing、家庭长期试用、Yeelight 直连、边缘镜像部署不进入 MVP。

### Milestone 1：Studio 创建 Pack，Runtime 加载 Pack（第 1-2 周）

| 编号 | 名称 | 优先级 | 状态 |
|---|---|---:|---|
| [RQ001](#rq001) | Scenario Pack Schema 定义 | P0 | pending |
| [RQ002](#rq002) | melon Studio 项目骨架 | P0 | pending |
| [RQ003](#rq003) | Pack List 页面 | P0 | pending |
| [RQ004](#rq004) | Pack Editor 框架 | P0 | pending |
| [RQ005](#rq005) | Manifest Editor | P0 | pending |
| [RQ006](#rq006) | Role Editor | P1 | pending |
| [RQ007](#rq007) | Tools Config Editor | P1 | pending |
| [RQ008](#rq008) | Permissions Config Editor | P1 | pending |
| [RQ009](#rq009) | Knowledge Sources Config Editor | P1 | pending |
| [RQ010](#rq010) | UI Layout Config Editor | P1 | pending |
| [RQ011](#rq011) | Eval Cases Config Editor | P1 | pending |
| [RQ012](#rq012) | Pack Validation Panel | P0 | pending |
| [RQ013](#rq013) | Pack Import / Export | P1 | pending |
| [RQ014](#rq014) | melon Runtime 最小 Daemon | P0 | pending |
| [RQ015](#rq015) | Pack Loader | P0 | pending |
| [RQ016](#rq016) | SQLite 基础表 | P0 | pending |

### Milestone 2：Tool + Governance + Knowledge + Demo Ops Pack（第 3-5 周）

| 编号 | 名称 | 优先级 | 状态 |
|---|---|---:|---|
| [RQ017](#rq017) | Task Manager | P0 | pending |
| [RQ018](#rq018) | Run / Debug Panel | P0 | pending |
| [RQ019](#rq019) | Tool Registry | P0 | pending |
| [RQ020](#rq020) | Mock Tool Adapter | P0 | pending |
| [RQ021](#rq021) | Policy Engine | P0 | pending |
| [RQ022](#rq022) | Approval Panel | P0 | pending |
| [RQ023](#rq023) | Audit Log | P0 | pending |
| [RQ024](#rq024) | Trace Inspector | P0 | pending |
| [RQ025](#rq025) | Knowledge Layer 最小实现 | P0 | pending |
| [RQ026](#rq026) | Eval Runner | P0 | pending |
| [RQ027](#rq027) | Demo Ops Pack — 巡检流程 | P0 | pending |
| [RQ028](#rq028) | Demo Ops Pack — 审批流程 | P0 | pending |
| [RQ029](#rq029) | Demo Ops Pack — 知识报告流程 | P0 | pending |
| [RQ030](#rq030) | 通用 UI Panel 协议 | P0 | pending |

### Milestone 3：melon Home Pack + 真实设备控制（第 6-8 周）

| 编号 | 名称 | 优先级 | 状态 |
|---|---|---:|---|
| [RQ031](#rq031) | melon Home Pack 结构 | P0 | pending |
| [RQ032](#rq032) | Home Assistant Adapter | P0 | pending |
| [RQ033](#rq033) | Home 模型 | P0 | pending |
| [RQ034](#rq034) | Home Dashboard | P1 | pending |
| [RQ035](#rq035) | Room View | P1 | pending |
| [RQ036](#rq036) | Device Panel | P1 | pending |
| [RQ037](#rq037) | Scene Panel | P1 | pending |
| [RQ038](#rq038) | 灯光控制 | P0 | pending |
| [RQ039](#rq039) | 插座控制 | P1 | pending |
| [RQ040](#rq040) | 传感器状态读取 | P1 | pending |
| [RQ041](#rq041) | 中高风险动作审批 | P0 | pending |
| [RQ042](#rq042) | 设备操作 Trace / Audit | P0 | pending |

---

## 需求详情

### RQ001

**名称**：Scenario Pack Schema 定义
**优先级**：P0
**状态**：pending

**功能点**：
- 定义 `manifest.yaml` 的 JSON Schema（id、name、version、description、author、runtime、entry、permissions、dependencies）
- 定义 `role.md` 的格式约束
- 定义 `workflows/*.yaml` 的 step 结构（id、type、approval 等字段）
- 定义 `tools/*.yaml` 的工具注册结构
- 定义 `knowledge/sources.yaml` 的知识源结构
- 定义 `ui/layout.yaml` 的布局结构
- 定义 `permissions/policy.yaml` 的权限策略结构
- 定义 `evals/cases.yaml` 的评测用例结构
- 实现 schema validation 函数，输入 pack 目录输出验证结果

**验收方式**：
- 提供一个合法的 scenario pack 目录，validation 通过
- 删除 manifest 的必填字段，validation 报错并指出缺失字段
- 修改 workflow step type 为非法值，validation 报错

---

### RQ002

**名称**：melon Studio 项目骨架
**优先级**：P0
**状态**：pending

**功能点**：
- 创建 React + TypeScript 前端项目（apps/studio）
- 配置 Vite 或 Next.js 开发环境
- 配置 Tauri 项目骨架（开发期可用 Web dev server 调试）
- 配置路由结构
- 配置 UI 组件库（如 shadcn/ui 或 Radix）
- 建立与 Runtime 的 API 通信方式（HTTP / WebSocket）

**验收方式**：
- `npm run dev` 能启动 Studio 前端
- Web dev server 能访问 Studio 页面
- Tauri `npm run tauri dev` 能启动桌面壳

---

### RQ003

**名称**：Pack List 页面
**优先级**：P0
**状态**：pending

**功能点**：
- 列出本地 `scenarios/` 目录下的所有 scenario pack
- 展示每个 pack 的基本信息（name、version、description、status）
- 提供「新建 pack」入口
- 提供「打开本地 pack 目录」入口
- 提供「导入 pack」入口占位（P1，可先置灰）
- 点击 pack 进入 Editor 页面

**验收方式**：
- 打开 Pack List 能看到已有 pack
- 点击 pack 名称能进入 Editor 页面
- 空目录时显示引导文案

---

### RQ004

**名称**：Pack Editor 框架
**优先级**：P0
**状态**：pending

**功能点**：
- 提供 pack 文件树 + 编辑器布局
- 支持 YAML / Markdown 文件编辑
- 根据文件路径关联 JSON Schema，提供 schema-aware 校验提示
- 提供常用文件快捷 tab（Manifest、Role、Tools、Permissions、Knowledge、UI、Evals）
- 左侧显示 pack 目录树
- 右侧显示当前编辑内容
- 顶部显示保存 / 校验按钮
- 显示当前 pack 的 validation 状态指示器
- 专用可视化编辑器不属于 P0，由 RQ006-RQ011 覆盖

**验收方式**：
- 进入 Editor 能看到 pack 文件树
- 点击 YAML / Markdown 文件能编辑并保存
- schema 校验错误能在编辑器或 validation panel 中展示
- 保存按钮能触发保存到本地目录

---

### RQ005

**名称**：Manifest Editor
**优先级**：P0
**状态**：pending

**功能点**：
- 提供 manifest.yaml 最小表单（id、name、version、description、runtime）
- 高级字段（author、entry、permissions、dependencies）通过 YAML 编辑器维护
- 字段必填校验提示
- 实时更新 YAML 预览

**验收方式**：
- 修改字段后 YAML 预览同步更新
- 不填必填字段时显示红色提示
- 保存后 manifest.yaml 文件内容正确

---

### RQ006

**名称**：Role Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 提供 Markdown 文本编辑器编辑 role.md
- 支持预览渲染后的内容
- 支持角色、目标、边界的模板提示
- P0 阶段 role.md 通过 RQ004 通用 Markdown 编辑器维护

**验收方式**：
- 编辑 role.md 内容能保存
- 预览能正确渲染 Markdown

---

### RQ007

**名称**：Tools Config Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 可视化配置工具列表（id、type、command、permissions、healthcheck、startup）
- 支持添加 / 删除 / 排序工具
- 支持 MCP、CLI、HTTP 三种工具类型
- 编辑工具权限声明
- P0 阶段 tools/*.yaml 通过 RQ004 通用 YAML 编辑器维护，Tool Registry 由 RQ019 实现

**验收方式**：
- 添加一个 mock tool 配置，保存后 tools.yaml 格式正确
- 删除工具后配置同步更新

---

### RQ008

**名称**：Permissions Config Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 可视化配置权限策略（policy name、default action、scopes）
- 支持 default 选项：allow、ask、deny、allow_once、allow_session
- 配置审计日志开关和保留天数
- 提供常见权限模板
- P0 阶段 permissions/policy.yaml 通过 RQ004 通用 YAML 编辑器维护，策略执行由 RQ021 实现

**验收方式**：
- 配置一条 read_files: ask 策略，保存后 policy.yaml 正确
- 配置 audit.enabled = true 能写入文件

---

### RQ009

**名称**：Knowledge Sources Config Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 可视化配置知识源列表（source id、uri、type、description）
- 支持文件、目录、网页三种来源类型；MVP P0 只支持本地 Markdown / txt 文件，由 RQ025 实现
- 支持上传本地文件到 pack 的 knowledge/ 目录

**验收方式**：
- 添加一个文件类型的知识源，保存后 sources.yaml 正确
- 上传 Markdown 文件到 knowledge/ 目录

---

### RQ010

**名称**：UI Layout Config Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 可视化配置布局（default layout name、view 列表）
- 支持 view 类型：chat、document、table、kanban、task_graph、device_panel、approval
- 支持 view region 配置：left、main、right、bottom
- 提供布局预览图
- P0 阶段 ui/layout.yaml 通过 RQ004 通用 YAML 编辑器维护，实际渲染协议由 RQ030 实现

**验收方式**：
- 配置一个包含 4 个 view 的布局，保存后 layout.yaml 正确
- 布局预览图能反映 view 的位置

---

### RQ011

**名称**：Eval Cases Config Editor
**优先级**：P1
**状态**：pending

**功能点**：
- 可视化配置评测用例（id、goal、expected.must_include、expected.must_not）
- 支持添加 / 删除用例
- 支持编辑预期条件
- P0 阶段 evals/cases.yaml 通过 RQ004 通用 YAML 编辑器维护，执行由 RQ026 实现

**验收方式**：
- 添加一个 eval case，保存后 cases.yaml 格式正确
- 编辑 must_include / must_not 列表能正确序列化

---

### RQ012

**名称**：Pack Validation Panel
**优先级**：P0
**状态**：pending

**功能点**：
- 点击「Validate」按钮触发对当前 pack 的全量校验
- 展示校验结果：通过 / 失败
- 失败时列出具体错误信息和文件路径
- 在 Editor 顶部显示 validation 状态指示器

**验收方式**：
- 合法 pack 点击 Validate 显示通过
- 删除必填字段后点击 Validate 显示错误列表
- 错误列表包含错误位置和描述

---

### RQ013

**名称**：Pack Import / Export
**优先级**：P1
**状态**：pending

**功能点**：
- 从 ZIP 文件导入 scenario pack 到 scenarios/ 目录
- 将已有 scenario pack 导出为 ZIP 文件
- 导入前校验 pack 格式
- 导出时排除本地 secret 值，仅保留 secret ref

**验收方式**：
- 导入一个合法 ZIP，pack 出现在 Pack List
- 导出一个 pack，解压后目录结构完整
- 导出的 pack 中不包含 token / api key 明文
- 导入格式错误的 ZIP 显示错误提示

---

### RQ014

**名称**：melon Runtime 最小 Daemon
**优先级**：P0
**状态**：pending

**功能点**：
- Rust 实现的后台进程
- 提供 HTTP API（health、packs、tasks 等基础端点）
- 读取 scenarios/ 目录下的 pack
- 支持 WebSocket 推送事件（task 状态变更、trace 更新）
- 支持 graceful shutdown

**验收方式**：
- `cargo run` 能启动 daemon
- `GET /api/health` 返回 200
- `GET /api/packs` 返回已发现的 pack 列表

---

### RQ015

**名称**：Pack Loader
**优先级**：P0
**状态**：pending

**功能点**：
- 读取 scenario pack 目录并解析所有 YAML 文件
- 验证 pack 结构完整性
- 将 pack 元数据存入 SQLite
- 支持 pack 热加载（文件变更后重新加载）

**验收方式**：
- Studio 保存 pack 后 Runtime 能发现新 pack
- pack 文件变更后 Runtime 能重新加载
- 非法 pack 目录不加载并记录错误

---

### RQ016

**名称**：SQLite 基础表
**优先级**：P0
**状态**：pending

**功能点**：
- 创建以下基础表：scenario_packs、tasks、trace_events、approval_requests、tools、permissions、knowledge_sources、knowledge_items、knowledge_chunks、eval_cases、eval_runs、audit_logs、ui_sessions
- 提供 migration 机制（版本管理）
- 提供 CRUD 封装

**验收方式**：
- 首次启动自动创建所有表
- 能插入和查询 scenario_packs 记录
- 表结构变更通过 migration 文件管理

---

### RQ017

**名称**：Task Manager
**优先级**：P0
**状态**：pending

**功能点**：
- 创建任务：关联 scenario pack、用户目标
- 任务状态机：created → planning → awaiting_approval → running → completed / failed / cancelled
- 暂停 / 恢复 / 取消任务
- 查询任务列表和详情
- 任务事件通过 WebSocket 推送

**验收方式**：
- 创建任务后 status 为 created
- 更新状态到 running 成功
- 取消任务后 status 为 cancelled
- Studio 能实时看到任务状态变化

---

### RQ018

**名称**：Run / Debug Panel
**优先级**：P0
**状态**：pending

**功能点**：
- 在 Studio 中选择 pack 并点击 Run
- 显示当前任务状态和进度
- 显示实时 trace 事件流
- 支持暂停 / 恢复 / 取消运行中的任务
- 显示历史运行记录

**验收方式**：
- 点击 Run 后 Runtime 创建任务并执行
- Panel 显示 task 状态和 trace 事件
- 暂停后任务状态变为 paused
- 取消后任务停止

---

### RQ019

**名称**：Tool Registry
**优先级**：P0
**状态**：pending

**功能点**：
- 注册 / 注销工具 adapter
- 存储工具配置（id、type、command、permissions）
- 工具健康检查（healthcheck）
- 工具按需启动和停止
- 工具调用日志

**验收方式**：
- 注册一个 mock tool 后能在 registry 中查到
- 健康检查返回工具状态
- 启动 / 停止工具进程正常

---

### RQ020

**名称**：Mock Tool Adapter
**优先级**：P0
**状态**：pending

**功能点**：
- 实现 Adapter interface 的 mock 实现
- 支持预设返回值（成功 / 失败 / 延迟）
- 模拟服务状态、存储状态、网络状态检查
- 模拟文件清理操作
- 记录所有调用输入和输出

**验收方式**：
- 调用 mock_check_service 返回预设结果
- 调用 mock_cleanup_temp 触发 approval
- 所有调用记录在 tool_calls 表中

---

### RQ021

**名称**：Policy Engine
**优先级**：P0
**状态**：pending

**功能点**：
- 加载 pack 的 permissions/policy.yaml
- 评估动作的权限（allow / ask / deny / allow_once / allow_session）
- 支持按 action 和 scope 匹配策略
- 返回权限评估结果和建议

**验收方式**：
- read_files 在 workspace scope 下返回 ask
- shell 命令返回 deny
- allow_once 动作批准后本次通过，下次仍需批准
- allow_session 动作批准后本次会话通过

---

### RQ022

**名称**：Approval Panel
**优先级**：P0
**状态**：pending

**功能点**：
- 展示待审批动作：动作名称、范围、风险等级
- 用户操作：批准 / 拒绝 / 修改
- 审批结果回传给 Executor
- 显示审批历史记录

**验收方式**：
- 中风险动作触发审批，Panel 显示动作详情
- 批准后任务继续执行
- 拒绝后任务标记为 cancelled
- 审批记录写入 audit_logs

---

### RQ023

**名称**：Audit Log
**优先级**：P0
**状态**：pending

**功能点**：
- 记录每次有副作用操作的审计信息（task id、scenario id、tool id、action、input summary、output summary、approval status、timestamp、rollback hint）
- 提供查询接口（按 task、按 scenario、按时间范围）
- 在 Studio 中展示审计记录

**验收方式**：
- 每次 tool call 后 audit_logs 增加一条记录
- 按 task id 查询能返回该任务的所有审计记录
- Studio 中能看到审计记录列表

---

### RQ024

**名称**：Trace Inspector
**优先级**：P0
**状态**：pending

**功能点**：
- 展示任务的所有 trace 事件（type、summary、inputRef、outputRef、timestamp）
- 支持按类型过滤（model、tool、knowledge、ui、approval、system）
- 点击 trace 事件查看输入输出详情
- 时间线视图展示事件顺序

**验收方式**：
- 任务执行后能看到完整的 trace 事件列表
- 按 type 过滤能正确筛选事件
- 点击事件能查看输入输出摘要

---

### RQ025

**名称**：Knowledge Layer 最小实现
**优先级**：P0
**状态**：pending

**功能点**：
- 导入 Markdown / txt 文件到 knowledge/ 目录
- 解析文件内容，提取 metadata（title、contentType、hash）
- 将 metadata 存入 knowledge_items 表
- 将文件内容按段落 chunk 存入 knowledge_chunks 表
- 基于 SQLite FTS 实现全文检索
- 实现 per-scenario knowledge source 管理
- 实现 source tracking 数据结构（检索结果关联 source id）
- 记录 knowledge retrieval trace
- MVP 不实现目录监听、网页抓取、embedding、向量检索；这些进入 P1 / P2

**验收方式**：
- 导入一个 Markdown 文件，能在 knowledge_items 中查到记录
- 搜索关键词能返回匹配的 chunk 和对应的 source
- 检索结果包含 source id 和引用信息

---

### RQ026

**名称**：Eval Runner
**优先级**：P0
**状态**：pending

**功能点**：
- 读取 pack 的 evals/cases.yaml
- 逐个执行 eval case
- 检查 expected.must_include 条件
- 检查 expected.must_not 条件
- 记录 eval run 结果（pass / fail）到 eval_runs 表
- 在 Studio 中展示 eval 结果列表

**验收方式**：
- 运行一个 eval case，输出 pass 或 fail
- must_include 条件不满足时标记为 fail
- must_not 条件被违反时标记为 fail
- eval run 记录可查询

---

### RQ027

**名称**：Demo Ops Pack — 巡检流程
**优先级**：P0
**状态**：pending

**功能点**：
- 创建 scenarios/demo-ops/ 目录和完整 pack 结构
- 定义巡检 workflow：生成检查项 → 检查服务 → 检查存储 → 检查网络 → 发现异常 → 生成摘要
- 配置 mock_check_service、mock_check_storage、mock_check_network 三个 mock tool
- 预设 storage warning 场景
- 配置对应的 eval case

**验收方式**：
- 发起"执行今日系统巡检"能走完完整流程
- UI 显示服务/存储/网络状态卡片
- 发现 storage warning 并展示异常卡片
- 生成巡检摘要
- eval case 通过

---

### RQ028

**名称**：Demo Ops Pack — 审批流程
**优先级**：P0
**状态**：pending

**功能点**：
- 定义审批 workflow：用户发起清理 → 判断副作用 → 触发审批 → 用户批准 → 执行清理 → 记录审计
- 配置 mock_cleanup_temp 工具（中风险，需审批）
- 审批 Panel 展示动作、范围、风险等级
- 配置对应的 eval case

**验收方式**：
- 发起"清理临时文件"触发审批
- Approval Panel 展示完整动作信息
- 批准后 mock_cleanup_temp 执行成功
- audit log 记录完整结果

---

### RQ029

**名称**：Demo Ops Pack — 知识报告流程
**优先级**：P0
**状态**：pending

**功能点**：
- 在 demo-ops/knowledge/ 下放置 inspection_runbook.md 样例文件
- 定义知识报告 workflow：检索知识 → 记录 source ref → 生成处理建议 → 展示引用
- 配置知识检索和 source tracking
- 配置对应的 eval case（必须包含 source ref）

**验收方式**：
- 发起"根据巡检手册生成处理建议"能完成流程
- 处理建议中展示来源引用
- eval case 检查必须包含 source ref 并通过

---

### RQ030

**名称**：通用 UI Panel 协议
**优先级**：P0
**状态**：pending

**功能点**：
- 定义 UI panel 的数据协议（panel type、data schema、actions）
- 实现 Panel Registry：注册可用 panel 类型
- 实现 Panel Router：根据 pack 的 ui/layout.yaml 渲染对应 panel
- 支持 Entity List Panel、Entity Detail Panel、Action Panel
- Panel 数据通过 WebSocket 从 Runtime 推送

**验收方式**：
- 根据 layout.yaml 配置能正确渲染对应 panel
- Entity List Panel 能展示实体列表
- Action Panel 能触发动作并反馈结果

---

### RQ031

**名称**：melon Home Pack 结构
**优先级**：P0
**状态**：pending

**功能点**：
- 创建 scenarios/melon-home/ 目录和完整 pack 结构
- manifest.yaml：melon.home、name、version、dependencies
- home/rooms.yaml：房间定义
- home/devices.yaml：设备定义（占位，由 HA 动态填充）
- home/members.yaml：家庭成员
- home/preferences.yaml：场景偏好
- workflows/scenes.yaml：场景工作流
- tools/home_assistant.yaml：HA adapter 配置
- ui/layout.yaml：home dashboard 布局
- permissions/policy.yaml：home 权限策略
- evals/cases.yaml：home 评测用例
- P0 只要求最小 home pack 能承载一个 HA 灯光设备；完整全屋模型归 P1

**验收方式**：
- pack 结构完整，validation 通过
- 所有 YAML 文件格式正确
- Studio 能编辑 home pack 各部分

---

### RQ032

**名称**：Home Assistant Adapter
**优先级**：P0
**状态**：pending

**功能点**：
- 实现 Adapter interface 对接 Home Assistant REST API
- 配置 Home Assistant URL 和 access token secret ref
- access token 写入本地加密 secret store，不允许明文写入 `tools/home_assistant.yaml`
- Studio 配置界面中 token 默认脱敏展示
- pack export 必须排除真实 token，仅保留 secret ref
- GET /api/states 获取所有 entities
- POST /api/services/light/turn_on 控制灯
- POST /api/services/light/turn_off 控制灯
- POST /api/services/switch/turn_on 控制插座（P1）
- POST /api/services/switch/turn_off 控制插座（P1）
- 映射 HA entities 为 melonOS 设备模型（light、switch、sensor）
- 工具调用写入 trace 和 audit log
- 错误处理和展示

**验收方式**：
- 配置 HA URL 和 token secret ref 后能读取设备列表
- `tools/home_assistant.yaml` 中不出现明文 token
- 能控制一个灯的开关
- 能调整灯的亮度
- token 脱敏展示，导出 pack 时不包含真实 token
- 每次调用都有 trace 和 audit 记录

---

### RQ033

**名称**：Home 模型
**优先级**：P0
**状态**：pending

**功能点**：
- SQLite 表：homes、rooms、home_devices、home_scenes、home_events
- home model schema：id、name、rooms、members、devices、scenes
- room schema：id、name、type、deviceIds
- device schema：id、name、roomId、vendor、category、capabilities、state、riskLevel、adapterRef
- scene schema：id、name、intentExamples、triggers、actions、riskLevel、approval
- 从 HA entities 自动映射到 home_devices
- 支持房间分组和设备关联
- P0 最小模型只要求支持 light entity；switch、sensor、复杂 scene 进入 P1

**验收方式**：
- 从 HA 导入 entities 后 home_devices 表有记录
- 能把一个 light entity 映射为 melonOS 设备
- 设备 riskLevel 正确分级（low / medium / high）

---

### RQ034

**名称**：Home Dashboard
**优先级**：P1
**状态**：pending

**功能点**：
- 展示家庭总览：在线设备数、异常设备数、当前活跃场景
- 快捷场景按钮：观影模式、睡眠模式、离家模式
- Agent 任务面板
- 审批队列
- 事件时间线摘要

**验收方式**：
- Dashboard 显示家庭基本信息
- 点击场景按钮能执行对应场景
- Agent 任务面板显示当前任务状态

---

### RQ035

**名称**：Room View
**优先级**：P1
**状态**：pending

**功能点**：
- 展示房间列表（客厅、卧室、书房等）
- 点击进入房间详情
- 房间详情：设备列表、设备状态、快捷操作
- 支持按房间过滤设备

**验收方式**：
- Room View 显示所有房间
- 点击进入房间能看到该房间的设备列表
- 设备状态实时更新

---

### RQ036

**名称**：Device Panel
**优先级**：P1
**状态**：pending

**功能点**：
- 展示设备详情：名称、类型、能力、当前状态
- 设备操作按钮（开关、亮度滑块、色温滑块）
- 操作触发审批流程（中高风险）
- 显示设备操作历史

**验收方式**：
- Device Panel 显示设备当前状态
- 点击开关能控制设备
- 中高风险操作触发审批

---

### RQ037

**名称**：Scene Panel
**优先级**：P1
**状态**：pending

**功能点**：
- 展示可用场景列表（观影模式、睡眠模式、离家模式、起夜模式等）
- 场景描述和风险提示
- 一键执行场景
- 显示场景执行进度

**验收方式**：
- Scene Panel 显示所有定义的场景
- 点击执行场景能完成所有动作
- 执行过程显示进度

---

### RQ038

**名称**：灯光控制
**优先级**：P0
**状态**：pending

**功能点**：
- 通过 HA Adapter 控制灯光开关
- 控制灯光亮度（0-100%）
- 控制灯光色温
- 灯光控制属于低风险操作，默认允许执行

**验收方式**：
- 能开关一盏灯
- 能调整灯的亮度
- 能调整灯的色温
- 操作记录写入 trace

---

### RQ039

**名称**：插座控制
**优先级**：P1
**状态**：pending

**功能点**：
- 通过 HA Adapter 控制插座开关
- 插座控制属于中风险操作，触发审批

**验收方式**：
- 能控制一个插座的开关
- 控制前触发审批流程
- 批准后插座状态改变

---

### RQ040

**名称**：传感器状态读取
**优先级**：P1
**状态**：pending

**功能点**：
- 通过 HA Adapter 读取传感器状态（温湿度、人体感应等）
- 传感器读取属于低风险操作
- 在 Room View 和 Device Panel 中展示传感器状态

**验收方式**：
- 能读取温湿度传感器当前值
- 能读取人体传感器状态
- 传感器状态在 UI 中正确展示

---

### RQ041

**名称**：中高风险动作审批
**优先级**：P0
**状态**：pending

**功能点**：
- 复用 RQ022 Approval Panel
- 根据 home 设备 riskLevel 触发不同审批策略
- 低风险（灯光）：默认允许
- 中风险（插座、窗帘、空调）：触发审批
- 高风险（门锁、摄像头、安防）：默认拒绝
- 审批策略由 permissions/policy.yaml 配置

**验收方式**：
- 灯光控制不触发审批
- mock 中风险设备动作触发审批，用户批准后执行
- mock 高风险设备动作默认拒绝

---

### RQ042

**名称**：设备操作 Trace / Audit
**优先级**：P0
**状态**：pending

**功能点**：
- 复用 RQ023 Audit Log 和 RQ024 Trace Inspector
- 每次 HA Adapter 调用写入 trace_events
- 每次有副作用操作写入 audit_logs
- Trace Inspector 能按 home scenario 过滤
- Audit Log 能按 home 设备查询

**验收方式**：
- 每次设备控制后 trace_events 有对应记录
- 每次有副作用操作后 audit_logs 有对应记录
- Trace Inspector 能查看完整操作链路
- Audit Log 能按设备查询历史记录

---

## 需求统计

| 优先级 | 数量 |
|---:|---|
| P0 | 29 |
| P1 | 13 |

| 里程碑 | 数量 |
|---|---|
| Milestone 1 | 16 |
| Milestone 2 | 14 |
| Milestone 3 | 12 |

| 里程碑 | P0 数量 | P1 数量 |
|---|---:|---:|
| Milestone 1 | 9 | 7 |
| Milestone 2 | 14 | 0 |
| Milestone 3 | 6 | 6 |

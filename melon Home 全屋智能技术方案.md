# melon Home 全屋智能技术方案

创建日期：2026-05-18

关联文档：[[melonOS 技术方案]]

开发计划：[[melonOS MVP 开发计划]]

## 1. 定位

melon Home 是 melonOS 的第一批验证场景之一。

它不是 melonOS 的产品重心，也不是要把 melonOS 变成智能家居平台。它的定位是作为第一个真实 scenario pack，由 melon Studio 创建和维护，用来验证 melon Runtime 的通用能力。

在这个场景里，melon Home 表现为：

> 一个本地优先的家庭 agent 中枢，用自然语言、场景包、家庭知识库、动态 UI 和可审计自动化来编排全屋设备。

它和 Home Assistant 的关系：

- Home Assistant 是规则引擎、设备抽象层和控制面板；
- melon Home 是运行在 melonOS 上的家庭 agent scenario pack；
- melon Home 不替代 HA 的设备生态，而是在 HA 之上提供模糊意图理解、任务 trace、权限审批、审计记录和可迭代的场景包。

核心差异：

> HA 解决“设备怎么接、规则怎么跑”；melon Home 验证“用户说出模糊意图后，agent 如何把它变成可解释、可审批、可追踪的一组动作”。

本文档只定义 melon Home 这个验证场景，不改变 melonOS 的 runtime-first 主线。

## 2. 为什么全屋智能适合 melonOS

全屋智能天然需要 melon Runtime 的所有核心 layers：

- `Knowledge Layer`：房间、设备、家庭成员、习惯、说明书、故障记录、场景偏好；
- `Tool / MCP Layer`：Home Assistant、小米、Matter、MQTT、Yeelight、Zigbee、红外、局域网设备；
- `Skill Layer`：照明策略、离家策略、观影策略、睡眠策略、异常诊断策略；
- `Agent Layer`：理解「观影模式」「老人起夜」「离家省电」「孩子睡了」这类模糊意图；
- `Adaptive UI Layer`：房间视图、设备面板、场景面板、自动化编辑、告警时间线；
- `Governance Layer`：对门锁、摄像头、空调、强电、安防等高风险动作做确认、审计和回滚。

这个场景比纯 research/coding 更有 OS 感，因为 agent 会直接编排物理空间里的设备。

## 3. 复杂度判断

| 能力 | 复杂度 | 说明 |
|---|---:|---|
| 灯光、插座、空调等单设备控制 | 低到中 | 通过 Home Assistant / 小米 / Matter / Yeelight 可实现 |
| 房间/场景 UI | 中 | 需要设备建模、房间建模、状态同步 |
| 自然语言控制 | 中 | LLM + intent parser + tool call 即可 |
| 多设备联动自动化 | 中高 | 需要处理条件、冲突、失败、恢复 |
| 家庭习惯学习 | 高 | 涉及长期数据、隐私、安全策略 |
| 安防/门锁/摄像头 agent | 高 | 权限、误操作和隐私风险大 |
| 自研硬件生态 | 很高 | 供应链、固件、认证、售后都重 |

MVP 应从灯光、插座、传感器、房间视图和场景模式开始，不要第一版就做门锁、摄像头、燃气、强电控制。

## 4. 接入架构

推荐第一阶段把 Home Assistant 作为设备抽象层。

```text
melon Home Pack
      ↓
melon Runtime
      ↓
Home Assistant Adapter / Matter Adapter / Yeelight Adapter / MQTT Adapter
      ↓
小米灯 / Matter 灯 / 插座 / 传感器 / 空调 / 网关
```

这样 melonOS 不需要一开始直接适配所有品牌设备，而是先专注：

- agent 编排；
- 场景理解；
- 家庭知识建模；
- 权限审计；
- 动态 UI；
- 设备 adapter 抽象。

## 5. 小米灯设备接入路线

小米/米家设备接入建议分三条路径。

### 路径 A：Home Assistant + Xiaomi Home Integration

这是最快的 MVP 路线。

```text
melonOS
  ↓
Home Assistant Adapter
  ↓
Xiaomi Home Integration / Xiaomi Miio / MIoT
  ↓
小米灯 / 插座 / 传感器
```

优点：

- 设备覆盖广；
- 原型速度快；
- 可复用 Home Assistant 的实体模型、状态同步和自动化能力；
- 适合先验证 melon Home Pack。

限制：

- 部分小米设备依赖 MIoT Cloud；
- 局域网控制能力取决于设备型号、网关和地区；
- 状态同步与高级能力可能不稳定；
- 需要处理小米账号、地区和云端 API 限制。

### 路径 B：Yeelight LAN / 局域网灯光 Adapter

部分 Yeelight Wi-Fi 灯支持官方第三方局域网控制协议，可通过局域网发现设备，并用 JSON 命令控制开关、亮度、色温、RGB 等能力。

```text
melonOS Tool Layer
  ↓
Yeelight LAN Adapter
  ↓
局域网发现 / 开关 / 调光 / 色温 / RGB / 场景
```

优点：

- 本地控制；
- 低延迟；
- 适合做第一版灯光 demo；
- 能体现 melonOS 的 local-first 价值。

限制：

- 不是所有小米/Yeelight 灯都支持 LAN Control；
- 需要用户在 App 中开启局域网控制/开发者模式；
- 协议明文传输，安全依赖家庭局域网；
- 设备固件和型号差异需要适配。

### 路径 C：Matter

Matter 是长期应该支持的标准路线。

```text
melonOS
  ↓
Matter Adapter / Home Assistant Matter
  ↓
Matter 灯 / 插座 / 传感器 / 窗帘 / 空调
```

优点：

- 标准化；
- 本地优先；
- 跨品牌；
- 更符合 melonOS 的平台中立定位。

限制：

- Matter 对部分设备只暴露基础能力；
- 品牌特有能力仍可能需要原生集成；
- Thread/Matter 网络调试有门槛；
- 旧设备覆盖不足。

## 6. melon Home Pack 结构

```text
melon-home/
├── manifest.yaml
├── role.md
├── home/
│   ├── rooms.yaml
│   ├── devices.yaml
│   ├── members.yaml
│   └── preferences.yaml
├── workflows/
│   ├── lighting.yaml
│   ├── scenes.yaml
│   ├── diagnostics.yaml
│   └── energy_saving.yaml
├── tools/
│   ├── home_assistant.yaml
│   ├── matter.yaml
│   ├── yeelight.yaml
│   └── mqtt.yaml
├── ui/
│   ├── layout.yaml
│   ├── room_view.yaml
│   ├── device_panel.yaml
│   ├── scene_panel.yaml
│   └── timeline.yaml
├── permissions/
│   └── policy.yaml
└── evals/
    ├── lighting_cases.yaml
    ├── scene_cases.yaml
    └── safety_cases.yaml
```

## 7. 家庭设备模型

```ts
type Home = {
  id: string;
  name: string;
  rooms: Room[];
  members: HomeMember[];
  devices: HomeDevice[];
  scenes: HomeScene[];
};

type Room = {
  id: string;
  name: string;
  floor?: string;
  type: "living_room" | "bedroom" | "kitchen" | "bathroom" | "study" | "balcony" | "other";
  deviceIds: string[];
};

type HomeDevice = {
  id: string;
  name: string;
  roomId: string;
  vendor: "xiaomi" | "yeelight" | "matter" | "mqtt" | "home_assistant" | "custom";
  category: "light" | "plug" | "sensor" | "curtain" | "climate" | "camera" | "lock" | "gateway" | "other";
  capabilities: DeviceCapability[];
  state: Record<string, unknown>;
  riskLevel: "low" | "medium" | "high" | "critical";
  adapterRef: string;
};
```

能力模型：

```ts
type DeviceCapability =
  | "power"
  | "brightness"
  | "color_temperature"
  | "rgb"
  | "scene"
  | "occupancy"
  | "temperature"
  | "humidity"
  | "energy"
  | "lock"
  | "camera_stream"
  | "ir_control";
```

## 8. 场景自动化模型

```ts
type HomeScene = {
  id: string;
  name: string;
  intentExamples: string[];
  triggers: SceneTrigger[];
  conditions: SceneCondition[];
  actions: SceneAction[];
  riskLevel: "low" | "medium" | "high" | "critical";
  approval: "never" | "ask" | "ask_if_high_risk";
};
```

第一批场景：

- 回家模式；
- 离家模式；
- 观影模式；
- 睡眠模式；
- 起夜模式；
- 阅读模式；
- 会客模式；
- 节能模式；
- 全屋关灯；
- 设备异常诊断。

## 9. 权限策略

全屋智能必须把设备动作按风险分级。

低风险，可默认执行：

- 开关灯；
- 调整亮度；
- 调整色温；
- 切换灯光场景；
- 读取温湿度；
- 读取设备在线状态。

中风险，建议按场景确认：

- 控制空调温度；
- 控制插座；
- 控制窗帘；
- 执行全屋联动；
- 夜间自动化；
- 修改长期自动化规则。

高风险，必须确认：

- 门锁；
- 摄像头；
- 安防布撤防；
- 强电设备；
- 燃气/热水器；
- 对外发送家庭状态；
- 删除历史日志或家庭记忆。

权限策略示例：

```yaml
policies:
  light.control:
    default: allow_session
    audit: true
  plug.control:
    default: ask
    audit: true
  climate.control:
    default: ask_if_unattended
    audit: true
  lock.control:
    default: deny
    can_override: ask_owner
    audit: true
  camera.view:
    default: ask
    audit: true
  automation.modify:
    default: ask
    audit: true
```

## 10. UI 设计

melon Home 的第一版 UI 不应该是聊天框，而应该是家庭控制台：

- 房间总览；
- 房间详情；
- 设备卡片；
- 场景按钮；
- 自动化时间线；
- 家庭状态摘要；
- agent 任务面板；
- 审批面板；
- 设备异常面板；
- 操作审计记录。

UI 示例：

```text
Home Dashboard
├── Home Status
├── Rooms
│   ├── Living Room
│   ├── Bedroom
│   └── Study
├── Scenes
│   ├── Movie
│   ├── Sleep
│   └── Away
├── Agent Task
├── Approval Queue
└── Event Timeline
```

## 11. 是否需要自有硬件

短期不需要自研灯泡、插座、传感器等设备。

第一阶段只需要让 melonOS 运行在已有硬件上：

- Mac / Linux；
- x86 mini PC；
- Raspberry Pi；
- RK3588；
- Home Assistant 主机。

中期建议做 `melon Home Node` 参考硬件，而不是自研全套智能家居设备。

melon Home Node 可以是：

- x86 mini PC / RK3588；
- 预装 melon Runtime；
- 预装 melon Home Pack；
- 可选 Home Assistant；
- 可选 Zigbee/Thread dongle；
- 可选麦克风/音箱；
- 可选触摸屏；
- 本地知识库和审计日志。

长期如果要做硬件，优先级应该是：

1. 家庭 agent 中枢；
2. 智能中控屏；
3. 多协议网关；
4. 语音控制器；
5. 传感器套装。

不建议早期做：

- 灯泡；
- 插座；
- 门锁；
- 摄像头；
- 强电开关。

## 12. melon Home MVP

第一版支持：

- Home Assistant Adapter；
- Yeelight LAN Adapter；
- Matter Adapter 基础支持；
- MQTT Adapter；
- 房间建模；
- 设备建模；
- 灯光开关；
- 亮度/色温/RGB；
- 插座开关；
- 简单传感器状态；
- 回家、离家、观影、睡眠、起夜等场景；
- 操作审计；
- 高风险动作确认；
- 设备异常诊断。

第一版不做：

- 门锁自动开门；
- 摄像头人脸识别；
- 燃气/热水器控制；
- 强电自动控制；
- 自动购买耗材；
- 复杂家庭成员权限；
- 完全自学习的无人确认自动化；
- 自研灯泡/插座/传感器。

## 13. 参考资料

- Xiaomi Home Integration for Home Assistant: https://github.com/XiaoMi/ha_xiaomi_home
- Home Assistant Xiaomi Home integration: https://www.home-assistant.io/integrations/xiaomi_miio/
- Home Assistant Matter integration: https://www.home-assistant.io/integrations/matter/
- Yeelight Developer / LAN Control: https://www.yeelight.com/en_US/developer
- Xiaomi Vela developer platform: https://dev.mi.com/xiaomihyperos/xiaomivela

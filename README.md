# 跨平台代理客户端

为 OpenClaw 网关提供远程设备管理能力的客户端，支持 Windows、macOS、Linux 三大平台，实现命令远程执行、系统信息获取、进程管理等功能。

---

## 📋 目录

1. [项目概述](#项目概述)
2. [核心原理](#核心原理)
3. [功能列表](#功能列表)
4. [安装部署](#安装部署)
5. [配置说明](#配置说明)
6. [支持命令](#支持命令)
7. [通信协议](#通信协议)
8. [故障排除](#故障排除)
9. [版本历史](#版本历史)
10. [相关链接](#相关链接)

---

## 项目概述

### 是什么？

clw-agent-client-rs 是一个 Rust 编写的跨平台代理客户端，配合 OpenClaw 网关的 Remote Agent 插件使用，部署在远程设备上以实现远程控制和管理。

### 能做什么？

- 🖥️ **远程执行命令** - 在本设备上执行服务端推送的 Shell 命令
- 📊 **获取系统信息** - 向服务端提供本设备的主机名、操作系统、CPU、内存等信息
- 📁 **文件管理** - 列出目录、读写文件等操作
- 📱 **进程管理** - 列出进程、启动/停止进程
- 🌐 **浏览器控制** - 打开指定网址
- 🔧 **软件管理** - 通过 winget（Windows）、brew（macOS）、apt（Linux）安装和卸载软件
- 🔐 **环境变量管理** - 获取和设置系统/用户环境变量
- ♻️ **系统操作** - 重启、关机等操作

### 适用场景

- 远程服务器管理
- 家庭/办公室电脑远程控制
- 自动化运维任务
- 跨平台设备统一管理

---

## 核心原理

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      远程设备（客户端）                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Cross-Platform Agent                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │   Platform  │  │    Auth     │  │   Config    │ │   │
│  │  │   Layer     │  │   Module    │  │   Loader    │ │   │
│  │  │ (Win/Mac/Ln)│  │  (HMAC)     │  │  (YAML)     │ │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │   │
│  └─────────┼────────────────┼────────────────┼────────┘   │
└────────────┼────────────────┼────────────────┼──────────────┘
             │                │                │
             │ WebSocket      │ Token          │ Config
             │ Connection     │ Auth           │ Load
             │                │                │
┌────────────┴────────────────┴────────────────┴──────────────┐
│                        OpenClaw 网关                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Remote Agent 插件                       │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │ WebSocket   │  │ Unix Socket │  │   Tools     │ │   │
│  │  │   Server    │  │   Server    │  │  Registry   │ │   │
│  │  │  (8765端口)  │  │  (实时事件)  │  │  (AI工具)   │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### 工作流程

1. **启动连接**：客户端启动后，通过 WebSocket 连接到服务端 `server_url`
2. **身份认证**：客户端发送 `agent_id` 和 `token` 进行认证
3. **保持连接**：认证成功后，客户端保持长连接，等待服务端推送命令
4. **命令执行**：服务端发送命令（通过 AI 工具调用或直接推送），客户端执行后返回结果
5. **自动重连**：连接断开后，客户端会自动尝试重新连接

### 关键技术点

| 组件 | 技术 | 说明 |
|------|------|------|
| WebSocket 客户端 | tokio-tungstenite | 与服务端建立长连接，支持双向通信 |
| 平台抽象层 | async-trait | 统一接口，支持 Windows/macOS/Linux |
| 配置加载 | serde_yaml | 读取 YAML 格式配置文件 |
| 日志系统 | tracing | 结构化日志，支持多级别输出 |
| 异步运行时 | tokio | 异步 I/O 事件处理 |

---

## 功能列表

### 客户端支持的命令

| 命令 | 说明 | 参数 |
|------|------|------|
| `system.info` | 获取系统信息 | 无 |
| `shell.execute` | 执行 Shell 命令 | `command`, `timeout` |
| `process.list` | 列出进程 | 无 |
| `software.list` | 列出已安装软件 | 无 |
| `env.list` | 列出环境变量 | `scope`（user/system/session） |
| `file.list` | 列出文件 | `path` |

### 平台特定功能

#### Windows 平台

| 功能 | 说明 |
|------|------|
| 注册表操作 | 读取/写入 Windows 注册表 |
| WMI 查询 | 系统信息查询 |
| 软件管理 | 通过 winget 安装/卸载软件 |
| 服务管理 | Windows 服务控制 |

#### macOS 平台

| 功能 | 说明 |
|------|------|
| Launchd 管理 | 启动项管理 |
| AppleScript | 脚本自动化 |
| Homebrew | 软件包管理 |

#### Linux 平台

| 功能 | 说明 |
|------|------|
| D-Bus 通信 | 系统服务交互 |
| systemd | 服务管理 |
| APT/DNF/YUM | 软件包管理 |

---

## 安装部署

### 前置条件

- Rust 1.70+ 编译环境
- 目标平台：Windows 10+、macOS 10.15+、Linux（主流发行版）
- 可访问 OpenClaw 服务器的 8765 端口

### 步骤一：克隆仓库

```bash
git clone https://github.com/easy-do/claw-agent-client-rs.git
cd claw-agent-client-rs
```

### 步骤二：编译项目

```bash
# 开发模式编译
cargo build

# 生产模式编译（推荐）
cargo build --release
```

编译产物位于 `target/release/claw-agent-client-rs`（Linux/macOS）或 `target/release/claw-agent-client-rs.exe`（Windows）。

### 步骤三：配置文件


编辑 `config/agent.yml`：

```yaml
# 设备 ID（必须与服务端配置的 key 匹配）
agent_id: "台式机"

# OpenClaw 服务器 WebSocket 地址
server_url: "ws://your-server.com:8765"

# 认证配置
auth:
  token: "agent-desktop-xxx"

# 可选：功能开关 (未实现)
capabilities:
  system_info: true
  process_control: true
  env_management: true
  software_install: true
  file_operations: true
```

### 步骤四：运行客户端

```bash
# 开发模式运行
cargo run

# 生产模式运行
./target/release/claw-agent-client-rs
```

### 步骤五：验证连接

客户端成功连接后，服务端日志会显示设备上线信息：

```
[Remote Agent] Agent connected: 台式机, session: xxx
```

---

## 配置说明

### 配置文件结构

```yaml
# ==========================================
# 必填配置项
# ==========================================

# 设备唯一标识符
agent_id: "设备名称"

# OpenClaw 服务器地址（WebSocket）
server_url: "ws://服务器地址:8765"

# 认证令牌（与服务端配置匹配）
auth:
  token: "agent-xxx"

# ==========================================
# 可选配置项
# ==========================================

# 代理能力配置（默认全部开启 未实现）
capabilities:
  system_info: true        # 系统信息查询
  process_control: true    # 进程控制
  env_management: true     # 环境变量管理
  software_install: true   # 软件安装卸载
  file_operations: true    # 文件操作

```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `AGENT_CONFIG` | 配置文件路径 | `config/agent.yml` |
| `AGENT_LOG_LEVEL` | 日志级别 | `info` |
| `AGENT_SECRET_KEY` | 密钥（预留） | - |

---

## 支持命令

### system.info

获取本机系统信息：

```json
{
  "action": "system.info",
  "params": {}
}
```

返回：

```json
{
  "hostname": "DESKTOP-PC",
  "os_type": "Windows",
  "os_version": "10.0.19045",
  "arch": "x64",
  "username": "admin",
  "uptime_secs": 86400,
  "total_memory_gb": 16.0,
  "available_memory_gb": 8.5,
  "cpu_count": 8,
  "cpu_usage_percent": 15.5
}
```

### shell.execute

执行 Shell 命令：

```json
{
  "action": "shell.execute",
  "params": {
    "command": "dir C:\\",
    "timeout": 30
  }
}
```

返回：

```json
{
  "stdout": " 驱动器 C 中的卷没有标签。\n...",
  "stderr": "",
  "exit_code": 0,
  "platform": "windows"
}
```

### process.list

列出正在运行的进程：

```json
{
  "action": "process.list",
  "params": {}
}
```

返回：

```json
[
  {
    "pid": 0,
    "name": "System Idle Process",
    "cmd": "",
    "cpu_percent": 0.0,
    "memory_mb": 0,
    "status": "Running"
  }
]
```

### software.list

列出已安装软件：

```json
{
  "action": "software.list",
  "params": {}
}
```

返回：

```json
[
  {
    "name": "Google Chrome",
    "version": "120.0.6099.130",
    "publisher": "Google LLC",
    "install_path": "C:\\Program Files\\Google\\Chrome\\Application"
  }
]
```

### env.list

列出环境变量：

```json
{
  "action": "env.list",
  "params": {
    "scope": "user"
  }
}
```

返回：

```json
[
  {
    "name": "PATH",
    "value": "C:\\Windows\\system32;..."
  }
]
```

### file.list

列出目录内容：

```json
{
  "action": "file.list",
  "params": {
    "path": "C:\\Users"
  }
}
```

返回：

```json
[
  {
    "name": "Admin",
    "path": "C:\\Users\\Admin",
    "is_dir": true,
    "size_bytes": 0,
    "modified": 1704067200
  }
]
```

---

## 通信协议

### WebSocket 连接

客户端连接到服务端 URL：`{server_url}/agent/ws`

### 消息格式

#### 1. 服务端欢迎消息

服务端 → 客户端：

```json
{
  "type": "welcome",
  "version": "0.1.0",
  "platform": "openclaw-remote-agent"
}
```

#### 2. 客户端认证

客户端 → 服务端：

```json
{
  "type": "auth",
  "agent_id": "台式机",
  "token": "agent-desktop-xxx"
}
```

服务端 → 客户端：

```json
{
  "type": "auth_response",
  "success": true,
  "session_id": "uuid",
  "message": "认证成功"
}
```

#### 3. 命令推送

服务端 → 客户端：

```json
{
  "command_id": "cmd-1234567890-1",
  "action": "shell.execute",
  "params": {
    "command": "whoami",
    "timeout": 30000
  }
}
```

#### 4. 命令响应

客户端 → 服务端：

```json
{
  "type": "command_response",
  "command_id": "cmd-1234567890-1",
  "success": true,
  "data": {
    "stdout": "admin\n",
    "stderr": "",
    "exit_code": 0
  }
}
```

#### 5. 心跳（Ping/Pong）

服务端 → 客户端：

```json
{
  "type": "ping"
}
```

客户端 → 服务端：

```json
{
  "type": "pong"
}
```

（实际实现中使用 WebSocket 原生 Ping/Pong 帧）

---

## 故障排除

### 常见问题

#### 1. 连接失败

**症状**：客户端日志显示连接失败

**排查步骤**：

```bash
# 检查网络连通性
ping your-server.com

# 检查端口是否开放
telnet your-server.com 8765

# 查看客户端日志
tail -f log/cmd.log
```

**可能原因**：

- 服务器地址错误
- 服务器未运行
- 防火墙阻止连接
- 网络不可达

#### 2. 认证失败

**症状**：服务端返回 "Authentication failed"

**排查步骤**：

1. 确认 `agent_id` 与服务端配置一致
2. 确认 `token` 与服务端配置完全匹配
3. 检查配置文件中是否有多余空格或换行

#### 3. 命令执行超时

**症状**：发送命令后长时间无响应

**排查步骤**：

```bash
# 查看客户端日志
tail -f log/cmd.log

# 检查命令是否正确送达
# 日志中应有 "Received command" 信息
```

**可能原因**：

- 客户端处理命令耗时过长
- 网络延迟过高
- 命令阻塞

#### 4. 权限不足

**症状**：部分操作返回权限错误

**排查步骤**：

```bash
# Windows：以管理员身份运行客户端
# macOS：确保有 Full Disk Access 权限
# Linux：检查是否需要 sudo
```

---

## 相关链接

- **服务端插件仓库**：https://gitee.com/yuzhanfeng/claw-remote-agent-plugin.git
- **服务端插件仓库**：https://github.com/easy-do/claw-remote-agent-plugin.git
- **OpenClaw 文档**：https://docs.openclaw.ai


# 跨平台代理客户端

为 OpenClaw 网关提供远程设备管理能力的客户端，支持 Windows、macOS、Linux 三大平台，实现命令远程执行、系统信息获取、进程管理等功能。

---

## 📋 目录

1. [项目概述](#项目概述)
2. [核心原理](#核心原理)
3. [功能列表](#功能列表)
4. [构建部署](#构建部署)
5. [安装部署](#安装部署)
6. [配置说明](#配置说明)
7. [支持命令](#支持命令)
8. [通信协议](#通信协议)
9. [故障排除](#故障排除)

---

## 项目概述

claw-agent-client-rs 是一个 Rust 编写的跨平台代理客户端，配合 OpenClaw 网关的 Remote Agent 插件使用，部署在远程设备上以实现远程控制和管理。

### 功能

- 🖥️ **远程执行命令** - 执行服务端推送的 Shell 命令
- 📊 **获取系统信息** - 主机名、操作系统、CPU、内存等信息
- 📁 **文件管理** - 读写文件、复制移动、下载等
- 📱 **进程管理** - 列出进程、停止进程
- 🔧 **软件管理** - 搜索、安装、卸载软件
- ⚙️ **配置管理** - 读取和写入系统配置
- ♻️ **系统操作** - 重启、关机

---

## 核心原理

### 认证机制

1. **单一 Token**：服务端和客户端使用相同的 Token 进行认证
2. **自动注册**：客户端连接时只要 Token 校验通过，即可自动注册
3. **唯一在线**：同一 agent_id 只能有一个在线连接

### 工作流程

1. 获取 Token（从服务端）
2. 配置客户端 `agent.yml`
3. 启动客户端，连接 WebSocket
4. 身份认证
5. 保持长连接，等待命令
6. 执行命令，返回结果

---

## 功能列表

### 客户端支持的命令

| 命令 | 说明 | 参数 |
|------|------|------|
| `capabilities` | 获取客户端支持的所有命令列表 | 无 |
| `system.info` | 获取系统信息 | 无 |
| `system.reboot` | 重启系统 | 无 |
| `system.shutdown` | 关闭系统 | 无 |
| `process.list` | 列出进程 | 无 |
| `process.stop` | 停止进程 | `pid`, `force` |
| `software.list` | 列出已安装软件 | 无 |
| `software.search` | 搜索软件 | `query` |
| `software.install` | 安装软件 | `package`, `silent` |
| `software.uninstall` | 卸载软件 | `package` |
| `env.list` | 列出环境变量 | `scope` |
| `env.get` | 获取环境变量 | `name`, `scope` |
| `env.set` | 设置环境变量 | `name`, `value`, `scope` |
| `env.delete` | 删除环境变量 | `name`, `scope` |
| `file.list` | 列出文件 | `path` |
| `file.read` | 读取文件内容 | `path` |
| `file.write` | 写入文件内容 | `path`, `content` |
| `file.delete` | 删除文件 | `path` |
| `file.create_dir` | 创建目录 | `path`, `recursive` |
| `file.copy` | 复制文件 | `src`, `dst` |
| `file.move` | 移动文件 | `src`, `dst` |
| `file.download` | 下载文件 | `url`, `dest` |
| `config.get` | 获取配置 | `path` |
| `config.set` | 设置配置 | `path`, `value` |
| `shell.execute` | 执行 Shell 命令 | `command`, `timeout` |

---

## 构建部署

### 方法一：GitHub Release

从 [GitHub Releases](https://github.com/easy-do/claw-agent-client-rs/releases) 下载预编译的压缩包。

### 方法二：本地构建

#### Windows
```powershell
.\scripts\builds\windows_build.bat
```

#### macOS
```bash
chmod +x scripts/builds/macos_build.sh
./scripts/builds/macos_build.sh
```

#### Linux
```bash
chmod +x scripts/builds/linux_build.sh
sudo ./scripts/builds/linux_build.sh
```

---

## 安装部署

### 步骤一：获取 Token

从服务端 Remote Agent 插件生成 Token。

### 步骤二：配置

编辑 `config/agent.yml`：
```yaml
agent_id: "设备名称"
server_url: "ws://your-server.com:8765"
auth:
  token: "your-token"
```

### 步骤三：安装服务

#### Windows
```powershell
.\compose\windows_install.bat install
```

#### macOS / Linux
```bash
chmod +x compose/*_install.sh
sudo ./compose/*_install.sh install
```

---

## 配置说明

### 配置文件

```yaml
agent_id: "设备名称"
server_url: "ws://服务器地址:8765"
auth:
  token: "your-token"

capabilities:
  shell.execute:
    enabled: true
    name: "Execute Shell Command"
    description: "执行Shell命令"
    category: "shell"
```

---

## 支持命令

### capabilities
```json
{ "action": "capabilities", "params": {} }
```

### system.info
```json
{ "action": "system.info", "params": {} }
```

### shell.execute
```json
{ "action": "shell.execute", "params": { "command": "dir", "timeout": 30 } }
```

### process.list
```json
{ "action": "process.list", "params": {} }
```

### process.stop
```json
{ "action": "process.stop", "params": { "pid": 1234, "force": false } }
```

### software.list
```json
{ "action": "software.list", "params": {} }
```

### software.search
```json
{ "action": "software.search", "params": { "query": "chrome" } }
```

### software.install
```json
{ "action": "software.install", "params": { "package": "Google Chrome", "silent": true } }
```

### software.uninstall
```json
{ "action": "software.uninstall", "params": { "package": "Google Chrome" } }
```

### env.list
```json
{ "action": "env.list", "params": { "scope": "user" } }
```

### env.get
```json
{ "action": "env.get", "params": { "name": "PATH", "scope": "user" } }
```

### env.set
```json
{ "action": "env.set", "params": { "name": "TEST", "value": "123", "scope": "user" } }
```

### env.delete
```json
{ "action": "env.delete", "params": { "name": "TEST", "scope": "user" } }
```

### file.list
```json
{ "action": "file.list", "params": { "path": "C:\\Users" } }
```

### file.read
```json
{ "action": "file.read", "params": { "path": "C:\\test\\file.txt" } }
```

### file.write
```json
{ "action": "file.write", "params": { "path": "C:\\test\\file.txt", "content": "hello" } }
```

### config.get
```json
{ "action": "config.get", "params": { "path": "HKEY_CURRENT_USER\\Software\\Microsoft" } }
```

### config.set
```json
{ "action": "config.set", "params": { "path": "HKEY_CURRENT_USER\\Test", "value": "123" } }
```

### system.reboot
```json
{ "action": "system.reboot", "params": {} }
```

---

## 通信协议

### WebSocket 连接
`{server_url}/agent/ws`

### 消息格式

#### 认证
```json
{ "type": "auth", "agent_id": "设备名", "token": "token" }
```

#### 命令推送
```json
{ "command_id": "cmd-1", "action": "shell.execute", "params": { "command": "whoami" } }
```

#### 命令响应
```json
{ "type": "command_response", "command_id": "cmd-1", "success": true, "data": {...} }
```

---

## 故障排除

### 连接失败
- 检查网络连通性
- 确认服务器地址和端口正确

### 认证失败
- 确认 Token 与服务端配置一致

### 权限不足
- Windows：以管理员身份运行
- Linux/macOS：使用 sudo

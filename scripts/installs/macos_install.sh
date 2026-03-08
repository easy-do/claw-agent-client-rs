#!/bin/bash

set -e

AGENT_NAME="com.claw.agent-client-rs"
AGENT_USER="root"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${SCRIPT_DIR}"
BINARY_NAME="claw-agent-client-rs"
PLIST_FILE="${AGENT_NAME}.plist"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

show_banner() {
    clear
    echo -e "${GREEN}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║         Claw Agent Client Rs 安装脚本 v1.0                ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

show_menu() {
    echo -e "${BLUE}请选择操作:${NC}"
    echo ""
    echo "  1) 安装服务"
    echo "  2) 启动服务"
    echo "  3) 停止服务"
    echo "  4) 重启服务"
    echo "  5) 查看服务状态"
    echo "  6) 卸载服务"
    echo "  7) 查看日志"
    echo "  8) 帮助信息"
    echo "  0) 退出"
    echo ""
    echo -n "请输入选项 [0-8]: "
}

check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        echo -e "${RED}错误: 请使用 root 用户运行此脚本${NC}"
        echo "用法: sudo $0"
        exit 1
    fi
}

install_binary() {
    echo -e "${GREEN}[1/4] 检查二进制文件...${NC}"

    BINARY_PATH="${SCRIPT_DIR}/${BINARY_NAME}"

    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}错误: 未找到二进制文件 ${BINARY_PATH}${NC}"
        echo "请确保二进制文件在脚本同级目录"
        return 1
    fi

    chmod +x "$BINARY_PATH"
    echo "二进制文件: $BINARY_PATH"
}

install_config() {
    echo -e "${GREEN}[2/4] 安装配置文件...${NC}"

    CONFIG_SOURCE="${SCRIPT_DIR}/agent.yml"

    mkdir -p "$INSTALL_DIR/config"

    if [ -f "$CONFIG_SOURCE" ]; then
        cp "$CONFIG_SOURCE" "$INSTALL_DIR/config/agent.yml"
        echo "配置文件已安装到: $INSTALL_DIR/config/agent.yml"
    else
        echo -e "${YELLOW}警告: 未找到配置文件 ${CONFIG_SOURCE}${NC}"
        echo "请手动创建: $INSTALL_DIR/config/agent.yml"
    fi
}

create_service() {
    echo -e "${GREEN}[3/4] 创建 launchd 服务...${NC}"

    PLIST_PATH="/Library/LaunchDaemons/${PLIST_FILE}"

    cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${AGENT_NAME}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${SCRIPT_DIR}/${BINARY_NAME}</string>
    </array>
    <key>WorkingDirectory</key>
    <string>${INSTALL_DIR}</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>${INSTALL_DIR}/agent.log</string>
    <key>StandardErrorPath</key>
    <string>${INSTALL_DIR}/agent.error.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>AGENT_LOG_LEVEL</key>
        <string>info</string>
        <key>AGENT_CONFIG</key>
        <string>${INSTALL_DIR}/config/agent.yml</string>
    </dict>
</dict>
</plist>
EOF

    chown root:wheel "$PLIST_PATH"
    chmod 644 "$PLIST_PATH"

    echo "launchd 服务文件已创建: $PLIST_PATH"
}

enable_service() {
    echo -e "${GREEN}[4/4] 启用服务...${NC}"

    launchctl load "/Library/LaunchDaemons/${PLIST_FILE}"
    launchctl start "${AGENT_NAME}"

    echo -e "${GREEN}安装完成!${NC}"
}

do_install() {
    check_root
    install_binary
    install_config
    create_service
    enable_service
    show_status
}

do_start() {
    check_root
    launchctl start "${AGENT_NAME}"
    echo -e "${GREEN}服务已启动${NC}"
}

do_stop() {
    check_root
    launchctl stop "${AGENT_NAME}"
    echo -e "${GREEN}服务已停止${NC}"
}

do_restart() {
    check_root
    launchctl stop "${AGENT_NAME}"
    sleep 1
    launchctl start "${AGENT_NAME}"
    echo -e "${GREEN}服务已重启${NC}"
}

show_status() {
    echo ""
    echo -e "${BLUE}服务状态:${NC}"
    launchctl list | grep "${AGENT_NAME}" || echo "服务未运行"
}

show_logs() {
    echo -e "${BLUE}查看日志:${NC}"
    if [ -f "${INSTALL_DIR}/agent.log" ]; then
        tail -f "${INSTALL_DIR}/agent.log"
    else
        echo "日志文件不存在"
    fi
}

show_help() {
    echo ""
    echo -e "${BLUE}===== 帮助信息 =====${NC}"
    echo ""
    echo "服务名称: ${AGENT_NAME}"
    echo "安装目录: ${INSTALL_DIR}"
    echo "二进制文件: ${INSTALL_DIR}/${AGENT_NAME}"
    echo "配置文件: ${INSTALL_DIR}/config/agent.yml"
    echo ""
    echo "常用命令:"
    echo "  启动服务: sudo launchctl start ${AGENT_NAME}"
    echo "  停止服务: sudo launchctl stop ${AGENT_NAME}"
    echo "  重启服务: sudo launchctl stop ${AGENT_NAME} && sudo launchctl start ${AGENT_NAME}"
    echo "  查看状态: sudo launchctl list | grep ${AGENT_NAME}"
    echo ""
    echo "配置文件说明:"
    echo "  - agent_id: 代理唯一标识"
    echo "  - server_url: OpenClaw 服务器地址"
    echo "  - auth.token: 认证 Token"
    echo "  - capabilities: 命令开关配置"
    echo ""
}

uninstall_service() {
    echo -e "${YELLOW}卸载服务...${NC}"

    launchctl stop "${AGENT_NAME}" 2>/dev/null || true
    launchctl unload "/Library/LaunchDaemons/${PLIST_FILE}" 2>/dev/null || true
    rm -f "/Library/LaunchDaemons/${PLIST_FILE}"

    echo "服务已卸载"
    echo "安装目录 ${INSTALL_DIR} 保留未删除"
}

pause() {
    echo ""
    echo -n "按回车键继续..."
    read -r dummy
}

main() {
    if [ -n "${1:-}" ]; then
        case "${1}" in
            uninstall)
                check_root
                uninstall_service
                ;;
            start)
                do_start
                ;;
            stop)
                do_stop
                ;;
            restart)
                do_restart
                ;;
            status)
                show_status
                ;;
            logs)
                show_logs
                ;;
            help|--help|-h)
                show_help
                ;;
            install)
                do_install
                ;;
            *)
                echo "未知参数: $1"
                echo "用法: $0 [install|start|stop|restart|status|logs|uninstall|help]"
                exit 1
                ;;
        esac
        return
    fi

    while true; do
        show_banner
        show_menu

        read -r choice
        echo ""

        case "$choice" in
            1)
                do_install
                pause
                ;;
            2)
                do_start
                pause
                ;;
            3)
                do_stop
                pause
                ;;
            4)
                do_restart
                pause
                ;;
            5)
                show_status
                pause
                ;;
            6)
                check_root
                echo -n "确定要卸载服务吗? (y/N): "
                read -r confirm
                if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                    uninstall_service
                fi
                pause
                ;;
            7)
                show_logs
                ;;
            8)
                show_help
                pause
                ;;
            0)
                echo -e "${GREEN}退出${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}无效选项，请重新选择${NC}"
                sleep 1
                ;;
        esac
    done
}

main "$@"

#!/bin/bash
set -e

# 启动系统服务
service nginx start
service fail2ban start
service docker start

# 等待 Docker 就绪
sleep 5

# 输出成功信息
echo "================================================"
echo "✅ Lightweight Secure Docker Host 启动成功！"
echo "📦 Docker Version: $(docker --version)"
echo "🧩 Docker Compose: $(docker compose version)"
echo "🌐 Nginx: 运行中"
echo "🛡️  Fail2ban: 运行中"
echo "================================================"

# 保持容器运行
tail -f /dev/null

FROM debian:12-slim

LABEL author="Yulong Deng"
LABEL description="Lightweight Secure Docker Container Hosting Service for Vultr Marketplace"
LABEL vendor="Vultr Marketplace"

ENV DEBIAN_FRONTEND=noninteractive

# 安装基础依赖与Docker官方源
RUN apt update && apt install -y \
    curl \
    gnupg \
    lsb-release \
    ca-certificates \
    apt-transport-https \
    software-properties-common \
    nginx \
    fail2ban \
    git

# 添加Docker官方GPG密钥与源
RUN install -m 0755 -d /etc/apt/keyrings
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
RUN echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

# 安装Docker核心组件
RUN apt update && apt install -y \
    docker-ce \
    docker-ce-cli \
    containerd.io \
    docker-compose-plugin

# 清理缓存减少镜像体积
RUN apt clean && rm -rf /var/lib/apt/lists/*

# 工作目录与启动脚本
WORKDIR /app
COPY start.sh /app/start.sh
RUN chmod +x /app/start.sh

# 暴露端口
EXPOSE 80 443

# 启动命令
CMD ["/app/start.sh"]

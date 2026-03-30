# Stage 1: Builder - 使用 Rust 镜像构建
FROM rust:1.94 AS builder

# 安装 Node.js 和 npm
RUN apt-get update && apt-get install -y \
    curl \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# 复制源代码
COPY Cargo.toml ./
COPY crates/ ./crates/
COPY server/ ./server/
COPY src/ ./src/
COPY frontend/ ./frontend/
COPY model/ ./model/

# 构建 release 版本（启用编译缓存）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release --bin nihility-server

# Stage 2: Runtime - 使用最小镜像
FROM debian:trixie-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libnss3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libasound2 \
    libpango-1.0-0 \
    libpangocairo-1.0-0 \
    fonts-liberation \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制和模型
COPY --from=builder /build/target/release/nihility-server /app/
COPY --from=builder /build/model/ /app/model/

# 创建 config 目录（运行时生成）
RUN mkdir -p /app/config

# 创建非 root 用户并设置权限
RUN groupadd --gid 1000 nihility && \
    useradd --uid 1000 --gid nihility --shell /bin/sh --create-home --home-dir /home/nihility nihility && \
    chown -R nihility:nihility /app

VOLUME ["/app/config"]

USER nihility
ENTRYPOINT ["/app/nihility-server"]

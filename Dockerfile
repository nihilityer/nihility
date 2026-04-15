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

# 构建 release 版本（启用编译缓存）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/target \
    cargo build --release --bin nihility-server

# Stage 2: Runtime - 使用最小镜像
FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /build/target/release/nihility-server /app/

ENv NIHILITY_IN_CONTAINER true

ENTRYPOINT ["/app/nihility-server"]

# syntax=docker/dockerfile:1
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    NVM_DIR=/usr/local/nvm \
    PATH=/usr/local/cargo/bin:$NVM_DIR/versions/node/v20/bin:/usr/local/nvm:$PATH

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    git \
    # Tauri 2 system dependencies
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    # Python 3.12 (ships with 24.04)
    python3 \
    python3-pip \
    python3-venv \
    # Misc
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable \
    && rustup component add rustfmt clippy

# Install Node.js 20 via nvm
RUN mkdir -p "$NVM_DIR" \
    && curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash \
    && . "$NVM_DIR/nvm.sh" \
    && nvm install 20 \
    && nvm alias default 20 \
    && nvm use default

# Install Tauri CLI
RUN cargo install tauri-cli --version "^2"

# Install pre-commit
RUN pip3 install pre-commit

# Set working directory
WORKDIR /workspace

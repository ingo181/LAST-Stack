FROM rust:1.87-bookworm

# System dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    git \
    cmake \
    libsasl2-dev \
    && rm -rf /var/lib/apt/lists/*

# WASM target für Leptos/trunk
RUN rustup target add wasm32-unknown-unknown

# Cargo tools
RUN cargo install trunk cargo-watch

# Non-root user (entspricht dem Host-User)
ARG UID=1000
ARG GID=1000
RUN groupadd -g ${GID} dev && \
    useradd -u ${UID} -g dev -m -s /bin/bash dev

USER dev
WORKDIR /workspace

# Cargo cache vorwärmen (wird via Volume gemountet)
ENV CARGO_HOME=/home/dev/.cargo
ENV PATH="/home/dev/.cargo/bin:${PATH}"

CMD ["sleep", "infinity"]

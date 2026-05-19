# 🦀 LAST Stack

> **L**eptos · **A**xum · **S**urrealDB · **T**auri — A full-stack Rust example application

[![Rust](https://img.shields.io/badge/Rust-1.78+-orange?logo=rust)](https://www.rust-lang.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.6-red)](https://leptos.dev)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue)](https://github.com/tokio-rs/axum)
[![SurrealDB](https://img.shields.io/badge/SurrealDB-1.x-teal)](https://surrealdb.com)
[![Tauri](https://img.shields.io/badge/Tauri-1.x-purple)](https://tauri.app)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

A production-inspired, end-to-end Rust stack demonstrating a **Todo application** built on a microservice architecture with async messaging via Apache Kafka in **KRaft mode** (no Zookeeper). Inspired by the [RSTY Stack](https://letsgetrusty.com) and the microservice patterns from the *Digital Frontiers Rust Book*.

---

## 📦 Stack Overview

| Letter | Technology | Role |
|--------|------------|------|
| **L** | [Leptos](https://leptos.dev) | Reactive WASM frontend (SSR-ready) |
| **A** | [Axum](https://github.com/tokio-rs/axum) | Async HTTP backend & API gateway |
| **S** | [SurrealDB](https://surrealdb.com) | Multi-model database (SQL + graph + realtime) |
| **T** | [Tauri](https://tauri.app) + [Thaw UI](https://thaw.rs) / [Floem](https://github.com/lapce/floem) | Desktop shell & native UI |

**Message bus:** Apache Kafka (KRaft — no Zookeeper)  
**Dev environment:** Dev Container (all-in-one)

---

## 🏗️ Architecture

```
Browser (Leptos WASM)          Desktop (Tauri + Floem/Thaw UI)
        │  REST / WebSocket               │
        └──────────────┬──────────────────┘
                       ▼
             ┌─────────────────┐
             │   API Gateway   │  JWT auth · CORS · TenantContext
             │   Axum :8080    │  X-Tenant-ID → tenant_id per request
             └────────┬────────┘
                      │ sync REST + event publish
          ┌───────────┼────────────┐
          ▼           ▼            ▼
    user-service  order-service  notify-service  …(add more)
     Axum :8081    Axum :8082    Axum :8083
          │           │            ▲
          └─────┬─────┘            │ consume
                ▼                  │
         ┌─────────────┐           │
         │    Kafka    │───────────┘
         │   (KRaft)   │  user.events · order.events
         │   :9092     │  notifications · dead.letter
         └─────────────┘
                │
     ┌──────────┴──────────────────┐
     ▼                             ▼
SurrealDB Primary            SurrealDB Replica
    :8000                        :8001
ns=last  db=tenant_{id}       (read traffic)
```

Each service is a standalone Rust binary in a shared Cargo workspace. Inter-service communication is **async via Kafka topics**. The API gateway handles synchronous REST calls and publishes domain events for all state changes.

---

## 🔐 Multi-tenancy

Tenant isolation is a first-class concern, built in from the start:

- Every JWT carries a `tenant_id` claim (and a reserved `locale` field for future i18n).
- The gateway middleware extracts `tenant_id` into a `TenantContext` Axum extension — no service needs to parse the JWT again.
- SurrealDB isolates tenants at the **database level**: `use ns "last" db "tenant_{id}"`. No extra discriminator columns are needed.
- All Kafka messages include `tenant_id` in the event envelope.

Adding a new tenant requires no code changes — only provisioning a new SurrealDB database and issuing a JWT with the correct claim.

---

## 🌍 Internationalisation (i18n)

Planned for a post-MVP increment. The `shared` crate already reserves a `locale` field in `TenantContext` and JWT claims, so no structural changes will be needed later. The frontend will use [Fluent](https://projectfluent.org) via the Leptos integration for runtime locale switching.

---

## 📐 Scalability

All services are **stateless by design**:

| Concern | Solution |
|---------|----------|
| Session state | JWT only — no server-side sessions |
| Horizontal scaling | Kafka consumer groups, multiple instances share partitions |
| Read scaling | SurrealDB replica for read traffic |
| Configuration | Environment variables and mounted secrets only |
| Liveness | `/health` endpoint on every service |
| Readiness | `/ready` endpoint — only healthy after DB + Kafka connected |
| Graceful shutdown | `SIGTERM` handler — in-flight requests complete before exit |
| Resource limits | CPU/memory defined in `docker-compose.yml` deploy blocks |

The compose file uses `deploy:` blocks compatible with `docker stack deploy` (Swarm). Kubernetes manifests are in `k8s/` — one `Deployment`, `Service`, `ConfigMap`, and `Secret` per microservice.

---

## 📁 Project Structure

```
last-stack/
├── Cargo.toml                   # Workspace root
├── Cargo.lock
├── docker-compose.yml           # Kafka (KRaft), SurrealDB, services
├── .devcontainer/
│   ├── devcontainer.json
│   └── setup.sh                 # Toolchain bootstrap (runs once)
├── infra/
│   ├── Dockerfile.service       # Dev image with cargo-watch
│   ├── kafka/                   # KRaft config
│   └── surreal/                 # Schema migrations
├── k8s/                         # Kubernetes manifests
│   ├── gateway/
│   ├── user-service/
│   ├── order-service/
│   ├── notify-service/
│   └── surrealdb/
├── crates/
│   ├── shared/                  # Common types, errors, traits, Kafka events
│   ├── gateway/                 # JWT middleware, CORS, reverse proxy
│   ├── user-service/            # Users, auth, SurrealDB, Kafka producer
│   ├── order-service/           # Orders, SurrealDB, Kafka producer
│   ├── notify-service/          # Kafka consumer, WebSocket push
│   ├── frontend/                # Leptos WASM app
│   └── desktop/                 # Tauri + Floem/Thaw UI
└── docs/
    └── architecture.md
```

---

## 🚀 Getting Started

### Prerequisites

- [Docker](https://www.docker.com) or [Podman](https://podman.io) with Compose
- [VS Code](https://code.visualstudio.com) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

### 1. Clone & open in Dev Container

```bash
git clone https://github.com/your-username/last-stack
cd last-stack
code .
# → Click "Reopen in Container" when prompted
```

The container automatically installs:
- Rust nightly + `wasm32-unknown-unknown` target
- `trunk` and `cargo-leptos` (Leptos build tools)
- `cargo-watch` for live reloading
- `wasm-bindgen-cli`
- SurrealDB CLI
- Kafka CLI tools

### 2. Start infrastructure

```bash
docker compose up -d
# Starts: Kafka (KRaft), SurrealDB primary + replica, Kafka UI
```

### 3. Start services

```bash
# Each in its own terminal, or use a multiplexer like tmux/zellij
cargo watch -x "run -p gateway"
cargo watch -x "run -p user-service"
cargo watch -x "run -p order-service"
cargo watch -x "run -p notify-service"

# Leptos frontend
cd crates/frontend && trunk serve
```

### 4. Open the app

| URL | What |
|-----|------|
| `http://localhost:3000` | Leptos frontend |
| `http://localhost:8080` | API gateway |
| `http://localhost:8090` | Kafka UI |
| `http://localhost:8000` | SurrealDB (Surrealist) |

---

## ⚙️ Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KAFKA_BROKERS` | `kafka:29092` | Kafka bootstrap servers |
| `KAFKA_CLUSTER_ID` | *(generated)* | KRaft cluster ID (set once, never change) |
| `SURREAL_URL` | `ws://surreal-primary:8000` | SurrealDB WebSocket URL |
| `SURREAL_USER` | `root` | SurrealDB username |
| `SURREAL_PASS` | `root` | SurrealDB password |
| `SURREAL_NS` | `last` | SurrealDB namespace |
| `JWT_SECRET` | *(required)* | HMAC secret for JWT signing |
| `GATEWAY_PORT` | `8080` | API gateway port |
| `USER_SERVICE_PORT` | `8081` | User service port |
| `ORDER_SERVICE_PORT` | `8082` | Order service port |
| `NOTIFY_SERVICE_PORT` | `8083` | Notify service port |

Copy `.env.example` to `.env` for local development. Never commit `.env` with real secrets.

---

## 🧪 Running Tests

```bash
# All workspace tests
cargo test --workspace

# A specific service
cargo test -p user-service

# With log output
cargo test --workspace -- --nocapture
```

---

## 🏭 Production Build

```bash
# Optimised release binaries
cargo build --release --workspace

# Frontend WASM bundle
cd crates/frontend && trunk build --release
```

### Docker Swarm

```bash
docker stack deploy -c docker-compose.yml last-stack
```

### Kubernetes

```bash
kubectl apply -f k8s/
```

---

## 📚 References & Inspiration

- [Let's Get Rusty – RSTY Stack](https://letsgetrusty.com)
- [Digital Frontiers – Microservices in Rust](https://digitalfrontiers.de)
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [SurrealDB Rust SDK](https://surrealdb.com/docs/sdk/rust)
- [Tauri Guides](https://tauri.app/v1/guides/)
- [Kafka KRaft documentation](https://kafka.apache.org/documentation/#kraft)

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.
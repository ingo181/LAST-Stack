# 🦀 LAST Stack

> **L**eptos · **A**xum · **S**urrealDB · **T**auri — A full-stack Rust example application

[![Rust](https://img.shields.io/badge/Rust-1.78+-orange?logo=rust)](https://www.rust-lang.org)
[![Leptos](https://img.shields.io/badge/Leptos-0.6-red)](https://leptos.dev)
[![Axum](https://img.shields.io/badge/Axum-0.7-blue)](https://github.com/tokio-rs/axum)
[![SurrealDB](https://img.shields.io/badge/SurrealDB-1.x-teal)](https://surrealdb.com)
[![Tauri](https://img.shields.io/badge/Tauri-1.x-purple)](https://tauri.app)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

A production-inspired, end-to-end Rust stack demonstrating a **Todo application** built on a microservice architecture with async messaging via Apache Kafka. Inspired by the [RSTY Stack](https://letsgetrusty.com) and the microservice patterns from the *Digital Frontiers Rust Book*.

---

## 📦 Stack Overview

| Letter | Technology | Role |
|--------|------------|------|
| **L** | [Leptos](https://leptos.dev) | Reactive WASM frontend (SSR-ready) |
| **A** | [Axum](https://github.com/tokio-rs/axum) | Async HTTP backend & API gateway |
| **S** | [SurrealDB](https://surrealdb.com) | Multi-model database (SQL + graph + realtime) |
| **T** | [Tauri](https://tauri.app) + [Thaw UI](https://thaw.rs) / [Floem](https://github.com/lapce/floem) | Desktop shell & native UI |

**Infrastructure:** Apache Kafka · Zookeeper · Dev Container (all-in-one)

---

## 🏗️ Architecture

```
Browser (Leptos WASM)
        │  REST / WebSocket
        ▼
  ┌─────────────┐        ┌─────────────────────────────┐
  │ API Gateway │──pub──▶│         Kafka Bus           │
  │  Axum :8080 │        │  todos.created              │
  └──────┬──────┘        │  todos.updated              │
         │ sync REST      │  todos.deleted              │
         ▼               │  audit.events               │
  ┌─────────────┐        │  notifications              │
  │ Todo Service│◀──sub──│                             │
  │  Axum :8081 │        │  [Zookeeper coordination]   │
  └──────┬──────┘        └──────────┬──────────────────┘
         │                          │
         ▼                 ┌────────┴────────┐
    SurrealDB               │                │
      :8000            ┌────▼────┐     ┌─────▼──────┐
                       │ Notify  │     │   Audit    │
                       │ Service │     │  Service   │
                       │ :8082   │     │  :8083     │
                       └────┬────┘     └─────┬──────┘
                            │ WebSocket       │
                            ▼                ▼
                         Browser          SurrealDB
```

Each service is a standalone Rust binary in a shared Cargo workspace. All inter-service communication is **async via Kafka topics**. The API gateway handles synchronous REST calls to the todo service and publishes events to Kafka for everything else.

---

## 📁 Project Structure

```
last-stack/
├── Cargo.toml                  # Workspace root
├── Cargo.lock
├── docker-compose.yml          # Kafka, Zookeeper, SurrealDB, dev container
├── Dockerfile.dev
├── .devcontainer/
│   └── devcontainer.json
│
├── shared/                     # Common types shared across all services
│   └── src/
│       ├── lib.rs
│       ├── models.rs           # Todo, User structs
│       ├── events.rs           # Kafka event types
│       └── errors.rs
│
├── services/
│   ├── api-gateway/            # Entry point, routing, auth, rate limiting
│   ├── todo-service/           # CRUD operations, SurrealDB, Kafka producer
│   ├── notification-service/   # Kafka consumer, WebSocket push
│   └── audit-service/         # Kafka consumer, event logging
│
├── frontend/                   # Leptos WASM app
│   └── src/
│       ├── app.rs
│       └── components/
│           ├── todo_list.rs
│           ├── todo_item.rs
│           └── add_form.rs
│
└── desktop/                    # Tauri + Thaw UI / Floem
    └── src/
        ├── main.rs
        └── ui.rs
```

---

## 🚀 Getting Started

### Prerequisites

- [Docker](https://www.docker.com) & [Docker Compose](https://docs.docker.com/compose/)
- [VS Code](https://code.visualstudio.com) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

### 1. Clone & open in Dev Container

```bash
git clone https://github.com/your-username/last-stack
cd last-stack
code .
# → Click "Reopen in Container" when prompted
```

The container automatically installs:
- Rust stable + `wasm32-unknown-unknown` target
- `trunk` (Leptos build tool)
- `cargo-watch` for live reloading
- Kafka, Zookeeper, SurrealDB as companion services

### 2. Start all services

Open separate terminals for each service (or use a process manager like `cargo-run-script`):

```bash
# API Gateway
cargo watch -x "run --bin api-gateway"

# Todo Service
cargo watch -x "run --bin todo-service"

# Notification Service
cargo watch -x "run --bin notification-service"

# Audit Service
cargo watch -x "run --bin audit-service"

# Leptos Frontend
cd frontend && trunk serve
```

### 3. Open the app

```
http://localhost:8080
```

---

## ⚙️ Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KAFKA_BROKERS` | `kafka:29092` | Kafka bootstrap servers |
| `SURREAL_URL` | `ws://surrealdb:8000` | SurrealDB WebSocket URL |
| `SURREAL_USER` | `root` | SurrealDB username |
| `SURREAL_PASS` | `root` | SurrealDB password |
| `GATEWAY_PORT` | `8080` | API Gateway port |
| `TODO_SERVICE_PORT` | `8081` | Todo Service port |
| `NOTIFY_PORT` | `8082` | Notification Service port |
| `AUDIT_PORT` | `8083` | Audit Service port |

---

## 🧪 Running Tests

```bash
# All workspace tests
cargo test --workspace

# A specific service
cargo test --package todo-service

# With output
cargo test --workspace -- --nocapture
```

---

## 🏭 Production Build

```bash
# Optimised release binaries (size + LTO)
cargo build --release --workspace

# Frontend WASM bundle
cd frontend && trunk build --release
```

Release profile uses `opt-level = "z"`, `lto = true`, and `strip = true` for minimal binary sizes.

---

## 📚 References & Inspiration

- [Let's Get Rusty – RSTY Stack](https://letsgetrusty.com)
- [Digital Frontiers – Microservices in Rust](https://digitalfrontiers.de)
- [WhiteSponge – How to Build A Full Stack Rust Dashboard App with Leptos, Actix Web & SurrealDB (Full Tutorial)](https://www.youtube.com/watch?v=) *(YouTube)*
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [SurrealDB Rust SDK](https://surrealdb.com/docs/sdk/rust)
- [Tauri Guides](https://tauri.app/v1/guides/)

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to open an issue or submit a pull request.

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Commit your changes: `git commit -m "feat: add my feature"`
4. Push and open a PR

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

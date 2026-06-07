# Developer Setup

This guide covers everything you need to get the LAST Stack running locally
for development.

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Podman](https://podman.io) | ≥ 5.0 | Container runtime |
| [podman-compose](https://github.com/containers/podman-compose) | ≥ 1.0 | Compose orchestration |
| Git | any | |

For **web frontend + backend** development, that's it. Everything else
(Rust, trunk, SurrealDB, Kafka) runs inside the Dev Container.

For **desktop app** development (Tauri), some tools must additionally be
installed on the **host** — see the [Desktop App Development](#desktop-app-development-tauri)
section below.

---

## Quick Start (Web + Backend)

### 1. Clone the repository

```bash
git clone https://github.com/ingo181/LAST-Stack.git
cd LAST-Stack
```

### 2. Start the infrastructure

```bash
podman-compose up -d
```

This starts:
- **SurrealDB** (primary + replica) on ports 8000 / 8001
- **Kafka** (KRaft mode, no Zookeeper) on port 9092
- **Kafka UI** on port 8090
- **Surrealist** (SurrealDB UI) on port 8080
- **Dev Container** (`last-dev`) exposing the backend on port 8081

### 3. Enter the Dev Container

```bash
podman exec -it last-dev bash
```

### 4. Start the backend

```bash
cargo run -p apqp-service
```

### 5. Start the frontend

Open a second terminal in the container:

```bash
podman exec -it last-dev bash
cd crates/frontend
trunk serve
```

### 6. Open the app

| URL | What |
|-----|------|
| http://localhost:3000 | Leptos frontend |
| http://localhost:8081/health | apqp-service health check |
| http://localhost:8080 | Surrealist (SurrealDB UI) |
| http://localhost:8090 | Kafka UI |

> **Note:** When developing the web frontend inside the container, trunk
> serves on port 3000. The container forwards this port. If you instead
> develop the **desktop app** (see below), trunk runs on the *host* and the
> container only provides the backend on 8081.

---

## Project Structure

```
LAST-Stack/
├── Cargo.toml              # Workspace root
├── docker-compose.yml      # Infrastructure + Dev Container
├── Dockerfile.dev          # Dev Container image
├── .env                    # UID/GID for the container (see below)
├── .devcontainer/
│   └── devcontainer.json   # Dev Container config
├── infra/
│   └── surreal/
│       └── schema.surql    # SurrealDB schema (import once)
└── crates/
    ├── shared/             # Domain types, Kafka events, errors
    ├── apqp-service/       # Axum REST API (port 8081)
    ├── gateway/            # API Gateway (planned)
    ├── notify-service/     # Kafka consumer + WebSocket (planned)
    ├── frontend/           # Leptos + Thaw UI WASM app (crate: task-mgmt)
    └── desktop/            # Tauri 2.x desktop shell
```

---

## User Mapping (`.env`)

The Dev Container runs rootless under Podman and maps the host user 1:1
into the container via `userns_mode: "keep-id"` (set in
`docker-compose.yml`). For this to work, the container must be built with
your host UID/GID. These are read from a `.env` file in the repository
root:

```env
UID=1000
GID=1000
```

Create it once after cloning, matching your host user:

```bash
printf 'UID=%s\nGID=%s\n' "$(id -u)" "$(id -g)" > .env
```

> **Why this matters:** Without `keep-id` and matching UIDs, files created
> on the host appear as `root` inside the container (and vice versa),
> leading to `Permission denied` errors on the shared `/workspace` mount.
> The `keep-id` setup makes host and container agree on UID 1000, so both
> can read and write the same files.

---

## Environment Variables

The Dev Container sets these automatically via `docker-compose.yml`.
For running a service on the host (outside the container), create a `.env`
or export them:

```env
SURREAL_URL=ws://localhost:8000
SURREAL_USER=root
SURREAL_PASS=root
SURREAL_NS=opcaq
APQP_PORT=8081
RUST_LOG=apqp_service=debug,tower_http=info
```

---

## Database Schema

The schema must be imported once into SurrealDB after first start:

```bash
# Using Surrealist UI at http://localhost:8080
# or via surreal CLI:
surreal import \
  --conn ws://localhost:8000 \
  --user root --pass root \
  --ns opcaq --db tenant_00000000000000000000000000000001 \
  infra/surreal/schema.surql
```

---

## Desktop App Development (Tauri)

The Tauri desktop app (`crates/desktop`) reuses the **same Leptos CSR
frontend** that runs in the browser. The UI is built once and runs both in
the browser and as a native desktop window — and is prepared for mobile
(Android/iOS) via the `crate-type` and `mobile_entry_point` setup in the
crate.

### Why this part runs on the host

Tauri renders through the operating system's native WebView (WebKitGTK on
Linux). A WebView needs a graphical display server and the GTK/WebKit system
libraries, which are not present in the headless Dev Container. Therefore the
desktop app is built and run **on the host**, while the backend continues to
run in the container.

The split is clean:
- **Container:** backend (`apqp-service` on port 8081), SurrealDB, Kafka
- **Host:** trunk (frontend dev server) + the Tauri window

Tauri starts trunk itself (configured in `tauri.conf.json` →
`beforeDevCommand`), and the frontend talks to the backend at
`http://localhost:8081` through trunk's `/api/` proxy (see
`crates/frontend/Trunk.toml`).

### Host prerequisites

You need a Rust toolchain on the host (not just in the container), the
`wasm32-unknown-unknown` target, the Tauri system libraries, and the
`trunk` + `tauri-cli` tools.

**1. Rust toolchain + wasm target** (all distros):

```bash
# If Rust is not yet installed, use rustup: https://rustup.rs
rustup target add wasm32-unknown-unknown
```

**2. System libraries for Tauri's WebView:**

*Arch / EndeavourOS:*
```bash
sudo pacman -S --needed webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg base-devel
```

*Debian / Ubuntu (incl. WSL):*
```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

*Fedora:*
```bash
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel
sudo dnf group install -y "C Development Tools and Libraries"
```

> For other distributions, see the official Tauri prerequisites:
> https://v2.tauri.app/start/prerequisites/

**3. CLI tools** (all distros):

```bash
# Prefer prebuilt binaries to avoid long compile times:
cargo install cargo-binstall      # if not already present
cargo binstall trunk tauri-cli

# Alternatively, build from source:
# cargo install trunk tauri-cli --locked
```

Verify:
```bash
cargo tauri --version   # tauri-cli 2.x
trunk --version         # trunk 0.21.x
```

### Running the desktop app

1. **Start the backend** in the container (if not already running):
   ```bash
   podman exec -it last-dev bash -c "cd /workspace && cargo run -p apqp-service"
   ```
   Leave this terminal open. Verify from the host:
   ```bash
   curl -s "http://localhost:8081/tasks?page=1&page_size=25" \
     -o /dev/null -w "Backend: HTTP %{http_code}\n"
   ```
   You should see `HTTP 200`.

2. **Start the desktop app** on the host:
   ```bash
   cd crates/desktop
   cargo tauri dev
   ```
   Tauri compiles the app (the first build takes a few minutes), starts
   trunk automatically, and opens a native window with the task list.

> **Note:** Do **not** run a separate `trunk serve` on the host before
> `cargo tauri dev` — Tauri starts trunk itself. A second instance would
> fail with `Address already in use` on port 3000.

### Building a release bundle

```bash
cd crates/desktop
cargo tauri build
```

This produces platform-specific installers/binaries under
`target/release/bundle/`. App icons are generated from `app-icon.png` via
`cargo tauri icon app-icon.png` (already committed under `icons/`).

---

## Editor Setup

The repository includes a `.devcontainer/devcontainer.json` for Dev
Container–capable editors (e.g. VS Code / VS Codium with the
*Dev Containers* extension).

1. Install the editor and its Dev Containers support
2. Open the project folder
3. Choose **"Reopen in Container"** when prompted

The container provides:
- `rust-analyzer` with the correct toolchain
- `even-better-toml` for Cargo.toml editing
- Format on save via `rustfmt`
- Clippy as the check command

> **Note (Podman):** Dev Containers may need the Docker socket path. Set
> `DOCKER_HOST=unix:///run/user/1000/podman/podman.sock` in your shell
> profile, or configure it in the editor's Dev Containers settings.

> **Note (desktop development):** rust-analyzer for the `desktop` crate
> works best when the editor runs on the **host**, since the crate is built
> on the host. A container-based editor still handles `shared`,
> `apqp-service`, and `frontend`.

---

## Useful Commands

```bash
# Check all crates compile
cargo check --workspace

# Run all tests
cargo test --workspace

# Check a specific crate
cargo check -p apqp-service
cargo check -p task-mgmt

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Rebuild Dev Container image
podman-compose build dev --no-cache
```

---

## Known Issues

- **SurrealDB 3.x:** Use `type::record()` instead of `type::thing()` in
  SurrealQL queries. The Rust SDK requires owned `String` values in
  `.bind()` calls (no `&str`).

- **Cargo registry permissions:** The Dev Container sets
  `CARGO_HOME=/usr/local/cargo`. If you see permission errors there, run:
  ```bash
  podman exec -it -u root last-dev chown -R dev:dev /usr/local/cargo
  ```

- **`target/` permissions after switching between host and container
  builds:** The `cargo-target` volume is owned by the container build user.
  If a host build complains about `target/`, the simplest fix is to use
  separate target directories — or run the occasional:
  ```bash
  podman exec -it -u root last-dev chown -R dev:dev /workspace/target
  ```

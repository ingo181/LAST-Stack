# LAST-Stack

This is the last rust stack you need.

The stack consists of the following components:
- **L**eptos
- **A**xum
- **S**urrealDB
- **T**auri
- Thaw

The microservice architecture with docker/podman:
- SurrealDB database instance (primary + read replica)
- Apache Kafka (KRaft mode — no Zookeeper required)

## Getting started

See **[docs/dev-setup.md](docs/dev-setup.md)** for the full developer setup
— prerequisites, starting the infrastructure, running the backend and
frontend, and building the desktop app. Contributions are welcome; please
read **[CONTRIBUTING.md](CONTRIBUTING.md)** first.

## Block diagram
  <img width="1568" height="367" alt="image" src="https://github.com/user-attachments/assets/f9fcac30-d06c-4e27-be3b-f35c55702a5f" />

## Data flow diagram
<img width="240" height="516" alt="Pasted image 20250606174812" src="https://github.com/user-attachments/assets/268875e8-bbc7-43f5-bd06-db99511935fd" />

The diagram shows the data flow from the request to data storage:
- The client sends requests via HTTP to the Axum router
- The authentication layer checks access rights and tokens
- The tenant context is extracted from the JWT and forwarded to all services
- SurrealDB processes the authorized requests in the tenant-specific database
- The data is stored in the selected storage system (e.g., SurrealKV for local development)

## Desktop app (Tauri)

The same Leptos frontend runs both in the browser and as a native desktop
app via Tauri 2.x — one UI codebase, multiple targets (Linux/Windows/macOS,
with mobile prepared). The frontend is built with Leptos in CSR mode so the
identical WASM bundle is served in the browser and embedded in the Tauri
window.

Because Tauri renders through the operating system's native WebView, the
desktop app is built and run on the host while the backend keeps running in
the container. See the **Desktop App Development** section in
[docs/dev-setup.md](docs/dev-setup.md) for host prerequisites (Arch /
Debian / Ubuntu / Fedora) and the run/build workflow.

## Multi-tenancy

Tenant isolation is built in from the start. Every JWT contains a `tenant_id` claim. The API gateway extracts it into a `TenantContext` extension that is forwarded to all downstream services. SurrealDB uses one database per tenant (`use ns "last" db "tenant_{id}"`), providing native isolation without extra columns in every table.

## Internationalisation (i18n)

Planned for a post-MVP increment. The `shared` crate already reserves a `locale` field in the JWT claims and `TenantContext` so no structural changes will be needed when i18n is implemented. The frontend will use [Fluent](https://projectfluent.org) via the Leptos integration.

## Scalability

All services are designed to be **stateless** — no local session state, no in-process caches that differ between instances. This makes horizontal scaling straightforward under both Docker Swarm and Kubernetes:

- Each service exposes a `/health` (liveness) and `/ready` (readiness) endpoint
- Kafka consumer groups allow multiple instances of a service to share topic partitions
- SurrealDB read traffic is distributed across replicas via the gateway
- All configuration is supplied via environment variables or mounted secrets (no config files baked into images)
- Services handle `SIGTERM` with a graceful shutdown — in-flight requests complete before the process exits

The `docker-compose.yml` includes `deploy` blocks compatible with `docker stack deploy` (Swarm). Kubernetes manifests are provided separately in `k8s/`.

## Docker-specific files

Each sample app remains standalone and can be developed, tested, and deployed independently. The extended architecture offers:
- Asynchronous communication between services via Kafka topics
- Scalable message processing through consumer groups
- Fault tolerance through message queuing and dead-letter topics
- Persistent message processing with configurable retention
- Central coordination handled natively by Kafka KRaft (no separate Zookeeper service)
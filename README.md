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

## Status

**MVP.** The implemented and working part is: the `shared` domain crate,
the `apqp-service` Axum REST API (task CRUD, `/health` + `/ready` probes,
tenant-aware handlers), the Leptos + Thaw frontend (`task-mgmt`), and the
Tauri desktop app — all running against SurrealDB. Kafka event types are
defined in `shared`.

Everything below marked _planned_ describes the intended target
architecture and is **not yet implemented**: the API gateway and
`notify-service` are empty placeholders, there is no JWT auth layer or
running Kafka producer/consumer yet, and i18n is only prepared (a reserved
`locale` field), not built. Contributions toward these are welcome.

## Getting started

See **[docs/dev-setup.md](docs/dev-setup.md)** for the full developer setup
— prerequisites, starting the infrastructure, running the backend and
frontend, and building the desktop app. Contributions are welcome; please
read **[CONTRIBUTING.md](CONTRIBUTING.md)** first.

## Block diagram
  <img width="1568" height="367" alt="image" src="https://github.com/user-attachments/assets/f9fcac30-d06c-4e27-be3b-f35c55702a5f" />

## Data flow diagram
<img width="240" height="516" alt="Pasted image 20250606174812" src="https://github.com/user-attachments/assets/268875e8-bbc7-43f5-bd06-db99511935fd" />

The diagram shows the _planned_ data flow from the request to data storage
(the auth layer and gateway are not yet implemented — see Status above):
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

## Multi-tenancy _(planned)_

Tenant isolation is a design goal from the start. The intended model: every JWT contains a `tenant_id` claim, the API gateway extracts it into a `TenantContext` extension forwarded to all downstream services, and SurrealDB uses one database per tenant (`use ns "last" db "tenant_{id}"`), providing native isolation without extra columns in every table. The `TenantContext` type and per-tenant database selection already exist in `apqp-service`; the JWT extraction and gateway are not yet built.

## Internationalisation (i18n) _(planned)_

Planned for a post-MVP increment. The `shared` crate already reserves a `locale` field in the JWT claims and `TenantContext` so no structural changes will be needed when i18n is implemented. The frontend will use [Fluent](https://projectfluent.org) via the Leptos integration.

## Scalability _(planned)_

All services are designed to be **stateless** — no local session state, no in-process caches that differ between instances. The intent is straightforward horizontal scaling under both Docker Swarm and Kubernetes:

- Each service exposes `/health` (liveness) and `/ready` (readiness) endpoints — _implemented in `apqp-service`_
- Kafka consumer groups allow multiple instances of a service to share topic partitions — _planned_
- SurrealDB read traffic is distributed across replicas via the gateway — _planned_
- All configuration is supplied via environment variables or mounted secrets (no config files baked into images)
- Services handle `SIGTERM` with a graceful shutdown — in-flight requests complete before the process exits

> Note: Docker Swarm `deploy` blocks and Kubernetes manifests (`k8s/`) are
> part of the target setup but are **not included yet**.

## Messaging architecture _(planned)_

Each sample app remains standalone and can be developed, tested, and deployed independently. The intended extended architecture offers:
- Asynchronous communication between services via Kafka topics
- Scalable message processing through consumer groups
- Fault tolerance through message queuing and dead-letter topics
- Persistent message processing with configurable retention
- Central coordination handled natively by Kafka KRaft (no separate Zookeeper service)

The Kafka event types are defined in the `shared` crate; the producing and
consuming services (e.g. `notify-service`) are not yet implemented.
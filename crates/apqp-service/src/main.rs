mod db;
mod handlers;
mod state;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::net::SocketAddr;
use surrealdb::{engine::remote::ws::{Client, Ws}, opt::auth::Root, Surreal};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let cfg = Config::from_env()?;

    info!("Connecting to SurrealDB at {}", cfg.surreal_url);

    let db = Surreal::new::<Ws>(cfg.surreal_url.trim_start_matches("ws://")).await?;
    db.signin(Root {
        username: cfg.surreal_user,
        password: cfg.surreal_pass,
    })
    .await?;
    db.use_ns(&cfg.surreal_ns).use_db("tenant_00000000000000000000000000000001").await?;
    info!("SurrealDB connected — ns={}", cfg.surreal_ns);

    let state = AppState::new(db);

    let app = Router::new()
        .route("/health", get(handlers::health::liveness))
        .route("/ready",  get(handlers::health::readiness))
        .nest("/tasks", task_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!("apqp-service listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn task_routes() -> Router<AppState> {
    Router::new()
        .route("/",            get(handlers::task::list))
        .route("/",            post(handlers::task::create))
        .route("/{id}",         get(handlers::task::get_by_id))
        .route("/{id}",         patch(handlers::task::update))
        .route("/{id}",         delete(handlers::task::soft_delete))
        .route("/{id}/status",  patch(handlers::task::transition_status))
        .route("/{id}/risk",    patch(handlers::task::update_risk))
        .route("/{id}/history", get(handlers::task::history))
}

struct Config {
    surreal_url:  String,
    surreal_user: String,
    surreal_pass: String,
    surreal_ns:   String,
    port:         u16,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            surreal_url:  env("SURREAL_URL",  "ws://localhost:8000")?,
            surreal_user: env("SURREAL_USER", "root")?,
            surreal_pass: env("SURREAL_PASS", "root")?,
            surreal_ns:   env("SURREAL_NS",   "opcaq")?,
            port:         env("APQP_PORT",    "8081")?.parse()?,
        })
    }
}

fn env(key: &str, default: &str) -> anyhow::Result<String> {
    Ok(std::env::var(key).unwrap_or_else(|_| default.to_owned()))
}

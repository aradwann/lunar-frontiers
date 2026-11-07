use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use log::error;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

use crate::models::BuildingReadModel;
use crate::projectors::BuildingProjector;

/// Shorter than the Docker healthcheck timeout, so a hung database gets a 503.
const HEALTH_DB_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct ApiState {
    pool: Arc<Pool<Postgres>>,
    projector: BuildingProjector,
}

pub fn router(pool: Arc<Pool<Postgres>>, projector: BuildingProjector) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/buildings", get(list_buildings))
        .route("/buildings/{id}", get(get_building))
        .with_state(ApiState { pool, projector })
}

async fn health(State(state): State<ApiState>) -> StatusCode {
    let ping = sqlx::query("SELECT 1").execute(state.pool.as_ref());

    match timeout(HEALTH_DB_TIMEOUT, ping).await {
        Ok(Ok(_)) => StatusCode::OK,
        Ok(Err(e)) => {
            error!("Health check failed: {}", e);
            StatusCode::SERVICE_UNAVAILABLE
        }
        Err(_) => {
            error!("Health check timed out");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

async fn list_buildings(State(state): State<ApiState>) -> Json<Vec<BuildingReadModel>> {
    Json(state.projector.list_buildings())
}

async fn get_building(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<BuildingReadModel>, StatusCode> {
    state
        .projector
        .get_building(id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

use dashmap::DashMap;
use log::{error, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::events::{
    BuildingCompleted, BuildingEvent, BuildingProgressed, BuildingSpawned, BuildingSpawnedV2,
};
use crate::message_broadcaster::MessageBroadcaster;
use crate::models::{BoxError, BuildingReadModel};

/// Building projector that maintains read model in both memory (DashMap) and database.
/// Subscribes to events via the message broadcaster — no other component should push to it.
///
/// Handles both legacy V1 flow (construction site events + building spawned) and
/// the new V2 flow (all events on the building stream).
#[derive(Clone)]
pub struct BuildingProjector {
    pool: Arc<Pool<Postgres>>,
    buildings: Arc<DashMap<Uuid, BuildingReadModel>>,
}

impl BuildingProjector {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self {
            pool,
            buildings: Arc::new(DashMap::new()),
        }
    }

    /// Start listening to events from the broadcaster.
    /// Spawns background tasks that update the read model on each event.
    pub fn start(&self, broadcaster: &MessageBroadcaster) {
        let mut building_rx = broadcaster.subscribe_building();

        let projector = self.clone();
        tokio::spawn(async move {
            loop {
                match building_rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = projector.handle_building_event(&event).await {
                            error!("BuildingProjector building event error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("BuildingProjector building channel error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    /// Handle V1 legacy BuildingSpawned (already-completed building from process manager)
    async fn handle_building_spawned_v1(&self, event: &BuildingSpawned) -> Result<(), BoxError> {
        info!(
            "Building spawned V1 (legacy, already complete): {} at ({}, {})",
            event.site_id, event.location.x, event.location.y
        );

        let building = BuildingReadModel {
            site_id: event.site_id,
            site_type: event.site_type.clone(),
            location: event.location.clone(),
            player_id: event.player_id,
            complete_percentage: 100.0,
            ready: true,
            progressed_ticks: None,
            required_ticks: None,
        };

        self.buildings.insert(event.site_id, building.clone());

        let location_json = serde_json::to_value(&building.location)?;
        sqlx::query!(
            r#"
            INSERT INTO buildings_read_model
                (site_id, site_type, location, player_id, complete_percentage, ready, progressed_ticks, required_ticks)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (site_id) DO UPDATE SET
                complete_percentage = EXCLUDED.complete_percentage,
                ready = EXCLUDED.ready,
                progressed_ticks = EXCLUDED.progressed_ticks,
                required_ticks = EXCLUDED.required_ticks
            "#,
            building.site_id,
            building.site_type.to_string(),
            location_json,
            building.player_id,
            building.complete_percentage,
            building.ready,
            building.progressed_ticks,
            building.required_ticks,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Handle V2 BuildingSpawnedV2 (new building under construction)
    async fn handle_building_spawned_v2(&self, event: &BuildingSpawnedV2) -> Result<(), BoxError> {
        info!(
            "Building spawned V2: {} at ({}, {}) - requires {} ticks",
            event.building_id, event.location.x, event.location.y, event.required_ticks
        );

        let building = BuildingReadModel {
            site_id: event.building_id,
            site_type: event.site_type.clone(),
            location: event.location.clone(),
            player_id: event.player_id,
            complete_percentage: 0.0,
            ready: false,
            progressed_ticks: Some(0),
            required_ticks: Some(event.required_ticks as i64),
        };

        self.buildings.insert(event.building_id, building.clone());

        let location_json = serde_json::to_value(&building.location)?;
        sqlx::query!(
            r#"
            INSERT INTO buildings_read_model
                (site_id, site_type, location, player_id, complete_percentage, ready, progressed_ticks, required_ticks)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (site_id) DO UPDATE SET
                site_type = EXCLUDED.site_type,
                location = EXCLUDED.location,
                player_id = EXCLUDED.player_id,
                complete_percentage = EXCLUDED.complete_percentage,
                ready = EXCLUDED.ready,
                progressed_ticks = EXCLUDED.progressed_ticks,
                required_ticks = EXCLUDED.required_ticks
            "#,
            building.site_id,
            building.site_type.to_string(),
            location_json,
            building.player_id,
            building.complete_percentage,
            building.ready,
            building.progressed_ticks,
            building.required_ticks,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Handle BuildingProgressed event
    async fn handle_building_progressed(&self, event: &BuildingProgressed) -> Result<(), BoxError> {
        let percentage = (event.progressed_ticks as f32 / event.required_ticks as f32) * 100.0;

        info!(
            "Building construction progressed: {} - {:.1}%",
            event.building_id, percentage
        );

        self.buildings
            .entry(event.building_id)
            .and_modify(|building| {
                building.complete_percentage = percentage;
                building.progressed_ticks = Some(event.progressed_ticks as i64);
                building.required_ticks = Some(event.required_ticks as i64);
            });

        sqlx::query!(
            r#"
            UPDATE buildings_read_model
            SET complete_percentage = $1,
                progressed_ticks = $2,
                required_ticks = $3
            WHERE site_id = $4
            "#,
            percentage,
            event.progressed_ticks as i64,
            event.required_ticks as i64,
            event.building_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Handle BuildingCompleted event
    async fn handle_building_completed(&self, event: &BuildingCompleted) -> Result<(), BoxError> {
        info!("Building construction completed: {}", event.building_id);

        self.buildings
            .entry(event.building_id)
            .and_modify(|building| {
                building.complete_percentage = 100.0;
                building.ready = true;
                building.progressed_ticks = None;
                building.required_ticks = None;
            });

        sqlx::query!(
            r#"
            UPDATE buildings_read_model
            SET complete_percentage = $1,
                ready = $2,
                progressed_ticks = NULL,
                required_ticks = NULL
            WHERE site_id = $3
            "#,
            100.0_f32,
            true,
            event.building_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Main event handler - dispatches to specific handlers based on event type
    async fn handle_building_event(&self, event: &BuildingEvent) -> Result<(), BoxError> {
        match event {
            BuildingEvent::BuildingSpawned(evt) => self.handle_building_spawned_v1(evt).await,
            BuildingEvent::BuildingSpawnedV2(evt) => self.handle_building_spawned_v2(evt).await,
            BuildingEvent::BuildingProgressed(evt) => self.handle_building_progressed(evt).await,
            BuildingEvent::BuildingCompleted(evt) => self.handle_building_completed(evt).await,
        }
    }
}

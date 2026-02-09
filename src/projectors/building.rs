use dashmap::DashMap;
use log::{error, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::events::{
    BuildingEvent, BuildingSpawned, ConstructionProgressed, ConstructionSiteEvent, SiteSpawned,
};
use crate::message_broadcaster::MessageBroadcaster;
use crate::models::{BoxError, BuildingReadModel};

/// Building projector that maintains read model in both memory (DashMap) and database.
/// Subscribes to events via the message broadcaster — no other component should push to it.
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
        let mut construction_rx = broadcaster.subscribe_construction();
        let mut building_rx = broadcaster.subscribe_building();

        let projector = self.clone();
        tokio::spawn(async move {
            loop {
                match construction_rx.recv().await {
                    Ok(event) => {
                        if let Err(e) = projector.handle_construction_event(&event).await {
                            error!("BuildingProjector construction event error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("BuildingProjector construction channel error: {}", e);
                        break;
                    }
                }
            }
        });

        let projector = self.clone();
        tokio::spawn(async move {
            loop {
                match building_rx.recv().await {
                    Ok(event) => {
                        let BuildingEvent::BuildingSpawned(evt) = &event;
                        if let Err(e) = projector.handle_building_spawned(evt).await {
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

    /// Handle SiteSpawned event
    async fn handle_site_spawned(&self, event: &SiteSpawned) -> Result<(), BoxError> {
        info!(
            "Site spawned: {} at ({}, {})",
            event.site_id, event.location.x, event.location.y
        );

        let building = BuildingReadModel {
            site_id: event.site_id,
            site_type: event.site_type.clone(),
            location: event.location.clone(),
            player_id: event.player_id,
            complete_percentage: 0.0,
            ready: false,
            progressed_ticks: Some(0),
            required_ticks: Some(event.remaining_ticks as i64),
        };

        // Update in-memory cache
        self.buildings.insert(event.site_id, building.clone());

        // Update database
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

    /// Handle ConstructionProgressed event
    async fn handle_construction_progressed(
        &self,
        event: &ConstructionProgressed,
    ) -> Result<(), BoxError> {
        let percentage = (event.progressed_ticks as f32 / event.required_ticks as f32) * 100.0;

        info!(
            "Construction progressed: {} - {:.1}%",
            event.site_id, percentage
        );

        // Update in-memory cache
        self.buildings.entry(event.site_id).and_modify(|building| {
            building.complete_percentage = percentage;
            building.progressed_ticks = Some(event.progressed_ticks as i64);
            building.required_ticks = Some(event.required_ticks as i64);
        });

        // Update database
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
            event.site_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Handle BuildingSpawned event (construction completed)
    async fn handle_building_spawned(&self, event: &BuildingSpawned) -> Result<(), BoxError> {
        info!(
            "Building spawned (construction complete): {} at ({}, {})",
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

        // Update in-memory cache
        self.buildings.insert(event.site_id, building.clone());

        // Update database
        sqlx::query!(
            r#"
            UPDATE buildings_read_model
            SET complete_percentage = $1,
                ready = $2,
                progressed_ticks = NULL,
                required_ticks = NULL
            WHERE site_id = $3
            "#,
            100.0,
            true,
            event.site_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Main event handler - dispatches to specific handlers
    async fn handle_construction_event(
        &self,
        event: &ConstructionSiteEvent,
    ) -> Result<(), BoxError> {
        match event {
            ConstructionSiteEvent::SiteSpawned(evt) => self.handle_site_spawned(evt).await,
            ConstructionSiteEvent::ConstructionProgressed(evt) => {
                self.handle_construction_progressed(evt).await
            }
            ConstructionSiteEvent::ConstructionCompleted(_) => {
                // Handled via building events channel when BuildingSpawned arrives
                Ok(())
            }
        }
    }
}

use sqlx::{Pool, Postgres};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use crate::aggregates::{Building, ConstructionSite, Gameloop};
use crate::events::{BuildingEvent, ConstructionSiteEvent, GameloopEvent};

mod events;
mod template;

use events::{BuildingEventTypes, ConstructionEventTypes, GameloopEventTypes};
pub use events::{BuildingEvents, ConstructionSiteEvents, GameloopEvents};

#[derive(Error, Debug)]
pub enum EventStoreError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(Uuid),
}

/// Event store for Gameloop aggregates
#[derive(Clone)]
pub struct GameloopEventStore {
    pool: Arc<Pool<Postgres>>,
}

impl GameloopEventStore {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn store_event(
        &self,
        game_id: Uuid,
        event: GameloopEvent,
        event_id: Uuid,
        version: u64,
    ) -> Result<(), EventStoreError> {
        let serialized: GameloopEvents = event.into();
        let event_type = serialized.event_type();
        let payload = serde_json::to_value(&serialized)?;

        sqlx::query!(
            r#"
            INSERT INTO gameloop_events (id, game_id, event_type, version, payload)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            event_id,
            game_id,
            event_type as GameloopEventTypes,
            version as i64,
            payload
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_events(&self, game_id: Uuid) -> Result<Vec<GameloopEvent>, EventStoreError> {
        template::get_gameloop_events(&self.pool, game_id).await
    }

    pub async fn get_aggregate(&self, game_id: Uuid) -> Result<Option<Gameloop>, EventStoreError> {
        let events = self.get_events(game_id).await?;
        if events.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Gameloop::hydrate(game_id, events)))
        }
    }
}

/// Event store for ConstructionSite aggregates
#[derive(Clone)]
pub struct ConstructionSiteEventStore {
    pool: Arc<Pool<Postgres>>,
}

impl ConstructionSiteEventStore {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn store_event(
        &self,
        site_id: Uuid,
        event: ConstructionSiteEvent,
        event_id: Uuid,
        version: u64,
    ) -> Result<(), EventStoreError> {
        let serialized: ConstructionSiteEvents = event.into();
        let event_type = serialized.event_type();
        let payload = serde_json::to_value(&serialized)?;

        sqlx::query!(
            r#"
            INSERT INTO construction_site_events (id, site_id, event_type, version, payload)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            event_id,
            site_id,
            event_type as ConstructionEventTypes,
            version as i64,
            payload
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_events(
        &self,
        site_id: Uuid,
    ) -> Result<Vec<ConstructionSiteEvent>, EventStoreError> {
        template::get_construction_site_events(&self.pool, site_id).await
    }

    pub async fn get_aggregate(
        &self,
        site_id: Uuid,
    ) -> Result<Option<ConstructionSite>, EventStoreError> {
        let events = self.get_events(site_id).await?;
        Ok(ConstructionSite::hydrate(site_id, events))
    }

    /// Get all site IDs that have been spawned but not yet completed
    pub async fn get_active_site_ids(&self) -> Result<Vec<Uuid>, EventStoreError> {
        template::get_active_construction_site_ids(&self.pool).await
    }
}

/// Event store for Building aggregates
#[derive(Clone)]
pub struct BuildingEventStore {
    pool: Arc<Pool<Postgres>>,
}

impl BuildingEventStore {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn store_event(
        &self,
        site_id: Uuid,
        event: BuildingEvent,
        event_id: Uuid,
        version: u64,
    ) -> Result<(), EventStoreError> {
        let serialized: BuildingEvents = event.into();
        let event_type = serialized.event_type();
        let payload = serde_json::to_value(&serialized)?;

        sqlx::query!(
            r#"
            INSERT INTO building_events (id, site_id, event_type, version, payload)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            event_id,
            site_id,
            event_type as BuildingEventTypes,
            version as i64,
            payload
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_events(&self, site_id: Uuid) -> Result<Vec<BuildingEvent>, EventStoreError> {
        template::get_building_events(&self.pool, site_id).await
    }

    pub async fn get_aggregate(&self, site_id: Uuid) -> Result<Option<Building>, EventStoreError> {
        let events = self.get_events(site_id).await?;
        Ok(Building::hydrate(events))
    }
}

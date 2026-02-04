use log::info;
use uuid::Uuid;

use crate::commands::SpawnBuilding;
use crate::event_store::{BuildingEventStore, EventStoreError};
use crate::events::{BuildingEvent, ConstructionCompleted, ConstructionSiteEvent};
use crate::models::BoxError;
use crate::projectors::BuildingProjector;

/// Process manager that handles construction completion workflow
/// When construction completes, spawns a building
pub struct ConstructionProcessManager {
    building_store: BuildingEventStore,
    building_projector: BuildingProjector,
}

impl ConstructionProcessManager {
    pub fn new(building_store: BuildingEventStore, building_projector: BuildingProjector) -> Self {
        Self {
            building_store,
            building_projector,
        }
    }

    /// Handle ConstructionCompleted event and spawn building
    pub async fn handle_construction_completed(
        &self,
        event: &ConstructionCompleted,
    ) -> Result<(), BoxError> {
        info!(
            "Construction completed for site {}, spawning building",
            event.site_id
        );

        let cmd = SpawnBuilding {
            site_id: event.site_id,
            player_id: event.player_id,
            site_type: event.site_type.clone(),
            location: event.location.clone(),
            tick: event.tick,
        };

        // Get the aggregate (or create new one)
        let _aggregate = self.building_store.get_aggregate(event.site_id).await?;

        // Handle command to generate event
        let building_event = crate::aggregates::Building::handle_spawn(cmd)?;

        // Store the event
        let event_id = Uuid::now_v7();
        self.building_store
            .store_event(event.site_id, building_event.clone(), event_id, 1)
            .await?;

        // Update projector
        if let BuildingEvent::BuildingSpawned(evt) = building_event {
            self.building_projector
                .handle_building_spawned(&evt)
                .await?;
        }

        Ok(())
    }

    /// Main event handler for construction site events
    pub async fn handle_event(&self, event: &ConstructionSiteEvent) -> Result<(), BoxError> {
        match event {
            ConstructionSiteEvent::ConstructionCompleted(evt) => {
                self.handle_construction_completed(evt).await
            }
            _ => Ok(()), // Only interested in completed events
        }
    }
}

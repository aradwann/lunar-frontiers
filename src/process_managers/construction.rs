use log::info;
use uuid::Uuid;

use crate::commands::SpawnBuilding;
use crate::event_store::BuildingEventStore;
use crate::events::{ConstructionCompleted, ConstructionSiteEvent};
use crate::models::BoxError;

/// Process manager that handles construction completion workflow
/// When construction completes, spawns a building
pub struct ConstructionProcessManager {
    building_store: BuildingEventStore,
}

impl ConstructionProcessManager {
    pub fn new(building_store: BuildingEventStore) -> Self {
        Self { building_store }
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

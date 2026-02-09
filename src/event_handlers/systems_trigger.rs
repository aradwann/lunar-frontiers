use log::{error, info};
use uuid::Uuid;

use crate::commands::AdvanceConstruction;
use crate::event_store::ConstructionSiteEventStore;
use crate::events::GameloopAdvanced;
use crate::models::BoxError;
use crate::projectors::BuildingProjector;

/// Event handler that triggers systems on each game tick
/// Similar to ECS pattern - advances all active systems when gameloop advances
#[derive(Clone)]
pub struct SystemsTrigger {
    construction_store: ConstructionSiteEventStore,
    building_projector: BuildingProjector,
}

impl SystemsTrigger {
    pub fn new(
        construction_store: ConstructionSiteEventStore,
        building_projector: BuildingProjector,
    ) -> Self {
        Self {
            construction_store,
            building_projector,
        }
    }

    /// Advance construction for all active sites
    async fn advance_construction(&self, tick: u64) -> Result<(), BoxError> {
        let active_sites = self.building_projector.active_sites();

        info!(
            "Advancing construction for {} active sites at tick {}",
            active_sites.len(),
            tick
        );

        for site_id in active_sites {
            if let Err(e) = self.advance_single_site(site_id, tick).await {
                error!("Failed to advance construction for site {}: {}", site_id, e);
                // Continue with other sites even if one fails
            }
        }

        Ok(())
    }

    async fn advance_single_site(&self, site_id: Uuid, tick: u64) -> Result<(), BoxError> {
        // Get aggregate
        let aggregate = self
            .construction_store
            .get_aggregate(site_id)
            .await?
            .ok_or_else(|| format!("Construction site {} not found", site_id))?;

        // Get current version (number of events)
        let current_version = self.construction_store.get_events(site_id).await?.len() as u64;

        // Handle command
        let cmd = AdvanceConstruction {
            site_id,
            tick,
            advance_ticks: 1,
        };

        let events = aggregate.handle_advance(cmd)?;

        // Store all events generated
        for (i, event) in events.iter().enumerate() {
            let event_id = Uuid::now_v7();
            let version = current_version + i as u64 + 1;

            self.construction_store
                .store_event(site_id, event.clone(), event_id, version)
                .await?;
        }

        Ok(())
    }

    /// Handle GameloopAdvanced event
    pub async fn handle_gameloop_advanced(&self, event: &GameloopAdvanced) -> Result<(), BoxError> {
        info!("Gameloop advanced to tick {}", event.tick);

        // Advance construction system every tick
        self.advance_construction(event.tick).await?;

        // Other systems could be triggered here with different frequencies:
        // - Combat every 2 ticks: if event.tick % 2 == 0
        // - Resource generation every 3 ticks: if event.tick % 3 == 0
        // - Movement every tick

        Ok(())
    }
}

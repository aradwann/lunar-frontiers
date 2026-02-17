use log::{error, info};
use uuid::Uuid;

use crate::commands::AdvanceBuilding;
use crate::event_store::BuildingEventStore;
use crate::events::GameloopAdvanced;
use crate::message_broadcaster::MessageBroadcaster;
use crate::models::BoxError;

/// Event handler that triggers systems on each game tick
/// Similar to ECS pattern - advances all active systems when gameloop advances
#[derive(Clone)]
pub struct SystemsTrigger {
    building_store: BuildingEventStore,
    broadcaster: MessageBroadcaster,
}

impl SystemsTrigger {
    pub fn new(building_store: BuildingEventStore, broadcaster: MessageBroadcaster) -> Self {
        Self {
            building_store,
            broadcaster,
        }
    }

    /// Advance construction for all active buildings (V2 flow)
    async fn advance_buildings(&self, tick: u64) -> Result<(), BoxError> {
        let active_buildings = self.building_store.get_active_building_ids().await?;

        info!(
            "Advancing construction for {} active buildings at tick {}",
            active_buildings.len(),
            tick
        );

        for building_id in active_buildings {
            if let Err(e) = self.advance_single_building(building_id, tick).await {
                error!("Failed to advance building {}: {}", building_id, e);
            }
        }

        Ok(())
    }

    async fn advance_single_building(&self, building_id: Uuid, tick: u64) -> Result<(), BoxError> {
        let aggregate = self
            .building_store
            .get_aggregate(building_id)
            .await?
            .ok_or_else(|| format!("Building {} not found", building_id))?;

        let current_version = self.building_store.get_events(building_id).await?.len() as u64;

        let cmd = AdvanceBuilding {
            building_id,
            tick,
            advance_ticks: 1,
        };

        let events = aggregate.handle_advance(cmd)?;

        for (i, event) in events.iter().enumerate() {
            let event_id = Uuid::now_v7();
            let version = current_version + i as u64 + 1;

            self.building_store
                .store_event(building_id, event.clone(), event_id, version)
                .await?;

            // Broadcast each event so projectors see it
            if let Err(e) = self.broadcaster.broadcast_building(event.clone()) {
                error!("Failed to broadcast building event: {}", e);
            }
        }

        Ok(())
    }

    /// Handle GameloopAdvanced event
    pub async fn handle_gameloop_advanced(&self, event: &GameloopAdvanced) -> Result<(), BoxError> {
        info!("Gameloop advanced to tick {}", event.tick);

        // Advance building construction system every tick
        self.advance_buildings(event.tick).await?;

        // Other systems could be triggered here with different frequencies:
        // - Combat every 2 ticks: if event.tick % 2 == 0
        // - Resource generation every 3 ticks: if event.tick % 3 == 0
        // - Movement every tick

        Ok(())
    }
}

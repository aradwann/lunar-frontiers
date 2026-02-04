use uuid::Uuid;

use crate::commands::SpawnBuilding;
use crate::events::{BuildingEvent, BuildingSpawned};
use crate::models::{BoxError, Location, SiteType};

#[derive(Debug, Clone)]
pub struct Building {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
}

impl Building {
    /// Handle SpawnBuilding command
    pub fn handle_spawn(cmd: SpawnBuilding) -> Result<BuildingEvent, BoxError> {
        Ok(BuildingEvent::BuildingSpawned(BuildingSpawned {
            site_id: cmd.site_id,
            site_type: cmd.site_type,
            location: cmd.location,
            player_id: cmd.player_id,
        }))
    }

    /// Apply an event to update state
    pub fn apply(&mut self, event: &BuildingEvent) {
        match event {
            BuildingEvent::BuildingSpawned(evt) => {
                self.site_id = evt.site_id;
                self.site_type = evt.site_type.clone();
                self.location = evt.location.clone();
                self.player_id = evt.player_id;
            }
        }
    }

    /// Hydrate aggregate from event history
    pub fn hydrate(events: Vec<BuildingEvent>) -> Option<Self> {
        if events.is_empty() {
            return None;
        }

        let first_event = &events[0];
        let mut building = match first_event {
            BuildingEvent::BuildingSpawned(evt) => Building {
                site_id: evt.site_id,
                site_type: evt.site_type.clone(),
                location: evt.location.clone(),
                player_id: evt.player_id,
            },
        };

        for event in events.iter().skip(1) {
            building.apply(event);
        }

        Some(building)
    }
}

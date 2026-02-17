use uuid::Uuid;

use crate::commands::{AdvanceBuilding, SpawnBuilding, SpawnBuildingV2};
use crate::events::{
    BuildingCompleted, BuildingEvent, BuildingProgressed, BuildingSpawned, BuildingSpawnedV2,
};
use crate::models::{BoxError, Location, SiteType};

#[derive(Debug, Clone)]
pub struct Building {
    pub building_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
    /// None for V1 legacy buildings (already completed), Some for V2 buildings under construction
    pub required_ticks: Option<u64>,
    pub completed_ticks: u64,
    pub completed: bool,
}

impl Building {
    /// Legacy V1: Handle SpawnBuilding command (building already completed via process manager)
    pub fn handle_spawn(cmd: SpawnBuilding) -> Result<BuildingEvent, BoxError> {
        Ok(BuildingEvent::BuildingSpawned(BuildingSpawned {
            site_id: cmd.site_id,
            site_type: cmd.site_type,
            location: cmd.location,
            player_id: cmd.player_id,
        }))
    }

    /// V2: Handle SpawnBuildingV2 command (building with construction tracking)
    pub fn handle_spawn_v2(cmd: SpawnBuildingV2) -> Result<BuildingEvent, BoxError> {
        Ok(BuildingEvent::BuildingSpawnedV2(BuildingSpawnedV2 {
            building_id: cmd.building_id,
            site_type: cmd.site_type,
            location: cmd.location,
            player_id: cmd.player_id,
            tick: cmd.tick,
            required_ticks: cmd.required_ticks,
        }))
    }

    /// V2: Handle AdvanceBuilding command - returns progress + optional completion events
    pub fn handle_advance(&self, cmd: AdvanceBuilding) -> Result<Vec<BuildingEvent>, BoxError> {
        if self.completed {
            return Ok(vec![]);
        }

        let required = self
            .required_ticks
            .ok_or("Cannot advance a V1 legacy building")?;

        let mut events = vec![];

        let new_completed = self.completed_ticks + cmd.advance_ticks;

        events.push(BuildingEvent::BuildingProgressed(BuildingProgressed {
            building_id: self.building_id,
            progressed_ticks: new_completed,
            required_ticks: required,
            tick: cmd.tick,
        }));

        if new_completed >= required {
            events.push(BuildingEvent::BuildingCompleted(BuildingCompleted {
                building_id: self.building_id,
                tick: cmd.tick,
            }));
        }

        Ok(events)
    }

    /// Apply an event to update state
    pub fn apply(&mut self, event: &BuildingEvent) {
        match event {
            BuildingEvent::BuildingSpawned(evt) => {
                self.building_id = evt.site_id;
                self.site_type = evt.site_type.clone();
                self.location = evt.location.clone();
                self.player_id = evt.player_id;
                self.required_ticks = None;
                self.completed_ticks = 0;
                self.completed = true; // V1 buildings are already completed
            }
            BuildingEvent::BuildingSpawnedV2(evt) => {
                self.building_id = evt.building_id;
                self.site_type = evt.site_type.clone();
                self.location = evt.location.clone();
                self.player_id = evt.player_id;
                self.required_ticks = Some(evt.required_ticks);
                self.completed_ticks = 0;
                self.completed = false;
            }
            BuildingEvent::BuildingProgressed(evt) => {
                self.completed_ticks = evt.progressed_ticks;
            }
            BuildingEvent::BuildingCompleted(_) => {
                self.completed = true;
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
                building_id: evt.site_id,
                site_type: evt.site_type.clone(),
                location: evt.location.clone(),
                player_id: evt.player_id,
                required_ticks: None,
                completed_ticks: 0,
                completed: true,
            },
            BuildingEvent::BuildingSpawnedV2(evt) => Building {
                building_id: evt.building_id,
                site_type: evt.site_type.clone(),
                location: evt.location.clone(),
                player_id: evt.player_id,
                required_ticks: Some(evt.required_ticks),
                completed_ticks: 0,
                completed: false,
            },
            _ => return None, // Invalid event stream
        };

        for event in events.iter().skip(1) {
            building.apply(event);
        }

        Some(building)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_spawn_v2() {
        let building_id = Uuid::now_v7();
        let player_id = Uuid::now_v7();

        let cmd = SpawnBuildingV2 {
            building_id,
            player_id,
            site_type: SiteType::PowerPlant,
            location: Location { x: 10, y: 10 },
            required_ticks: 5,
            tick: 0,
        };

        let event = Building::handle_spawn_v2(cmd).unwrap();
        match event {
            BuildingEvent::BuildingSpawnedV2(evt) => {
                assert_eq!(evt.building_id, building_id);
                assert_eq!(evt.required_ticks, 5);
            }
            _ => panic!("Expected BuildingSpawnedV2 event"),
        }
    }

    #[test]
    fn test_building_construction_completes() {
        let building_id = Uuid::now_v7();
        let player_id = Uuid::now_v7();

        let events = vec![BuildingEvent::BuildingSpawnedV2(BuildingSpawnedV2 {
            building_id,
            site_type: SiteType::Mine,
            location: Location { x: 0, y: 0 },
            player_id,
            tick: 0,
            required_ticks: 3,
        })];

        let mut building = Building::hydrate(events).unwrap();
        assert!(!building.completed);
        assert_eq!(building.required_ticks, Some(3));

        // Advance 3 ticks - should complete
        let cmd = AdvanceBuilding {
            building_id,
            tick: 3,
            advance_ticks: 3,
        };

        let result_events = building.handle_advance(cmd).unwrap();
        assert_eq!(result_events.len(), 2); // Progress + Completed

        for event in &result_events {
            building.apply(event);
        }

        assert!(building.completed);
        assert_eq!(building.completed_ticks, 3);
    }

    #[test]
    fn test_building_v1_legacy_is_completed() {
        let events = vec![BuildingEvent::BuildingSpawned(BuildingSpawned {
            site_id: Uuid::now_v7(),
            site_type: SiteType::Habitat,
            location: Location { x: 5, y: 5 },
            player_id: Uuid::now_v7(),
        })];

        let building = Building::hydrate(events).unwrap();
        assert!(building.completed);
        assert_eq!(building.required_ticks, None);
    }
}

use uuid::Uuid;

use crate::commands::{AdvanceConstruction, SpawnSite};
use crate::events::{
    ConstructionCompleted, ConstructionProgressed, ConstructionSiteEvent, SiteSpawned,
};
use crate::models::{BoxError, Location, SiteType};

#[derive(Debug, Clone)]
pub struct ConstructionSite {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub required_ticks: u64,
    pub completed_ticks: u64,
    pub created_tick: u64,
    pub player_id: Uuid,
    pub completed: bool,
    pub completed_tick: Option<u64>,
}

impl ConstructionSite {
    pub fn new(site_id: Uuid) -> Option<Self> {
        // Return None for uninitialized aggregate
        None
    }

    /// Handle SpawnSite command
    pub fn handle_spawn(cmd: SpawnSite) -> Result<ConstructionSiteEvent, BoxError> {
        Ok(ConstructionSiteEvent::SiteSpawned(SiteSpawned {
            site_id: cmd.site_id,
            site_type: cmd.site_type,
            location: cmd.location,
            tick: cmd.tick,
            remaining_ticks: cmd.completion_ticks,
            player_id: cmd.player_id,
        }))
    }

    /// Handle AdvanceConstruction command - returns multiple events
    pub fn handle_advance(
        &self,
        cmd: AdvanceConstruction,
    ) -> Result<Vec<ConstructionSiteEvent>, BoxError> {
        if self.completed {
            return Ok(vec![]); // No-op if already completed
        }

        let mut events = vec![];

        // Progress event
        events.push(ConstructionSiteEvent::ConstructionProgressed(
            ConstructionProgressed {
                site_id: self.site_id,
                site_type: self.site_type.clone(),
                location: self.location.clone(),
                progressed_ticks: cmd.advance_ticks,
                required_ticks: self.required_ticks,
                tick: cmd.tick,
            },
        ));

        // Check if construction is complete after this progress
        let new_completed_ticks = self.completed_ticks + cmd.advance_ticks;
        if new_completed_ticks >= self.required_ticks {
            events.push(ConstructionSiteEvent::ConstructionCompleted(
                ConstructionCompleted {
                    site_id: self.site_id,
                    site_type: self.site_type.clone(),
                    location: self.location.clone(),
                    tick: cmd.tick,
                    player_id: self.player_id,
                },
            ));
        }

        Ok(events)
    }

    /// Apply an event to update state
    pub fn apply(&mut self, event: &ConstructionSiteEvent) {
        match event {
            ConstructionSiteEvent::SiteSpawned(evt) => {
                self.site_id = evt.site_id;
                self.site_type = evt.site_type.clone();
                self.location = evt.location.clone();
                self.player_id = evt.player_id;
                self.created_tick = evt.tick;
                self.required_ticks = evt.remaining_ticks;
                self.completed_ticks = 0;
                self.completed = false;
            }
            ConstructionSiteEvent::ConstructionProgressed(evt) => {
                self.completed_ticks += evt.progressed_ticks;
            }
            ConstructionSiteEvent::ConstructionCompleted(evt) => {
                self.completed_tick = Some(evt.tick);
                self.completed = true;
            }
        }
    }

    /// Hydrate aggregate from event history
    pub fn hydrate(site_id: Uuid, events: Vec<ConstructionSiteEvent>) -> Option<Self> {
        if events.is_empty() {
            return None;
        }

        // Initialize with first event
        let first_event = &events[0];
        let mut site = match first_event {
            ConstructionSiteEvent::SiteSpawned(evt) => ConstructionSite {
                site_id: evt.site_id,
                site_type: evt.site_type.clone(),
                location: evt.location.clone(),
                required_ticks: evt.remaining_ticks,
                completed_ticks: 0,
                created_tick: evt.tick,
                player_id: evt.player_id,
                completed: false,
                completed_tick: None,
            },
            _ => return None, // Invalid event stream
        };

        // Apply remaining events
        for event in events.iter().skip(1) {
            site.apply(event);
        }

        Some(site)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_site_spawn() {
        let cmd = SpawnSite {
            site_id: Uuid::now_v7(),
            player_id: Uuid::now_v7(),
            site_type: SiteType::PowerPlant,
            completion_ticks: 10,
            location: Location { x: 5, y: 10 },
            tick: 1,
        };

        let event = ConstructionSite::handle_spawn(cmd.clone()).unwrap();

        match event {
            ConstructionSiteEvent::SiteSpawned(evt) => {
                assert_eq!(evt.site_id, cmd.site_id);
                assert_eq!(evt.remaining_ticks, 10);
            }
            _ => panic!("Expected SiteSpawned event"),
        }
    }

    #[test]
    fn test_construction_completes() {
        let site_id = Uuid::now_v7();
        let player_id = Uuid::now_v7();

        let events = vec![ConstructionSiteEvent::SiteSpawned(SiteSpawned {
            site_id,
            site_type: SiteType::Mine,
            location: Location { x: 0, y: 0 },
            tick: 1,
            remaining_ticks: 5,
            player_id,
        })];

        let mut site = ConstructionSite::hydrate(site_id, events).unwrap();

        // Advance construction by 5 ticks - should complete
        let cmd = AdvanceConstruction {
            site_id,
            tick: 6,
            advance_ticks: 5,
        };

        let result_events = site.handle_advance(cmd).unwrap();
        assert_eq!(result_events.len(), 2); // Progress + Completed

        // Apply events
        for event in result_events {
            site.apply(&event);
        }

        assert!(site.completed);
        assert_eq!(site.completed_tick, Some(6));
    }
}

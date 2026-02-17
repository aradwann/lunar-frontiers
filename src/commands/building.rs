use uuid::Uuid;

use crate::models::{Location, SiteType};

/// Legacy: spawn a completed building (used by construction process manager)
#[derive(Debug, Clone)]
pub struct SpawnBuilding {
    pub site_id: Uuid,
    pub player_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub tick: u64,
}

/// V2: spawn a building that tracks its own construction progress
#[derive(Debug, Clone)]
pub struct SpawnBuildingV2 {
    pub building_id: Uuid,
    pub player_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub required_ticks: u64,
    pub tick: u64,
}

/// Advance building construction by a number of ticks
#[derive(Debug, Clone)]
pub struct AdvanceBuilding {
    pub building_id: Uuid,
    pub tick: u64,
    pub advance_ticks: u64,
}

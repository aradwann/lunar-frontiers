use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Location, SiteType};

/// V1: Legacy event - building spawned after construction completed (via process manager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSpawned {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
}

/// V2: Building spawned with construction tracking built-in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSpawnedV2 {
    pub building_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
    pub tick: u64,
    pub required_ticks: u64,
}

/// Building construction progressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingProgressed {
    pub building_id: Uuid,
    pub progressed_ticks: u64,
    pub required_ticks: u64,
    pub tick: u64,
}

/// Building construction completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingCompleted {
    pub building_id: Uuid,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildingEvent {
    /// Legacy: building created by process manager after construction site completed
    BuildingSpawned(BuildingSpawned),
    /// V2: building created with embedded construction lifecycle
    BuildingSpawnedV2(BuildingSpawnedV2),
    /// V2: construction progress on the building
    BuildingProgressed(BuildingProgressed),
    /// V2: building construction completed
    BuildingCompleted(BuildingCompleted),
}

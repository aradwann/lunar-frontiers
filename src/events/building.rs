use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Location, SiteType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSpawned {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildingEvent {
    BuildingSpawned(BuildingSpawned),
}

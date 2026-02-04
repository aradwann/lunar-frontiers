use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Location, SiteType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSpawned {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub tick: u64,
    pub remaining_ticks: u64,
    pub player_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionProgressed {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub progressed_ticks: u64,
    pub required_ticks: u64,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionCompleted {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub tick: u64,
    pub player_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstructionSiteEvent {
    SiteSpawned(SiteSpawned),
    ConstructionProgressed(ConstructionProgressed),
    ConstructionCompleted(ConstructionCompleted),
}

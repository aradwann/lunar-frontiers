use uuid::Uuid;

use crate::models::{Location, SiteType};

#[derive(Debug, Clone)]
pub struct SpawnSite {
    pub site_id: Uuid,
    pub player_id: Uuid,
    pub site_type: SiteType,
    pub completion_ticks: u64,
    pub location: Location,
    pub tick: u64,
}

#[derive(Debug, Clone)]
pub struct AdvanceConstruction {
    pub site_id: Uuid,
    pub tick: u64,
    pub advance_ticks: u64,
}

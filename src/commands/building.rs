use uuid::Uuid;

use crate::models::{Location, SiteType};

#[derive(Debug, Clone)]
pub struct SpawnBuilding {
    pub site_id: Uuid,
    pub player_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub tick: u64,
}

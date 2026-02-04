use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SiteType {
    PowerPlant,
    Mine,
    Habitat,
    Factory,
}

impl std::fmt::Display for SiteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SiteType::PowerPlant => write!(f, "power_plant"),
            SiteType::Mine => write!(f, "mine"),
            SiteType::Habitat => write!(f, "habitat"),
            SiteType::Factory => write!(f, "factory"),
        }
    }
}

impl std::str::FromStr for SiteType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "power_plant" => Ok(SiteType::PowerPlant),
            "mine" => Ok(SiteType::Mine),
            "habitat" => Ok(SiteType::Habitat),
            "factory" => Ok(SiteType::Factory),
            _ => Err(format!("Unknown site type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildingReadModel {
    pub site_id: Uuid,
    pub site_type: SiteType,
    pub location: Location,
    pub player_id: Uuid,
    pub complete_percentage: f32,
    pub ready: bool,
    pub progressed_ticks: Option<i64>,
    pub required_ticks: Option<i64>,
}

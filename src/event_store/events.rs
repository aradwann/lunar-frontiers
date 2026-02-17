use serde::{Deserialize, Serialize};

use crate::events::*;

// Gameloop serialization types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameloopAdvancedV1 {
    pub game_id: uuid::Uuid,
    pub tick: u64,
}

#[derive(Deserialize)]
pub enum GameloopEvents {
    GameloopAdvancedV1(GameloopAdvancedV1),
}

impl From<GameloopEvent> for GameloopEvents {
    fn from(event: GameloopEvent) -> Self {
        match event {
            GameloopEvent::Advanced(e) => Self::GameloopAdvancedV1(GameloopAdvancedV1 {
                game_id: e.game_id,
                tick: e.tick,
            }),
        }
    }
}

impl From<GameloopEvents> for GameloopEvent {
    fn from(events: GameloopEvents) -> Self {
        match events {
            GameloopEvents::GameloopAdvancedV1(e) => Self::Advanced(GameloopAdvanced {
                game_id: e.game_id,
                tick: e.tick,
            }),
        }
    }
}

impl Serialize for GameloopEvents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::GameloopAdvancedV1(e) => e.serialize(serializer),
        }
    }
}

impl GameloopEvents {
    pub fn event_type(&self) -> GameloopEventTypes {
        match self {
            Self::GameloopAdvancedV1(_) => GameloopEventTypes::GameloopAdvancedV1,
        }
    }
}

#[derive(sqlx::Type, Clone, Debug)]
#[sqlx(type_name = "gameloop_event_type", rename_all = "snake_case")]
pub enum GameloopEventTypes {
    GameloopAdvancedV1,
}

// Construction Site serialization types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SiteSpawnedV1 {
    pub site_id: uuid::Uuid,
    pub site_type: crate::models::SiteType,
    pub location: crate::models::Location,
    pub tick: u64,
    pub remaining_ticks: u64,
    pub player_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConstructionProgressedV1 {
    pub site_id: uuid::Uuid,
    pub site_type: crate::models::SiteType,
    pub location: crate::models::Location,
    pub progressed_ticks: u64,
    pub required_ticks: u64,
    pub tick: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConstructionCompletedV1 {
    pub site_id: uuid::Uuid,
    pub site_type: crate::models::SiteType,
    pub location: crate::models::Location,
    pub tick: u64,
    pub player_id: uuid::Uuid,
}

#[derive(Deserialize)]
pub enum ConstructionSiteEvents {
    SiteSpawnedV1(SiteSpawnedV1),
    ConstructionProgressedV1(ConstructionProgressedV1),
    ConstructionCompletedV1(ConstructionCompletedV1),
}

impl From<ConstructionSiteEvent> for ConstructionSiteEvents {
    fn from(event: ConstructionSiteEvent) -> Self {
        match event {
            ConstructionSiteEvent::SiteSpawned(e) => Self::SiteSpawnedV1(SiteSpawnedV1 {
                site_id: e.site_id,
                site_type: e.site_type,
                location: e.location,
                tick: e.tick,
                remaining_ticks: e.remaining_ticks,
                player_id: e.player_id,
            }),
            ConstructionSiteEvent::ConstructionProgressed(e) => {
                Self::ConstructionProgressedV1(ConstructionProgressedV1 {
                    site_id: e.site_id,
                    site_type: e.site_type,
                    location: e.location,
                    progressed_ticks: e.progressed_ticks,
                    required_ticks: e.required_ticks,
                    tick: e.tick,
                })
            }
            ConstructionSiteEvent::ConstructionCompleted(e) => {
                Self::ConstructionCompletedV1(ConstructionCompletedV1 {
                    site_id: e.site_id,
                    site_type: e.site_type,
                    location: e.location,
                    tick: e.tick,
                    player_id: e.player_id,
                })
            }
        }
    }
}

impl From<ConstructionSiteEvents> for ConstructionSiteEvent {
    fn from(events: ConstructionSiteEvents) -> Self {
        match events {
            ConstructionSiteEvents::SiteSpawnedV1(e) => Self::SiteSpawned(SiteSpawned {
                site_id: e.site_id,
                site_type: e.site_type,
                location: e.location,
                tick: e.tick,
                remaining_ticks: e.remaining_ticks,
                player_id: e.player_id,
            }),
            ConstructionSiteEvents::ConstructionProgressedV1(e) => {
                Self::ConstructionProgressed(ConstructionProgressed {
                    site_id: e.site_id,
                    site_type: e.site_type,
                    location: e.location,
                    progressed_ticks: e.progressed_ticks,
                    required_ticks: e.required_ticks,
                    tick: e.tick,
                })
            }
            ConstructionSiteEvents::ConstructionCompletedV1(e) => {
                Self::ConstructionCompleted(ConstructionCompleted {
                    site_id: e.site_id,
                    site_type: e.site_type,
                    location: e.location,
                    tick: e.tick,
                    player_id: e.player_id,
                })
            }
        }
    }
}

impl Serialize for ConstructionSiteEvents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::SiteSpawnedV1(e) => e.serialize(serializer),
            Self::ConstructionProgressedV1(e) => e.serialize(serializer),
            Self::ConstructionCompletedV1(e) => e.serialize(serializer),
        }
    }
}

impl ConstructionSiteEvents {
    pub fn event_type(&self) -> ConstructionEventTypes {
        match self {
            Self::SiteSpawnedV1(_) => ConstructionEventTypes::SiteSpawnedV1,
            Self::ConstructionProgressedV1(_) => ConstructionEventTypes::ConstructionProgressedV1,
            Self::ConstructionCompletedV1(_) => ConstructionEventTypes::ConstructionCompletedV1,
        }
    }
}

#[derive(sqlx::Type, Clone, Debug)]
#[sqlx(type_name = "construction_event_type", rename_all = "snake_case")]
pub enum ConstructionEventTypes {
    SiteSpawnedV1,
    ConstructionProgressedV1,
    ConstructionCompletedV1,
}

// Building serialization types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingSpawnedV1 {
    pub site_id: uuid::Uuid,
    pub site_type: crate::models::SiteType,
    pub location: crate::models::Location,
    pub player_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingSpawnedV2Payload {
    pub building_id: uuid::Uuid,
    pub site_type: crate::models::SiteType,
    pub location: crate::models::Location,
    pub player_id: uuid::Uuid,
    pub tick: u64,
    pub required_ticks: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingProgressedV1 {
    pub building_id: uuid::Uuid,
    pub progressed_ticks: u64,
    pub required_ticks: u64,
    pub tick: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingCompletedV1 {
    pub building_id: uuid::Uuid,
    pub tick: u64,
}

#[derive(Deserialize)]
pub enum BuildingEvents {
    BuildingSpawnedV1(BuildingSpawnedV1),
    BuildingSpawnedV2(BuildingSpawnedV2Payload),
    BuildingProgressedV1(BuildingProgressedV1),
    BuildingCompletedV1(BuildingCompletedV1),
}

impl From<BuildingEvent> for BuildingEvents {
    fn from(event: BuildingEvent) -> Self {
        match event {
            BuildingEvent::BuildingSpawned(e) => Self::BuildingSpawnedV1(BuildingSpawnedV1 {
                site_id: e.site_id,
                site_type: e.site_type,
                location: e.location,
                player_id: e.player_id,
            }),
            BuildingEvent::BuildingSpawnedV2(e) => {
                Self::BuildingSpawnedV2(BuildingSpawnedV2Payload {
                    building_id: e.building_id,
                    site_type: e.site_type,
                    location: e.location,
                    player_id: e.player_id,
                    tick: e.tick,
                    required_ticks: e.required_ticks,
                })
            }
            BuildingEvent::BuildingProgressed(e) => {
                Self::BuildingProgressedV1(BuildingProgressedV1 {
                    building_id: e.building_id,
                    progressed_ticks: e.progressed_ticks,
                    required_ticks: e.required_ticks,
                    tick: e.tick,
                })
            }
            BuildingEvent::BuildingCompleted(e) => Self::BuildingCompletedV1(BuildingCompletedV1 {
                building_id: e.building_id,
                tick: e.tick,
            }),
        }
    }
}

impl From<BuildingEvents> for BuildingEvent {
    fn from(events: BuildingEvents) -> Self {
        match events {
            BuildingEvents::BuildingSpawnedV1(e) => Self::BuildingSpawned(BuildingSpawned {
                site_id: e.site_id,
                site_type: e.site_type,
                location: e.location,
                player_id: e.player_id,
            }),
            BuildingEvents::BuildingSpawnedV2(e) => {
                Self::BuildingSpawnedV2(crate::events::BuildingSpawnedV2 {
                    building_id: e.building_id,
                    site_type: e.site_type,
                    location: e.location,
                    player_id: e.player_id,
                    tick: e.tick,
                    required_ticks: e.required_ticks,
                })
            }
            BuildingEvents::BuildingProgressedV1(e) => {
                Self::BuildingProgressed(crate::events::BuildingProgressed {
                    building_id: e.building_id,
                    progressed_ticks: e.progressed_ticks,
                    required_ticks: e.required_ticks,
                    tick: e.tick,
                })
            }
            BuildingEvents::BuildingCompletedV1(e) => {
                Self::BuildingCompleted(crate::events::BuildingCompleted {
                    building_id: e.building_id,
                    tick: e.tick,
                })
            }
        }
    }
}

impl Serialize for BuildingEvents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::BuildingSpawnedV1(e) => e.serialize(serializer),
            Self::BuildingSpawnedV2(e) => e.serialize(serializer),
            Self::BuildingProgressedV1(e) => e.serialize(serializer),
            Self::BuildingCompletedV1(e) => e.serialize(serializer),
        }
    }
}

impl BuildingEvents {
    pub fn event_type(&self) -> BuildingEventTypes {
        match self {
            Self::BuildingSpawnedV1(_) => BuildingEventTypes::BuildingSpawnedV1,
            Self::BuildingSpawnedV2(_) => BuildingEventTypes::BuildingSpawnedV2,
            Self::BuildingProgressedV1(_) => BuildingEventTypes::BuildingProgressedV1,
            Self::BuildingCompletedV1(_) => BuildingEventTypes::BuildingCompletedV1,
        }
    }
}

#[derive(sqlx::Type, Clone, Debug)]
#[sqlx(type_name = "building_event_type", rename_all = "snake_case")]
pub enum BuildingEventTypes {
    BuildingSpawnedV1,
    BuildingSpawnedV2,
    BuildingProgressedV1,
    BuildingCompletedV1,
}

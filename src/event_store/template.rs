use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::EventStoreError;
use super::events::*;
use crate::events::*;

pub async fn get_gameloop_events(
    pool: &Pool<Postgres>,
    game_id: Uuid,
) -> Result<Vec<GameloopEvent>, EventStoreError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, event_type as "event_type!: GameloopEventTypes", payload, version
        FROM gameloop_events
        WHERE game_id = $1
        ORDER BY version ASC
        "#,
        game_id
    )
    .fetch_all(pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let serialized = match row.event_type {
                GameloopEventTypes::GameloopAdvancedV1 => {
                    let inner: GameloopAdvancedV1 = serde_json::from_value(row.payload)?;
                    GameloopEvents::GameloopAdvancedV1(inner)
                }
            };
            Ok(serialized.into())
        })
        .collect::<Result<Vec<_>, EventStoreError>>()?;

    Ok(events)
}

pub async fn get_construction_site_events(
    pool: &Pool<Postgres>,
    site_id: Uuid,
) -> Result<Vec<ConstructionSiteEvent>, EventStoreError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, event_type as "event_type!: ConstructionEventTypes", payload, version
        FROM construction_site_events
        WHERE site_id = $1
        ORDER BY version ASC
        "#,
        site_id
    )
    .fetch_all(pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let serialized = match row.event_type {
                ConstructionEventTypes::SiteSpawnedV1 => {
                    let inner: SiteSpawnedV1 = serde_json::from_value(row.payload)?;
                    ConstructionSiteEvents::SiteSpawnedV1(inner)
                }
                ConstructionEventTypes::ConstructionProgressedV1 => {
                    let inner: ConstructionProgressedV1 = serde_json::from_value(row.payload)?;
                    ConstructionSiteEvents::ConstructionProgressedV1(inner)
                }
                ConstructionEventTypes::ConstructionCompletedV1 => {
                    let inner: ConstructionCompletedV1 = serde_json::from_value(row.payload)?;
                    ConstructionSiteEvents::ConstructionCompletedV1(inner)
                }
            };
            Ok(serialized.into())
        })
        .collect::<Result<Vec<_>, EventStoreError>>()?;

    Ok(events)
}

pub async fn get_active_construction_site_ids(
    pool: &Pool<Postgres>,
) -> Result<Vec<Uuid>, EventStoreError> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT site_id
        FROM construction_site_events
        WHERE event_type = 'site_spawned_v1'
          AND site_id NOT IN (
              SELECT site_id FROM construction_site_events
              WHERE event_type = 'construction_completed_v1'
          )
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| row.site_id).collect())
}

pub async fn get_building_events(
    pool: &Pool<Postgres>,
    site_id: Uuid,
) -> Result<Vec<BuildingEvent>, EventStoreError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, event_type as "event_type!: BuildingEventTypes", payload, version
        FROM building_events
        WHERE site_id = $1
        ORDER BY version ASC
        "#,
        site_id
    )
    .fetch_all(pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let serialized = match row.event_type {
                BuildingEventTypes::BuildingSpawnedV1 => {
                    let inner: BuildingSpawnedV1 = serde_json::from_value(row.payload)?;
                    BuildingEvents::BuildingSpawnedV1(inner)
                }
                BuildingEventTypes::BuildingSpawnedV2 => {
                    let inner: BuildingSpawnedV2Payload = serde_json::from_value(row.payload)?;
                    BuildingEvents::BuildingSpawnedV2(inner)
                }
                BuildingEventTypes::BuildingProgressedV1 => {
                    let inner: BuildingProgressedV1 = serde_json::from_value(row.payload)?;
                    BuildingEvents::BuildingProgressedV1(inner)
                }
                BuildingEventTypes::BuildingCompletedV1 => {
                    let inner: BuildingCompletedV1 = serde_json::from_value(row.payload)?;
                    BuildingEvents::BuildingCompletedV1(inner)
                }
            };
            Ok(serialized.into())
        })
        .collect::<Result<Vec<_>, EventStoreError>>()?;

    Ok(events)
}

pub async fn get_active_building_ids(pool: &Pool<Postgres>) -> Result<Vec<Uuid>, EventStoreError> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT site_id
        FROM building_events
        WHERE event_type = 'building_spawned_v2'
          AND site_id NOT IN (
              SELECT site_id FROM building_events
              WHERE event_type = 'building_completed_v1'
          )
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| row.site_id).collect())
}

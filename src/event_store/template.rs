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
            let serialized: GameloopEvents = serde_json::from_value(row.payload)?;
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
            let serialized: ConstructionSiteEvents = serde_json::from_value(row.payload)?;
            Ok(serialized.into())
        })
        .collect::<Result<Vec<_>, EventStoreError>>()?;

    Ok(events)
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
            let serialized: BuildingEvents = serde_json::from_value(row.payload)?;
            Ok(serialized.into())
        })
        .collect::<Result<Vec<_>, EventStoreError>>()?;

    Ok(events)
}

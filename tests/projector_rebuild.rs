use std::sync::Arc;
use std::time::Duration;

use lunar_frontiers::event_store::BuildingEventStore;
use lunar_frontiers::events::{
    BuildingCompleted, BuildingEvent, BuildingProgressed, BuildingSpawnedV2,
};
use lunar_frontiers::message_broadcaster::MessageBroadcaster;
use lunar_frontiers::models::{BuildingReadModel, Location, SiteType};
use lunar_frontiers::projectors::BuildingProjector;
use sqlx::PgPool;
use uuid::Uuid;

fn spawned(building_id: Uuid, required_ticks: u64) -> BuildingEvent {
    BuildingEvent::BuildingSpawnedV2(BuildingSpawnedV2 {
        building_id,
        site_type: SiteType::Mine,
        location: Location { x: 1, y: 2 },
        player_id: Uuid::now_v7(),
        tick: 0,
        required_ticks,
    })
}

fn progressed(building_id: Uuid, progressed_ticks: u64, required_ticks: u64) -> BuildingEvent {
    BuildingEvent::BuildingProgressed(BuildingProgressed {
        building_id,
        progressed_ticks,
        required_ticks,
        tick: progressed_ticks,
    })
}

fn completed(building_id: Uuid, tick: u64) -> BuildingEvent {
    BuildingEvent::BuildingCompleted(BuildingCompleted { building_id, tick })
}

async fn store_all(store: &BuildingEventStore, building_id: Uuid, events: &[BuildingEvent]) {
    for (i, event) in events.iter().enumerate() {
        store
            .store_event(building_id, event.clone(), Uuid::now_v7(), i as u64 + 1)
            .await
            .unwrap();
    }
}

async fn wait_for_building(
    projector: &BuildingProjector,
    building_id: Uuid,
    done: impl Fn(&BuildingReadModel) -> bool,
) -> BuildingReadModel {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match projector.get_building(building_id) {
                Some(building) if done(&building) => return building,
                _ => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        }
    })
    .await
    .expect("read model never reached the expected state")
}

/// Events stored but never broadcast, as after a crash between the two steps.
#[sqlx::test]
async fn rebuild_recovers_events_that_were_never_broadcast(pool: PgPool) {
    let pool = Arc::new(pool);
    let store = BuildingEventStore::new(pool.clone());
    let projector = BuildingProjector::new(pool.clone(), store.clone());

    let in_progress = Uuid::now_v7();
    store_all(
        &store,
        in_progress,
        &[spawned(in_progress, 4), progressed(in_progress, 1, 4)],
    )
    .await;

    let finished = Uuid::now_v7();
    store_all(
        &store,
        finished,
        &[
            spawned(finished, 1),
            progressed(finished, 1, 1),
            completed(finished, 1),
        ],
    )
    .await;

    projector.rebuild().await.unwrap();

    let building = projector.get_building(in_progress).unwrap();
    assert_eq!(building.complete_percentage, 25.0);
    assert_eq!(building.progressed_ticks, Some(1));
    assert!(!building.ready);

    let building = projector.get_building(finished).unwrap();
    assert_eq!(building.complete_percentage, 100.0);
    assert_eq!(building.progressed_ticks, None);
    assert!(building.ready);

    let (percentage, ready): (f32, bool) = sqlx::query_as(
        "SELECT complete_percentage, ready FROM buildings_read_model WHERE site_id = $1",
    )
    .bind(in_progress)
    .fetch_one(pool.as_ref())
    .await
    .unwrap();
    assert_eq!(percentage, 25.0);
    assert!(!ready);
}

/// A capacity of 1 forces the receiver to lag. The projector has to survive
/// that, recover the skipped events, and keep handling new ones.
#[sqlx::test]
async fn projector_recovers_from_lag_and_keeps_running(pool: PgPool) {
    let pool = Arc::new(pool);
    let store = BuildingEventStore::new(pool.clone());
    let projector = BuildingProjector::new(pool.clone(), store.clone());
    let broadcaster = MessageBroadcaster::new(1);
    projector.start(&broadcaster);

    let building_id = Uuid::now_v7();
    let events = [
        spawned(building_id, 2),
        progressed(building_id, 1, 2),
        progressed(building_id, 2, 2),
        completed(building_id, 2),
    ];
    store_all(&store, building_id, &events).await;

    // No await between sends, so the projector task can't drain the channel
    for event in events {
        broadcaster.broadcast_building(event).unwrap();
    }

    wait_for_building(&projector, building_id, |building| building.ready).await;

    let next_id = Uuid::now_v7();
    broadcaster.broadcast_building(spawned(next_id, 3)).unwrap();

    let building = wait_for_building(&projector, next_id, |_| true).await;
    assert_eq!(building.required_ticks, Some(3));
}

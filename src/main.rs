use log::{error, info, warn};
use lunar_frontiers::aggregates::*;
use lunar_frontiers::api;
use lunar_frontiers::commands::*;
use lunar_frontiers::event_handlers::SystemsTrigger;
use lunar_frontiers::event_store::*;
use lunar_frontiers::events::*;
use lunar_frontiers::message_broadcaster::MessageBroadcaster;
use lunar_frontiers::models::*;
use lunar_frontiers::projectors::BuildingProjector;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{Duration, interval};
use uuid::Uuid;

const HTTP_ADDR: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    env_logger::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/lunar_frontiers".to_string());

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let pool = Arc::new(pool);

    // Initialize event stores
    let gameloop_store = GameloopEventStore::new(pool.clone());
    let building_store = BuildingEventStore::new(pool.clone());

    // Initialize message broadcaster
    let broadcaster = MessageBroadcaster::new(1000);

    // Initialize projectors — they subscribe to events via the broadcaster
    let building_projector = BuildingProjector::new(pool.clone(), building_store.clone());
    building_projector.start(&broadcaster);

    // Catch up on anything that was stored but never projected
    building_projector.rebuild().await?;

    // Initialize event handlers (SystemsTrigger now broadcasts events directly)
    let systems_trigger = SystemsTrigger::new(building_store.clone(), broadcaster.clone());

    // Resume the latest game, or start a new one with its initial buildings
    let gameloop = match gameloop_store.get_latest_game_id().await? {
        Some(game_id) => {
            let gameloop = gameloop_store
                .get_aggregate(game_id)
                .await?
                .ok_or(EventStoreError::AggregateNotFound(game_id))?;
            info!("Resuming game {} at tick {}", game_id, gameloop.tick);
            gameloop
        }
        None => {
            let gameloop = Gameloop::new(Uuid::now_v7());
            info!("Starting game with ID: {}", gameloop.game_id);

            // V2 flow — buildings track their own construction
            info!("Spawning initial buildings...");
            spawn_initial_buildings(&building_store, &broadcaster).await?;
            gameloop
        }
    };

    // Serve the read model and health endpoint
    let listener = TcpListener::bind(HTTP_ADDR).await?;
    info!("HTTP server listening on {}", HTTP_ADDR);
    let router = api::router(pool.clone(), building_projector.clone());
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            error!("HTTP server error: {}", e);
        }
    });

    // Subscribe before the first tick so no gameloop event is missed
    let mut gameloop_rx = broadcaster.subscribe_gameloop();

    // Start gameloop ticker
    let gameloop_store_clone = gameloop_store.clone();
    let broadcaster_clone = broadcaster.clone();
    tokio::spawn(async move {
        if let Err(e) = run_gameloop(gameloop, gameloop_store_clone, broadcaster_clone).await {
            error!("Gameloop error: {}", e);
        }
    });

    // Trigger systems on each gameloop event
    let systems_trigger_clone = systems_trigger.clone();
    tokio::spawn(async move {
        loop {
            match gameloop_rx.recv().await {
                Ok(event) => {
                    let GameloopEvent::Advanced(evt) = event;
                    if let Err(e) = systems_trigger_clone.handle_gameloop_advanced(&evt).await {
                        error!("Systems trigger error: {}", e);
                    }
                }
                // Missed ticks are not replayed; the next tick carries on from here
                Err(RecvError::Lagged(skipped)) => {
                    warn!("Systems trigger lagged by {} ticks", skipped);
                }
                Err(RecvError::Closed) => {
                    info!("Gameloop channel closed, stopping systems trigger");
                    break;
                }
            }
        }
    });

    info!("Lunar Frontiers is running! Press Ctrl+C to exit.");

    // Keep the main task alive
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn spawn_initial_buildings(
    building_store: &BuildingEventStore,
    broadcaster: &MessageBroadcaster,
) -> Result<(), BoxError> {
    let player_id = Uuid::now_v7();

    let buildings: Vec<(SiteType, Location, u64)> = vec![
        (SiteType::PowerPlant, Location { x: 10, y: 10 }, 5),
        (SiteType::Mine, Location { x: 20, y: 15 }, 8),
        (SiteType::Habitat, Location { x: 5, y: 25 }, 10),
    ];

    for (site_type, location, required_ticks) in buildings {
        let building_id = Uuid::now_v7();

        let cmd = SpawnBuildingV2 {
            building_id,
            player_id,
            site_type: site_type.clone(),
            location: location.clone(),
            required_ticks,
            tick: 0,
        };

        let event = Building::handle_spawn_v2(cmd)?;

        building_store
            .store_event(building_id, event.clone(), Uuid::now_v7(), 1)
            .await?;

        // Broadcast the event — projector will pick it up
        if let Err(e) = broadcaster.broadcast_building(event) {
            error!("Failed to broadcast building event: {}", e);
        }

        info!(
            "Spawned {} at ({}, {}) - requires {} ticks",
            site_type, location.x, location.y, required_ticks
        );
    }

    Ok(())
}

async fn run_gameloop(
    mut gameloop: Gameloop,
    gameloop_store: GameloopEventStore,
    broadcaster: MessageBroadcaster,
) -> Result<(), BoxError> {
    let mut tick_interval = interval(Duration::from_secs(2));

    // This task is the only writer, so the aggregate stays in memory instead of
    // being replayed from the event store every tick.
    loop {
        tick_interval.tick().await;
        let tick = gameloop.tick + 1;

        let cmd = AdvanceGameloop {
            game_id: gameloop.game_id,
            tick,
        };

        let event = gameloop.handle_advance(cmd)?;

        // Store event
        gameloop_store
            .store_event(gameloop.game_id, event.clone(), Uuid::now_v7(), tick)
            .await?;

        gameloop.apply(&event);

        // Broadcast event
        if let Err(e) = broadcaster.broadcast_gameloop(event) {
            error!("Failed to broadcast gameloop event: {}", e);
        }

        info!("Tick {}", tick);
    }
}

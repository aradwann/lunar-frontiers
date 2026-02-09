use log::{error, info};
use lunar_frontiers::aggregates::*;
use lunar_frontiers::commands::*;
use lunar_frontiers::event_handlers::SystemsTrigger;
use lunar_frontiers::event_store::*;
use lunar_frontiers::events::*;
use lunar_frontiers::message_broadcaster::MessageBroadcaster;
use lunar_frontiers::models::*;
use lunar_frontiers::process_managers::ConstructionProcessManager;
use lunar_frontiers::projectors::BuildingProjector;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use uuid::Uuid;

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
    let construction_store = ConstructionSiteEventStore::new(pool.clone());
    let building_store = BuildingEventStore::new(pool.clone());

    // Initialize message broadcaster
    let broadcaster = MessageBroadcaster::new(1000);

    // Initialize projectors — they subscribe to events via the broadcaster
    let building_projector = BuildingProjector::new(pool.clone());
    building_projector.start(&broadcaster);

    // Initialize process managers
    let construction_pm = ConstructionProcessManager::new(building_store.clone());

    // Initialize event handlers
    let systems_trigger = SystemsTrigger::new(construction_store.clone());

    // Game ID
    let game_id = Uuid::now_v7();
    info!("Starting game with ID: {}", game_id);

    // Spawn some initial construction sites
    info!("Spawning initial construction sites...");
    spawn_initial_sites(&construction_store, &broadcaster, game_id).await?;

    // Start gameloop ticker
    let gameloop_store_clone = gameloop_store.clone();
    let broadcaster_clone = broadcaster.clone();
    tokio::spawn(async move {
        if let Err(e) = run_gameloop(game_id, gameloop_store_clone, broadcaster_clone).await {
            error!("Gameloop error: {}", e);
        }
    });

    // Subscribe to gameloop events and trigger systems
    let mut gameloop_rx = broadcaster.subscribe_gameloop();
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
                Err(e) => {
                    error!("Error receiving gameloop event: {}", e);
                    break;
                }
            }
        }
    });

    // Subscribe to construction events and handle with process manager
    let mut construction_rx = broadcaster.subscribe_construction();
    tokio::spawn(async move {
        loop {
            match construction_rx.recv().await {
                Ok(event) => {
                    if let Err(e) = construction_pm.handle_event(&event).await {
                        error!("Process manager error: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error receiving construction event: {}", e);
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

async fn spawn_initial_sites(
    construction_store: &ConstructionSiteEventStore,
    broadcaster: &MessageBroadcaster,
    _game_id: Uuid,
) -> Result<(), BoxError> {
    let player_id = Uuid::now_v7();

    let sites: Vec<(SiteType, Location, u64)> = vec![
        (SiteType::PowerPlant, Location { x: 10, y: 10 }, 5),
        (SiteType::Mine, Location { x: 20, y: 15 }, 8),
        (SiteType::Habitat, Location { x: 5, y: 25 }, 10),
    ];

    for (site_type, location, completion_ticks) in sites {
        let site_id = Uuid::now_v7();

        let cmd = SpawnSite {
            site_id,
            player_id,
            site_type: site_type.clone(),
            completion_ticks,
            location: location.clone(),
            tick: 0,
        };

        let event = ConstructionSite::handle_spawn(cmd)?;

        construction_store
            .store_event(site_id, event.clone(), Uuid::now_v7(), 1)
            .await?;

        // Broadcast the event — projector will pick it up
        if let Err(e) = broadcaster.broadcast_construction(event) {
            error!("Failed to broadcast construction event: {}", e);
        }

        info!(
            "Spawned {} at ({}, {}) - requires {} ticks",
            site_type, location.x, location.y, completion_ticks
        );
    }

    Ok(())
}

async fn run_gameloop(
    game_id: Uuid,
    gameloop_store: GameloopEventStore,
    broadcaster: MessageBroadcaster,
) -> Result<(), BoxError> {
    let mut tick_interval = interval(Duration::from_secs(2));
    let mut current_tick = 0u64;

    loop {
        tick_interval.tick().await;
        current_tick += 1;

        let cmd = AdvanceGameloop {
            game_id,
            tick: current_tick,
        };

        // Get or create gameloop aggregate
        let aggregate: Gameloop = gameloop_store
            .get_aggregate(game_id)
            .await?
            .unwrap_or_else(|| Gameloop::new(game_id));

        let event = aggregate.handle_advance(cmd)?;

        // Store event
        gameloop_store
            .store_event(game_id, event.clone(), Uuid::now_v7(), current_tick)
            .await?;

        // Broadcast event
        if let Err(e) = broadcaster.broadcast_gameloop(event) {
            error!("Failed to broadcast gameloop event: {}", e);
        }

        info!("Tick {}", current_tick);
    }
}

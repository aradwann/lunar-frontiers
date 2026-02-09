# Lunar Frontiers - Rust

A Rust implementation of an event-sourced game system, converted from the Elixir/Commanded "Lunar Frontiers" example.

## Overview

Lunar Frontiers is a space colony simulation game that demonstrates event sourcing patterns. The game features:

- **Gameloop**: Tick-based game progression
- **Construction Sites**: Buildings under construction with progress tracking
- **Buildings**: Completed structures
- **Process Managers**: Automated workflows (e.g., spawning buildings when construction completes)
- **Event Handlers**: System triggers that advance game state on each tick

## Architecture

### Event Sourcing Pattern

The system uses **event sourcing** where all state changes are captured as immutable events:

```
Command → Aggregate → Event → Event Store → Projections
                                    ↓
                              Read Models
```

### Components

#### Aggregates
- **Gameloop**: Tracks game tick progression
- **ConstructionSite**: Manages construction progress
- **Building**: Represents completed buildings

#### Event Stores
- `GameloopEventStore`: Stores gameloop events
- `ConstructionSiteEventStore`: Stores construction events  
- `BuildingEventStore`: Stores building events

Each event store provides:
- Event persistence with versioning
- Idempotency via unique event IDs
- Aggregate hydration from event history

#### Projectors (Read Models)
- **BuildingProjector**: Maintains queryable view of buildings and construction sites
- Updates both in-memory cache (DashMap) and PostgreSQL database
- Provides fast queries for active construction sites

#### Process Managers
- **ConstructionProcessManager**: Listens for `ConstructionCompleted` events and spawns buildings

#### Event Handlers
- **SystemsTrigger**: Advances all game systems when the gameloop ticks
- Similar to ECS (Entity Component System) pattern

## Database Schema

Three event tables (one per aggregate):
- `gameloop_events`
- `construction_site_events`
- `building_events`

One read model table:
- `buildings_read_model`

Each event table has:
- Unique event IDs for idempotency
- Version numbers for ordering
- JSON payloads for flexibility
- Unique constraints on (aggregate_id, version)

## Running the Project

### Prerequisites

- Docker and Docker Compose
- Rust 1.85+ (for local development)

### Docker Compose (recommended)

Build and start the database, run migrations, and launch the app in one command:

```bash
docker compose up -d
```

This starts three services:
- **db** — PostgreSQL 18 on port 5433
- **migrate** — applies SQL migrations automatically
- **app** — the game binary

View logs:
```bash
docker compose logs -f app
```

Stop everything:
```bash
docker compose down
```

### Local Development

1. **Start the database**:
```bash
docker compose up -d db
```

2. **Run migrations**:
```bash
cargo sqlx migrate run
```

3. **Run the game**:
```bash
cargo run
```

The `.env` file already contains `DATABASE_URL` and `RUST_LOG` so no manual exports are needed.

### What Happens

1. Game initializes with a new game ID
2. Three construction sites are spawned:
   - Power Plant (5 ticks to complete)
   - Mine (8 ticks to complete)
   - Habitat (10 ticks to complete)
3. Gameloop ticks every 2 seconds
4. On each tick:
   - All active construction sites advance by 1 tick
   - When construction reaches required ticks, it completes
   - Process manager spawns a building
   - Building appears in read model as "ready"

### Expected Output

```
[INFO] Starting game with ID: 01935f8a-1234-7890-abcd-ef0123456789
[INFO] Spawning initial construction sites...
[INFO] Spawned power_plant at (10, 10) - requires 5 ticks
[INFO] Spawned mine at (20, 15) - requires 8 ticks
[INFO] Spawned habitat at (5, 25) - requires 10 ticks
[INFO] Lunar Frontiers is running! Press Ctrl+C to exit.
[INFO] Tick 1
[INFO] Advancing construction for 3 active sites at tick 1
[INFO] Tick 2
[INFO] Advancing construction for 3 active sites at tick 2
[INFO] Tick 5
[INFO] Construction completed for site ..., spawning building
[INFO] Building spawned (construction complete): ...
```

## Design Patterns

### 1. **Event Sourcing**
- All state changes are events
- Events are immutable and append-only
- State reconstructed by replaying events

### 2. **CQRS (Command Query Responsibility Segregation)**
- Commands write to event store
- Queries read from projections/read models
- Separate models for writing and reading

### 3. **Process Manager (Saga)**
- Coordinates workflows across aggregates
- Listens to events and dispatches commands
- Example: Construction completion → Building spawn

### 4. **Projections**
- Transform event stream into queryable views
- Updated asynchronously from events
- Can be rebuilt from event history

### 5. **Entity Component System (ECS-like)**
- Systems (construction, movement, combat) advance on tick
- Different systems can have different frequencies
- Decoupled game logic

## Code Structure

```
src/
├── aggregates/          # Domain logic and state
│   ├── gameloop.rs
│   ├── construction_site.rs
│   └── building.rs
├── commands/            # Command definitions
├── events/              # Event definitions  
├── event_store/         # Event persistence
│   ├── mod.rs          # Event store implementations
│   ├── events.rs       # Serialization types
│   └── template.rs     # Query helpers
├── projectors/          # Read model updaters
│   └── building.rs
├── process_managers/    # Workflow coordinators
│   └── construction.rs
├── event_handlers/      # Event processors
│   └── systems_trigger.rs
├── message_broadcaster.rs  # Event distribution
├── models.rs            # Shared types
└── main.rs              # Application entry point
```

## Testing

Run unit tests (no database required):
```bash
SQLX_OFFLINE=true cargo test
```

Example test:
```rust
#[test]
fn test_construction_completes() {
    let site = spawn_site(completion_ticks: 5);
    advance_construction(site, ticks: 5);
    assert!(site.completed);
}
```

## Extending the Game

### Add a New System

1. Define events in `src/events/`
2. Add system logic to `src/event_handlers/systems_trigger.rs`
3. Choose tick frequency (every tick, every 2 ticks, etc.)

Example:
```rust
// In SystemsTrigger::handle_gameloop_advanced
if event.tick % 3 == 0 {
    self.advance_resource_generation(event.tick).await?;
}
```

### Add a New Aggregate

1. Create aggregate in `src/aggregates/`
2. Define commands in `src/commands/`
3. Define events in `src/events/`
4. Create event store in `src/event_store/`
5. Add migration for event table
6. Optional: Add projector for read model

## Performance Considerations

- **In-memory cache**: DashMap provides O(1) lookups for active sites
- **Event versioning**: Prevents lost updates with optimistic locking
- **Idempotency**: Duplicate events safely ignored
- **Indexes**: PostgreSQL indexes on aggregate_id for fast event retrieval
- **Batch operations**: Multiple events can be stored in transactions

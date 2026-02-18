# Lunar Frontiers - Rust

A Rust implementation of an event-sourced game system demonstrating event sourcing, CQRS, and ECS-like patterns in the context of a space colony simulation.

## Overview

Lunar Frontiers is a tick-based colony builder where buildings are constructed over time. The system is built around event sourcing: all state changes are captured as immutable events, and current state is derived by replaying those events.

The game currently simulates:
- A tick-based gameloop (one tick every 2 seconds)
- Buildings that self-track their own construction progress
- A read model projector that maintains a queryable view of all buildings

## Architecture

### Event Sourcing Pattern

```
Command → Aggregate → Event → Event Store → (broadcast) → Subscribers
                                                               ├── Projectors (read models)
                                                               └── Event Handlers (game systems)
```

All writes go through aggregates. Aggregates validate commands and emit events. Events are stored append-only, then broadcast to subscribers.

### CQRS

- **Write side**: Commands → Aggregates → Events → PostgreSQL event tables
- **Read side**: Events → `BuildingProjector` → `buildings_read_model` table + in-memory DashMap

### Components

#### Aggregates (`src/aggregates/`)

| Aggregate | Purpose |
|---|---|
| `Gameloop` | Tracks tick count; emits `GameloopAdvanced` on each tick |
| `Building` | Manages a building's full lifecycle — spawning, construction progress, completion |
| `ConstructionSite` | **Legacy V1** — previously tracked construction; kept for historical data |

#### Event Stores (`src/event_store/`)

Each aggregate has a dedicated event store backed by a PostgreSQL table:

| Store | Table | Key Methods |
|---|---|---|
| `GameloopEventStore` | `gameloop_events` | `store_event`, `get_aggregate` |
| `BuildingEventStore` | `building_events` | `store_event`, `get_aggregate`, `get_active_building_ids` |
| `ConstructionSiteEventStore` | `construction_site_events` | `store_event`, `get_aggregate` (legacy) |

Events are stored with a `event_type` VARCHAR discriminator and a JSON payload. Deserialization uses the discriminator to select the correct variant — not `serde`'s tagged enum format — because the stored payload is a flat struct, not a tagged wrapper.

#### Projectors (`src/projectors/`)

**`BuildingProjector`** maintains the `buildings_read_model` table and an in-memory `DashMap`. It subscribes to the building broadcast channel and handles all building event types (V1 and V2).

#### Event Handlers (`src/event_handlers/`)

**`SystemsTrigger`** is the game's ECS-like system runner. On each `GameloopAdvanced` event, it fetches all active building IDs and advances each one by one tick. Buildings that complete emit a `BuildingCompleted` event.

#### Process Managers (`src/process_managers/`) — Legacy

**`ConstructionProcessManager`** was the V1 bridge between `ConstructionSite` and `Building`. It is no longer used in the V2 flow but is kept for historical completeness.

#### Message Broadcaster (`src/message_broadcaster.rs`)

Wraps Tokio `broadcast` channels for each event stream. Components subscribe to receive events without coupling to the event stores directly.

---

## Building V2: Self-Contained Construction

### Why V2?

The original V1 design split construction across two aggregates:

```
ConstructionSite: SiteSpawned → ConstructionProgressed → ConstructionCompleted
                                                               ↓
                                           ConstructionProcessManager
                                                               ↓
                                  Building: BuildingSpawned (already complete)
```

This required a process manager to bridge two aggregates and two event streams. V2 eliminates this by having buildings track their own construction:

```
Building: BuildingSpawnedV2 → BuildingProgressed → BuildingCompleted
```

### V2 Events

| Event | When emitted | Key fields |
|---|---|---|
| `BuildingSpawnedV2` | Building created under construction | `building_id`, `site_type`, `location`, `player_id`, `required_ticks` |
| `BuildingProgressed` | Each tick while under construction | `progressed_ticks`, `required_ticks` |
| `BuildingCompleted` | When `progressed_ticks >= required_ticks` | `building_id`, `tick` |

### Backward Compatibility

- V1 `BuildingSpawned` events are still handled by the projector
- `construction_site_events` table is preserved (event history is immutable)
- `ConstructionSiteEventStore` and `ConstructionProcessManager` remain as unused legacy code
- The V2 migration adds new values to the existing `building_event_type` PostgreSQL enum

---

## Database Schema

### Event Tables

All event tables share the same structure with aggregate-specific variants:

```sql
-- Example: building_events
id          UUID PRIMARY KEY
site_id     UUID NOT NULL          -- building_id for V2 buildings
event_type  building_event_type    -- enum discriminator
version     BIGINT NOT NULL
payload     JSON NOT NULL
timestamp   TIMESTAMPTZ DEFAULT now()
UNIQUE (site_id, version)
```

The `UNIQUE (aggregate_id, version)` constraint provides optimistic locking.

### Event Type Enums

```sql
CREATE TYPE gameloop_event_type AS ENUM (
    'gameloop_advanced_v1'
);

CREATE TYPE construction_event_type AS ENUM (
    'site_spawned_v1', 'construction_progressed_v1', 'construction_completed_v1'
);

CREATE TYPE building_event_type AS ENUM (
    'building_spawned_v1',
    'building_spawned_v2',       -- V2
    'building_progressed_v1',    -- V2
    'building_completed_v1'      -- V2
);
```

### Read Model

```sql
buildings_read_model (
    site_id             UUID PRIMARY KEY,
    site_type           VARCHAR,
    location            JSON,
    player_id           UUID,
    complete_percentage REAL DEFAULT 0.0,
    ready               BOOLEAN DEFAULT false,
    progressed_ticks    BIGINT,   -- NULL when complete
    required_ticks      BIGINT,   -- NULL when complete
    created_at          TIMESTAMPTZ,
    updated_at          TIMESTAMPTZ   -- auto-updated by trigger
)
```

---

## Code Structure

```
src/
├── lib.rs                        # Module exports
├── main.rs                       # App entry point
├── models.rs                     # Shared types (Location, SiteType, BuildingReadModel)
├── message_broadcaster.rs        # Tokio broadcast channel wrappers
├── aggregates/
│   ├── gameloop.rs               # Gameloop aggregate
│   ├── building.rs               # Building aggregate (V1 + V2)
│   └── construction_site.rs      # ConstructionSite aggregate (legacy V1)
├── commands/
│   ├── gameloop.rs               # AdvanceGameloop
│   ├── building.rs               # SpawnBuilding (V1), SpawnBuildingV2, AdvanceBuilding
│   └── construction_site.rs      # SpawnSite, AdvanceConstruction (legacy)
├── events/
│   ├── gameloop.rs               # GameloopAdvanced
│   ├── building.rs               # BuildingSpawned, BuildingSpawnedV2, BuildingProgressed, BuildingCompleted
│   └── construction_site.rs      # SiteSpawned, ConstructionProgressed, ConstructionCompleted (legacy)
├── event_store/
│   ├── mod.rs                    # Event store implementations
│   ├── events.rs                 # Serialization types
│   └── template.rs               # SQL query helpers + event deserialization
├── projectors/
│   └── building.rs               # BuildingProjector
├── process_managers/
│   └── construction.rs           # ConstructionProcessManager (legacy)
└── event_handlers/
    └── systems_trigger.rs        # SystemsTrigger
```

---

## Running the Project

### Prerequisites

- Docker and Docker Compose
- Rust 1.85+ (for local development)

### Docker Compose (recommended)

```bash
docker compose up -d
```

This starts three services in order:
1. **db** — PostgreSQL 18 on port 5433
2. **migrate** — runs SQL migrations against the database
3. **app** — builds and runs the game binary

```bash
docker compose logs -f app   # follow logs
docker compose down          # stop and remove containers
```

### Local Development

1. Start the database:
```bash
docker compose up -d db
```

2. Run migrations:
```bash
# Requires sqlx-cli: cargo install sqlx-cli
DATABASE_URL="postgres://postgres:postgres@localhost:5433/lunar_frontiers" cargo sqlx migrate run
```

3. Run the game:
```bash
SQLX_OFFLINE=true cargo run
```

The `.env` file contains `DATABASE_URL` and `RUST_LOG` for local runs.

### What Happens at Startup

1. Three buildings are spawned under construction (V2 flow):
   - Power Plant at (10, 10) — requires 5 ticks
   - Mine at (20, 15) — requires 8 ticks
   - Habitat at (5, 25) — requires 10 ticks
2. The gameloop starts ticking every 2 seconds
3. Each tick, `SystemsTrigger` advances all active buildings by 1 tick
4. Buildings emit `BuildingProgressed` each tick and `BuildingCompleted` when done
5. `BuildingProjector` updates the read model on each event

### Expected Output

```
[INFO] Starting game ...
[INFO] Spawning Power Plant at (10, 10), requires 5 ticks
[INFO] Spawning Mine at (20, 15), requires 8 ticks
[INFO] Spawning Habitat at (5, 25), requires 10 ticks
[INFO] Tick 1 — advancing 3 buildings
[INFO] Tick 2 — advancing 3 buildings
[INFO] Tick 5 — Power Plant completed
[INFO] Tick 8 — Mine completed
[INFO] Tick 10 — Habitat completed
```

---

## Testing

Unit tests live in the aggregate files and run without a database:

```bash
SQLX_OFFLINE=true cargo test
```

Current tests (7 total):

| Test | Location |
|---|---|
| `test_gameloop_advance` | `aggregates/gameloop.rs` |
| `test_gameloop_hydration` | `aggregates/gameloop.rs` |
| `test_building_spawn_v2` | `aggregates/building.rs` |
| `test_building_construction_completes` | `aggregates/building.rs` |
| `test_building_v1_legacy_is_completed` | `aggregates/building.rs` |
| `test_construction_site_spawn` | `aggregates/construction_site.rs` |
| `test_construction_completes` | `aggregates/construction_site.rs` |

---

## SQLx Offline Mode

Queries are validated at compile time against a cached schema in `.sqlx/`. To regenerate after schema changes:

```bash
DATABASE_URL="postgres://postgres:postgres@localhost:5433/lunar_frontiers" cargo sqlx prepare
```

---

## Extending the Game

### Add a New System

Add a method to `SystemsTrigger` and call it from `handle_gameloop_advanced`. Systems can run at different tick frequencies:

```rust
async fn handle_gameloop_advanced(&self, event: &GameloopAdvanced) -> Result<(), BoxError> {
    self.advance_buildings(event.tick).await?;

    if event.tick % 3 == 0 {
        self.advance_resource_generation(event.tick).await?;
    }
    Ok(())
}
```

### Add a New Aggregate

1. Define events in `src/events/`
2. Define commands in `src/commands/`
3. Implement the aggregate in `src/aggregates/`
4. Implement an event store in `src/event_store/`
5. Add a SQL migration for the event table and enum type
6. Regenerate the SQLx cache
7. Optionally add a projector for a read model

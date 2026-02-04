-- Create enum types for event discrimination
DO $$ BEGIN
    CREATE TYPE gameloop_event_type AS ENUM (
        'gameloop_advanced_v1'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE construction_event_type AS ENUM (
        'site_spawned_v1',
        'construction_progressed_v1',
        'construction_completed_v1'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE building_event_type AS ENUM (
        'building_spawned_v1'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Gameloop events table
CREATE TABLE gameloop_events (
    id uuid NOT NULL PRIMARY KEY,
    game_id uuid NOT NULL,
    event_type gameloop_event_type NOT NULL,
    version bigint NOT NULL,
    payload json NOT NULL,
    timestamp timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT uq_gameloop_events_stream_version UNIQUE (game_id, version)
);

CREATE INDEX idx_gameloop_events_stream ON gameloop_events(game_id, version);

-- Construction site events table
CREATE TABLE construction_site_events (
    id uuid NOT NULL PRIMARY KEY,
    site_id uuid NOT NULL,
    event_type construction_event_type NOT NULL,
    version bigint NOT NULL,
    payload json NOT NULL,
    timestamp timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT uq_construction_site_events_stream_version UNIQUE (site_id, version)
);

CREATE INDEX idx_construction_site_events_stream ON construction_site_events(site_id, version);

-- Building events table
CREATE TABLE building_events (
    id uuid NOT NULL PRIMARY KEY,
    site_id uuid NOT NULL,
    event_type building_event_type NOT NULL,
    version bigint NOT NULL,
    payload json NOT NULL,
    timestamp timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT uq_building_events_stream_version UNIQUE (site_id, version)
);

CREATE INDEX idx_building_events_stream ON building_events(site_id, version);

-- Read model: Buildings and construction sites
CREATE TABLE buildings_read_model (
    site_id uuid PRIMARY KEY,
    site_type varchar NOT NULL,
    location json NOT NULL,
    player_id uuid,
    complete_percentage real NOT NULL DEFAULT 0.0,
    ready boolean NOT NULL DEFAULT false,
    progressed_ticks bigint,
    required_ticks bigint,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_buildings_by_player ON buildings_read_model(player_id) WHERE player_id IS NOT NULL;
CREATE INDEX idx_buildings_not_ready ON buildings_read_model(site_id) WHERE ready = false;

-- Trigger to update timestamps
CREATE OR REPLACE FUNCTION set_buildings_updated_timestamp()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_buildings_updated_timestamp
    BEFORE UPDATE ON buildings_read_model
    FOR EACH ROW
    EXECUTE FUNCTION set_buildings_updated_timestamp();

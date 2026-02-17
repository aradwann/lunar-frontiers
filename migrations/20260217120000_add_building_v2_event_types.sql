-- Add new building event type variants for the V2 building lifecycle.
-- Buildings now track their own construction progress instead of using
-- a separate ConstructionSite aggregate.
--
-- Existing construction_site_events table and its enum are preserved
-- as immutable historical data.

ALTER TYPE building_event_type ADD VALUE IF NOT EXISTS 'building_spawned_v2';
ALTER TYPE building_event_type ADD VALUE IF NOT EXISTS 'building_progressed_v1';
ALTER TYPE building_event_type ADD VALUE IF NOT EXISTS 'building_completed_v1';

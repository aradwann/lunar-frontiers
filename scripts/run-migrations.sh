#!/bin/sh
set -e

echo "Running migrations..."

# Create tracking table if it doesn't exist
psql -c "
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL
);"

# Apply each migration file in order
for migration in /migrations/*.sql; do
    filename=$(basename "$migration")
    # Extract version number (timestamp prefix before the first underscore-separated name)
    version=$(echo "$filename" | grep -o '^[0-9]*')

    # Check if already applied
    applied=$(psql -tAc "SELECT count(*) FROM _sqlx_migrations WHERE version = $version AND success = true;")

    if [ "$applied" = "0" ]; then
        echo "Applying migration: $filename"
        psql -f "$migration"
        psql -c "INSERT INTO _sqlx_migrations (version, description, success) VALUES ($version, '$filename', true);"
        echo "Applied: $filename"
    else
        echo "Already applied: $filename"
    fi
done

echo "Migrations complete."

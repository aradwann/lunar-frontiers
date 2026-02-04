#!/bin/bash
set -e

echo "🚀 Setting up Lunar Frontiers database..."
echo ""

# Configuration
CONTAINER_NAME="lunar-frontiers-db"
DB_USER="postgres"
DB_PASSWORD="postgres"
DB_NAME="lunar_frontiers"
DB_PORT="5433"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running"
    exit 1
fi

# Remove old container if exists
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "🧹 Removing old container..."
    docker stop "${CONTAINER_NAME}" > /dev/null 2>&1 || true
    docker rm "${CONTAINER_NAME}" > /dev/null 2>&1 || true
fi

# Create and start PostgreSQL container
echo "📦 Creating PostgreSQL container..."
docker run -d \
    --name "${CONTAINER_NAME}" \
    -e POSTGRES_PASSWORD="${DB_PASSWORD}" \
    -e POSTGRES_DB="${DB_NAME}" \
    -p "${DB_PORT}:5432" \
    postgres:18-alpine

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for database to start..."
sleep 3
until docker exec "${CONTAINER_NAME}" pg_isready > /dev/null 2>&1; do
    echo -n "."
    sleep 1
done
echo " Ready!"

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "📥 Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Set DATABASE_URL and run migrations
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
echo "🔄 Running migrations..."
sqlx migrate run

# Create .env file
echo "📝 Creating .env file..."
cat > .env << EOF
DATABASE_URL=${DATABASE_URL}
RUST_LOG=info
EOF

echo ""
echo "✅ Setup complete!"
echo ""
echo "Database URL: ${DATABASE_URL}"
echo ""
echo "To start the app:"
echo "  cargo run"
echo ""
echo "To connect with psql:"
echo "  docker exec -it ${CONTAINER_NAME} psql -U ${DB_USER} -d ${DB_NAME}"
echo ""

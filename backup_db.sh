#!/bin/bash

# Set variables
CONTAINER_NAME="semperfliesDB"
DB_USER="postgres"
DB_NAME="your_database"
BACKUP_DIR="/home/youruser/postgres_backups"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILE="$BACKUP_DIR/db_backup_$TIMESTAMP.sql"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

# Run pg_dump inside the container and copy the backup to the VPS
docker exec "$CONTAINER_NAME" pg_dump -U "$DB_USER" -d "$DB_NAME" > "$BACKUP_FILE"

# Optional: Delete old backups (keep last 7)
find "$BACKUP_DIR" -type f -name "*.sql" -mtime +7 -delete

echo "Backup completed: $BACKUP_FILE"


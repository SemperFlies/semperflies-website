#!/bin/bash

BACKUP_DIR="$HOME/semperflies_backups"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILE="$BACKUP_DIR/dump_$TIMESTAMP.sql"

# Ensure the backup directory exists
mkdir -p $BACKUP_DIR

# Perform the backup
sudo docker exec -t semperfliesDB pg_dumpall -c -U admin > $BACKUP_FILE

echo "Backup completed: $BACKUP_FILE"


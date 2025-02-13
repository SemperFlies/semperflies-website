#!/bin/bash

# Define the backup directory
BACKUP_DIR="$HOME/semperflies_backups/sql"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILE="$BACKUP_DIR/dump_$TIMESTAMP.sql"

# Ensure the backup directory exists
mkdir -p "$BACKUP_DIR"

# Change to the backup directory
cd "$BACKUP_DIR" || exit

# List all files in the directory sorted by modification time, keeping the latest 5
echo "Cleaning up old backups, keeping the latest 5..."

# Get a list of files sorted by modification time (oldest first) and delete everything except the last 5
ls -t | tail -n +6 | while read -r file; do
    # Check if the file exists before attempting to delete
    if [ -f "$file" ]; then
        echo "Deleting old backup: $file"
        rm -f "$file"
    fi
done

echo "Cleanup completed. Only the latest 5 backups are kept."

# Perform the backup (docker exec assumes you have correct permissions to run the command)
echo "Starting backup..."
sudo docker exec -t semperfliesDB pg_dumpall -c -U admin > "$BACKUP_FILE"

echo "Backup completed: $BACKUP_FILE"

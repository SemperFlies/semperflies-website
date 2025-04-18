#!/bin/bash

# Define source and target directories
SOURCE_DIR="$HOME/semperflies_backups/imgs_tmp/images"

# Define your Docker container name
CONTAINER_NAME="semperflies-website-semperflies-1"
TARGET_DIR="/usr/src/app/public/assets/images"

# Function to move files into Docker container
move_files_to_container() {
    local source="$1"
    local container="$2"
    local target_path="$3"

    # Loop through the files in the source directory
    for item in "$source"/*; do
        # Skip if no files are found
        [ -e "$item" ] || continue

        # Get the basename (file or directory name)
        basename_item=$(basename "$item")
        
        # Check if the item exists in the target directory inside the container
        echo "Copying $item to container $container:$target_path"
        sudo docker cp "$item" "$container:$target_path"
    done
}

# Move the files to the container
move_files_to_container "$SOURCE_DIR" "$CONTAINER_NAME" "$TARGET_DIR"

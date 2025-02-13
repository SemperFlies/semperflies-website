#!/bin/bash

# Define the source and target directories
SOURCE_DIR="$HOME/semperflies_backups/imgs_tmp"
TARGET_DIR="/usr/src/app/public/assets/images"

# Function to move files and directories
move_files() {
    local source="$1"
    local target="$2"

    # Loop through the items (files or directories) in the source directory
    for item in "$source"/*; do
        # Skip if no files are found
        [ -e "$item" ] || continue

        # Get the basename (file or directory name)
        basename_item=$(basename "$item")
        
        # Check if the item exists in the target directory
        if [ ! -e "$target/$basename_item" ]; then
            if [ -d "$item" ]; then
                # If it's a directory, recursively call the function
                echo "Moving directory $item to $target"
                mv -v "$item" "$target"
            else
                # If it's a file, move it
                echo "Moving file $item to $target"
                mv -v "$item" "$target"
            fi
        fi
    done
}

# Call the function to move files and directories from source to target
move_files "$SOURCE_DIR" "$TARGET_DIR"


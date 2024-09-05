#!/bin/bash

setup_nfs() {
    echo "Setting up NFS server..."

    # Install NFS server if not already installed
    if ! dpkg -l | grep -q nfs-kernel-server; then
        apt-get update
        apt-get install -y nfs-kernel-server
    fi

    # Create NFS share directory
    # mkdir -p /srv/nfs/share
    # touch /srv/nfs/share/test.txt
    # echo "your mom" > /srv/nfs/share/test.txt
    # chown -R nobody:nogroup /srv/nfs/share
    # chmod 755 /srv/nfs/share

    # Configure NFS exports
    sudo echo "${APP}/public *(rw,sync,no_subtree_check)" > /etc/exports
    exportfs -a

    # Start NFS server
    service nfs-kernel-server start

    echo "NFS server setup completed."
}

# Start NFS setup
setup_nfs

# Run the web server
echo "Starting web server..."
exec "${APP}/semperflies"

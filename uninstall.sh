#!/bin/bash

# Check if the systemd service is active, and stop it if necessary.
if systemctl is-active --quiet rust-ddns.service; then
    echo "Stopping rust-ddns service..."
    sudo systemctl stop rust-ddns.service
    sudo systemctl disable rust-ddns.service
fi

# Check if the systemd timer is active, and stop it if necessary.
if systemctl is-active --quiet rust-ddns.timer; then
    echo "Stopping rust-ddns timer..."
    sudo systemctl stop rust-ddns.timer
    sudo systemctl disable rust-ddns.timer
fi

# Delete the systemd files.
if [[ -f '/etc/systemd/system/rust-ddns.service' ]]; then
    echo "Removing rust-ddns service..."
    sudo rm /etc/systemd/system/rust-ddns.service
fi

if [[ -f '/etc/systemd/system/rust-ddns.timer' ]]; then
    echo "Removing rust-ddns timer..."
    sudo rm /etc/systemd/system/rust-ddns.timer
fi

# Reload the systemd manager configuration.
sudo systemctl daemon-reload

# Delete the symbolic link from the binary folder.
if [[ -L "$HOME/.local/bin/rust-ddns" ]]; then
    echo "Deleting symbolic link to rust-ddns..."
    rm "$HOME/.local/bin/rust-ddns"
fi

# Confirm uninstallation.
echo "Uninstallation complete!"
#!/bin/bash

OS=$(uname)

if [[ "$OS" == "Linux" ]]; then
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

    # Delete the binary and wrapper from the binary folder.
    if [[ -L "$HOME/.local/bin/rust-ddns" ]] || [[ -f "$HOME/.local/bin/rust-ddns" ]]; then
        echo "Removing rust-ddns binary..."
        rm "$HOME/.local/bin/rust-ddns"
    fi

    if [[ -f "$HOME/.local/bin/ddnsd-rust-ddns" ]]; then
        echo "Removing ddnsd wrapper..."
        rm "$HOME/.local/bin/ddnsd-rust-ddns"
    fi

elif [[ "$OS" == "Darwin" ]]; then
    PLIST_PATH="$HOME/Library/LaunchAgents/com.rust-ddns.plist"

    if [[ -f "$PLIST_PATH" ]]; then
        echo "Unloading LaunchAgent..."
        launchctl bootout "gui/$UID" "$PLIST_PATH" 2>/dev/null || true
        echo "Removing plist..."
        rm "$PLIST_PATH"
    fi

    if [[ -f "$HOME/.local/bin/rust-ddns" ]]; then
        echo "Removing rust-ddns binary..."
        rm "$HOME/.local/bin/rust-ddns"
    fi

    if [[ -f "$HOME/.local/bin/ddnsd-rust-ddns" ]]; then
        echo "Removing ddnsd wrapper..."
        rm "$HOME/.local/bin/ddnsd-rust-ddns"
    fi

else
    echo "Unsupported OS: $OS. For Windows, use uninstall.ps1."
    exit 1
fi

echo "Uninstallation complete!"

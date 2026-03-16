#!/bin/bash

set -e

CURRENT_DIR=$(pwd)

if [[ -z $RUST_DDNS_INTERVAL ]]; then
    echo "RUST_DDNS_INTERVAL is not set, defaulting to 5min."
    RUST_DDNS_INTERVAL=5min
fi

OS=$(uname)

# Parse interval into seconds (for macOS launchd)
interval_to_seconds() {
    local interval="$1"
    if [[ "$interval" == *min ]]; then
        echo $(( ${interval%min} * 60 ))
    elif [[ "$interval" == *h ]]; then
        echo $(( ${interval%h} * 3600 ))
    else
        echo "$interval"
    fi
}

echo "Compiling Rust application..."
cargo build --release

mkdir -p "$HOME/.local/bin"

if [[ "$OS" == "Linux" ]]; then
    RUN_SCRIPT='ddnsd'
    if [[ ! -f 'install.sh' ]] || [[ ! -f $RUN_SCRIPT ]]; then
        echo "Required files not found. Make sure you are running the script from within the source directory" && exit 1
    fi

    echo "Creating rust-ddns systemd timer and service..."

    echo "\
[Unit]
Description=Runs rust-ddns every $RUST_DDNS_INTERVAL

[Timer]
OnBootSec=$RUST_DDNS_INTERVAL
OnUnitActiveSec=$RUST_DDNS_INTERVAL
Unit=rust-ddns.service

[Install]
WantedBy=multi-user.target
" | sudo tee /etc/systemd/system/rust-ddns.timer >/dev/null

    echo "\
[Unit]
Description=Run rust-ddns once

[Service]
User=$USER
WorkingDir=$HOME
ExecStart=$HOME/.local/bin/ddnsd-rust-ddns
Environment=HOME=$HOME
Environment=PATH=$HOME/.local/bin:$PATH
" | sudo tee /etc/systemd/system/rust-ddns.service >/dev/null

    BINARY_PATH="$HOME/.local/bin/rust-ddns"
    if [[ ! -f $BINARY_PATH ]]; then
        echo "Creating symbolic link to binary in local binaries folder..."
        ln -s "$CURRENT_DIR/target/release/rust-ddns" "$BINARY_PATH"
    else
        echo "Symbolic link already exists. Skipping creation."
    fi

    WRAPPER_PATH="$HOME/.local/bin/ddnsd-rust-ddns"
    echo '#!/bin/bash
if [[ -z $RUST_DDNS_LOG_FILE ]]; then
RUST_DDNS_LOG_FILE=$HOME/.rust-ddns.log
fi
touch $RUST_DDNS_LOG_FILE
cd $HOME || exit 1
rust-ddns &>> $RUST_DDNS_LOG_FILE
echo "$(tail -n 200 $RUST_DDNS_LOG_FILE)" > $RUST_DDNS_LOG_FILE' > "$WRAPPER_PATH"
    chmod +x "$WRAPPER_PATH"

    echo "Reloading systemd manager configuration..."
    sudo systemctl daemon-reload

    echo "Installation complete!"

    if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
        echo "Warning: Your PATH does not include the local binaries folder."
        echo "To add it to your PATH, append the following line to your shell's configuration file:"
        echo "export PATH=\$PATH:\$HOME/.local/bin"
    fi

    echo "Run: sudo systemctl enable --now rust-ddns.timer"

elif [[ "$OS" == "Darwin" ]]; then
    INTERVAL_SECS=$(interval_to_seconds "$RUST_DDNS_INTERVAL")
    BINARY_DEST="$HOME/.local/bin/rust-ddns"
    PLIST_DIR="$HOME/Library/LaunchAgents"
    PLIST_PATH="$PLIST_DIR/com.rust-ddns.plist"
    WRAPPER_PATH="$HOME/.local/bin/ddnsd-rust-ddns"
    LOG_FILE="${RUST_DDNS_LOG_FILE:-$HOME/.rust-ddns.log}"

    echo "Copying binary to $BINARY_DEST..."
    cp "$CURRENT_DIR/target/release/rust-ddns" "$BINARY_DEST"
    chmod +x "$BINARY_DEST"

    echo "Writing log rotation wrapper to $WRAPPER_PATH..."
    cat > "$WRAPPER_PATH" << 'WRAPPER'
#!/bin/bash
if [[ -z $RUST_DDNS_LOG_FILE ]]; then
RUST_DDNS_LOG_FILE=$HOME/.rust-ddns.log
fi
touch $RUST_DDNS_LOG_FILE
cd $HOME || exit 1
rust-ddns &>> $RUST_DDNS_LOG_FILE
echo "$(tail -n 200 $RUST_DDNS_LOG_FILE)" > $RUST_DDNS_LOG_FILE
WRAPPER
    chmod +x "$WRAPPER_PATH"

    mkdir -p "$PLIST_DIR"
    echo "Writing LaunchAgent plist to $PLIST_PATH..."
    cat > "$PLIST_PATH" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rust-ddns</string>
    <key>ProgramArguments</key>
    <array>
        <string>$WRAPPER_PATH</string>
    </array>
    <key>StartInterval</key>
    <integer>$INTERVAL_SECS</integer>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_FILE</string>
    <key>StandardErrorPath</key>
    <string>$LOG_FILE</string>
</dict>
</plist>
PLIST

    echo "Loading LaunchAgent..."
    launchctl bootstrap "gui/$UID" "$PLIST_PATH"

    echo "Installation complete!"

    if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
        echo "Warning: Your PATH does not include the local binaries folder."
        echo "Add to your shell config: export PATH=\$PATH:\$HOME/.local/bin"
    fi
else
    echo "Unsupported OS: $OS. For Windows, use install.ps1."
    exit 1
fi

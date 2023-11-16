#!/bin/bash

set -e

RUN_SCRIPT='ddnsd'
CURRENT_DIR=$(pwd)

if [[ -z $RUST_DDNS_INTERVAL ]]; then
    echo "RUST_DDNS_INTERVAL is not set, defaulting to 5min."
    RUST_DDNS_INTERVAL=5min
fi

if [[ -f 'install.sh' ]] && [[ -f $RUN_SCRIPT ]]; then
    echo "Creating rust-ddns systemd timer and service..."

    echo "\
[Unit]
Description=Runs rust-ddns every 5 minutes

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
ExecStart=$CURRENT_DIR/$RUN_SCRIPT
Environment=HOME=$HOME
Environment=PATH=$HOME/.local/bin:$PATH
" | sudo tee /etc/systemd/system/rust-ddns.service >/dev/null

    echo "Compiling Rust application..."
    cargo build --release

    mkdir -p $HOME/.local/bin
    SYMBOLIC_LINK_PATH="$HOME/.local/bin/rust-ddns"
    if [[ ! -f $SYMBOLIC_LINK_PATH ]]; then
        echo "Creating symbolic link to binary in local binaries folder..."
        ln -s $CURRENT_DIR/target/release/rust-ddns $SYMBOLIC_LINK_PATH
    else
        echo "Symbolic link already exists. Skipping creation."
    fi

    echo "Reloading systemd manager configuration..."
    sudo systemctl daemon-reload
else
    echo "Required files not found. Make sure you are running the script from within the source directory" && exit 1
fi

echo "Installation complete!"

if ! echo $PATH | grep -q "$HOME/.local/bin"; then
    echo "Warning: Your PATH does not include the local binaries folder."
    echo "To add it to your PATH, append the following line to your shell's configuration file (usually either ~/.bashrc or ~/.zshrc):"
    echo "export PATH=\$PATH:\$HOME/.local/bin"
    echo "Then, reload the configuration with the command 'source ~/.bashrc' or 'source ~/.zshrc'."
fi

#!/bin/bash

set -e

RUN_SCRIPT='ddnsd'
CURRENT_DIR=$(pwd)

if [[ -z $RUST_DDNS_INTERVAL ]]; then
RUST_DDNS_INTERVAL=5min
fi

if [[ -f 'install.sh' ]] && [[ -f $RUN_SCRIPT ]]; then
echo "\
[Unit]
Description=Runs rust-ddns every 5 minutes

[Timer]
OnBootSec=$RUST_DDNS_INTERVAL
OnUnitActiveSec=$RUST_DDNS_INTERVAL
Unit=rust-ddns.service

[Install]
WantedBy=multi-user.target
" | sudo tee /etc/systemd/system/rust-ddns.timer
echo "\
[Unit]
User=$USER
Description=Run rust-ddns once

[Service]
ExecStart=$CURRENT_DIR/$RUN_SCRIPT
" | sudo tee /etc/systemd/system/rust-ddns.service

cargo build --release

mkdir -p $HOME/.local/bin

ln -s $CURRENT_DIR/target/release/rust-ddns $HOME/.local/bin/rust-ddns

sudo systemctl daemon-reload

else
echo "Required files not found. Make sure you are running the script from within the source directory" && exit 1
fi

#!/bin/bash

set -e

RUN_SCRIPT='run_with_file_logging.sh'
CURRENT_DIR=$(pwd)

if [[ -f 'install.sh' ]] && [[ -f $RUN_SCRIPT ]]; then
echo "\
[Unit]
Description=Runs rust-ddns every 5 minutes

[Timer]
OnBootSec=5min
OnUnitActiveSec=5min
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
echo "required files not found" || exit 1
fi

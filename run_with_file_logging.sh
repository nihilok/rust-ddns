#!/bin/bash

LOG_FILE=$HOME/.rust-ddns.log

cd $HOME || exit 1

rust-ddns &>> $LOG_FILE

echo "$(tail -n 200 $LOG_FILE)" > $LOG_FILE
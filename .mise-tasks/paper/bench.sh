#!/usr/bin/env bun
#MISE description="Measure the compiler and write the CSV files the papers read"
#MISE dir="{{config_root}}"

cargo build -q --release -p zakac && docs/paper/bench/run.sh

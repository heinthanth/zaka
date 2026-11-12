#!/usr/bin/env bun
#MISE description="Run the examples of the papers and compare their output"
#MISE dir="{{config_root}}"

cargo build -q -p zakac && docs/paper/examples/check.sh

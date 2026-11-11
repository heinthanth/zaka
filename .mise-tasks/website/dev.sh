#!/usr/bin/env bun
#MISE description="Serve the website with live reload, the explorer page with it"
#MISE depends=["website:vendor", "font:fetch"]
#MISE dir="{{config_root}}"

LEPTOS_HASH_FILES=false cargo leptos watch --project zaka_website --split

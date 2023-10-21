#!/bin/bash 

root="$(dirname "${BASH_SOURCE[0]}")/.."
bin="$root/target/debug"

for util in "$bin/"{"styx","cocytus","lethe","phlegethon","acheron"}; do 
	if [ -f "$util" ]; then
		chmod +x "$util"
		cp "$util" "$MYTHOS_BIN_DIR"
	fi
done

cp "$root/files/plutonian-shores.toml" "$MYTHOS_CONFIG_DIR"
cp -n "$root/files/plutonian-shores.toml" "$MYTHOS_LOCAL_CONFIG_DIR"



unset root bin

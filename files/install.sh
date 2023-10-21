#!/bin/bash 

HOME="$(getent passwd $SUDO_USER | cut -d: -f6)"
if [ -z "$MYTHOS_BIN_DIR" ] && [ -f "/etc/profile.d/mythos-vars.sh" ]; then 
	source "/etc/profile.d/mythos-vars.sh"
else 
	echo "Could not get MYTHOS_DIR env vars"
	return 2
fi
	
root="$(dirname "${BASH_SOURCE[0]}")/.."
bin="$root/target/debug"

for util_name in "styx" "cocytus" "lethe" "phlegethon" "acheron"; do 
	util="$bin/$util_name"
	if [ -f "$util" ]; then
		chmod +x "$util"
		cp "$util" "$MYTHOS_BIN_DIR/$util_name"

		echo "Installed: $util_name"
	fi
done

cp "$root/files/plutonian-shores.toml" "$MYTHOS_CONFIG_DIR"
cp -n "$root/files/plutonian-shores.toml" "$MYTHOS_LOCAL_CONFIG_DIR"
chown hyrum "$MYTHOS_LOCAL_CONFIG_DIR/plutonian-shores.toml"


unset root bin util

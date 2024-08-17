#!/bin/bash
# Script intended to install charon, as it currently cannot install itself.

cargo build --bin charon

sudo cp "target/debug/charon" "/bin"
sudo charon "charon/charon.charon"


#!/bin/bash
# Local P2P Testing Helper
#
# This script launches two instances of emberware-zx for local P2P testing.
# The instances connect to each other via localhost UDP sockets.
#
# Usage: ./scripts/test-p2p.sh <rom_path> [input_delay]
#
# Examples:
#   ./scripts/test-p2p.sh games/pong.ewzx
#   ./scripts/test-p2p.sh games/pong.ewzx 2

set -e

ROM_PATH="${1:?Usage: $0 <rom_path> [input_delay]}"
INPUT_DELAY="${2:-2}"

if [ ! -f "$ROM_PATH" ]; then
    echo "Error: ROM file not found: $ROM_PATH"
    exit 1
fi

echo "Starting P2P test with ROM: $ROM_PATH (input delay: $INPUT_DELAY)"
echo ""
echo "Player 1: bind=7777, peer=7778, local_player=0"
echo "Player 2: bind=7778, peer=7777, local_player=1"
echo ""

# Start player 2 in background
cargo run -p emberware-zx --release -- "$ROM_PATH" \
    --p2p \
    --bind 7778 \
    --peer 7777 \
    --local-player 1 \
    --input-delay "$INPUT_DELAY" &
P2_PID=$!

# Give player 2 time to bind its port
sleep 0.5

# Start player 1 in foreground
cargo run -p emberware-zx --release -- "$ROM_PATH" \
    --p2p \
    --bind 7777 \
    --peer 7778 \
    --local-player 0 \
    --input-delay "$INPUT_DELAY"

# Clean up player 2 when player 1 exits
echo ""
echo "Player 1 exited, cleaning up..."
kill $P2_PID 2>/dev/null || true
wait $P2_PID 2>/dev/null || true

echo "P2P test complete."

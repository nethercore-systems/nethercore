#!/bin/bash
# NCHS Protocol Testing Helper
#
# This script runs the NCHS (Nethercore Handshake) protocol tests.
# These tests verify the handshake, validation, and session setup.
#
# Usage: ./scripts/test-nchs.sh [test_filter]
#
# Examples:
#   ./scripts/test-nchs.sh              # Run all NCHS tests
#   ./scripts/test-nchs.sh mismatch     # Run mismatch rejection tests
#   ./scripts/test-nchs.sh handshake    # Run handshake tests

set -e

TEST_FILTER="${1:-nchs}"

echo "Running NCHS protocol tests..."
echo ""

# Run the NCHS tests with optional filter
cargo test --lib --package nethercore-core "$TEST_FILTER" -- --nocapture

echo ""
echo "NCHS tests complete."
echo ""
echo "=== Test Coverage ==="
echo "- Message serialization (bitcode roundtrip)"
echo "- Socket binding and send/receive"
echo "- Host/Guest state machines"
echo "- Validation: ROM hash, console type, tick rate mismatches"
echo "- Lobby full rejection"
echo "- Game in progress rejection"
echo "- Full handshake flow: join -> ready -> start -> session sync"
echo ""
echo "=== Notes ==="
echo "The current --host/--join CLI modes bypass NCHS and use direct P2P."
echo "NCHS integration into the player app is pending."
echo "Use --p2p mode for direct testing without validation."

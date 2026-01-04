@echo off
REM NCHS Protocol Testing Helper for Windows
REM
REM This script runs the NCHS (Nethercore Handshake) protocol tests.
REM These tests verify the handshake, validation, and session setup.
REM
REM Usage: scripts\test-nchs.bat [test_filter]
REM
REM Examples:
REM   scripts\test-nchs.bat              # Run all NCHS tests
REM   scripts\test-nchs.bat mismatch     # Run mismatch rejection tests
REM   scripts\test-nchs.bat handshake    # Run handshake tests

setlocal

set TEST_FILTER=%~1
if "%TEST_FILTER%"=="" set TEST_FILTER=nchs

echo Running NCHS protocol tests...
echo.

REM Run the NCHS tests with optional filter
cargo test --lib --package nethercore-core %TEST_FILTER% -- --nocapture

echo.
echo NCHS tests complete.
echo.
echo === Test Coverage ===
echo - Message serialization (bitcode roundtrip)
echo - Socket binding and send/receive
echo - Host/Guest state machines
echo - Validation: ROM hash, console type, tick rate mismatches
echo - Lobby full rejection
echo - Game in progress rejection
echo - Full handshake flow: join -^> ready -^> start -^> session sync
echo.
echo === Notes ===
echo The current --host/--join CLI modes bypass NCHS and use direct P2P.
echo NCHS integration into the player app is pending.
echo Use --p2p mode for direct testing without validation.

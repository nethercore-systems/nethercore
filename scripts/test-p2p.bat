@echo off
REM Local P2P Testing Helper for Windows
REM
REM This script launches two instances of nethercore-zx for local P2P testing.
REM The instances connect to each other via localhost UDP sockets.
REM
REM Usage: scripts\test-p2p.bat <rom_path> [input_delay]
REM
REM Examples:
REM   scripts\test-p2p.bat games\pong.nczx
REM   scripts\test-p2p.bat games\pong.nczx 2

setlocal

if "%~1"=="" (
    echo Usage: %0 ^<rom_path^> [input_delay]
    exit /b 1
)

set ROM_PATH=%~1
set INPUT_DELAY=%~2
if "%INPUT_DELAY%"=="" set INPUT_DELAY=2

if not exist "%ROM_PATH%" (
    echo Error: ROM file not found: %ROM_PATH%
    exit /b 1
)

echo Starting P2P test with ROM: %ROM_PATH% (input delay: %INPUT_DELAY%)
echo.
echo Player 1: bind=7777, peer=7778, local_player=0
echo Player 2: bind=7778, peer=7777, local_player=1
echo.

REM Start player 2 in a new window
start "Player 2" cmd /c "cargo run -p nethercore-zx --release -- "%ROM_PATH%" --p2p --bind 7778 --peer 7777 --local-player 1 --input-delay %INPUT_DELAY%"

REM Wait a moment for player 2 to bind
timeout /t 1 /nobreak >nul

REM Start player 1 in this window
cargo run -p nethercore-zx --release -- "%ROM_PATH%" --p2p --bind 7777 --peer 7778 --local-player 0 --input-delay %INPUT_DELAY%

echo.
echo P2P test complete. Close the Player 2 window if still open.

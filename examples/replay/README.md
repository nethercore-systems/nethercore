# Replay Smoke Scripts

This directory contains console-level replay scripts that can be used with any compatible ROM.

## ZX Sync-Test Gate

`zx-sync-test-300f-2p.ncrs` is a 300-frame, two-player smoke script for the shared Nethercore sync-test path. It does not depend on a specific game's menus or rules.

From a ZX game project directory:

```powershell
nether run --sync-test --players 2 --exit-after-frames 300 --replay ..\nethercore\examples\replay\zx-sync-test-300f-2p.ncrs
```

Use `--no-build` after the ROM is already built.

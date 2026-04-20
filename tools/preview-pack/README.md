# Nethercore ZX Tool Preview Pack

This directory owns the reusable source of truth for the Nethercore ZX tool preview.

## Files

- `manifest.json`: canonical review goals, curated examples, feedback questions, hotkeys, and source links.
- `assemble-preview-pack.ps1`: builds a shareable packet from installed example ROMs and the manifest.

## Local Use

From the `nethercore` repo root:

```powershell
cargo xtask build-examples
pwsh .\tools\preview-pack\assemble-preview-pack.ps1 -IncludeLocalTools
```

The packet is written to:

```text
target/tool-preview/nethercore-zx-tool-preview/
```

## CI Use

The `Tool Preview Packet` workflow builds the player/CLI, builds examples, assembles the packet, zips it, and uploads it as a GitHub Actions artifact.

## Website Use

The website should mirror `manifest.json` into its own data file or consume a generated JSON artifact. Do not hand-maintain a separate list of examples.

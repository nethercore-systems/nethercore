param(
    [string]$WorkspaceRoot = (Resolve-Path "$PSScriptRoot\..\..").Path,
    [string]$ManifestPath = (Join-Path $PSScriptRoot "manifest.json"),
    [string]$OutputDir = (Join-Path (Resolve-Path "$PSScriptRoot\..\..").Path "target\tool-preview\nethercore-zx-tool-preview"),
    [string]$DataRoot = (Join-Path $env:APPDATA "Nethercore\data\games"),
    [switch]$IncludeLocalTools
)

$ErrorActionPreference = "Stop"

function Write-Utf8File {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [AllowEmptyString()][string]$Content
    )

    $directory = Split-Path -Parent $Path
    if ($directory) {
        New-Item -ItemType Directory -Force -Path $directory | Out-Null
    }
    Set-Content -Encoding UTF8 -Path $Path -Value $Content
}

function Find-ExampleManifest {
    param(
        [Parameter(Mandatory = $true)][string]$ExampleId
    )

    $examplesRoot = Join-Path $WorkspaceRoot "examples"
    if (!(Test-Path $examplesRoot)) {
        return $null
    }

    return Get-ChildItem -Path $examplesRoot -Recurse -Filter "nether.toml" |
        Where-Object { $_.Directory.Name -eq $ExampleId } |
        Select-Object -First 1
}

$manifest = Get-Content -Raw -Path $ManifestPath | ConvertFrom-Json

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutputDir "docs") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutputDir "roms") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutputDir "replays") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $OutputDir "scripts") | Out-Null

Copy-Item -Force -Path $ManifestPath -Destination (Join-Path $OutputDir "manifest.json")

$exampleTable = ($manifest.examples | ForEach-Object {
    "| ``$($_.id)`` | $($_.category) | $($_.why) |"
}) -join "`n"

$hotkeyTable = ($manifest.hotkeys | ForEach-Object {
    "| $($_.key) | $($_.purpose) |"
}) -join "`n"

$firstRun = ($manifest.examples | Where-Object { $_.firstRun } | ForEach-Object {
    "1. Run ``$($_.id)`` - $($_.why)"
}) -join "`n"

$feedback = ($manifest.feedbackQuestions | ForEach-Object {
    "1. $_"
}) -join "`n"

$readme = @"
# $($manifest.title)

$($manifest.description)

This packet is for a technical review of Nethercore ZX as a game development runtime/toolchain. It is intentionally not a showcase of prototype fighting-game content.

## Fast Path

$firstRun
1. Try local multiplayer with ``paddle`` if you want the smallest rollback-oriented sample.
1. Skim ``docs/feedback-questions.md``.
1. Send blunt technical feedback.

## Launch Paths

This packet has two launch modes:

- Direct CLI launch: fastest path for reviewing specific ROMs.
- Library app: browse installed ROMs and use the GUI multiplayer flow.

The CLI path is the least ambiguous way to run a specific curated ROM. The library path is useful when you want the launcher experience.

## Direct CLI Launch

Use ``tools\nether.exe`` if this packet includes local tools. Otherwise put ``nether`` on PATH from the latest Nethercore release.

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\debug-demo
.\tools\nether.exe run --no-build --project .\roms\skinned-mesh
.\tools\nether.exe run --no-build --project .\roms\glb-inline
.\tools\nether.exe run --no-build --project .\roms\viewport-test
~~~

Each ROM folder contains ``<example-id>.nczx`` and a matching ``nether.toml``.

## Local Multiplayer

Local multiplayer is launched with ``--players``. This is the quick path for checking multi-player rollback/session behavior without the library UI.

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\paddle --players 2
.\tools\nether.exe run --no-build --project .\roms\viewport-test --players 4
.\tools\nether.exe run --no-build --project .\roms\paddle --sync-test --players 2
~~~

Local P2P smoke testing spawns two connected local player instances:

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\paddle --p2p-test --input-delay 2
~~~

## Library App

``tools\nethercore.exe`` launches the Nethercore Library GUI. It is not the player binary; it is the browser/launcher app. The CLI and the library both need ``tools\nethercore-zx.exe`` beside them to launch ZX games.

To make this packet appear in the library, install the ROMs into the local Nethercore data directory, then open the library:

~~~powershell
.\scripts\install-roms-to-library.ps1
.\tools\nethercore.exe
~~~

You can also use the library's import/open controls for individual ``.nczx`` files.

## Curated ROMs

| ROM | Category | Why it is included |
|---|---|---|
$exampleTable

## Hotkeys

| Key | Purpose |
|---|---|
$hotkeyTable

## What Not To Infer

$($manifest.nonGoals | ForEach-Object { "- $_" } | Out-String)
"@

Write-Utf8File -Path (Join-Path $OutputDir "README.md") -Content $readme

$overview = @"
# $($manifest.title)

$($manifest.description)

## Review Ask

$($manifest.reviewGoals | ForEach-Object { "- $_" } | Out-String)

## Evaluation Path

$firstRun

## Launch Paths

- Direct launch: ``.\tools\nether.exe run --no-build --project .\roms\debug-demo``
- Local multiplayer: ``.\tools\nether.exe run --no-build --project .\roms\paddle --players 2``
- Local P2P smoke: ``.\tools\nether.exe run --no-build --project .\roms\paddle --p2p-test --input-delay 2``
- Library browser: ``.\scripts\install-roms-to-library.ps1`` then ``.\tools\nethercore.exe``

## Non-Goals

$($manifest.nonGoals | ForEach-Object { "- $_" } | Out-String)
"@

Write-Utf8File -Path (Join-Path $OutputDir "docs\overview.md") -Content $overview
Write-Utf8File -Path (Join-Path $OutputDir "docs\feedback-questions.md") -Content "# Feedback Questions`n`n$feedback"

$libraryDoc = @'
# Library And Multiplayer

## Direct Launch

Use this when you want one exact ROM without touching local library state:

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\debug-demo
.\tools\nether.exe run --no-build --project .\roms\skinned-mesh
~~~

## Local Multiplayer

Use `--players` for local multiplayer in one player process:

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\paddle --players 2
.\tools\nether.exe run --no-build --project .\roms\viewport-test --players 4
~~~

Use `--sync-test` to exercise deterministic rollback checks:

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\paddle --sync-test --players 2
~~~

Use `--p2p-test` for a local network smoke test. This starts two connected local instances:

~~~powershell
.\tools\nether.exe run --no-build --project .\roms\paddle --p2p-test --input-delay 2
~~~

## Library App

`tools\nethercore.exe` is the library GUI. It browses installed games, launches games through the player binary, and exposes the GUI multiplayer flow. It is not the direct player binary.

The player binary for ZX games is `tools\nethercore-zx.exe`. Keep it beside `nether.exe` and `nethercore.exe`.

Install the packet ROMs into the local library:

~~~powershell
.\scripts\install-roms-to-library.ps1
.\tools\nethercore.exe
~~~

If the library is already open, press Refresh after installing.
'@

Write-Utf8File -Path (Join-Path $OutputDir "docs\library-and-multiplayer.md") -Content $libraryDoc

$installScript = @'
param(
    [string]$PacketRoot = (Resolve-Path "$PSScriptRoot\..").Path,
    [string]$DataRoot = (Join-Path $env:APPDATA "Nethercore\data\games")
)

$ErrorActionPreference = "Stop"

function Write-Utf8File {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [AllowEmptyString()][string]$Content
    )

    $directory = Split-Path -Parent $Path
    if ($directory) {
        New-Item -ItemType Directory -Force -Path $directory | Out-Null
    }
    Set-Content -Encoding UTF8 -Path $Path -Value $Content
}

$manifestPath = Join-Path $PacketRoot "manifest.json"
if (!(Test-Path $manifestPath)) {
    throw "Missing packet manifest: $manifestPath"
}

$manifest = Get-Content -Raw -Path $manifestPath | ConvertFrom-Json
New-Item -ItemType Directory -Force -Path $DataRoot | Out-Null

foreach ($example in $manifest.examples) {
    $id = $example.id
    $sourceRom = Join-Path $PacketRoot "roms\$id\$id.nczx"
    if (!(Test-Path $sourceRom)) {
        Write-Warning "Skipping $id because $sourceRom is missing"
        continue
    }

    $gameDir = Join-Path $DataRoot $id
    New-Item -ItemType Directory -Force -Path $gameDir | Out-Null
    Copy-Item -Force -Path $sourceRom -Destination (Join-Path $gameDir "rom.nczx")

    $localManifest = [ordered]@{
        id = $id
        title = $example.title
        author = "Nethercore Examples"
        version = "0.1.0"
        downloaded_at = (Get-Date).ToUniversalTime().ToString("o")
        console_type = "zx"
    } | ConvertTo-Json

    Write-Utf8File -Path (Join-Path $gameDir "manifest.json") -Content $localManifest
    Write-Host "Installed $id"
}

Write-Host "Installed preview ROMs to $DataRoot"
Write-Host "Open tools\nethercore.exe, or press Refresh if the library is already open."
'@

Write-Utf8File -Path (Join-Path $OutputDir "scripts\install-roms-to-library.ps1") -Content $installScript

$sendNote = @'
# Send Note

~~~text
Hey, thanks again for being willing to take a look.

I realized the useful thing is not to show you rough prototype fighting-game content. That is not representative. I put together a short Nethercore ZX tool preview instead:

[link]

The ask is: from a fighting-game developer's perspective, does this runtime/tooling foundation seem credible? I am especially interested in rollback constraints, debug workflow, asset/animation path, and what would make you bounce off.

The fastest path is the README. No need to review the whole repo.
~~~
'@

Write-Utf8File -Path (Join-Path $OutputDir "docs\send-note.md") -Content $sendNote

$videoDoc = @'
# 60-90 Second Video Shot List

Goal: show that the toolchain exists and is navigable. Do not present this as a game trailer.

1. `debug-demo` with F4 inspector open.
2. `skinned-mesh` or `multi-skinned-rom`.
3. `glb-inline`.
4. `viewport-test`.
5. `audio-demo` or `tracker-demo-xm`.
6. Quick terminal shot of `nether run` and the ROM folders.

Do not include DUEL//FRAME gameplay.
'@

Write-Utf8File -Path (Join-Path $OutputDir "docs\video-shot-list.md") -Content $videoDoc

$replay = @'
console = "zx"
seed = 4919
players = 1

[[frames]]
f = 0
p1 = "idle"
screenshot = true

[[frames]]
f = 30
p1 = "idle"
screenshot = true

[[frames]]
f = 45
p1 = "a"

[[frames]]
f = 46
p1 = "idle"

[[frames]]
f = 75
screenshot = true

[[frames]]
f = 120
screenshot = true
'@

Write-Utf8File -Path (Join-Path $OutputDir "replays\tool-preview-capture.ncrs") -Content $replay

foreach ($example in $manifest.examples) {
    $exampleId = $example.id
    $sourceRom = Join-Path $DataRoot "$exampleId\rom.nczx"
    if (!(Test-Path $sourceRom)) {
        Write-Warning "Missing installed ROM for $exampleId at $sourceRom"
        continue
    }

    $destDir = Join-Path $OutputDir "roms\$exampleId"
    New-Item -ItemType Directory -Force -Path $destDir | Out-Null
    Copy-Item -Force -Path $sourceRom -Destination (Join-Path $destDir "$exampleId.nczx")
    Copy-Item -Force -Path $sourceRom -Destination (Join-Path $destDir "rom.nczx")

    $exampleManifest = Find-ExampleManifest -ExampleId $exampleId
    if ($exampleManifest) {
        Copy-Item -Force -Path $exampleManifest.FullName -Destination (Join-Path $destDir "nether.toml")
    } else {
        $fallbackManifest = @(
            "[game]",
            "id = `"$exampleId`"",
            "title = `"$($example.title)`"",
            "author = `"Nethercore Examples`"",
            "version = `"0.1.0`"",
            "",
            "[build]",
            "rom = `"rom.nczx`""
        ) -join "`n"
        Write-Utf8File -Path (Join-Path $destDir "nether.toml") -Content $fallbackManifest
    }
}

if ($IncludeLocalTools) {
    $toolsDir = Join-Path $OutputDir "tools"
    New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null

    $toolNames = @("nether.exe", "nethercore.exe", "nethercore-zx.exe")
    foreach ($toolName in $toolNames) {
        $candidatePaths = @(
            (Join-Path $WorkspaceRoot "target\release\$toolName"),
            (Join-Path $WorkspaceRoot "target\debug\$toolName")
        )
        $candidates = $candidatePaths | Where-Object { Test-Path $_ } | ForEach-Object { Get-Item $_ } | Sort-Object LastWriteTimeUtc -Descending

        $selected = $candidates | Select-Object -First 1
        if ($selected) {
            Copy-Item -Force -Path $selected.FullName -Destination $toolsDir
            Write-Host "Included tool: $toolName from $($selected.FullName)"
        } else {
            Write-Warning "Missing local tool binary: $toolName"
        }
    }
}

Write-Host "Preview packet assembled at $OutputDir"

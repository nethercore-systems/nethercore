# Roster

This file is the mutable source of truth for the target 20-preset roster.

## Default Shipping Target

| # | Preset | Temp / Feel | Genre Fit | Status |
|---|--------|-------------|-----------|--------|
| 1 | Neon Metropolis | hot, synthetic | cyberpunk shooter | parked fail - reopened Wave 52 branch still failed after exact-image review |
| 2 | Sakura Shrine | cool, sacred | fantasy adventure | parked fail - reopened Wave 56 branch still failed after exact-image review |
| 3 | Ocean Depths | cold, wet | underwater exploration | parked fail - Wave 45 review still atmosphere-first / no trench-basin owner |
| 4 | Void Station | cold, industrial | sci-fi traversal | banked strong near-pass - the older bank still stands, but the newest authored reset also hard-failed on `void-station-liveiter-pass29`, regressing into a darker uniform shell around the same capped technical-shell probe |
| 5 | Desert Mirage | hot, dry | desert action RPG | banked near-pass survivor - the older bank still stands, but the newer rewritten source hard-failed on `desert-mirage-liveiter-pass6` as a brighter flatter brown-wash regression |
| 6 | Enchanted Grove | temperate, lush | fairy tale platformer | rebuild lane - both the bounds-first and newer structure-first source bases still hard-fail canopy arch / clearing floor / shaft-family read |
| 7 | Astral Void | cold, surreal | cosmic puzzle | banked near-pass survivor - the older bank still stands, but the newest rewritten source hard-failed on `astral-void-liveiter-pass6` in the same dark-orb ring/shaft class |
| 8 | Hell Core | hot, hostile | demonic action | pass - protected shipping lane |
| 9 | Sky Ruins | airy, ancient | sky-fantasy exploration | rebuild lane - both the terrace and newer masonry/court rewrites still hard-fail ruins silhouette / marble floor / cloud-depth read |
| 10 | Combat Lab | synthetic, tactical | combat sandbox | guard lane - proof-of-life pass |
| 11 | Frozen Tundra | cold, exposed | arctic survival | benchmark-blocked - depends on Frozen Bed |
| 12 | Storm Front | cold, wet, violent | weather drama | benchmark-blocked - depends on Front Mass |
| 13 | Crystal Cavern | cold, radiant | cave crawler | parked fail - shell-first source reset still failed live on cavern enclosure and stayed in the same open radial petal/slab class |
| 14 | Moonlit Graveyard | cold, gothic | horror | parked fail - reused live lane still produced no grave-marker owner and no saved-capture mist motion |
| 15 | Alien Jungle | hot, humid, uncanny | alien exploration | parked fail - repaired live lane is truthful, but this scene still stays static or falls into seam/tiling artifacts |
| 16 | Gothic Cathedral | cool, sacred interior | stealth / puzzle | parked fail - the artifact-heavy tracery branch is still blocked, and the authoritative locked path restores the same bright tracery / guide-ring blocker family |
| 17 | Toxic Wasteland | hot, corrupted | post-apocalypse | parked fail - `MASS` envelope suppresses the shell arc, but still does not create a coherent ruined exterior |
| 18 | Neon Arcade | synthetic, retro | arcade racer / shooter | parked fail - `MASS/SHELF` suppresses the spokes, but still does not create a cabinet-row owner |
| 19 | War Zone | hot, smoky | military shooter | parked fail - no tested owner beat baseline, and saved motion still stays static |
| 20 | Digital Matrix | synthetic, abstract | cyberspace fiction | banked strong near-pass - the stronger banked baseline still stands after the newer stronger `APERTURE/RECT` gate-owner move produced no gain, and it still banks truthfully through `digital-matrix-liveiter-pass31` |

## Editing Rules

- Change this roster only from the `design` or `orchestrate` packs.
- If you rename, swap, or drop a target preset, add a dated note below.
- Use `Status` for real progress, not vague optimism.

## Change Log

| Date | Change | Reason |
|------|--------|--------|
| 2026-03-10 | seeded 20-preset target roster | split from monolithic prompt into shared SSOT |
| 2026-03-10 | marked Frozen Tundra, Storm Front, Crystal Cavern, and Moonlit Graveyard as implementation active | first expansion batch selected and handed to implementation |
| 2026-03-11 | marked Frozen Tundra and Storm Front as implemented in code pending capture/review; left Crystal Cavern and Moonlit Graveyard as next implementation work | first implemented mini-batch landed and planning state was re-audited against code |
| 2026-03-13 | marked Combat Lab as the protected indoor proof-of-life pass and parked Crystal Cavern / Moonlit Graveyard behind the outdoor gate | roster reconciled to the benchmark-first runbook and current open-gaps truth |
| 2026-03-13 | converted all 20 presets to explicit orchestration lanes | durable roster now distinguishes guard, quick-fix, benchmark-blocked, rebuild, and new-implementation status |
| 2026-03-13 | marked presets 13-20 as implemented in code and moved them to first-pass rebuild status | the full 20-scene replay now matches code, but the first integrated capture shows the new scenes are not showcase-ready yet |
| 2026-03-14 | reconciled roster statuses through Wave 52 | Hell Core and Combat Lab are protected passes, Void Station and Digital Matrix are banked strong near-pass stop lanes, Astral Void is banked near-pass, Ocean Depths remains parked fail, and Neon Metropolis reopened with review pending |
| 2026-03-14 | reconciled roster statuses through Wave 55 | Neon Metropolis reopened and validated cleanly through Waves 53-55 but still failed exact-image review, so it is parked again and Sakura Shrine becomes the next active survivor lane |
| 2026-03-14 | reconciled roster statuses through Wave 60 | Sakura Shrine reopened and validated cleanly through Wave 59 but still failed exact-image review, so it is parked again and Void Station becomes the reopened active review lane on a clean Wave 60 pair |
| 2026-03-14 | reconciled roster statuses through Wave 62 | Void Station held near-pass through Waves 61-62; Wave 62 is banked as the best seam-branch state and the lane returns to stop status due to diminishing returns |
| 2026-03-14 | reconciled roster statuses through Wave 64 | Digital Matrix held near-pass through Waves 63-64; Wave 64 did not materially beat the same radial-fan ownership limit, so the lane is banked again and Desert Mirage becomes the next active lane |
| 2026-03-14 | reconciled roster statuses through Wave 69 | Desert Mirage progressed from fail toward a banked near-pass survivor on the dune-horizon plus dust-suppression branch; Wave 69 is the current bank point and Enchanted Grove becomes the next active lane in `set_05_06.rs` |
| 2026-03-14 | reconciled roster statuses after the Wave 60 exact-image review | Void Station reopened on a clean Wave 60 pair but remained near-pass on exact-image review, so it is banked again as a strong near-pass stop lane |
| 2026-03-15 | reconciled the Wave 70 shared-artifact review | Enchanted Grove still failed, Sky Ruins becomes the next active rebuild lane, Crystal Cavern upgraded to a directional near-pass rebuild candidate, and Moonlit Graveyard remains a first-pass fail |
| 2026-03-15 | reconciled the Wave 71 Sky Ruins review and Frozen Bed benchmark follow-up | Sky Ruins failed its first rebuild pass and is blocked again, Crystal Cavern becomes the strongest next rebuild candidate, and the promoted Frozen Bed benchmark replay is closed as a water-read fail |
| 2026-03-15 | reconciled the Wave 72 Crystal review plus benchmark sidecars | Crystal Cavern still failed on enclosure/depth and now has one final truthful pass left before rotation, while both Front Mass and Frozen Bed sidecar sweeps closed without replay-worthy promotions |
| 2026-03-15 | reconciled the Wave 73 Crystal stop-loss review and lane rotation | Crystal Cavern failed its final allowed pass and is now rotated out as a parked fail; Alien Jungle becomes the next active rebuild lane |
| 2026-03-15 | reconciled the Alien Jungle live-motion diagnostics | Alien Jungle remains the active rebuild lane, but repeated live diagnostics on the reused `4581` lane still failed to prove motion, so replay stays blocked while the workbench animation-proof path is isolated |
| 2026-03-15 | reconciled the Alien Jungle live-path repair | the `4581` workbench lane can capture real motion again after preserving live phase in locked editor overrides, but Alien Jungle itself still captures static frames, so the blocker narrows back to scene-level authoring rather than a globally dead live path |
| 2026-03-15 | reconciled the Alien Jungle live direct-green sidecar | Alien Jungle remains the active rebuild lane, but the strongest live result (`alien-jungle-direct-k3-silhouette-green`) is still replay-blocked because direct/background motion stayed hash-identical after an ~8 second gap and the canopy owner is still too soft |
| 2026-03-15 | rotated the active live rebuild lane from Alien Jungle to Moonlit Graveyard | the repaired `4581` lane proved truthful, but Alien Jungle still stayed static or became visibly artifact-worse, so Moonlit Graveyard is now the next active workbench rebuild lane |
| 2026-03-15 | rotated the active live rebuild lane from Moonlit Graveyard to Gothic Cathedral | the reused `4581` lane stayed healthy, but Moonlit Graveyard still had no unmistakable grave markers and no saved-capture mist motion, so the next truthful live rebuild lane is Gothic Cathedral |
| 2026-03-15 | reconciled the first Gothic Cathedral live pass | the reused `4581` lane found a real cathedral-specific tracery owner with slight saved-capture motion, but visible seam/ring artifacts and missing stained-light support still block replay promotion, so the lane stays active |
| 2026-03-15 | reconciled the Gothic Cathedral live follow-up pass | the cathedral owner still survives, but editor cleanup and more `CELL` tuning did not fix the artifact blocker, so the next truthful pass must change owner family instead of churning the same geometry |
| 2026-03-15 | rotated the active live rebuild lane from Gothic Cathedral to Toxic Wasteland | the owner-family pass closed Gothic Cathedral truthfully as parked, so Toxic Wasteland is now the next active live rebuild lane on the reused workbench |
| 2026-03-15 | rotated the active live rebuild lane from Toxic Wasteland to Neon Arcade | the `MASS` envelope pass still failed to produce a coherent ruined exterior, so Toxic Wasteland is parked and Neon Arcade becomes the next active live rebuild lane |
| 2026-03-15 | rotated the active live rebuild lane from Neon Arcade to War Zone | the `MASS/SHELF` arcade pass still failed to produce a cabinet-row owner, so Neon Arcade is parked and War Zone becomes the next active live rebuild lane |
| 2026-03-15 | parked the War Zone live rebuild lane | no tested owner beat baseline and the restored baseline still stayed motion-static in saved captures, so the remaining `13-20` live rebuild sweep is now fully walked and durably parked/banked |
| 2026-03-15 | re-banked Astral Void after fresh rebuilt-lane validation | the refreshed `4581` lane on PID `27440` did export real full/probe/background deltas after two live state edits and an 8-second motion pair, but the direct-background change stayed too weak to justify reopening it as an active polish lane |
| 2026-03-15 | reconciled the Crystal source reset and fourth artifact-pass live recheck | Crystal Cavern still failed truthfully after a shell-first source rewrite, Gothic Cathedral still did not materially improve after the next shared artifact pass, and the reused live lane was cleanly rebuilt/relaunched onto PID `72072` |
| 2026-03-15 | re-banked Digital Matrix after rebuilt-lane live iteration | three targeted live branches still failed to create a shippable direct-view matrix owner, so the lane remains a banked strong near-pass stop |
| 2026-03-15 | reconciled the fifth shared artifact pass | Front Mass and Gothic Cathedral both changed truthfully on the rebuilt lane but remained in the same blocker classes, so the live lane was rebuilt/relaunched again onto PID `69512` without reopening either lane |
| 2026-03-15 | reconciled the new benchmark/showcase rewrites on the reused live lane | Enchanted Grove improved again but stayed blocked, while both Front Mass and Frozen Bed produced replay-worthy live baselines that still failed under fresh authoritative benchmark review |
| 2026-03-15 | reconciled the latest Frozen Bed and Sky Ruins rewrite passes | the second Frozen Bed rewrite regressed before replay, and the new Sky Ruins source rewrite is build-valid but still needs trustworthy live validation because the first post-switch capture came back effectively identical to Frozen Bed |
| 2026-03-15 | closed the Sky Ruins scene-switch diagnostic and reran live validation | the reused lane can produce scene-distinct Sky Ruins captures, but the rewritten source still hard-fails on the actual terrace/colonnade/floor/cloud brief |
| 2026-03-15 | closed the new Void Station and Digital Matrix authored resets | both aggressive source rewrites built cleanly, but the live lane still hard-failed them immediately, so neither lane reopened beyond its banked near-pass stop state |
| 2026-03-15 | closed the new Astral Void and Ocean Depths authored resets | both aggressive source rewrites built cleanly, but the live lane still hard-failed them immediately, so neither lane reopened beyond its current banked/parked state |
| 2026-03-15 | closed the new Neon Metropolis and Sakura Shrine authored resets | both aggressive source rewrites built cleanly, but the live lane still hard-failed them immediately, so neither parked fail lane reopened |
| 2026-03-15 | closed the new Toxic Wasteland and War Zone authored resets | both aggressive source rewrites built cleanly, but the live lane still hard-failed them immediately, so neither parked fail lane reopened |
| 2026-03-15 | closed the new Moonlit Graveyard and Gothic Cathedral authored resets | both aggressive source rewrites built cleanly, but the live lane still hard-failed them immediately, so neither parked fail lane reopened |
| 2026-03-15 | reconciled the latest live/source/replay wave | Void Station and Digital Matrix kept their older banked near-pass status even though the new authored resets also hard-failed live, Enchanted Grove stayed blocked after another live hard fail, Front Mass failed a second same-day authoritative benchmark replay, and Gothic Cathedral regressed under the newest artifact validation pass |
| 2026-03-15 | reconciled the latest benchmark/showcase/artifact semantics wave | Frozen Bed stayed blocked after a new grounded rewrite still hard-failed live, Desert Mirage stayed banked near-pass, Enchanted Grove and Sky Ruins both hard-failed new source rewrites on the rebuilt lane, and Gothic Cathedral's apparent pale-chamber regression was narrowed to unlocked workbench semantics while the authoritative locked path still shows the same old blocker family |
| 2026-03-15 | synced the latest Void Station, Desert Mirage, Front Mass, and artifact-pass state | Void Station's newer sector-box rewrite still hard-failed on the reused lane, Desert Mirage remained a banked near-pass after another unchanged live pass, Front Mass's dark coast-under-squall benchmark still hard-failed live, and the deeper artifact pass is landed but not yet live-validated on Gothic Cathedral |
| 2026-03-15 | synced the latest Void Station live wave | the new split-face rear-bulkhead direction built and replay-validated cleanly, then banked truthfully as a near-pass on `void-station-liveiter-pass7` after a same-lane relaunch cleared stale content |
| 2026-03-15 | synced the latest Digital Matrix live wave | the new box-chamber rewrite built and replay-validated cleanly, but `digital-matrix-liveiter-pass6` still hard-failed after two truthful rebalances and a same-lane relaunch to clear stale pre-reset content |
| 2026-03-15 | synced the latest Digital Matrix partition-owner wave | the new half-space partition rewrite built and replay-validated cleanly, then banked truthfully as a near-pass on `digital-matrix-liveiter-pass7` after a same-lane relaunch loaded the new WASM |
| 2026-03-15 | synced the latest Desert Mirage rewrite wave | the rewritten Desert Mirage source built and replay-validated cleanly, but `desert-mirage-liveiter-pass6` hard-failed as a brighter flatter brown-wash regression, so the older banked near-pass survivor remains the truthful stop state |

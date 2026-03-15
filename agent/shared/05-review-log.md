# Review Log

Append-only adversarial review history.

Do not silently rewrite prior verdicts. Add a new dated entry if a preset is re-reviewed.

## Scorecard Template

Use this format for each review entry:

```text
Preset:
Date:
Capture Run:
Frames Reviewed:
Brief Match:
Missed Brief Reads:
Hard-Fail Triggers:
text prompt fidelity:
visual identity:
depth / composition:
reflection / lighting read:
technical cleanliness:
animation quality:
loop quality or loopability readiness:
novelty versus roster:
verdict:
notes:
```

## Entries

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-02`
Frames Reviewed: provided screenshots 1-3 from the newest six local images
visual identity: 7/10
depth / composition: 6/10
reflection / lighting read: 6/10
technical cleanliness: 5/10
animation quality: 6/10
novelty versus roster: 8/10
verdict: fail
notes: Blizzard mood reads immediately, but the authored glacier ridge and polished ice plane do not. The scene collapses into a soft blue-white whiteout with weak sky/floor separation, the bright right-edge column reads like an artifact or seam, and the three reviewed frames do not show convincing surface-sheen or snow-drift motion.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-02`
Frames Reviewed: provided screenshots 4-6 from the newest six local images
visual identity: 8/10
depth / composition: 7/10
reflection / lighting read: 8/10
technical cleanliness: 6/10
animation quality: 7/10
novelty versus roster: 8/10
verdict: fail
notes: The storm mood is recognizable, but the readable sea horizon promised by the preset does not survive capture. Lightning is strongest only as a reflection on the hero sphere, the right-side rain wall reads as repetitive stamped circles instead of a natural squall sheet, and the three reviewed frames do not sell violent weather motion or striking frame-to-frame change.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-03`
Frames Reviewed: provided screenshots 1-3 from the newest six local images
visual identity: 8/10
depth / composition: 6/10
reflection / lighting read: 7/10
technical cleanliness: 7/10
animation quality: 6/10
novelty versus roster: 8/10
verdict: fail
notes: The repair clears the earlier bright edge artifact, but the scene still collapses into a dense blue-white blizzard instead of exposing the promised glacier ridge and ice-plane separation. Across all three frames the snowfall drift is mild, the surface sheen remains hard to locate in the world, and the low-sun lighting reads more on the hero sphere than in the environment, so it still misses showcase clarity.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-03`
Frames Reviewed: provided screenshots 4-6 from the newest six local images
visual identity: 8/10
depth / composition: 6/10
reflection / lighting read: 7/10
technical cleanliness: 6/10
animation quality: 5/10
novelty versus roster: 8/10
verdict: fail
notes: The rerender still does not produce a readable sea horizon or a background lightning event; the strike reads mainly as a reflection on the hero sphere. The scene is dominated by concentric contour-like rain-wall patterning and clipped bright masses at the top edge, and the three frames show too little weather evolution to sell a violent squall.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-04`
Frames Reviewed: provided screenshots 1-3 from the newest six local images
visual identity: 6/10
depth / composition: 4/10
reflection / lighting read: 5/10
technical cleanliness: 3/10
animation quality: 1/10
novelty versus roster: 5/10
verdict: fail
notes: Arctic intent is still recognizable, but the promised glacier ridge and ice sheet do not survive capture. The frame is dominated by looping/repeated concentric banding plus fixed snow-speckle, the world reads as one dark blue bowl around the sphere, and screenshots 1-3 do not show obvious snow drift, sheen travel, or halo motion. Because the same banding and apparent animation stall also show up in `Storm Front`, this now reads as a suspected engine/EPU issue, not only a preset-tuning miss.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-04`
Frames Reviewed: provided screenshots 4-6 from the newest six local images
visual identity: 7/10
depth / composition: 4/10
reflection / lighting read: 5/10
technical cleanliness: 3/10
animation quality: 1/10
novelty versus roster: 5/10
verdict: fail
notes: The palette and sphere reflection suggest storm weather, but the background scene does not hold up. The sea horizon is buried, lightning reads only as a reflected streak on the hero sphere instead of a scene event, the walls and floor collapse into the same looping/repeated concentric patterning seen in `Frozen Tundra`, and screenshots 4-6 show no obvious rain, wave, or lightning evolution. Cross-preset persistence makes this a suspected engine/EPU bug until isolated.

Preset: `Neon Metropolis`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 01-03
Brief Match: cyberpunk night palette and faint neon accents survive
Missed Brief Reads: city or alley silhouette, hero neon sign, wet reflective pavement, readable rain support
Hard-Fail Triggers: reads as abstract magenta fog instead of a city alley; no dominant sign; skyline disappears into noise
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 4/10
animation quality: 1/10
novelty versus roster: 3/10
verdict: fail
notes: All three frames are dominated by muddy purple haze with three top-edge light spikes, not an alley or skyline. The scene never establishes pavement, windows, or one hero sign, and the frame-to-frame change is too small to count as rain or shimmer motion.

Preset: `Sky Ruins`
Date: 2026-03-15
Capture Run: `20260315-114532-sky-ruins-rebuild`
Frames Reviewed: authoritative full-sweep frames 25-27 from `agent/runs/20260315-114532-sky-ruins-rebuild/screenshots/single`
Brief Match: warm stone palette and generalized ruin atmosphere survive
Missed Brief Reads: readable ruined skyline, marble platform floor, layered cloud-bank depth, open-air grandeur
Hard-Fail Triggers: direct view collapses into soft beige slab/column masses; probe only confirms generic stone breakup; no decisive ruin silhouette or cloud-depth hierarchy emerges
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 3/10
novelty versus roster: 4/10
verdict: fail
notes: The rebuilt third-pass shader binary does not materially change the lane. Review also found no protected-lane regression in `Hell Core` or `Combat Lab`, and only a slight cosmetic cleanup in `Digital Matrix`, so the shipping set remains materially unchanged.

Preset: `Sakura Shrine`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 04-06
Brief Match: warm dusk palette and pink petal color are present
Missed Brief Reads: shrine or pagoda silhouette, mossy stone path, sacred haze, controlled petal accent
Hard-Fail Triggers: reads as generic warm fantasy instead of shrine grounds; petals dominate like noise; floor no longer reads as stone path
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 4/10
animation quality: 1/10
novelty versus roster: 3/10
verdict: fail
notes: The capture is basically a brown field filled with evenly scattered pink dots. There is no shrine approach, no readable floor material, and no visible motion arc beyond tiny speckle changes that do not satisfy the petal-drift contract.

Preset: `Ocean Depths`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 07-09
Brief Match: underwater blue gradient, suspended particulate, and overhead light shafts read immediately
Missed Brief Reads: stronger trench or seabed anchor, clearer bioluminescent accent, more visible caustic or bubble motion
Hard-Fail Triggers: none
visual identity: 5/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 2/10
novelty versus roster: 5/10
verdict: fail
notes: This is one of the few presets that still reads as its named place on first glance, but it does not reach showcase quality. The abyss-to-surface gradient works, yet the trench floor stays too weak to anchor the frame and the three captures show only tiny deltas instead of obvious drifting underwater motion.

Preset: `Void Station`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 10-12
Brief Match: cool industrial palette and a rounded top opening hint survive
Missed Brief Reads: enclosed room bounds, one clear viewport, readable eclipse, deck spill connecting the window to the room
Hard-Fail Triggers: interior bounds collapse and the room reads as open space; the eclipse or viewport opening is not clearly legible; wall or floor separation is muddy
visual identity: 3/10
depth / composition: 3/10
reflection / lighting read: 3/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The frame suggests a cold sci-fi room only in the loosest sense. What should be a viewport scene instead reads like a dim blue chamber with an unreadable star panel at the top, and the three images are effectively static.

Preset: `Desert Mirage`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 13-15
Brief Match: hot tan atmosphere and airborne dust are present
Missed Brief Reads: dune horizon, textured sand floor, mirage pool, white-hot horizon shimmer
Hard-Fail Triggers: no dune horizon; mirage read is absent and the floor is just flat tan noise
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The capture reads as a monochrome tan dust cloud with no convincing desert geography. The required dune line and false-water mirage never materialize, and the frame-to-frame differences are far too weak to sell heat shimmer.

Preset: `Enchanted Grove`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 16-18
Brief Match: warm green atmosphere and glowing motes are visible
Missed Brief Reads: tree canopy, mossy clearing floor, shafts of sunlight, restrained firefly accents
Hard-Fail Triggers: reads as generic green fog instead of a forest clearing; no clear light-shaft structure; fireflies overwhelm the space
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The scene is just green haze plus bright dots. It never establishes a grove, canopy, or floor plane, and the motes read as a uniform particle blanket rather than sparse magical accents.

Preset: `Astral Void`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 19-21
Brief Match: dark space and dense stars read immediately
Missed Brief Reads: one large eclipsed body, a secondary moon, restrained nebular drift, compositional hierarchy
Hard-Fail Triggers: hero celestial body is missing or lost
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
novelty versus roster: 5/10
verdict: fail
notes: It is a starfield, but not the composed cosmic tableau promised by the brief. The stars overwhelm the frame, the required celestial bodies never resolve cleanly, and there is no visible nebular drift across the three frames.

Preset: `Hell Core`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 22-24
Brief Match: infernal orange-black palette and a low heat glow survive
Missed Brief Reads: cracked volcanic foundation, dominant lava fissures, readable lower hellgate structure
Hard-Fail Triggers: cracks are not the main read
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 7/10
novelty versus roster: 5/10
verdict: fail
notes: Minimal motion is fine here, but the scene still fails because it reads as soft orange cellular abstraction instead of shattered volcanic ground. The lava fissure language never becomes the primary focal structure, which is the whole contract.

Preset: `Sky Ruins`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 25-27
Brief Match: pale airy palette and some architectural line-work survive
Missed Brief Reads: ruined colonnade silhouette, marble platform floor, sunlit clouds, elevated open-air grandeur
Hard-Fail Triggers: no clear ruins read; the scene loses the elevated open-air feeling
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The frame looks like a pale gray chamber with dotted guide arcs, not floating ruins above blazing clouds. Whatever ruin geometry exists in the authoring does not survive capture, and the sequence is visually near-static.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 28-30
Brief Match: sterile interior, cyan grid language, and bright clinical lighting are present
Missed Brief Reads: rectangular holo volume, distinct HUD or card panels, stronger room-bound separation
Hard-Fail Triggers: none
visual identity: 6/10
depth / composition: 5/10
reflection / lighting read: 5/10
technical cleanliness: 6/10
animation quality: 2/10
novelty versus roster: 6/10
verdict: fail
notes: This is the closest current preset to a pass because it does read as a clean sci-fi training space. It still misses the showcase bar because the holographic props are too weak to register, the exposure is washed enough to flatten structure, and the three review frames do not show obvious scan or pulse advancement.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 31-33
Brief Match: cold blue palette, snow speckle, and polar light survive
Missed Brief Reads: glacier ridge, readable ice-sheet floor, crisp sky-ground separation, controlled snow drift
Hard-Fail Triggers: ridge and floor separation disappear; concentric ringing or bowl-like floor artifacts dominate the frame; snow becomes static speckle
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 2/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The run08 deterministic baseline confirms this is still a real visual defect, not a capture glitch. The scene is overwhelmed by concentric contouring and bowl-like striping, the glacier horizon is not readable, and the snow field looks nearly frozen between frames.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-08`
Frames Reviewed: authoritative deterministic 05:52 batch frames 34-36
Brief Match: bruised storm palette, a dark upper shelf, and reflective storm light survive
Missed Brief Reads: clear sea horizon, readable stormwater floor, world-space lightning event, convincing driven rain motion
Hard-Fail Triggers: horizon is buried; looping rings or repeated contour patterning dominate walls or floor; lightning reads only in reflection and not in the world
visual identity: 4/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 2/10
animation quality: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The deterministic batch removes the replay excuse and leaves the core problem exposed. This is still a contour-ring storm bowl with no stable sea horizon, and the only clear lightning-like event is a reflection highlight on the sphere rather than a readable strike in the world.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `authoritative deterministic 07:37 batch (paired against 07:36 with 0 hash mismatches)`
Frames Reviewed: authoritative deterministic 07:37 batch frames 28-30
Brief Match: clinical interior room, cyan floor grid, and bright lab lighting survive
Missed Brief Reads: readable holographic HUD panel, unmistakable rectangular holo volume, stronger wall-tech structure, obvious frame-to-frame scan or pulse motion
Hard-Fail Triggers: floor grid or holo cards are unreadable
visual identity: 6/10
depth / composition: 5/10
reflection / lighting read: 5/10
technical cleanliness: 6/10
animation quality: 2/10
novelty versus roster: 6/10
verdict: fail
notes: This is directionally better than run08 because the floor grid is finally a real read and the top bloom is less dominant, but it still does not cross the showcase bar. The room remains a pale gray chamber with only faint wall tech, the holographic anchors are too weak to carry the scene in world space, and frames 28-30 show only minor sweep changes rather than obvious scanner or holo motion.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `authoritative deterministic 07:37 batch (paired against 07:36 with 0 hash mismatches)`
Frames Reviewed: authoritative deterministic 07:37 batch frames 31-33
Brief Match: cold blue palette, snow support, and a distant ridge silhouette survive
Missed Brief Reads: polished blue ice floor, crisp sky-ground separation, low polar glow as a scene event, distinct arctic identity versus the storm preset
Hard-Fail Triggers: snow becomes static speckle
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 2/10
verdict: fail
notes: The worst concentric bowl artifact is no longer the main issue, but the replacement still fails the brief. The frame reads as a dark blue weather bowl with evenly distributed white speckle, not as polished ice under low arctic light, and the three review frames show no obvious sheen travel or snow drift. Most importantly, it is now too close to `Storm Front` in palette, silhouette, and scene structure, which is an automatic roster-level distinctness failure.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `authoritative deterministic 07:37 batch (paired against 07:36 with 0 hash mismatches)`
Frames Reviewed: authoritative deterministic 07:37 batch frames 34-36
Brief Match: dark storm shelf, black-water palette, and a readable upper-vs-lower split survive
Missed Brief Reads: convincing driven rain or squall structure, world-space lightning event, stronger storm cadence, clear separation from `Frozen Tundra`
Hard-Fail Triggers: lightning reads only in reflection and not in the world
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
novelty versus roster: 2/10
verdict: fail
notes: The scene is no longer dominated by the earlier contour-ring bug, but it still does not read as a violent squall line. The only visible change across frames is minor speckle shift, there is still no meaningful world-space lightning event, and the image now lands uncomfortably close to `Frozen Tundra` as another dark blue speckled weather scene. That distinctness collapse is now one of the main blockers.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-12` (paired against run11 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run12 batch frames 28-30
Brief Match: clinical interior room, cyan floor grid, bright fluorescent lighting, and some wall-scan language survive
Missed Brief Reads: readable holographic HUD cards in world space, unmistakable rectangular holo volume, stronger wall-tech structure, obvious scan or pulse advancement across all three frames
Hard-Fail Triggers: holographic cards remain effectively unreadable
text prompt fidelity: 6/10
visual identity: 7/10
depth / composition: 6/10
reflection / lighting read: 6/10
technical cleanliness: 8/10
animation quality: 4/10
loop quality or loopability readiness: 6/10
novelty versus roster: 7/10
verdict: fail
notes: This remains one of the clearer current presets and its roster-level distinctness survives. The room, grid, and bright lab lighting are readable, and the moving wall rectangles do show some authored motion, but the scene still fails the brief because the holographic anchors are too faint to register as hero lab elements and the frame-to-frame sweep is not obvious enough to clear the motion bar.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-12` (paired against run11 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run12 batch frames 31-33
Brief Match: the reauthored high-key arctic palette, dry cold air, and a distant ridge line do survive
Missed Brief Reads: hard glacier ridge, polished blue ice sheet underfoot, stronger low polar sunlight event, visible drifting snow or sheen travel
Hard-Fail Triggers: the floor still reads as a smooth blue bowl around the sphere instead of a readable ice sheet
text prompt fidelity: 5/10
visual identity: 6/10
depth / composition: 5/10
reflection / lighting read: 5/10
technical cleanliness: 7/10
animation quality: 1/10
loop quality or loopability readiness: 4/10
novelty versus roster: 7/10
verdict: fail
notes: The latest content state does verify the intended shift away from the older dark speckled storm-bowl look: this now reads brighter, drier, and more arctic than the prior failed batch. It still misses the brief because the foreground remains a soft curved blue field rather than polished ice, the ridge is too gentle to anchor the place, and frames 31-33 show no obvious snow drift or ice-sheen motion. This reads as a content-side miss, not a renderer/EPU failure.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-12` (paired against run11 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run12 batch frames 34-36
Brief Match: low-key bruised storm light and black-water mood survive mainly in the sphere reflection
Missed Brief Reads: readable sea horizon, dark stormwater floor in direct view, driven rain or squall structure, lightning as a world-space event, obvious storm cadence across frames
Hard-Fail Triggers: horizon is buried and lightning reads only in reflection, not in the world
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 1/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 5/10
verdict: fail
notes: The new low-key direction is distinct from `Frozen Tundra`, but it still fails decisively because almost all of the useful storm information is trapped in the sphere reflection while the direct environment collapses into a near-empty gray field. There is no readable horizon, no visible rain wall, and no world-space lightning strike, so this is a content-read failure rather than a renderer bug call.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-14` (paired against run13 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run14 batch frames 28-30
Brief Match: clinical interior room, bright cyan floor grid, harsh fluorescent wash, and moving wall-scan slabs survive
Missed Brief Reads: readable holographic HUD panels in direct world space, unmistakable rectangular holo volume, stronger wall-tech structure, motion that reads instantly without close comparison
Hard-Fail Triggers: holo cards or display volume remain unreadable as hero lab elements
text prompt fidelity: 6/10
visual identity: 7/10
depth / composition: 6/10
reflection / lighting read: 6/10
technical cleanliness: 8/10
animation quality: 5/10
loop quality or loopability readiness: 6/10
novelty versus roster: 7/10
verdict: fail
notes: Relative to the run12 verdict, motion is marginally easier to catch because the translucent wall panels step across the room more clearly, but the improvement is not material. Readability stays effectively failing because the scene still lands as a gray calibration chamber instead of a hero training room and the holo cards plus rectangular display volume never become direct-view anchors. Roster-level distinctness still survives versus the rest of the set, especially `Void Station`, but that is not enough to clear the prompt or motion bar.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-14` (paired against run13 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run14 batch frames 31-33
Brief Match: high-key arctic palette, dry cold air, and a distant glacier band survive
Missed Brief Reads: hard glacier ridge, polished blue ice sheet underfoot, stronger low polar sunlight event, visible drifting snow, obvious ice-sheen travel
Hard-Fail Triggers: the foreground still reads as a bowl-like blue field around the sphere instead of a readable ice sheet
text prompt fidelity: 5/10
visual identity: 6/10
depth / composition: 5/10
reflection / lighting read: 5/10
technical cleanliness: 7/10
animation quality: 2/10
loop quality or loopability readiness: 4/10
novelty versus roster: 7/10
verdict: fail
notes: Relative to the run12 verdict, this stays effectively failing. The brighter, drier arctic split from `Storm Front` still survives, but the speed-table change does not produce a meaningful review-level motion gain: frames 31-33 do not show obvious snow drift or sheen travel, and the foreground remains a soft blue bowl instead of polished ice. Readability is essentially unchanged and still below the preset brief.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-14` (paired against run13 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run14 batch frames 34-36
Brief Match: low-key bruised storm light and black-water mood survive only inside the sphere reflection
Missed Brief Reads: readable sea horizon, direct-view stormwater floor, visible rain or squall structure, world-space lightning event, convincing weather cadence across frames
Hard-Fail Triggers: horizon remains buried and lightning still reads only in reflection, not in the world
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 1/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 5/10
verdict: fail
notes: Relative to the run12 verdict, this is not materially better. The speed-table tweak does not surface a storm scene outside the sphere reflection, and frames 34-36 are effectively frozen, so both motion and direct readability stay effectively failing. The earlier distinctness collision with `Frozen Tundra` remains resolved because this stays darker and emptier, but that is not a positive read; the preset still misses nearly every direct-view requirement in its text prompt.

Preset: `Ocean Depths`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-19` (paired against run18 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run19 batch frames 07-09
Brief Match: cold underwater palette, suspended particulate, and a faint abyss-to-surface direction still survive
Missed Brief Reads: readable trench or basalt seabed anchor, convincing surface glow hierarchy, bioluminescent hero accent, motion that reads as underwater drift instead of near-static shimmer
Hard-Fail Triggers: the scene collapses into abstract teal debris and particle fog rather than a legible deep-sea trench
text prompt fidelity: 3/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 3/10
animation quality: 2/10
loop quality or loopability readiness: 2/10
novelty versus roster: 5/10
verdict: fail
notes: This pass overcorrects the prior review note. The added structural layer reads as large floating fragments instead of a trench shelf, the floor never resolves into a dark seabed anchor, and the three review frames still do not produce obvious underwater drift. The preset is now less legible as a place than the earlier baseline, so the next loop should retreat from the current `PATCHES`-heavy read and rebuild around a clearer trench silhouette.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-19` (paired against run18 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run19 batch frames 28-30
Brief Match: clinical interior bounds, strong cyan floor grid, bright lab lighting, broader scanner-bar motion, and a more legible central holo slab survive
Missed Brief Reads: unmistakable wall-mounted HUD cards in direct world space, a hero rectangular holo volume that dominates outside reflection, stronger contrast between framed display bay and surrounding room shell
Hard-Fail Triggers: none, but the preset still undershoots the direct-view holo-card requirement
text prompt fidelity: 7/10
visual identity: 7/10
depth / composition: 6/10
reflection / lighting read: 7/10
technical cleanliness: 8/10
animation quality: 7/10
loop quality or loopability readiness: 7/10
novelty versus roster: 7/10
verdict: fail
notes: This is a real improvement over the run16 baseline. The floor grid now reads immediately, the scanner slabs advance clearly across frames, and the room no longer feels as close to a washed calibration chamber. It still does not clear the showcase bar because the most important holographic read remains borderline: the central cyan slab is present, but the wall HUDs and rectangular display volume still rely too much on reflection and do not yet dominate the direct scene as hero lab elements.

Preset: `Ocean Depths`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-21` (paired against run20 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run21 batch frames 07-09
Brief Match: underwater palette, falling particulate, and broad overhead shafts survive again
Missed Brief Reads: readable trench-floor anchor, basalt seabed structure, bioluminescent hero accent, obvious underwater drift across frames
Hard-Fail Triggers: no trench or seabed read; the scene lands as generic blue underwater light instead of a deep-sea trench
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 2/10
loop quality or loopability readiness: 3/10
novelty versus roster: 5/10
verdict: fail
notes: This is cleaner than run19 and recovers the broad underwater mood, but it still does not read as a trench with a real seabed. The floor never resolves into a dark basalt anchor, the biolum event is too weak to become a hero focal point, and the three review frames remain effectively static. The current lesson is that the preset is easier to keep readable when it stays close to the original shaft-based composition, but it still needs a more explicit floor or horizon structure and a much stronger motion beat.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-21` (paired against run20 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run21 batch frames 28-30
Brief Match: sterile room bounds, strong cyan floor grid, obvious scanner-slab motion, and a clearer framed wall bay behind the hero object survive
Missed Brief Reads: unmistakable side HUD cards, a rectangular holo volume that dominates in direct view rather than reflection, stronger high-tech contrast between display bay and room shell
Hard-Fail Triggers: none, but the direct-view holo requirement is still below showcase bar
text prompt fidelity: 8/10
visual identity: 8/10
depth / composition: 7/10
reflection / lighting read: 7/10
technical cleanliness: 8/10
animation quality: 7/10
loop quality or loopability readiness: 7/10
novelty versus roster: 7/10
verdict: fail
notes: This is the strongest `Combat Lab` state so far. The framed bay is a real improvement, the grid and scan cadence hold up at a glance, and the central cyan slab is easier to read without relying entirely on reflection. It still fails under the showcase bar because the holo read is not yet emphatic enough: the side HUD elements remain faint, the rectangular display volume still does not dominate the direct scene as a hero prop, and the room overall is still a little too flat and gray for a final docs or marketing shot.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `run23` (provided frames 28-30 only)
Frames Reviewed: provided run23 Combat Lab frames 28-30
Brief Match: clinical interior room bounds, bright cyan floor grid, framed rear display bay, and harsh fluorescent sweep motion all read immediately
Missed Brief Reads: white-bound sterility, legible side HUD cards, rectangular holo volume as the hero prop in direct view, stronger wall scan language beyond the floor grid
Hard-Fail Triggers: holo cards remain unreadable in direct view
text prompt fidelity: 7/10
visual identity: 7/10
depth / composition: 7/10
reflection / lighting read: 7/10
technical cleanliness: 8/10
animation quality: 8/10
loop quality or loopability readiness: 7/10
novelty versus roster: 8/10
verdict: fail
notes: Run23 keeps the same near-pass foundation as run21 and improves the motion read: the fluorescent slabs and scan planes advance clearly across frames 28-30, and the room still survives as a sterile sci-fi bay rather than collapsing into abstract cyan space. The blocker does not materially change, though. The background display still reads as a soft curved wall panel more than a rectangular holographic volume, the side HUD cards are still too faint or absent to satisfy the brief's holographic-panel requirement, and the room shell remains gray enough that the promised white clinical cleanliness never becomes a hero screenshot. Distinctness versus the roster survives because this is still one of the only readable indoor tech spaces, but it is not at showcase pass.

Correction Note
Date: 2026-03-11
Subject: `Combat Lab` terminology clarification
Correction: Recent references in this log to `HUD cards`, `side HUD cards`, or `holo cards` should be interpreted as failed reads of world-anchored projection planes, room-mounted luminous structures, or a rectangular projection chamber rooted to the environment. They do not imply literal overlay UI; the target remains fully in-world reflected architecture.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `run25` (provided frames 1-3 only)
Frames Reviewed: provided run25 Combat Lab frames 1-3
Brief Match: clinical interior room bounds, bright cyan floor grid, room-anchored translucent scan planes, and harsh fluorescent sweep motion all read immediately
Missed Brief Reads: a clearly rectangular projection chamber or test field in direct view, stronger white structural bounds, stronger wall scan language beyond the floor grid, more emphatic luminous emitters rooted into the room shell
Hard-Fail Triggers: the rectangular projection chamber still does not hold as a clear direct-view hero read
text prompt fidelity: 7/10
visual identity: 7/10
depth / composition: 7/10
reflection / lighting read: 7/10
technical cleanliness: 8/10
animation quality: 8/10
loop quality or loopability readiness: 7/10
novelty versus roster: 8/10
verdict: fail
notes: Run25 keeps the same near-pass foundation as run23. The floor grid is still clean, the sweeping light slabs now read more obviously as world-space scan planes, and frames 1-3 show real motion. The decisive blocker does not move: the rear chamber still reads as a soft curved bay instead of a crisp rectangular projection field, so the preset never lands the direct-view test-room focal prop promised by the brief. Distinctness versus the roster survives, but this is still a competent lab backdrop rather than a showcase-finished `Combat Lab` hero shot.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `run25` (provided frames 4-6 only)
Frames Reviewed: provided run25 Frozen Tundra frames 4-6
Brief Match: cold blue-white aerial haze and a high-key arctic palette survive
Missed Brief Reads: a hard glacier ridge separating sky from ground, a readable planar ice-sheet floor, low polar sunlight, drifting snow, and obvious ice-sheen travel across frames
Hard-Fail Triggers: ridge and floor separation disappear, and the environment collapses into a soft bowl-like blue field instead of a tundra expanse
text prompt fidelity: 3/10
visual identity: 3/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 6/10
verdict: fail
notes: Run25 is not a recovery from the earlier failing arctic state. The frame no longer establishes a real horizon at all: the background breaks into pale faceted patches, the bright strip on the far right reads closer to an artifact or seam than a glacier feature, and the area under the sphere never resolves into polished ice. Frames 4-6 look effectively static, so both the world read and the motion contract fail at once. Distinctness versus `Storm Front` survives only because this stays brighter and icier, not because it convincingly depicts a wind-cut polar landscape.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `run27` (provided frames 1-3 only)
Frames Reviewed: provided run27 Combat Lab frames 1-3
Brief Match: clinical interior bounds, a clean embedded rear test-field panel, harsh fluorescent wash, and faint cyan grid language survive
Missed Brief Reads: strong floor grid in direct view, room-anchored projection planes or luminous emitters beyond the rear panel, stronger white structural bounds, and obvious scan or holo-pulse motion across frames
Hard-Fail Triggers: none, but the lab read now depends too heavily on the sphere reflection for its grid and tech detail
text prompt fidelity: 6/10
visual identity: 6/10
depth / composition: 5/10
reflection / lighting read: 6/10
technical cleanliness: 8/10
animation quality: 3/10
loop quality or loopability readiness: 4/10
novelty versus roster: 7/10
verdict: fail
notes: This is a regression from the run23/run25 near-pass baseline. The rear blue chamber is cleaner and more geometric, but the rest of the room flattens back into broad gray space: the cyan floor grid is now mostly legible only in the sphere reflection, the surrounding architecture loses the stronger projection-plane read, and frames 1-3 show only tiny shimmer instead of an immediate scan beat. The preset still reads as an indoor sci-fi room, but not as a bright projection-heavy training lab, so the next fix should keep the older run23/run25 base rather than build from this state.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `run27` (provided frames 4-6 only)
Frames Reviewed: provided run27 Frozen Tundra frames 4-6
Brief Match: high-key arctic palette, cold blue-white haze, and a faint distant horizon band survive
Missed Brief Reads: hard glacier ridge, readable polished ice-sheet floor, low polar sunlight, drifting snow, and obvious ice-sheen travel across frames
Hard-Fail Triggers: ridge and floor separation still collapse, and faceted patching plus the bright right-edge seam dominate more than a tundra landscape
text prompt fidelity: 3/10
visual identity: 3/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 4/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 6/10
verdict: fail
notes: This is not a material recovery from run25. The horizon remains a soft blue band instead of a hard glacier ridge, the foreground still breaks into pale faceted patches instead of a planar polished ice sheet, and the bright vertical strip on the right still reads more like a seam or wall artifact than part of the landscape. Frames 4-6 look effectively frozen, so both the place read and the motion contract keep failing together. The preset stays brighter and colder than `Storm Front`, but that distinctness still does not produce a convincing wind-cut polar expanse.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `run27` (provided frames 34-36 only)
Frames Reviewed: provided run27 Storm Front frames 34-36
Brief Match: bruised blue-gray storm light, a dark lower storm-water mass, and a lightning-like reflection event survive
Missed Brief Reads: readable sea horizon, direct-view black-water floor, coherent storm shelf or squall wall, driven rain in world space, lightning outside the sphere reflection, and obvious weather cadence across frames
Hard-Fail Triggers: horizon remains buried and lightning still reads only in reflection, not in the world
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 2/10
loop quality or loopability readiness: 2/10
novelty versus roster: 6/10
verdict: fail
notes: This is not a material recovery from run14. The reflection now shows slightly easier-to-catch rain-like scratches and a shifting lightning scar, but the direct environment still collapses into a nearly featureless blue-gray field with no credible sea horizon, no readable storm shelf, and no world-space lightning strike. Frames 34-36 show only tiny internal drift, so the motion contract still fails, and `Frozen Tundra` remains the stronger outdoor proof-of-life candidate because this preset still cannot project its storm into the world outside the hero sphere.

Preset: `Combat Lab`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-29` (paired against run28 with `0` hash mismatches noted in `06-open-gaps.md`)
Frames Reviewed: local run29 batch frames 28-30
Brief Match: clinical white room bounds, a bright cyan floor grid in direct view, a clear rectangular test field behind the hero object, room-anchored fluorescent scan slabs, and obvious scan cadence all read immediately
Missed Brief Reads: side-wall projection structure remains more minimalist than lush, but not below acceptance
Hard-Fail Triggers: none
text prompt fidelity: 8/10
visual identity: 8/10
depth / composition: 8/10
reflection / lighting read: 8/10
technical cleanliness: 9/10
animation quality: 8/10
loop quality or loopability readiness: 8/10
novelty versus roster: 8/10
verdict: pass
notes: Run29 closes the long-standing direct-view blocker. The room now reads cleanly without leaning on the sphere reflection: the pale structural shell stays legible, the cyan grid survives in the frame itself, the bright rectangular test field is obvious behind the hero object, and the sweeping fluorescent slabs provide a real review-level motion beat across frames 28-30. The composition is still sparse rather than lavish, but it now matches the `Combat Lab` brief cleanly enough to function as a docs or showcase screenshot, so preset 10 should count as the current roster's first indoor proof-of-life pass.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-29` (paired against run28 with `0` hash mismatches noted in `06-open-gaps.md`)
Frames Reviewed: local run29 batch frames 34-36
Brief Match: bruised blue-gray storm light and a lightning-like reflection scar survive
Missed Brief Reads: readable sea horizon, direct-view black-water floor, coherent storm shelf, visible rain or squall structure in world space, lightning outside the reflection, and obvious weather cadence across frames
Hard-Fail Triggers: horizon is still buried and lightning still reads only in reflection, not in the world
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 1/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 5/10
verdict: fail
notes: Run29 does not move the blocker and arguably strips the scene back even further. The direct frame is almost a flat slate-blue field with no credible sea-water separation, no readable rain wall, and no storm shelf; all of the drama still lives on the sphere as reflected lightning scratches. Frames 34-36 show only minute reflection drift, so both the place read and the motion contract fail together. `Storm Front` remains well behind `Frozen Tundra` as the outdoor proof-of-life candidate.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `run31` (provided frames 1-3 only)
Frames Reviewed: attached run31 Frozen Tundra frames 1-3
Brief Match: cold blue-white aerial haze, a readable icy floor plane, and a distant ridge band survive
Missed Brief Reads: a hard glacier ridge, a convincing open tundra expanse, low polar sunlight, drifting snow, and obvious review-level sheen or snow motion across frames
Hard-Fail Triggers: repeated striped basin patterning dominates the horizon wall and floor, so the scene reads as a synthetic ice amphitheater more than a wind-cut arctic expanse
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 5/10
technical cleanliness: 4/10
animation quality: 2/10
loop quality or loopability readiness: 3/10
novelty versus roster: 6/10
verdict: fail
notes: Run31 is a partial recovery from the run25/run27 faceted-seam state, but not a showcase recovery. The frame finally gives the sphere something closer to a planar ice sheet and a continuous distant ridge band, yet the environment now resolves as a striped ice bowl or glacial amphitheater with strong repeated vertical wall marks and tiled sheen on the ground. Frames 1-3 show only tiny drift in the sheen and speckle, not an obvious snow or ice-motion beat, so the preset still fails both the place read and motion contract. It remains ahead of `Storm Front` as the outdoor proof-of-life candidate because it at least projects some world structure outside the reflection, but it is still not near pass.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `run31` (provided frames 4-6 only)
Frames Reviewed: attached run31 Storm Front frames 4-6
Brief Match: bruised blue-gray storm light, a dark lower mass, and a slight horizon split survive
Missed Brief Reads: a convincing sea horizon, direct-view black-water floor, squall wall or driven rain in world space, lightning outside the sphere reflection, and obvious storm cadence across frames
Hard-Fail Triggers: lightning still reads only on the sphere reflection, while the direct environment remains nearly empty
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 6/10
verdict: fail
notes: Run31 does not materially move the preset off its long-standing blocker. The upper sky band and darker lower field create a cleaner horizontal split than run29, but the result is still just a bare blue-gray stage rather than a violent squall line over water. All meaningful action remains trapped in the sphere reflection: the only visible change across frames 4-6 is the small highlight and lightning-scratch shift on the hero sphere, while the direct background shows no rain wall, no wave or water texture, and no world-space lightning event. Distinctness from `Frozen Tundra` survives because this stays emptier and darker, but the preset still fails almost every direct-view requirement in its brief.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-33` (paired against run32 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run33 batch frames 31-33
Brief Match: pale arctic palette, a shallow icy floor plane, and a distant ridge band survive
Missed Brief Reads: a hard glacier ridge, a convincing open tundra expanse, low polar sunlight, drifting snow, and obvious sheen or spindrift motion across frames
Hard-Fail Triggers: repeated striped wall/floor patterning dominates the frame and the three review images are nearly static
text prompt fidelity: 3/10
visual identity: 3/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 5/10
verdict: fail
notes: The updated carrier mix does not materially clear preset 11's blocker. The direct frame still reads as a pale striped ice basin or synthetic bowl rather than a wind-cut arctic expanse: the back wall resolves into repeated vertical streaks, the floor is covered in evenly tiled horizontal scratches, and the low polar-light event never becomes a real scene focal point. Frames 31-33 are almost indistinguishable at review speed, so the promised spindrift or sheen motion still does not survive capture. Relative to run31, the failure mode is essentially unchanged rather than improved.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-35` (paired against run34 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run35 batch frames 31-33
Brief Match: pale arctic palette, a shallow icy floor plane, and a distant ridge band survive
Missed Brief Reads: a hard glacier ridge, a convincing open tundra expanse, low polar sunlight, drifting snow, and obvious sheen or spindrift motion across frames
Hard-Fail Triggers: repeated striped wall/floor patterning dominates the frame and the three review images are nearly static
text prompt fidelity: 3/10
visual identity: 3/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 5/10
verdict: fail
notes: Run35 confirms that the new structural variants alone did not move the outdoor blocker. The scene still reads as a pale striped ice basin or amphitheater rather than a wind-cut glacier field: the far ridge remains soft, the wall carries evenly spaced vertical streaks, and the floor is tiled with shallow horizontal scratches. Frames 31-33 are effectively frozen at review speed, so neither spindrift nor ice-sheen motion survives capture.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-35` (paired against run34 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run35 batch frames 34-36
Brief Match: bruised blue-gray storm light, a dark upper shelf, and a darker lower field survive
Missed Brief Reads: a convincing sea horizon, direct-view black-water floor, a readable squall wall, lightning outside the sphere reflection, and obvious storm cadence across frames
Hard-Fail Triggers: the direct environment is still nearly empty while the most legible storm read remains trapped in reflection
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 6/10
verdict: fail
notes: Run35 also fails to move preset 12 off its long-standing shelf-only state. The new split does create a clearer upper dark band, but the world still reads as a dim slate stage instead of a violent squall over water: there is no readable rain curtain, no convincing black-water surface, and no world-space lightning event. Frames 34-36 show only tiny reflection drift on the sphere, so the storm cadence still does not survive in direct view.

Preset: `Frozen Tundra`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-39` (paired against run38 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run39 batch frames 31-33
Brief Match: the pale arctic palette survives, the distant ridge is slightly clearer, and subtle sky breakup finally shows outside the reflection
Missed Brief Reads: a convincing open tundra expanse, a hard glacier ridge, low polar sunlight, drifting snow, and obvious sheen or spindrift motion across frames
Hard-Fail Triggers: the frame still reads as a pale ice basin and the motion beat remains nearly absent
text prompt fidelity: 3/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 5/10
verdict: fail
notes: Run39 is a slight improvement over run35, but only slight. The stronger multiply-based `MOTTLE` pass finally breaks the completely flat pale sky and adds faint clouded variation around the ridge, so the scene is no longer one pure shelf. That does not solve the actual blocker: the environment still resolves as a shallow icy basin with a soft ridge, broad low-contrast floor, and almost no direct-view motion across frames 31-33. This is proof that the new breakup carrier can help, but it is not enough by itself to carry `Frozen Tundra` to pass quality.

Preset: `Storm Front`
Date: 2026-03-11
Capture Run: `2026-03-11-epu-showcase-12preset-replay-41` (paired against run40 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run41 batch frames 34-36
Brief Match: dark water survives, the storm shelf remains distinct from `Frozen Tundra`, and a world-space weather sheet finally appears directly in the background
Missed Brief Reads: a convincing natural squall front, readable world-space lightning outside the sphere reflection, and a believable storm cadence rather than geometric sheet bars
Hard-Fail Triggers: the visible rain now reads as giant luminous pillar sheets, which is cleaner than the old arc failure but still not a convincing storm
text prompt fidelity: 3/10
visual identity: 4/10
depth / composition: 3/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 3/10
loop quality or loopability readiness: 3/10
novelty versus roster: 6/10
verdict: fail
notes: Run41 is the first `Storm Front` state in this loop that materially changes the direct-view blocker. Moving `VEIL/RAIN_WALL` from the planar sheet to the cylindrical sheet domain finally puts a weather event into the scene itself instead of trapping it in reflection or turning it into huge side arcs. The tradeoff is now obvious and evidence-backed: the rain wall reads as giant geometric pillar slabs rather than a natural squall front, and lightning is still mostly a reflection event. This is real progress, but it also demonstrates a surface limitation in the current `VEIL` domain vocabulary rather than just a bad preset dial.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-43` (paired against run42 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run43 batch frames 31-33
Brief Match: pale arctic palette, a distant ridge band, and a broad cold sky survive
Missed Brief Reads: a convincing open tundra expanse, a hard glacier ridge, low polar sunlight, drifting snow, and obvious spindrift or sheen motion across frames
Hard-Fail Triggers: the direct frame collapses into a near-monochrome pale basin and the three review images are effectively static
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The first `ADVECT/SPINDRIFT` pass does not recover preset 11. In this pale scene class the new transport carrier almost disappears into the same low-contrast basin read, so the frame is even flatter than the stronger `MOTTLE` state from run39. The result is useful evidence rather than noise: anti-flat-fill support already exists, but broad transport alone is not enough to make `Frozen Tundra` pass. Preset 11 still needs stronger far-field structure and likely a non-water frozen-surface response rather than more particle or sheet churn.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-43` (paired against run42 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run43 batch frames 34-36
Brief Match: dark water, a readable horizon split, and a moving storm mass now survive directly in the background
Missed Brief Reads: a convincing natural squall front, readable world-space lightning outside the sphere reflection, and a storm cadence that does not resolve as architectural geometry
Hard-Fail Triggers: the transported weather body still reads as giant translucent slabs or ceiling panels rather than a coherent squall curtain
text prompt fidelity: 4/10
visual identity: 5/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: The first `ADVECT/SQUALL` pass is a real step forward. Unlike the old `VEIL` domain experiments, the direct environment is no longer empty and the storm body now travels as a broad world-space event across frames 34-36. The remaining miss is now narrower and more actionable: the carrier shape still resolves as large translucent wall or ceiling slabs instead of a believable weather front, while lightning stays reflection-led. That makes `Storm Front` the clearest proof that the new opcode direction is correct but its first-pass shaping still needs refinement.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-45` (paired against run44 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run45 batch frames 31-33
Brief Match: pale arctic palette, a distant ridge band, and a broad cold sky survive
Missed Brief Reads: a convincing open tundra expanse, a hard glacier ridge, low polar sunlight, drifting snow, and obvious spindrift or sheen motion across frames
Hard-Fail Triggers: the direct frame is still a near-monochrome pale basin and the three review images are effectively static
text prompt fidelity: 2/10
visual identity: 3/10
depth / composition: 2/10
reflection / lighting read: 3/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 4/10
verdict: fail
notes: The first `ADVECT` refinement pass does not materially move preset 11 off the run43 result. Horizon anchoring is still too subtle to read as spindrift in direct view, so the environment remains functionally the same pale ice basin at review speed. That closes the question of whether more transport tuning is the main lever for this preset: it is not. `Frozen Tundra` now clearly needs stronger frozen-surface identity and stronger far-field organization rather than more sheet-carrier churn.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-45` (paired against run44 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run45 batch frames 34-36
Brief Match: dark water, a readable horizon split, and a moving storm mass survive directly in the background
Missed Brief Reads: a convincing natural squall front, readable world-space lightning outside the sphere reflection, and a storm cadence that does not resolve as planar synthetic geometry
Hard-Fail Triggers: large translucent rectangular storm panels still dominate the direct scene, so the storm reads as transported architecture rather than weather
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: The first `ADVECT` refinement pass softens the brightest pillar and ceiling artifacts a little, but not enough to change the verdict. The storm now reads as a dim field of transported translucent panels rather than hard `VEIL` bars, which is marginally better, yet the carrier is still obviously synthetic and lightning remains reflection-led. This keeps `Storm Front` on the right engine path, but the shaping problem is still open and not yet close to pass quality.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-53` (paired against run52 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run53 batch frames 31-33
Brief Match: cold arctic palette, fractured ice geometry, and a darker lower basin now survive directly in the background
Missed Brief Reads: a convincing open tundra expanse, a readable hard glacier horizon, polar-light focus, and obvious drifting snow or spindrift motion across frames
Hard-Fail Triggers: the scene still reads as an abstract faceted ice chamber or basin rather than an exposed outdoor tundra, and frames 31-33 remain near-static at review speed
text prompt fidelity: 3/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 6/10
animation quality: 1/10
loop quality or loopability readiness: 2/10
novelty versus roster: 5/10
verdict: fail
notes: The two-bounds restructure is the first `Frozen Tundra` change in a while that clearly alters the failure class. `SILHOUETTE` plus `SPLIT/FACE` finally break the pale bowl into faceted glacier-like structure, and the lower half no longer reads as pure chalk water. That is real progress. It still does not cross the line because the frame reads more like an abstract icy chamber than an open wind-cut expanse, and the promised motion is still essentially absent in direct view. This means structure was part of the blocker, but not the whole blocker.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-53` (paired against run52 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run53 batch frames 34-36
Brief Match: dark water, a readable horizon shelf, and a storm-toned direct background survive
Missed Brief Reads: a believable squall curtain, readable world-space lightning outside the reflection sphere, and a storm body that reads as weather rather than an abstract layer field
Hard-Fail Triggers: the direct scene still resolves as subdued synthetic slabs / curtains with weak visible cadence, so the weather read remains underpowered
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: The latest batch was focused on `Frozen Tundra`, and `Storm Front` is largely unchanged in review outcome. The revised `ADVECT` shaping avoids the worst earlier `VEIL` artifacts, but the storm front still does not read as one convincing squall body in direct view. At this point the preset is no longer obviously mis-authored; the remaining gap is the weather-carrier shaping itself.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-55` (paired against run54 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run55 batch frames 31-33
Brief Match: cold palette and faceted icy breakup survive faintly in direct view
Missed Brief Reads: an open tundra floor, a clear horizon ridge, polar-light depth, and any obvious wind-driven motion across frames
Hard-Fail Triggers: the first `SPLIT/TIER` retry collapses the scene into a near-blank pale field where the floor/wall split is barely legible
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 1/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 3/10
verdict: fail
notes: This batch closes the question of whether simply swapping `Frozen Tundra` to an open-shelf `SPLIT/TIER` organizer would unlock the outdoor read. It does not. The result loses too much usable floor ownership for the `SURFACE` layers and ends up reading as a washed pale void with faint polygonal breakup. That is worse than run53 and strongly suggests the remaining blocker is not just "try a different shelf shape."

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-59` (paired against run58 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run59 batch frames 31-33
Brief Match: cold palette, faint faceted breakup, and a slightly darker lower band survive directly in the background
Missed Brief Reads: a readable open tundra floor, a hard glacier horizon, convincing outdoor depth, and any unmistakable drifting snow or spindrift cadence
Hard-Fail Triggers: even after the `UP`-oriented `SPLIT/FACE` retry, the scene still resolves as a pale low-contrast void with only a weak lower-floor hint
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 3/10
verdict: fail
notes: The `UP`-oriented `SPLIT/FACE` retry is slightly better than the `SPLIT/TIER` blank-field failure because it restores a trace of lower-floor ownership, but it still does not come close to the brief. The key conclusion is now stronger: repeated structural retunes are no longer producing meaningful outdoor scene readability. `Frozen Tundra` is increasingly acting like a real EPU surface/region-read limitation for this brief, not a small preset-authoring miss.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-59` (paired against run58 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run59 batch frames 34-36
Brief Match: dark water, a readable horizon shelf, and a broad storm-toned body still survive
Missed Brief Reads: a natural squall curtain, readable world-space lightning outside the sphere reflection, and a storm cadence that feels like weather rather than a low-contrast synthetic layer
Hard-Fail Triggers: the storm body remains too subdued and slab-like in direct view, so the weather event never becomes the dominant read
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: Narrowing the squall coverage avoided the worst oversized panel problem, but it did not solve the core miss. `Storm Front` is still on the correct engine path, yet the direct read remains too synthetic and too weak. The carrier-quality/shaping problem is still open, while `Frozen Tundra` now looks more like the stronger systemic surface-limit case.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-63` (paired against run62 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run63 batch frames 31-33
Brief Match: cold palette and some faceted icy breakup survive faintly in direct view
Missed Brief Reads: an open tundra floor, a readable hard glacier horizon, polar-light focus, and any unmistakable wind-driven motion
Hard-Fail Triggers: even with composed bounds plus the `SPLIT/TIER` retest, the direct scene still resolves as a washed pale chamber-like void
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 3/10
verdict: fail
notes: The first runtime-level bounds-composition change was real, but it did not rescue `Frozen Tundra` in the way the authored model had implied. Reintroducing `SPLIT/TIER` on top of composed bounds still leaves the preset effectively washed out, which closes the hopeful theory that simple sequential composition was the missing unlock for this outdoor brief.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-63` (paired against run62 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run63 batch frames 34-36
Brief Match: dark water, a readable horizon shelf, and the same subdued storm body survive
Missed Brief Reads: a convincing squall front and readable world-space lightning outside the reflection sphere
Hard-Fail Triggers: the runtime bounds-composition change does not materially alter the synthetic slab / subdued curtain read
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: `Storm Front` is largely unchanged by the first bounds-composition patch. That result matters because it shows the weather carrier miss is not primarily a region-overwrite bug.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-65` (paired against run64 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run65 batch frames 31-33
Brief Match: the same faint faceted ice breakup survives in direct view
Missed Brief Reads: an open outdoor floor, a convincing horizon, strong material separation, and obvious motion
Hard-Fail Triggers: sharpening bounds composition is visually near a no-op; the scene still reads as a pale near-monochrome void
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 3/10
verdict: fail
notes: The sharpened composition helper is likely the more correct runtime rule, but it does not materially improve `Frozen Tundra` in review. That is evidence that the preset's remaining blocker is deeper than the simple "later bounds blur away winners" theory.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-65` (paired against run64 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run65 batch frames 34-36
Brief Match: sea horizon, dark water cadence, and the subdued storm body remain intact
Missed Brief Reads: a convincing single squall front and lightning that reads directly in the world
Hard-Fail Triggers: the sharpened bounds-composition follow-up is visually near a no-op on the storm read as well
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: `Storm Front` stays on the same engine path. The weather body is still too synthetic and subdued, and the new sharpened composition rule is not the lever that changes that verdict.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-67` (paired against run66 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run67 batch frames 31-33
Brief Match: the same pale icy breakup persists
Missed Brief Reads: readable open-air floor ownership, a clear far-field glacier horizon, and visible wind-driven cadence
Hard-Fail Triggers: even after boosting single-region feature masks, the floor/surface carriers still do not become meaningfully legible in direct view
text prompt fidelity: 1/10
visual identity: 2/10
depth / composition: 2/10
reflection / lighting read: 2/10
technical cleanliness: 5/10
animation quality: 1/10
loop quality or loopability readiness: 1/10
novelty versus roster: 3/10
verdict: fail
notes: This closes another plausible system theory. The new single-region mask boost is also likely a better runtime rule, but it does not materially recover `Frozen Tundra`'s direct-view floor or surface identity. The blocker is now very unlikely to be just diluted floor-mask math.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-67` (paired against run66 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run67 batch frames 34-36
Brief Match: dark water and the same broad storm shelf remain
Missed Brief Reads: a natural squall front and more direct-view lightning/world-event presence
Hard-Fail Triggers: the single-region mask boost is effectively a visual no-op for the storm read
text prompt fidelity: 4/10
visual identity: 4/10
depth / composition: 4/10
reflection / lighting read: 4/10
technical cleanliness: 5/10
animation quality: 4/10
loop quality or loopability readiness: 4/10
novelty versus roster: 6/10
verdict: fail
notes: `Storm Front` again confirms that the remaining miss is carrier-quality/shaping, not just feature-mask attenuation from softened bounds ownership.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-69` (paired against run68 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run69 batch frames 31-33
Brief Match: the lower scene finally picks up a darker band and slightly stronger grounded breakup
Missed Brief Reads: a convincing open tundra floor, a clear far-field glacier horizon, and obvious wind-driven motion
Hard-Fail Triggers: despite the new grounded breakup, the scene still reads mostly as a pale icy chamber rather than an outdoor expanse
verdict: fail
notes: Replacing the weak snow-support slot with grounded `MOTTLE/GRAIN` was a real improvement. It gave `Frozen Tundra` the first visibly stronger lower-scene texture since the earlier structure recovery, but it still did not produce an outdoor proof-of-life pass.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-71` (paired against run70 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run71 batch frames 31-33
Brief Match: a darker lower floor band and clearer floor/surface separation now survive directly in view
Missed Brief Reads: a true open-air tundra horizon and stronger obvious motion
Hard-Fail Triggers: the scene is improved, but it still resolves as an abstract icy chamber more than an exposed tundra landscape
verdict: fail
notes: Fixing the `SPLIT` blend-width bug materially changed the review outcome. This is the first engine-side fix in a while that clearly improved `Frozen Tundra` itself instead of only cleaning up internal semantics.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-71` (paired against run70 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run71 batch frames 34-36
Brief Match: the sea horizon and storm shelf are much firmer and more legible in direct view
Missed Brief Reads: a strong readable storm body and more direct-view lightning presence
Hard-Fail Triggers: the structural organizer is now solid, but the actual weather event is still too weak compared with the shelf / water read
verdict: fail
notes: The `SPLIT` bug fix was just as important for `Storm Front`. The horizon organizer now reads cleanly, which means the remaining miss is even more cleanly isolated to storm-carrier strength and shaping.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-73` (paired against run72 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run73 batch frames 31-33
Brief Match: the darker lower band and grounded breakup hold under the fixed `SPLIT` behavior
Missed Brief Reads: a clear outdoor horizon contract and stronger motion
Hard-Fail Triggers: the latest content retune preserves the gains but does not yet push the preset over the line
verdict: fail
notes: The post-`SPLIT` content pass is stable, but it is only incremental over run71 for `Frozen Tundra`.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-73` (paired against run72 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run73 batch frames 34-36
Brief Match: crisp horizon shelf, dark water read, and slightly fuller storm body all survive
Missed Brief Reads: a dominant believable squall front and more direct lightning/world-event presence
Hard-Fail Triggers: the new storm-body tuning helps only slightly; the weather mass still does not dominate the scene the way the brief requires
verdict: fail
notes: `Storm Front` is healthier after the `SPLIT` fix than before it, but still not at pass quality.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-75` (paired against run74 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run75 batch frames 31-33
Brief Match: the post-`SPLIT` darker lower band and grounded breakup still survive
Missed Brief Reads: a convincing open-air horizon, less chamber-like side structure, and more obvious motion
Hard-Fail Triggers: the first `ADVECT` tightening pass is effectively a visual no-op in review; the scene still reads as the same pale icy chamber
verdict: fail
notes: Tightening the `ADVECT` horizon band and slightly retuning the preset did not materially move `Frozen Tundra`. That closes another smaller theory: this preset is not going to recover from subtle spindrift-shaping changes alone.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-75` (paired against run74 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run75 batch frames 34-36
Brief Match: the firm sea horizon, dark water, and subdued storm shelf still survive
Missed Brief Reads: a dominant single squall body and stronger direct-view lightning/event presence
Hard-Fail Triggers: the first `ADVECT` tightening pass is also effectively a visual no-op; the storm still reads as a weak synthetic layer rather than one convincing front
verdict: fail
notes: The first post-`SPLIT` `ADVECT` tightening pass did not materially change the verdict. `Storm Front` stays on the correct carrier family, but the shaping delta here was too small to matter in review.

Preset: `Frozen Tundra`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-77` (paired against run76 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run77 batch frames 31-33
Brief Match: the pale arctic palette and fractured ice geometry still survive
Missed Brief Reads: a cleaner open-air horizon contract, less enclosing chamber structure, and obvious wind-driven motion
Hard-Fail Triggers: even after moving the support breakup off the walls, the scene still resolves as an icy chamber rather than an exposed tundra
verdict: fail
notes: The stronger region retargeting pass confirms that wall-carried support layers are not the only cause of the enclosure read. `Frozen Tundra` remains the stronger evidence of a deeper outdoor surface/readability limitation.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-77` (paired against run76 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run77 batch frames 34-36
Brief Match: sea horizon, black-water motion, and a cleaner shelf silhouette all survive
Missed Brief Reads: a forceful storm body that dominates the scene and a clearer world-space lightning event
Hard-Fail Triggers: reducing the competing breakup flattens the storm body slightly, but it still does not become the hero weather event
verdict: fail
notes: The stronger preset-only retargeting did narrow one question: `Storm Front` does not primarily need more wall breakup. The direct read is slightly cleaner, but the storm mass is still too weak and too synthetic to pass.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-79` (paired against run78 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run79 batch frames 34-36
Brief Match: the sea horizon, water cadence, and same broad storm shelf still survive
Missed Brief Reads: a storm body that clearly dominates the scene and a stronger direct lightning/world-event read
Hard-Fail Triggers: the continuity-heavy `ADVECT/SQUALL` shader pass is still only a marginal visual change; the storm remains too soft and too backgrounded
verdict: fail
notes: This closes another plausible shader-side hope. Giving `SQUALL` more body continuity and a darker carrier tint does not materially change the direct-view verdict. The current carrier family still does not produce a forceful squall front.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-81` (paired against run80 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run81 batch frames 34-36
Brief Match: horizon shelf and water remain legible, and the carrier now hugs the storm wall more cleanly
Missed Brief Reads: a dominant storm mass and more direct lightning/world-event presence
Hard-Fail Triggers: even after isolating `ADVECT/SQUALL` to `WALLS` with a harder imprinting blend, the storm still does not materially take over the scene
verdict: fail
notes: This is the clearest closure yet on the current `ADVECT` tuning track. If `BLEND_LERP` plus `REGION_WALLS` still does not materially change the review read, the remaining blocker is not just the wrong blend or region mask. The current storm-carrier surface is still too weak for this brief.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-83` (paired against run82 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run83 batch frames 34-36
Brief Match: sea horizon, black-water motion, and the same broad storm shelf still survive
Missed Brief Reads: a storm body that clearly dominates the scene, a believable banked wall-front weather mass, and a stronger direct lightning/world-event read
Hard-Fail Triggers: even the new `ADVECT/BANK` pass still reads as a weak synthetic layer rather than a forceful storm front
verdict: fail
notes: This closes the next obvious transport-family theory. If a banked `ADVECT` profile still does not materially improve direct-view dominance, the remaining blocker is no longer just `SQUALL` shaping or preset masking. The current weather-carrier surface still lacks the forceful banked wall-front behavior this brief needs.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-85` (paired against run84 with `0` hash mismatches verified locally)
Frames Reviewed: newest deterministic run85 batch frames 34-36
Brief Match: black-water motion, the same shelf/horizon structure, and a cleaner non-wall haze all survive
Missed Brief Reads: a dominant storm body, a believable front mass that owns the wall belt, and stronger direct lightning/world-event presence
Hard-Fail Triggers: even after thickening `ADVECT/BANK` and removing wall-haze washout, the direct read is still effectively the same pale synthetic storm shell
verdict: fail
notes: This is now a second closure on the `BANK` path, not just the first authored miss. Making the bank physically thicker and de-washing the wall belt still does not materially improve the direct-view verdict. The remaining gap is no longer plausibly just a thin-bank implementation mistake; the current transport surface still lacks the scene-owning front vocabulary this brief needs.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: exploratory full-sweep promotion check after the `Front Mass` benchmark follow-up (`2026-03-12 13:23:34..13:23:47` single full-sweep batch; not yet paired into a new deterministic showcase baseline)
Frames Reviewed: newest single-batch preset-12 frames at `2026-03-12 13:23:47.309`, `13:23:47.641`, and `13:23:47.974`
Brief Match: the black-water floor, the darkened storm shelf, and a more coherent upper wall-front body now survive together
Missed Brief Reads: a clearly dominant storm event, stronger direct-view motion in the body itself, and a more forceful world-space lightning/weather cadence
Hard-Fail Triggers: the promoted `ADVECT/FRONT` direction is materially better than the old pale shelf, but the three reviewed frames still look too similar to pass the motion/event bar
verdict: fail
notes: This is the first showcase pass that actually preserves the benchmark-side wall-front recovery. The scene is directionally healthier: the shelf/body now reads as one darker front instead of a nearly absent carrier. But the result is still not ready to pass because the weather event remains too subdued and too static across the reviewed frames. The next loop should stay on this `ADVECT/FRONT` track rather than reopening old `BANK` / `SQUALL` assumptions.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-87` (paired locally against the second full-sweep rerun at `2026-03-12 19:04:49..19:05:03` with `0` hash mismatches across all `36` frame pairs after filtering to the correct full-sweep window)
Frames Reviewed: newest deterministic preset-12 frames at `2026-03-12 19:05:02.808`, `19:05:03.142`, and `19:05:03.476`
Brief Match: the new `MASS + ADVECT` architecture is directionally correct, and the water bed remains readable underneath it
Missed Brief Reads: a dark scene-owning storm shelf, forceful direct-view weather mass, and obvious motion/event escalation across the three review frames
Hard-Fail Triggers: the first `MASS/SHELF` promotion washed out into a pale wall field instead of owning the scene
verdict: fail
notes: This closes the first “new opcode must solve it” hope. `MASS` is a real useful separation of concerns, but the first showcase promotion proves that simply adding the body carrier is not enough by itself. The real preset read regressed toward a pale wall field rather than a violent front, so the next loop must tune the new architecture harder instead of assuming the surface addition alone fixed the brief.

Preset: `Storm Front`
Date: 2026-03-12
Capture Run: `2026-03-12-epu-showcase-12preset-replay-88` (paired locally against the second full-sweep rerun at `2026-03-12 19:07:05..19:07:41` with `0` hash mismatches across all `36` frame pairs in the `19:07` window)
Frames Reviewed: newest deterministic preset-12 frames at `2026-03-12 19:07:40.537`, `19:07:40.869`, and `19:07:41.202`
Brief Match: determinism is clean, the water read survives, and the body layer is at least being driven through the new `MASS + ADVECT` split
Missed Brief Reads: the storm still does not darken into one dominant front, lightning remains the clearest event, and the wall body still looks too pale and too even
Hard-Fail Triggers: even the darker/stronger `MASS/SHELF` retune still fails to create a forceful direct-view front
verdict: fail
notes: The second `MASS` promotion confirms the remaining blocker is not simply “make the new body layer darker.” The architecture is still the right one, but the current shaping/blending does not yet sell a violent storm front in direct view. The next loop should stay on `MASS + ADVECT`, but it needs stronger body occupancy/contrast rather than reopening the old transport-only branch.
Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-01` (paired against the same-day second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: first-pass 20-scene triptych frames `14:02:23.210`, `14:02:23.544`, and `14:02:23.879`
Brief Match: stronger descending shafts and a clearer underwater mood survive; the frame is no longer just loose blue fog
Missed Brief Reads: a decisive trench-floor anchor, a stronger basin focal, and more convincing direct-view motion
Hard-Fail Triggers: the scene is still too pale and diffuse to read as a deep trench with one obvious abyssal floor event
verdict: fail
notes: This wave is directionally better than the old failed state, but it still does not clear the brief. The new organizer/vent stack helped, yet the world remains too soft and high-key to sell a dark seabed and strong depth gradient.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-01` (paired against the same-day second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: first-pass 20-scene triptych frames `14:02:24.378`, `14:02:24.711`, and `14:02:25.046`
Brief Match: the single round viewport, eclipse, and interior-vs-space split now read far more clearly
Missed Brief Reads: richer deck/wall panel separation and a less porthole-dominated room composition
Hard-Fail Triggers: this is still a sparse room study rather than a fully convincing station interior
verdict: fail
notes: This is the strongest result from the quick-fix wave. It is not a pass yet, but it is plausibly near-pass and should stay high in the next shipping-quality lane.

Preset: `Astral Void`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-01` (paired against the same-day second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: first-pass 20-scene triptych frames `14:02:27.046`, `14:02:27.379`, and `14:02:27.880`
Brief Match: star density still survives and the space remains dark enough to read as cosmic
Missed Brief Reads: stable hero celestial hierarchy, clear secondary body emphasis, and disciplined restrained nebular drift
Hard-Fail Triggers: the reviewed triptych still does not present one consistent hero-body-led composition
verdict: fail
notes: The intended hierarchy work is not reading cleanly enough yet. This still feels like an unstable starfield-first scene rather than a finished cosmic tableau.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-01` (paired against the same-day second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: first-pass 20-scene triptych frames `14:02:28.213`, `14:02:28.547`, and `14:02:29.050`
Brief Match: the infernal palette and darker grounded stack are directionally stronger than the older abstraction
Missed Brief Reads: dominant lava fissures and a stable lower hellgate read across the reviewed frames
Hard-Fail Triggers: the captured sequence still does not present the cracked volcanic floor as the clear primary read
verdict: fail
notes: This wave closes the old soft-orange abstraction problem somewhat, but the preset still is not reliably landing the fissure-first contract in the authoritative triptych.

Batch: `2026-03-13-epu-showcase-20preset-replay-01`
Date: 2026-03-13
Scope: first-pass review of newly implemented presets `13-20` using the first-frame contact sheet from the authoritative paired 20-scene batch
Verdict: fail
notes: Presets `13-20` now exist in code and are covered by the full sweep, but they are still first-pass quality rather than showcase-ready. `Digital Matrix` and `Neon Arcade` are the strongest directionally, `Void Station` is the strongest quick-fix result, and the weakest first-pass additions are `Crystal Cavern`, `Moonlit Graveyard`, `Alien Jungle`, `Gothic Cathedral`, `Toxic Wasteland`, and `War Zone`, all of which still miss their direct scene contract decisively.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-02` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 1 triptych frames `18:09:31.061`, `18:09:31.396`, and `18:09:31.729`
Brief Match: the abyssal lower basin is much darker now, the trench floor finally survives in direct view, and the biolum vent is a clearer basin event
Missed Brief Reads: stronger floor contour separation, a more decisive hero focal point, and more obvious motion within the trench body
Hard-Fail Triggers: although much healthier than the prior pass, the scene still reads as a dark underwater chamber more than a clear trench with one unmistakable abyssal floor anchor
verdict: fail
notes: This is a real improvement. The lower-scene ownership recovered and the preset is closer to its contract than before. The next pass should build on the darker basin instead of reopening pale upper-water noise.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-02` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 1 triptych frames `18:09:32.229`, `18:09:32.562`, and `18:09:32.897`
Brief Match: the viewport, eclipse, and cold room-vs-space split remain readable
Missed Brief Reads: stronger room architecture, deck/wall differentiation, and less composition dominance from the circular viewport
Hard-Fail Triggers: the room still reads as a sparse porthole study rather than a convincing derelict station interior
verdict: fail
notes: The replacement of the explicit rim with room-light structure did not yet solve the core composition problem. This lane still looks near-pass compared with most of the roster, but it needs more enclosure and less empty grey field.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-02` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 1 triptych frames `18:09:46.241`, `18:09:46.574`, and `18:09:46.910`
Brief Match: the old olive-haze failure is reduced; the frame is darker, more militarized, and the tracer/light language finally shows up
Missed Brief Reads: clearer wreckage silhouette, stronger battlefield depth, and a more forceful combat event through the smoke
Hard-Fail Triggers: the scene still collapses toward a dark smoke wall with only a small accent event instead of a readable battlefront
verdict: fail
notes: Directionally better than the first pass, but still not enough. The new debris emphasis helps, yet the battlefield identity is still too submerged inside the smoke mass.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-02` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 1 triptych frames `18:09:47.409`, `18:09:47.742`, and `18:09:48.076`
Brief Match: the digital gate and partition read are clearer, and the synthetic chamber identity is still distinct from the rest of the roster
Missed Brief Reads: higher-contrast partition depth, more luminous data-space structure, and less overall grey wash in the chamber volume
Hard-Fail Triggers: the chamber still feels too soft and muted to read as an unmistakable impossible digital space
verdict: fail
notes: This remains one of the strongest expansion lanes, but the current pass is still too restrained. The next move should intensify partitioned digital depth rather than add more generic fog or broad geometry.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:14:43.117`, `18:14:43.620`, and `18:14:43.954`
Brief Match: the dark basin, trench floor, and biolum vent still survive together; the preset is no longer pale or structureless
Missed Brief Reads: stronger floor contour definition around the vent and a more forceful single trench-floor focal read
Hard-Fail Triggers: the floor is healthier, but the scene still reads more as a dark underwater chamber than a decisive trench drop with one obvious abyssal event
verdict: fail
notes: The floor-contour move is directionally correct. This lane is improving incrementally and should stay active, but the final focal hierarchy is still not clean enough.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:14:44.288`, `18:14:44.788`, and `18:14:45.121`
Brief Match: the viewport, eclipse, and room-vs-space split still read immediately
Missed Brief Reads: richer room enclosure, more deck/wall architecture, and a composition that is less dominated by the circular opening
Hard-Fail Triggers: even after the wall-light adjustments, the preset still resolves as a sparse porthole study
verdict: fail
notes: This remains near-pass relative to the rest of the roster, but the architecture still is not doing enough work. The next pass should add enclosure and structural room language, not more viewport emphasis.

Preset: `Toxic Wasteland`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:15:17.264`, `18:15:17.597`, and `18:15:17.931`
Brief Match: the palette is decisively toxic and the scene now has more structured contamination rhythm than the previous diffuse haze pass
Missed Brief Reads: unmistakable ruined machinery silhouettes, a clearer contaminated ground plane, and hazard structure that reads as industrial rather than organic blotches
Hard-Fail Triggers: the scene still reads mostly as green corrupted patterning instead of a poisoned industrial ruin
verdict: fail
notes: The lane changed failure shape but did not pass. The current stack is still too abstract, so the next move should push rigid industrial silhouette and ground ownership harder.

Preset: `Neon Arcade`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:15:18.432`, `18:15:18.764`, and `18:15:19.098`
Brief Match: the glossy floor grid and interior synthetic palette survive, and the scene is still distinct from the alley preset
Missed Brief Reads: stronger cabinet-row clustering, brighter entertainment-space identity, and more direct playful marquee/cabinet ownership
Hard-Fail Triggers: the preset still reads more like a stylized synth room than an unmistakable arcade space
verdict: fail
notes: This remains one of the healthier expansion lanes, but the cabinet identity is still too implied. The next pass should turn more wall energy into explicit arcade-row rhythm.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:15:19.599`, `18:15:19.933`, and `18:15:20.266`
Brief Match: tracer/flare activity is stronger and the frame feels more militarized than the prior olive-haze state
Missed Brief Reads: clearer rigid wreckage silhouette, stronger layered battlefield depth, and a smoke field that supports rather than swallows the fight
Hard-Fail Triggers: the scene is still too abstract and smoke-driven to read as a convincing battlefront
verdict: fail
notes: This lane is moving, but it still needs a harder wreckage/structure carrier. The current smoke-debris mix is more legible than before without yet landing the scene.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-03` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 2 triptych frames `18:15:20.767`, `18:15:21.100`, and `18:15:21.433`
Brief Match: the impossible data gate is clearer, partition depth is stronger, and the digital chamber identity remains one of the stronger expansion reads
Missed Brief Reads: brighter luminous data-space structure and more assertive split-space contrast around the hero gate
Hard-Fail Triggers: the scene still remains too muted and gray-blue to feel like a fully realized high-contrast cyberspace chamber
verdict: fail
notes: This lane improved again. It is still a strong polish candidate rather than a system blocker, but it needs one more more-assertive contrast/data-light pass.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:20:35.050`, `18:20:35.553`, and `18:20:35.886`
Brief Match: the dark trench body and single vent event still survive, and the scene is more disciplined than the earlier pale versions
Missed Brief Reads: clearer floor falloff around the vent and a stronger unmistakable trench-drop composition
Hard-Fail Triggers: the scene remains closer to a dark underwater chamber with one glow than to a decisive abyssal trench floor composition
verdict: fail
notes: This lane is still improving slowly, but the gains are now small. It should stay active, though the next pass needs sharper composition rather than more generic darkening.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:20:36.221`, `18:20:36.721`, and `18:20:37.055`
Brief Match: the viewport/ecliptic read is still immediate, and the new lower light structure gives the room a little more authored architecture
Missed Brief Reads: stronger enclosing room bulk, more convincing deck/wall complexity, and less empty field surrounding the hero viewport
Hard-Fail Triggers: despite the added bay-light structure, this is still fundamentally a porthole composition instead of a full station interior
verdict: fail
notes: Still near-pass, still worth pushing. The next change should add enclosure or sector-like room mass, not more glow around the existing opening.

Preset: `Toxic Wasteland`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:21:09.185`, `18:21:09.519`, and `18:21:09.852`
Brief Match: the scene is now more rigid and industrial than before, and it no longer collapses into the same diffuse green haze failure
Missed Brief Reads: clearer machinery silhouette ownership, more readable contaminated ground, and less giant patterned wall dominance
Hard-Fail Triggers: the preset overcorrected into bold abstract industrial patterning rather than a poisoned ruin
verdict: fail
notes: This is a different failure class, but still a failure. It should not get another blind pattern-structure pass without a more grounded ruined-machinery strategy.

Preset: `Neon Arcade`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:21:10.352`, `18:21:10.686`, and `18:21:11.019`
Brief Match: the room is bright, synthetic, and clearly distinct from the alley preset
Missed Brief Reads: explicit cabinet-row clustering, playful arcade identity, and a floor/reflection read that supports an entertainment-space rather than a geometric stage
Hard-Fail Triggers: the latest pass regressed into a bold abstract purple chamber and lost too much of the arcade-specific read
verdict: fail
notes: This lane needs rescoping rather than another immediate push in the same direction. The cabinet strategy overcorrected into geometry and marquee planes without enough arcade specificity.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:21:11.520`, `18:21:11.854`, and `18:21:12.188`
Brief Match: smoke no longer completely owns the frame, and the scene is darker and more severe than its early olive-haze version
Missed Brief Reads: readable wreckage silhouette, battlefield layering, and a clear militarized event instead of large abstract breakage
Hard-Fail Triggers: replacing debris with `CELL/SHATTER` overcorrected into black abstract fracture patterning
verdict: fail
notes: This branch regressed. The lane still needs a stronger rigid battlefront carrier, but not this pattern family.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-04` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 3 triptych frames `18:21:12.687`, `18:21:13.021`, and `18:21:13.354`
Brief Match: the hero gate remains strong, the chamber is clearly synthetic, and the split-space read is still among the better new-scene results
Missed Brief Reads: brighter luminous data-space energy and a stronger partition/depth field around the gate
Hard-Fail Triggers: the chamber is still too muted and conservative to feel like a fully realized high-contrast cyberspace volume
verdict: fail
notes: This lane continues to improve without changing failure class. It remains a strong polish target for the next wave.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:00.938`, `18:27:01.272`, and `18:27:01.774`
Brief Match: the underwater place read survives and the upper-column shafts plus particulate still support a submerged abyssal mood in both background and probe
Missed Brief Reads: a decisive trench-floor anchor, a darker seabed mass, and one localized lower-band hero event that makes the scene read as trench instead of open water
Hard-Fail Triggers: the composition still resolves as blue water with geometry and shafts more than a memorable abyssal floor drop
verdict: fail
notes: This lane is still active, but the next pass should stop nibbling at atmosphere and push one lower-scene trench/body anchor with a cleaner vent focal.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:01.774`, `18:27:02.107`, and `18:27:02.440`
Brief Match: the circular viewport, starfield, and eclipse still read immediately, and the cold probe reflection supports the space-facing mood
Missed Brief Reads: stronger room enclosure, more deck/wall structure, and a station interior that owns more of the frame than the porthole itself
Hard-Fail Triggers: the preset still reads as a soft gray bowl around a viewport rather than a designed station room
verdict: fail
notes: Still near-pass and still one of the strongest lanes. The next move should add one harder room-anchored structure or spill layer around the viewport instead of more space detail.

Preset: `Astral Void`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:05.276`, `18:27:05.610`, and `18:27:05.944`
Brief Match: the starfield is cleaner, the space read is immediate, and the palette is closer to the intended composed cosmic calm
Missed Brief Reads: a stronger hero-body hierarchy and one celestial event that survives clearly in direct background, not mostly inside the reflective probe
Hard-Fail Triggers: the frame still resolves as attractive stars-on-black rather than a strong cosmic tableau with a winning hero body
verdict: fail
notes: Directionally better. The next pass should enlarge and promote one world-space celestial carrier while demoting supporting drift and secondary bodies.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:06.444`, `18:27:06.779`, and `18:27:07.113`
Brief Match: infernal warmth and grounded pressure are stronger, and the lower glow now reads like a real hostile place instead of generic amber abstraction
Missed Brief Reads: a dominant cracked volcanic floor, clearer lava fissure ownership, and a lower rift that beats the broad warm wall facets
Hard-Fail Triggers: the scene is still carried more by amber faceting and glow than by one unmistakable fractured hell floor
verdict: fail
notes: This lane is healthier than before. The next pass should push one rigid fissure/crack carrier to own the floor silhouette and pull back the broad warm facet contribution.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:19.291`, `18:27:19.623`, and `18:27:19.957`
Brief Match: the ash-lit battlefield mood is back, and the smoke mass no longer collapses into the earlier black fracture failure
Missed Brief Reads: one clear militarized vector, stronger ruined-structure silhouette, and tracer/searchlight language that survives both background and probe
Hard-Fail Triggers: the frame still reads more like grayscale storm-smoke than a battlefront with an unmistakable combat event
verdict: fail
notes: Reverting off `CELL/SHATTER` helped, but this branch still needs a stronger directional combat-light carrier instead of more diffuse smoke shaping.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-05` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 4 triptych frames `18:27:20.457`, `18:27:20.790`, and `18:27:21.124`
Brief Match: the synthetic chamber, hero gate, and impossible partitioning survive clearly, and this remains one of the strongest expansion lanes
Missed Brief Reads: crisper rectilinear partition ownership and more disciplined data-space contrast without relying on extra teal fan glow
Hard-Fail Triggers: the chamber still leans too much on radial teal energy, so the digital-matrix identity is softer than it should be
verdict: fail
notes: Still a strong polish target. The next pass should sharpen one hard partition/bar layer rather than adding more bloom or drift.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:27.597`, `18:37:28.097`, and `18:37:28.432`
Brief Match: the submerged mood still reads cleanly in both background and probe, and the lower band is a little steadier than the previous wave
Missed Brief Reads: a decisive abyssal floor anchor, a memorable trench-drop composition, and one localized deep-floor hero event
Hard-Fail Triggers: the preset still resolves as attractive underwater gradient and shafts more than a trench-world with a winning lower focal
verdict: fail
notes: Only a marginal improvement. The next pass should add one unmistakable deep-floor focal structure rather than keep tuning top light, haze, or particulate.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:28.765`, `18:37:29.267`, and `18:37:29.600`
Brief Match: the bright side spill finally makes the viewport feel embedded in an interior shell, and the station read is materially stronger in direct background
Missed Brief Reads: clearer authored panel/frame language and a stronger room identity around the opening
Hard-Fail Triggers: the scene is still more of a lit chamber with a window than a fully realized sci-fi station room
verdict: fail
notes: Clear improvement and still one of the best lanes. The next move should add one stronger room-anchored panel/frame carrier instead of more space-side detail.

Preset: `Astral Void`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:32.602`, `18:37:32.935`, and `18:37:33.268`
Brief Match: the frame remains clean and cosmically calm, with supporting clutter held down better than earlier versions
Missed Brief Reads: one direct-view hero celestial event and a stronger background hierarchy that does not live mostly in the reflective probe
Hard-Fail Triggers: the scene still resolves as beautiful stars-on-black more than a composed cosmic tableau
verdict: fail
notes: Effectively unchanged from the prior wave. The next pass should sharply increase one celestial carrier and cut remaining competing body or nebular information.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:33.770`, `18:37:34.271`, and `18:37:34.604`
Brief Match: infernal pressure is stronger and the lower event is more assertive than the prior wave
Missed Brief Reads: a dominant volcanic crack network, clearer fissure-first floor ownership, and less competition from broad warm side-wall facets
Hard-Fail Triggers: the scene still reads as an amber infernal chamber before it reads as a shattered lava-floor world
verdict: fail
notes: Slight improvement. The next pass should strengthen one floor-anchored crack/rift carrier again and cut back the large warm chamber planes that dilute the fissure-first read.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:44.631`, `18:37:44.963`, and `18:37:45.296`
Brief Match: the warm ground flare or searchlight cue is clearer and the scene has a stronger directional conflict hint than Wave 4
Missed Brief Reads: readable ruined structure, stronger tracer or fireline ownership, and a combat event that clearly beats the smoky abstraction
Hard-Fail Triggers: the scene still reads more like smoky fractured atmosphere than a battlefield with a winning military vector
verdict: fail
notes: Directionally better, but still blocked. The next pass should replace one broad smoke or texture support layer with a single explicit tracer or fireline carrier.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-06` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 5 triptych frames `18:37:45.797`, `18:37:46.130`, and `18:37:46.464`
Brief Match: the harder cyan beams and darker wedges materially improve partitioning, and the chamber now feels more disciplined and synthetic than the prior wave
Missed Brief Reads: stronger rectilinear wall-space structure and less reliance on pure radial symmetry
Hard-Fail Triggers: the chamber still leans a bit too much on spotlight-like fan geometry, so the matrix or code-space read is not fully owned by digital partitions
verdict: fail
notes: Material improvement and still one of the best lanes. The next pass should add one rectilinear partition or slab layer to break the radial symmetry rather than more glow.

Preset: `Ocean Depths`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:19.752`, `18:44:20.085`, and `18:44:20.418`
Brief Match: the underwater gradient, shafts, and probe reflection still hold the submerged mood
Missed Brief Reads: a trench-floor anchor, a memorable abyssal drop, and one deep-floor hero event
Hard-Fail Triggers: the preset is still atmosphere-first water space instead of a distinct trench-world
verdict: fail
notes: Essentially unchanged from Wave 5. The next move must be one darker seabed silhouette or vent-body focal, not more atmospheric support tuning.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:20.920`, `18:44:21.254`, and `18:44:21.586`
Brief Match: the viewport still reads immediately, along with the starfield and eclipse beyond it
Missed Brief Reads: stronger architectural ownership around the opening and a more convincing station shell
Hard-Fail Triggers: the new rectangular frame pass regressed the room back toward a softer bowl-with-window read instead of strengthening the interior
verdict: fail
notes: Slight regression from Wave 5. The next pass should restore one simpler structural framing or spill layer around the viewport rather than a harder competing frame.

Preset: `Astral Void`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:24.089`, `18:44:24.421`, and `18:44:24.755`
Brief Match: the field remains clean and calm, with support clutter held down
Missed Brief Reads: a direct-view hero celestial event and stronger background hierarchy
Hard-Fail Triggers: the frame still reads as stars-on-black more than a cosmic tableau with a winning body
verdict: fail
notes: No material gain. This lane is now effectively stalled until one celestial carrier is promoted much more aggressively.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:25.591`, `18:44:25.924`, and `18:44:26.258`
Brief Match: the lower infernal event and fissure energy are clearer than Wave 5, and the scene is closer to a floor-led hell read
Missed Brief Reads: a truly dominant volcanic crack network and less competition from the warm chamber planes
Hard-Fail Triggers: the broad amber chamber faceting still competes too much with the floor fracture
verdict: fail
notes: Real improvement. The next pass should reduce the remaining warm wall-plane contribution one more step so the floor crack/rift stack can dominate.

Preset: `War Zone`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:36.921`, `18:44:37.255`, and `18:44:37.588`
Brief Match: the lower hot accents and tracer-like cues are a little cleaner and more intentional than Wave 5
Missed Brief Reads: a ruined battlefield silhouette and a combat-light pattern that makes the scene specifically militarized
Hard-Fail Triggers: the scene still reads as smoke plus abstract hot shards rather than a battlefront
verdict: fail
notes: Only a small directional gain. The next move should add a battlefield-specific ruined-horizon or industrial-ruin structure layer for the combat lighting to play against.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-07` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: Wave 6 triptych frames `18:44:38.420`, `18:44:38.755`, and `18:44:39.089`
Brief Match: the chamber is still clean, synthetic, and strongly partitioned in a way that survives both background and probe
Missed Brief Reads: harder code-like wall partitions that break the radial spotlight symmetry more decisively
Hard-Fail Triggers: the result remains too close to the same radial cyan-fan solution class as Wave 5
verdict: fail
notes: Almost no meaningful gain over Wave 5. The next pass must add a genuinely hard rectilinear wall-space partition layer instead of more beam tuning.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-08` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 7 board frames `18:50:18.714`, `18:50:19.046`, and `18:50:19.380`
Brief Match: the viewport again feels embedded in an interior shell, and the room read recovered after backing off the hard rectangular frame
Missed Brief Reads: stronger station-specific panel and deck framing around the lower room
Hard-Fail Triggers: the preset is still too minimal and smooth around the opening, so it reads as a windowed chamber more than a distinct sci-fi station room
verdict: fail
notes: This regains the healthier Wave 5 gain. The next pass should add one restrained lower-room structural panel or deck framing layer without hurting the clean viewport contract.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-08` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 7 board frames `18:50:25.719`, `18:50:26.052`, and `18:50:26.386`
Brief Match: the lower infernal event and cracked floor read more clearly now, and the broad chamber planes have been pushed back again
Missed Brief Reads: a dominant widened fissure network that unquestionably owns the floor silhouette
Hard-Fail Triggers: the scene still lands as a dark infernal chamber with a strong lower rift more than a volcanic floor split by lava cracks
verdict: fail
notes: Real improvement again. The next pass should strengthen and widen the floor-anchored crack or fissure carrier itself, not the surrounding glow.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-08` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 7 board frames `18:50:37.229`, `18:50:37.562`, and `18:50:37.895`
Brief Match: the chamber remains clean, synthetic, and reflection-friendly, and the gate still reads well
Missed Brief Reads: a truly dominant hard wall-space partition that breaks the radial fan geometry
Hard-Fail Triggers: the direct view is still owned by teal fan beams converging to a point, so the rectilinear partition remains an accent instead of the chamber owner
verdict: fail
notes: No meaningful breakout. The next pass must make one hard rectilinear wall-space partition the primary wall owner instead of tuning the beams further.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-09` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 9 board frames `19:03:38.556`, `19:03:39.053`, and `19:03:39.387`
Brief Match: the viewport, eclipse ring, and cold space-facing probe still work, and the preset still lands as a coherent window-into-space mood
Missed Brief Reads: stronger authored station identity in the enclosing room, especially below the opening
Hard-Fail Triggers: the enclosure still reads as a soft gray shell with only faint lower spill, so the direct background remains a viewport in a bowl rather than a cold metallic station interior
verdict: fail
notes: Flat. The wall-plus-floor spill restoration was too soft to change the failure class. The next pass must introduce one stronger lower-room structural frame or deck silhouette tied to the viewport spill.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-09` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 9 board frames `19:03:42.558`, `19:03:42.891`, and `19:03:43.224`
Brief Match: the scene still reads as an oppressive infernal chamber with a strong lower rift and coherent amber-black pressure in both background and probe
Missed Brief Reads: a dominant volcanic floor fracture network that overtakes the glow source
Hard-Fail Triggers: the floor crack web still reads as support around the rift instead of the hero structure, so the preset remains rift-first rather than fissure-first
verdict: fail
notes: Flat. The narrow crack-carrier boost was too small to change ownership. The next pass must increase visible floor crack branching and coverage around the central rift, not just brightness.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-09` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 9 board frames `19:04:14.538`, `19:04:15.037`, and `19:04:15.371`
Brief Match: the chamber still holds a clean cyan-black synthetic mood, and the reflective probe remains stronger than the background
Missed Brief Reads: direct-view rectilinear or code-partition structure that overtakes the radial fan geometry
Hard-Fail Triggers: the direct background still behaves like the same radial spotlight/fan solution, so the digital partition idea remains mostly probe-led
verdict: fail
notes: Flat. The previous worker attempt produced no effective diff, and the latest validated batch confirms no breakout. The next pass must force one visible wall-space partition layer to dominate direct view, even if current beam drama has to be reduced.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-10` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 10 board frames `19:09:38.522`, `19:09:38.855`, and `19:09:39.189`
Brief Match: the viewport and eclipse remain intact, and the larger lower-side cues finally restore a clearer chamber anchor in both direct view and probe
Missed Brief Reads: one coherent metallic station base or deck under the viewport rather than separated bright accents
Hard-Fail Triggers: the room still reads as a soft chamber with side fins instead of a distinctly authored station interior
verdict: fail
notes: Improved. The lower-room cues are finally helping, but they still need to connect into one continuous deck or frame base.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-10` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 10 board frames `19:09:42.356`, `19:09:42.690`, and `19:09:43.192`
Brief Match: the fractured hell-floor is much more present, and the scene now sells a coherent infernal fracture field in both background and probe
Missed Brief Reads: the widened floor fissure network still shares too much attention with the brightest top band and central vertical rift
Hard-Fail Triggers: the composition is still not fully fissure-first because the central glow path competes too hard with the floor network
verdict: fail
notes: Improved. The next pass should slightly reduce top-band or vertical-rift dominance so the widened floor fissure network becomes the clear hero.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-10` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 10 board frames `19:10:14.537`, `19:10:14.871`, and `19:10:15.204`
Brief Match: the scene still provides soft synthetic ambient lighting and a usable probe-side cyan-violet mood
Missed Brief Reads: hard digital chamber structure and a direct-view partition owner
Hard-Fail Triggers: the direct background regressed into a soft violet ambient wash, losing both the prior tech energy and any meaningful partition/data-structure read
verdict: fail
notes: Regressed. Park this lane until it gets a harder structural reset instead of more soft partition tuning.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-11` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 11 board frames `19:12:55.717`, `19:12:56.051`, and `19:12:56.383`
Brief Match: the viewport/eclipsed space read remains strong, and the larger lower-room base now gives the chamber its clearest station-window-in-a-room state so far
Missed Brief Reads: a more coherent metallic station base with clearer deck or panel ownership
Hard-Fail Triggers: the enclosure still reads as a soft chamber with bright side wedges rather than one authored station structure
verdict: fail
notes: Improved again. The next pass should connect the lower-side cues into one coherent deck/frame silhouette under the viewport.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-11` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 11 board frames `19:12:59.555`, `19:12:59.887`, and `19:13:00.388`
Brief Match: the scene now clearly presents a cracked infernal floor with a healthier fracture field around the rift in both background and probe
Missed Brief Reads: the widened floor fissure network still competes somewhat with the brightest top band and central vertical rift
Hard-Fail Triggers: the preset still is not fully fissure-first because the central glow path remains too dominant
verdict: fail
notes: Improved again. The next pass should slightly reduce top-band or vertical-rift dominance so the widened floor fissure network becomes the unmistakable hero read.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-12` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 12 board frames `19:17:53.081`, `19:17:53.415`, and `19:17:53.916`
Brief Match: the viewport/eclipsed space read remains solid, and the connected lower frame now gives the room its strongest station-like architectural base so far
Missed Brief Reads: a bit more darker structural panel or deck character above and behind the bright base frame
Hard-Fail Triggers: the chamber is still slightly too clean and abstract to read as a distinct metallic station interior
verdict: fail
notes: Improved again. The next pass should add one restrained darker structural panel layer above or behind the bright lower frame so the room gains material station depth without hurting the viewport.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-12` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 12 board frames `19:17:56.585`, `19:17:56.918`, and `19:17:57.417`
Brief Match: the scene now reads as convincing hell-core world art, with a strong infernal fracture field, clear rift, and solid amber-black pressure in both background and probe
Missed Brief Reads: the floor fissure network is near-hero but not fully unquestioned because the top hot band and central vertical rift still overclaim a little attention
Hard-Fail Triggers: the composition still shares too much focus with the upper band and central glow path instead of letting the floor network fully own the frame
verdict: fail
notes: Flat-to-near-pass. The next pass should trim the brightness or weight of the top hot band so the floor crack network becomes the unquestioned primary read.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-13` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 13 board frames `19:26:14.829`, `19:26:15.163`, and `19:26:15.496`
Brief Match: the viewport/eclipsed space read remains strong, the connected lower frame still anchors the room, and the deeper backplate finally adds some metallic station depth behind the bright base
Missed Brief Reads: one subtle live interior panel or deck cadence so the room reads as an active metallic chamber instead of a clean static shell
Hard-Fail Triggers: the room is still slightly too static and clean to feel like a fully authored station interior
verdict: near-pass
notes: Improved. The darker backplate helped, and the remaining miss is now a very small one: a restrained animated interior panel/deck cadence inside the lower chamber so the room feels lived-in without hurting the clean viewport read.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-13` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 13 board frames `19:26:19.501`, `19:26:19.834`, and `19:26:20.168`
Brief Match: the scene now convincingly reads as hell-core world art, with a strong infernal fracture field, a clear rift, and solid pressure in both direct background and probe
Missed Brief Reads: the surrounding floor fissure network still needs to become the unquestioned hero over the remaining central vertical-rift emphasis
Hard-Fail Triggers: the composition still shares a little too much focus with the central glow path instead of letting the floor cracks fully dominate
verdict: near-pass
notes: Near-pass. The top-band suppression helped again; the remaining miss is narrower now and is best addressed by slightly reducing the central vertical rift/column emphasis so the floor fissure network becomes the dominant infernal read.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-14` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 14 board frames `19:32:35.566`, `19:32:35.901`, and `19:32:36.233`
Brief Match: the viewport/eclipsed space read remains strong, the lower frame and darker backplate still sell a station room, and the new lower-chamber cadence now contributes some life to the metallic interior
Missed Brief Reads: slightly more readable interior cadence in direct background so the room feels unmistakably active rather than mostly implied through the probe
Hard-Fail Triggers: the animated room cadence is still a touch too subtle in direct background
verdict: near-pass
notes: Near-pass. This is still the right branch. The next pass should make the interior cadence slightly more readable in direct background without compromising the clean viewport or bright lower base.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-14` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 14 board frames `19:32:40.238`, `19:32:40.573`, and `19:32:40.906`
Brief Match: the scene continues to read as convincing hell-core world art, with a strong infernal fracture field and stable pressure in both direct background and probe
Missed Brief Reads: the side floor fissures still need slightly more dominance so they overtake the last central-axis symmetry and make the fracture field feel broader than the vertical core
Hard-Fail Triggers: the central-axis composition still holds a bit too much weight relative to the lateral floor crack network
verdict: near-pass
notes: Near-pass. The central-rift suppression helped, but the best remaining lever is now on the fissure network itself: slightly widen and brighten the side floor fissures so they become more dominant than the vertical core.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-15` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 15 board frames `19:39:42.680`, `19:39:43.013`, and `19:39:43.347`
Brief Match: the viewport/eclipsed space read remains strong, the lower frame and cadence both read more clearly in direct background, and the chamber now feels closer to a live station room than a static shell
Missed Brief Reads: one slightly darker layered interior back-structure behind the cadence so the room reads as a more materially layered station chamber
Hard-Fail Triggers: the chamber still sits a little too cleanly behind the viewport frame
verdict: near-pass
notes: Near-pass. The cadence boost helped in direct background. The best remaining lever is a slightly darker interior back-structure behind that cadence so the room reads as layered metallic architecture, not just a clean frame.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-15` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 15 board frames `19:39:47.350`, `19:39:47.684`, and `19:39:48.018`
Brief Match: the infernal fracture field remains strong and readable in both direct background and probe, and the lateral fissures now carry more of the floor read than before
Missed Brief Reads: the top hot ceiling band still holds a little too much weight over the floor-side fracture field
Hard-Fail Triggers: the overhead glow still competes slightly with the floor fissures for the dominant infernal structure read
verdict: near-pass
notes: Near-pass. The lateral fissure push helped, but the best remaining move is a final small reduction in the top hot ceiling band so the floor-side fissure field wins cleanly.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-16` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 16 board frames `19:49:31.674`, `19:49:32.008`, and `19:49:32.341`
Brief Match: the viewport/eclipsed space read is stable again, the layered metallic depth and live cadence both survive, and the room once more reads as `Void Station` rather than flipping into a different scene class
Missed Brief Reads: one clearer metallic panel or deck articulation around the lower backplate so the chamber reads as a distinct station interior instead of a very clean viewport shell
Hard-Fail Triggers: none beyond the remaining lack of distinct station-specific panel articulation
verdict: near-pass
notes: Near-pass. The stability repair worked. The next pass should add one clearer metallic panel/deck articulation around the lower backplate so the room reads as authored station architecture rather than a clean generic shell.

Preset: `Hell Core`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-16` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 16 board frames `19:49:36.345`, `19:49:36.679`, and `19:49:37.012`
Brief Match: the infernal fracture field now dominates the frame convincingly, with the floor-side fissures and lower-rift relationship reading as the primary structure in both direct background and reflective probe
Missed Brief Reads: none material enough to block the preset
Hard-Fail Triggers: none
verdict: pass
notes: Pass. The crack-linked floor segmentation finally broke the remaining smooth chamber planes enough for the fissure-first contract to win cleanly.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-17` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 17 board frames `19:55:43.428`, `19:55:54.852`, and `19:56:06.276`
Brief Match: the viewport/eclipsed-space read remains solid and the room still sits on the healthier near-pass branch
Missed Brief Reads: clearer metallic panel/deck articulation around the lower backplate
Hard-Fail Triggers: the room is still too clean and smooth around the lower chamber to read as authored station structure
verdict: near-pass
notes: Essentially flat from the prior near-pass state. The viewport and room-vs-space split still work, but the lower-room articulation did not become clearer enough to move the failure class.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-17` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 17 board frames `19:55:43.428`, `19:55:54.852`, and `19:56:06.276`
Brief Match: reflective-probe synthetic chamber support survives
Missed Brief Reads: hard wall-space partition ownership in direct background
Hard-Fail Triggers: the hard-rect reset materially regressed into a pale mint-white wash with no convincing dominant partition owner
verdict: fail
notes: Material regression. The branch lost the harder digital chamber identity and collapsed toward a pale synthetic wash.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-18` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 18 board frames `19:59:04.891`, `19:59:16.309`, and `19:59:27.728`
Brief Match: the bright lower frame returned, the viewport/eclipsed-space read stayed clean, and the station-room base recovered materially
Missed Brief Reads: one very subtle darker metallic panel break in the lower chamber
Hard-Fail Triggers: the room still reads a little too cleanly around the lower backplate
verdict: near-pass
notes: Materially improved over the regressed pass because the bright lower frame returned. The remaining miss is now very small again.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-18` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 18 board frames `19:59:04.891`, `19:59:16.309`, and `19:59:27.728`
Brief Match: the synthetic chamber recovered from the blown-out branch and the probe still carries coherent digital-space mood
Missed Brief Reads: a dominant direct-view partition owner
Hard-Fail Triggers: the split-first correction helps, but the background still does not let one hard partition clearly dominate
verdict: fail
notes: Materially improved from the blown-out branch, but still not enough. The next truthful lever was a darker, wider primary split.

Preset: `Void Station`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-19` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 19 board frames `20:05:31.023`, `20:05:42.440`, and `20:05:53.857`
Brief Match: the viewport/eclipsed-space read and bright lower frame still work cleanly
Missed Brief Reads: one readable lower-chamber panel seam or deck break
Hard-Fail Triggers: the micro-backplate deepen was too subtle to add distinct authored station structure
verdict: near-pass
notes: Flat. The micro-backplate deepen was too small to change the failure class. The next lever is one readable lower-chamber panel seam or deck break.

Preset: `Digital Matrix`
Date: 2026-03-13
Capture Run: `2026-03-13-epu-showcase-20preset-replay-19` (paired against the same-window second sweep with `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: focused Wave 19 board frames `20:05:31.023`, `20:05:42.440`, and `20:05:53.857`
Brief Match: the direct background now has a real dominant hard split/partition, and the reflective probe still carries a coherent synthetic chamber read
Missed Brief Reads: slightly brighter primary partition edge/event without reintroducing wash
Hard-Fail Triggers: none beyond the remaining need for a slightly stronger partition-edge event
verdict: near-pass
notes: Materially improved. The darker-wider split produced the first real near-pass for this lane and puts it back into the active shipping sequence.

Preset: `Toxic Wasteland`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-33` (Wave 31-33 lane all validated with `0` hash mismatches across `60` frame pairs per wave)
Frames Reviewed: exact-image Wave 33 board frames from the latest paired full sweep
Brief Match: the toxic palette and corrosive glow still survive, and the reflective probe still carries poisoned-atmosphere support
Missed Brief Reads: a coherent ruined exterior shell, readable toxic-industrial envelope, and one stable exterior silhouette that beats the current chamber-like bowl
Hard-Fail Triggers: the scene still slips between exterior ruin and enclosed shell, so the direct background does not resolve as a distinct toxic wasteland exterior
verdict: fail
notes: Still blocked. Repeated exact-image review says the lane is no longer failing on color or corruption mood; it is failing on shell/exterior confusion. The next truthful lever has to make the exterior envelope win decisively instead of adding more toxic support effects.

Preset: `War Zone`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-33` (Wave 31-33 lane all validated with `0` hash mismatches across `60` frame pairs per wave)
Frames Reviewed: exact-image Wave 33 board frames from the latest paired full sweep
Brief Match: smoke, flare heat, and militarized tracer accents survive, and the reflective probe still reads as battle-adjacent pressure
Missed Brief Reads: one coherent ruined-front or trench-line silhouette, a stable battle-line owner in direct background, and clearer front-to-back battlefield structure
Hard-Fail Triggers: the scene still reads as an abstract brown-black smoke field because no single battle-line silhouette beats the haze
verdict: fail
notes: Still blocked. Repeated exact-image review says the lane is not waiting on more tracer or smoke polish; it is waiting on one coherent battle-line silhouette that owns the frame ahead of the haze.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-39` (the same branch also held through replay-38; both paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 39 board frames from the latest paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower bay anchor, and room-vs-space split all remain stable in both direct background and reflective probe
Missed Brief Reads: one clearer lower-chamber panel seam or deck articulation so the room stops reading as slightly too clean around the backplate
Hard-Fail Triggers: none beyond the remaining small authored-station-structure miss
verdict: near-pass
notes: Stable near-pass on the same branch. The lane is no longer oscillating; the remaining miss is still one readable lower-chamber panel/deck break, not a larger composition or identity failure.

Preset: `Digital Matrix`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-39` (the same branch also held through replay-38; both paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 39 board frames from the latest paired full sweep
Brief Match: direct-view partition identity is stable again, and the reflective probe still carries coherent synthetic chamber mood
Missed Brief Reads: a slightly stronger primary partition-edge event so the chamber reads as fully authored digital architecture rather than a restrained near-pass
Hard-Fail Triggers: none beyond the remaining need for a stronger partition-edge owner
verdict: near-pass
notes: Stable near-pass. The replay-17 regression state is no longer the live truth; the darker split-first identity is holding and the next lever remains polish, not rescue.

Preset: `Ocean Depths`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-40`
Frames Reviewed: exact-image Wave 40 board frames from the latest authoritative paired full sweep
Brief Match: underwater palette, suspended particulate, and broad pressure still survive in both direct background and reflective probe
Missed Brief Reads: one unmistakable trench-floor owner, a darker seabed anchor, and a cleaner hero vent/focal that beats the atmosphere
Hard-Fail Triggers: the lane still reads atmosphere-first because no trench-floor owner beats the soft water column
verdict: fail
notes: Still blocked. The direct scene remains underwater mood first and trench structure second; the next truthful lever is still one darker lower-scene trench/floor owner rather than more haze, shafts, or particulate.

Preset: `Neon Arcade`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-40`
Frames Reviewed: exact-image Wave 40 board frames from the latest authoritative paired full sweep
Brief Match: bright entertainment-space palette and reflective probe energy still survive
Missed Brief Reads: explicit cabinet-row clustering, a readable arcade interior, and wall-side fixture ownership in direct background
Hard-Fail Triggers: the lane still reads as a radial purple chamber rather than an arcade/cabinet-row room
verdict: fail
notes: Still blocked. The latest pass did not break the chamber/fan read; the next truthful lever must replace one broad radial wall-energy carrier with a tighter repeated cabinet/fixture owner.

Preset: `Astral Void`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-43` (the same branch also held through replay-41 and replay-42; all three paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 43 board frames from the latest authoritative paired full sweep
Brief Match: one hero celestial event now survives clearly in direct background, the secondary support body stays subordinate, and the reflective probe still carries a coherent cosmic tableau
Missed Brief Reads: only minor final-polish restraint remains if the lane is ever reopened
Hard-Fail Triggers: none material enough to block banking this branch as a survivor near-pass
verdict: near-pass
notes: Stable banked near-pass. The lane moved from parked candidate to active survivor and then held through three validated full sweeps without reopening the old starfield-first failure class.

Preset: `Astral Void`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-41..43` (all three paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 43 board plus state-hold confirmation through the same survivor branch
Brief Match: the hero celestial event, subordinate support body, and coherent probe-side cosmic tableau all remain stable
Missed Brief Reads: only low-risk final-polish restraint if the lane is ever reopened
Hard-Fail Triggers: none material enough to justify reopening the old starfield-first failure class
verdict: near-pass
notes: Durable state sync: this lane is now banked as a stable near-pass rather than an active candidate.

Preset: `Neon Arcade`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-44` (the late branch now holds through replay-40 and replay-44; all Waves 41-44 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 44 board frames from the latest authoritative paired full sweep
Brief Match: bright entertainment palette and reflective probe energy still survive
Missed Brief Reads: explicit cabinet-row clustering, a readable arcade interior, and wall-side fixture ownership in direct background
Hard-Fail Triggers: the lane still reads as a radial purple chamber rather than an arcade/cabinet-row room
verdict: fail
notes: Still blocked. This branch is not the next truthful push; the chamber/fan read still beats cabinet ownership, so the lane should stay parked until a tighter cabinet-row reset is ready.

Preset: `Ocean Depths`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-45` (paired full sweep validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: exact-image Wave 45 board frames from the latest authoritative paired full sweep
Brief Match: underwater palette, depth haze, and pale shafts still survive in both direct background and reflective probe
Missed Brief Reads: one darker trench-basin owner, a readable lower seabed anchor, and a focal that beats the water column
Hard-Fail Triggers: the lane still reads as pale shafts and haze first because no trench-basin owner beats the atmosphere
verdict: fail
notes: Parked again. This branch is still blocked on a real lower-scene trench owner, not more atmospheric support.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-48` (the same late branch held through replay-46 and replay-47; all three paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 48 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower bay anchor, and room-vs-space split remain stable in both direct background and reflective probe
Missed Brief Reads: one stronger lower-backplate break so the room stops reading slightly too clean around the lower chamber
Hard-Fail Triggers: none beyond the remaining small authored-station-structure miss
verdict: near-pass
notes: Stable near-pass through Wave 48. The seam-only micro branch is now closed as too weak; the next truthful lever is one slightly bolder but still singular lower-backplate break.

Preset: `Digital Matrix`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-48` (the same late branch held through replay-46 and replay-47; all three paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 48 board frames from the latest authoritative paired full sweep
Brief Match: direct-view partition identity remains stable and the reflective probe still carries coherent synthetic chamber mood
Missed Brief Reads: one slightly brighter tiny central node core so the partition system reads fully authored without widening the event again
Hard-Fail Triggers: none beyond the remaining tiny central-node emphasis miss
verdict: near-pass
notes: Stable near-pass through Wave 48. The convergence-wide micro branch is now closed as too weak; the next truthful lever is brighten only the tiny central node core one last step.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-49` (paired full sweep validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: exact-image Wave 49 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower bay anchor, and room-vs-space split remain stable in both direct background and reflective probe
Missed Brief Reads: one darker, more continuous lower structural plate so the lower room reads as authored station architecture instead of a slightly clean shell
Hard-Fail Triggers: none beyond the remaining small authored-station-structure miss
verdict: near-pass
notes: Stable near-pass through Wave 49. The seam-only branch is closed as too weak, and the bolder lower-backplate break helped but still did not cross; the next truthful lever is make that lower break slightly darker and more continuous as one clear structural plate.

Preset: `Digital Matrix`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-49` (paired full sweep validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: exact-image Wave 49 board frames from the latest authoritative paired full sweep
Brief Match: direct-view partition identity remains stable and the reflective probe still carries coherent synthetic chamber mood
Missed Brief Reads: one tiny crisp highlight on the primary vertical partition edge so the chamber reads fully authored without widening or washing the event
Hard-Fail Triggers: none beyond the remaining partition-edge micro-emphasis miss
verdict: near-pass
notes: Stable near-pass through Wave 49. The convergence-wide and node-core branches are both now closed as too weak; the next truthful lever is a tiny crisp highlight on the primary vertical partition edge.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-51` (the same late branch held through replay-50; all Waves 46-51 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 51 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower bay anchor, and room-vs-space split remain stable in both direct background and reflective probe
Missed Brief Reads: only a small authored lower-plate polish remains if the lane is ever reopened
Hard-Fail Triggers: none material enough to justify more churn on the current branch
verdict: near-pass
notes: Banked strong near-pass through Wave 51. Stop recommendation: do not spend another active wave here unless stronger lanes stall or a final shipping-polish pass is explicitly needed.

Preset: `Digital Matrix`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-51` (the same late branch held through replay-50; all Waves 46-51 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 51 board frames from the latest authoritative paired full sweep
Brief Match: direct-view partition identity remains stable and the reflective probe still carries coherent synthetic chamber mood
Missed Brief Reads: only a tiny final partition-edge polish remains if the lane is ever reopened
Hard-Fail Triggers: none material enough to justify more churn on the current branch
verdict: near-pass
notes: Banked strong near-pass through Wave 51. Stop recommendation: do not spend another active wave here unless stronger lanes stall or a final shipping-polish pass is explicitly needed.

Preset: `Neon Metropolis`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-52` (paired full sweep validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: pending
Brief Match: pending
Missed Brief Reads: pending
Hard-Fail Triggers: pending
verdict: pending
notes: Reopened in Wave 52 on a clean validated pair. Review is the next required step before this lane can move further.

Preset: `Neon Metropolis`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-55` (the reopened branch held through replay-53 and replay-54; all Waves 53-55 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 55 board frames from the latest authoritative paired full sweep
Brief Match: neon palette, wet-floor energy, and reflective probe glow still survive
Missed Brief Reads: one coherent alley silhouette, a hero sign/event, and a direct-view urban place owner that beats the current abstract neon field
Hard-Fail Triggers: the reopened branch still fails because the scene remains a diffuse neon pattern space instead of a decisive city alley environment
verdict: fail
notes: Park this branch again. The reopened Wave 52 path validated cleanly through Waves 53-55 but did not cross on exact-image review; `Sakura Shrine` is now the next active survivor lane.

Preset: `Sakura Shrine`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-59` (the reopened branch held through replay-56, replay-57, and replay-58; all Waves 56-59 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 59 board frames from the latest authoritative paired full sweep
Brief Match: cool sacred palette, soft petal energy, and reflective probe atmosphere still survive
Missed Brief Reads: one decisive shrine silhouette, mossy path ownership, and a direct-view sacred place owner that beats the current diffuse decorative field
Hard-Fail Triggers: the reopened branch still fails because the scene remains an abstract cool decorative space instead of an unmistakable shrine environment
verdict: fail
notes: Park this branch again. The reopened Wave 56 path validated cleanly through Wave 59 but did not cross on exact-image review; the next active survivor selection must come from outside this branch.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-60` (paired full sweep validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: exact-image Wave 60 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower-bay anchor, and room-vs-space split remain stable in both direct background and reflective probe
Missed Brief Reads: only a small authored lower-room plate polish remains if the lane is ever reopened again
Hard-Fail Triggers: none material enough to justify further churn on the current branch
verdict: near-pass
notes: Reopened in Wave 60 on a clean validated pair, but exact-image review still holds the lane on the same strong near-pass branch. Rebank it with the same stop recommendation unless a final shipping-polish pass is explicitly needed.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-62` (the reopened seam branch held through replay-61; both paired sweeps validated at `0` hash mismatches across all `60` frame pairs)
Frames Reviewed: exact-image Wave 62 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower-bay anchor, and room-vs-space split remain stable, and this is the healthiest metallic-seam branch on the late reopen
Missed Brief Reads: only a tiny adjacent panel-break / layered-metal cue remains if the lane is ever reopened
Hard-Fail Triggers: none material enough to justify keeping the same seam-only branch active
verdict: near-pass
notes: Wave 62 is the best seam-branch state, but reviewers called diminishing returns here and banked it. `Digital Matrix` is now the next active survivor lane.

Preset: `Void Station`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-62` (the reopened seam branch held through replay-61; both Waves 61-62 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 62 board frames from the latest authoritative paired full sweep
Brief Match: the circular viewport, eclipse read, bright lower-bay anchor, and room-vs-space split remain stable in both direct background and reflective probe
Missed Brief Reads: only tiny authored lower-room seam polish remains if the lane is ever reopened again
Hard-Fail Triggers: none material enough to justify more churn on the current branch
verdict: near-pass
notes: Stable near-pass through Wave 62. This is the best state of the seam branch, and reviewers called diminishing returns here; bank this branch again with a stop recommendation.

Preset: `Digital Matrix`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-64` (the reopened branch held through replay-63; both Waves 63-64 paired sweeps validated at `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 64 board frames from the latest authoritative paired full sweep
Brief Match: direct-view partition identity remains stable and the reflective probe still carries coherent synthetic chamber mood
Missed Brief Reads: a decisive partition owner that clearly beats the residual radial-fan ownership without washing out the chamber
Hard-Fail Triggers: the reopened branch still does not materially overtake the same radial-fan ownership limit
verdict: near-pass
notes: Stable near-pass through Wave 64. This reopen did not materially beat the same radial-fan ownership limit; bank the lane here due to diminishing returns. `Desert Mirage` is now the next active lane.

Preset: `Desert Mirage`
Date: 2026-03-14
Capture Run: `2026-03-14-epu-showcase-20preset-replay-69` (the dune-horizon plus dust-suppression branch held through replay-65, replay-67, and replay-69; the reviewed exact-image waves split `fail / near-pass / near-pass`, and Wave 69 validated cleanly with `0` hash mismatches across `60` frame pairs)
Frames Reviewed: exact-image Wave 69 board frames from the latest authoritative paired full sweep
Brief Match: the dark dune horizon and sand-floor owners now survive clearly enough in both direct background and reflective probe to read as a real desert place rather than just hot tan atmosphere
Missed Brief Reads: the place still lands slightly soft/generic, and some residual side-wall wash keeps the scene from feeling fully authored
Hard-Fail Triggers: none material enough to justify discarding the branch; the remaining miss is restraint/polish rather than a structural place-read collapse
verdict: near-pass
notes: The dune-horizon plus dust-suppression branch progressed honestly from fail toward survivor status across Waves 65-69. Bank this lane as a near-pass survivor on Wave 69 rather than churning it immediately. `Enchanted Grove` in `set_05_06.rs` is now the next active lane.

Preset: `Enchanted Grove`
Date: 2026-03-15
Capture Run: `20260315-043247-enchanted-grove-rebuild` (authoritative single 20-scene sweep reused as a shared-artifact review for `Crystal Cavern` and `Moonlit Graveyard`)
Frames Reviewed: fresh subagent review of the three Enchanted Grove frames copied under `agent/runs/20260315-043247-enchanted-grove-rebuild/screenshots/single/`
Brief Match: warm green atmosphere and magical motes survive, but the scene does not reliably read as an enchanted forest clearing
Missed Brief Reads: no clear forest silhouette or canopy framing the sky, the floor does not read as a mossy/grassy ground plane, and the sunlight is still diffuse patchwork instead of readable warm shafts
Hard-Fail Triggers: hard fail triggered on `generic green fog instead of a forest clearing` and `no clear light-shaft structure`; the fireflies also drift toward ember-spam density
verdict: fail
notes: Fresh-context reviewer verdict. Motion exists only as soft texture shimmer in the light pattern and mote field; the next truthful lever is an unmistakable clearing layout with dark canopy silhouettes, a grounded moss floor owner, a few strong diagonal shafts, and fewer motes.

Preset: `Crystal Cavern`
Date: 2026-03-15
Capture Run: `20260315-043247-enchanted-grove-rebuild` (shared-artifact review from the same authoritative single 20-scene sweep)
Frames Reviewed: fresh subagent review of the three Crystal Cavern frames copied under `agent/runs/20260315-043247-enchanted-grove-rebuild/screenshots/single/`
Brief Match: faceted icy structure and cold reflective light read immediately, and the glowing seams partially satisfy the luminous-vein requirement
Missed Brief Reads: cave enclosure and chamber depth remain weak because the scene is still too blown out and spatial planes feel shallow and abstract rather than cavernous
Hard-Fail Triggers: none of the explicit hard-fail conditions fire; it does not read as a generic blue room and the faceted crystalline structure is clearly present
verdict: near-pass
notes: Fresh-context reviewer verdict. Motion only barely registers as shimmer, so the next truthful lever is darker recess planes and clearer wall-to-floor cavern framing rather than more brightness or seam energy.

Preset: `Moonlit Graveyard`
Date: 2026-03-15
Capture Run: `20260315-043247-enchanted-grove-rebuild` (shared-artifact review from the same authoritative single 20-scene sweep)
Frames Reviewed: fresh subagent review of the three Moonlit Graveyard frames copied under `agent/runs/20260315-043247-enchanted-grove-rebuild/screenshots/single/`
Brief Match: the palette is cold and nocturnal, and there is some recession toward a bright focal point
Missed Brief Reads: grave-marker language is absent, cemetery silhouette does not read, low mist is not visible, and the supernatural event remains too abstract to parse as a spectral rift inside a graveyard scene
Hard-Fail Triggers: hard fail triggered because there is still no graveyard read; the scene lands as an abstract dark outdoor or tunnel-like composition rather than a moonlit cemetery
verdict: fail
notes: Fresh-context reviewer verdict. Motion reads as abstract lighting movement rather than drifting mist or spectral disturbance; the next truthful lever is unmistakable crooked headstones plus ground-hugging mist before refining the rift.

Preset: `Sky Ruins`
Date: 2026-03-15
Capture Run: `20260315-045409-sky-ruins-rebuild` (authoritative single 20-scene sweep for the first dedicated rebuild pass after the Wave 70 `Enchanted Grove` fail)
Frames Reviewed: fresh subagent review of the three Sky Ruins frames copied under `agent/runs/20260315-045409-sky-ruins-rebuild/screenshots/single/`
Brief Match: weak at best; only a warm stone / beige palette survives consistently
Missed Brief Reads: no readable ruined skyline or colonnade silhouette, no convincing marble floor plane, no layered cloud depth, and the central orb dominates instead of helping the environment read
Hard-Fail Triggers: hard fail triggered because there is no clear ruins read, the clouds flatten into a generic warm wash, and the elevated open-air feeling is lost
verdict: fail
notes: Fresh-context reviewer verdict. Motion is effectively unreadable across the reviewed frames; the next truthful lever is a readable ruin silhouette and floor plane against layered sunlit cloud depth with obvious cloud/light drift.

Preset: `Crystal Cavern`
Date: 2026-03-15
Capture Run: `20260315-050734-crystal-cavern-implementation` (authoritative single 20-scene sweep for the first dedicated Crystal rebuild pass after it became the next active lane)
Frames Reviewed: fresh subagent review of Crystal Cavern frames `36-38` from `agent/runs/20260315-050734-crystal-cavern-implementation/screenshots/single/`
Brief Match: faceted crystal structure survives clearly, cold reflective palette survives, and the probe still picks up crystalline light/shard breakup
Missed Brief Reads: cave enclosure is still not established in direct view, chamber depth remains too weak and overexposed, luminous veins collapse into generic bright seam lines, and the direct-view motion read is still too slight
Hard-Fail Triggers: the latest lever did not fix the known blocker because broad flat white shelves/wedges still dominate the frame and the frame-to-frame delta is borderline-to-tiny in direct view
verdict: fail
notes: Fresh-context reviewer verdict. This is still a crystal scene but not yet a crystal cavern; the probe carries more of the identity than the background. Next truthful move is one more darker enclosure/depth pass, but if that still yields only micro-delta improvement the lane should rotate instead of churning.

Preset: `Crystal Cavern`
Date: 2026-03-15
Capture Run: `20260315-052122-crystal-cavern-implementation` (authoritative single 20-scene sweep for the final allowed Crystal rebuild wave before stop-loss rotation)
Frames Reviewed: fresh subagent review of Crystal Cavern frames `36-38` from `agent/runs/20260315-052122-crystal-cavern-implementation/screenshots/single/`
Brief Match: cold reflective palette survives, faceted/crystalline structural language is present, and motion across frames `36-38` is now visibly advancing
Missed Brief Reads: direct background still does not read as an enclosed cave chamber, broad bright shelves still flatten depth instead of giving wall-to-floor cavern framing, luminous veins still read mostly as seam lines, and the probe still does not show a convincing cold cavern volume with readable enclosure
Hard-Fail Triggers: hard fail on cave read; despite better motion, the scene still presents as an abstract overlit blue-white room/dome rather than a cave enclosure
verdict: fail
notes: Fresh-context reviewer verdict. Motion is no longer the main blocker, but scene identity still fails decisively. This final pass did not cross or become bankable; rotate the lane by stop-loss.

Benchmark: `Frozen Bed`
Date: 2026-03-15
Capture Run: `20260315-132701-frozen-bed-identity` (authoritative benchmark replay for the authored Frozen Bed floor-owner rewrite after live validation promoted the authored baseline)
Frames Reviewed: fresh subagent review of the Frozen Bed benchmark triplet under `agent/runs/20260315-132701-frozen-bed-identity/screenshots/a/`
Brief Match: directionally closer than the old dark-waterline branch; cracked icy/glass language survives
Missed Brief Reads: the scene still does not read as a grounded frozen bed or outdoor frozen floor owner, the probe stays glossy/refractive, and frame-to-frame motion is effectively absent
Hard-Fail Triggers: hard fail triggered because the direct view now reads as bright cracked glass / polished pane ice on a white void rather than a grounded frozen place
verdict: fail
notes: Fresh-context reviewer verdict. Live iteration found a real improvement over the old waterline class, but the authoritative benchmark still does not pass the gate into Frozen Tundra.

Benchmark: `Front Mass`
Date: 2026-03-15
Capture Run: `20260315-134149-front-mass-body-ownership` (authoritative benchmark replay for the tightened Front Mass storm-body rewrite after live validation promoted the authored baseline)
Frames Reviewed: fresh subagent review of the Front Mass benchmark triplet under `agent/runs/20260315-134149-front-mass-body-ownership/screenshots/a/`
Brief Match: better than the old explicit panel-wall family; storm-light intent survives faintly
Missed Brief Reads: the direct frame still lacks a decisive storm body owner, the probe read is weak, and motion is effectively absent
Hard-Fail Triggers: hard fail triggered because the reviewed frames still collapse toward a flat blue-gray field / washed slab with no convincing front mass taking ownership
verdict: fail
notes: Fresh-context reviewer verdict. The new concept is directionally healthier, but it still does not unblock Storm Front.

Benchmark: `Front Mass`
Date: 2026-03-15
Capture Run: `20260315-150518-front-mass-storm-owner` (authoritative paired benchmark replay for the darker squall-shelf authored reset after the reused live lane promoted the authored baseline)
Frames Reviewed: fresh subagent review of the Front Mass benchmark pair under `agent/runs/20260315-150518-front-mass-storm-owner/screenshots/a/` and `agent/runs/20260315-150518-front-mass-storm-owner/screenshots/b/`
Brief Match: storm-owner intent survives slightly more clearly than the prior run, and the darker plume is at least visible
Missed Brief Reads: the direct frame still does not land as one scene-owning weather front, the probe still reads as a hemispheric waterline split, motion remains negligible, and seam/panel guides are still visible
Hard-Fail Triggers: hard fail triggered because the reviewed frames still read as a generic pale split wall with a thin dark plume rather than a decisive front body
verdict: fail
notes: Fresh-context reviewer verdict. The second same-day replay is still below benchmark pass quality and does not unblock `Storm Front`.

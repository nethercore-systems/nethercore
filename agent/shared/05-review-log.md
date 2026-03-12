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

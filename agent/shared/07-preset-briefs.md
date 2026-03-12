# Preset Briefs

This file is the visual source of truth for the EPU showcase roster.

Every design, implementation, fix, and review run should treat these briefs as the target image contract.

The `Text prompt:` line under each preset is the canonical single-prompt description to hand to improvers and reviewers. Treat it as the scene-intent contract, not optional flavor text and not a demand for literal prop-by-prop rendering.

## Procedural Art Constraints

- Treat EPU presets as procedural/generative world art. The target is a strong metaphorical place read, not literal modeled props or screen-space UI.
- Judge for shippable cubemap-replacement value: ambient lighting, reflections, and direct-view mood should all benefit, even when the result stays abstract and constrained.
- The showcase captures include a reflective hero probe in the foreground while the EPU renders in the background. Reviewers must judge both the direct world read and what the probe proves about ambient/reflection contribution.
- Bounds layers establish the scene envelope, horizon, enclosure, and region masks. Feature layers must carry the readable world detail and most of the motion.
- Motion is variant-specific, not opcode-generic. Do not assume every layer animates from `param_d`.
- In the current runtime, reliable phase-driven movers include `FLOW`, `GRID`, `LOBE`, `DECAL`, `VEIL/RAIN_WALL`, `PLANE/WATER`, and `PORTAL/VORTEX`.
- `PORTAL/RECT` is a static shape, `TRACE/LIGHTNING` is a static strike form, `SCATTER` is seed-driven shimmer/respawn, `APERTURE` is a bounds remapper, and `BAND` phase is azimuthal modulation rather than a general horizon scroll.
- If a brief starts requiring missing directionality, variant families, or animation behavior that the current opcode surface cannot provide, log that as an opcode-surface gap instead of pretending more tuning will solve it.

## Current Target Guardrails

- `Combat Lab`: judge it as a reflected world-space training bay. Projection planes, scan walls, and test-field frames must read as room-integrated light structures rooted to the architecture, not like HUD/UI overlays.
- `Combat Lab`: `GRID`, `VEIL/RAIN_WALL`, `LOBE`, and room-anchored projection emitters are the preferred carriers. `PORTAL/RECT` can frame a bay, but it is static and should not be expected to carry cadence by itself.
- `Frozen Tundra`: the glacier ridge and `PLANE/WATER` ice sheet must carry the place read before weather support layers are added.
- `Frozen Tundra`: primary motion should come from `FLOW`, `PLANE/WATER`, `LOBE`, or another proven mover. `SCATTER` snow is support only, and `BAND` should not be treated as the main horizon or motion carrier.

## A+ Acceptance Rules

- The preset must read as the named scene immediately at the metaphor/place level, not only after explanation.
- The required foreground, midground, and background reads must survive reflection and direct background view.
- The reflective probe can carry backside/environment proof, but it must not become the only place the hero event exists when the brief calls for a direct-view world read.
- It does not need feature-film literal clarity. It does need to be strong enough that a developer would ship it instead of building a skybox.
- Every showcase preset must show obvious change across the reviewed frames. Tiny deltas fail.
- The preset must stay visually distinct within the shipping roster in mood, palette, shape language, and scene structure. Near-duplicates fail.
- Visible seams, clipping, stamped repetition, or obvious rendering errors are hard fails.
- When the authored effect can loop cleanly through `param_d`-driven phase motion, that is preferred over noisy non-repeatable motion.
- Loopable motion is a stated showcase goal but also a known defect area. Do not assume it works; validate it in capture and escalate if the engine path fights the authored loop.
- If a hard-fail condition appears, the preset fails even if the overall mood is attractive.
- If repeated patterning, looping seams, or likely engine artifacts dominate the frame, fail and escalate instead of grading generously.
- If giant flat bands or broad solid-color shelves dominate the frame without being the intended feature read, fail and escalate instead of grading generously.

## Current Shipping 12

### 1. Neon Metropolis

Text prompt: A rain-soaked cyberpunk alley at night, with a black city skyline, one dominant neon sign, sparse lit windows, and wet pavement catching magenta-cyan reflections.

Must read:
- unmistakable city or alley silhouette in the sky band
- one hero neon sign brighter than the rest of the scene
- wet pavement clearly reading as reflective ground
- rain or airborne shimmer supporting the alley mood

Motion contract:
- rain streaks or pavement shimmer should visibly advance across review frames

Hard fail:
- scene reads as abstract magenta fog instead of a city alley
- no dominant sign or no clear wet pavement read
- skyline disappears into noise

### 2. Sakura Shrine

Text prompt: A weathered shrine approach at golden hour, with torii or pagoda-like silhouettes, mossy stone path, warm sunlight, drifting pink petals, and soft sacred haze.

Must read:
- temple-spire or shrine silhouette against sky
- mossy stone floor with warm gold light
- pink petal accent visible without overwhelming the frame
- calm, sacred outdoor atmosphere

Motion contract:
- petals and/or floor shimmer should visibly drift between review frames

Hard fail:
- reads as generic warm fantasy instead of shrine grounds
- petals are invisible or dominate like noise
- floor no longer reads as stone path

### 3. Ocean Depths

Text prompt: A deep-sea trench with bright surface glow above, abyssal black below, basalt seabed, falling marine snow, faint bubbles, and subtle bioluminescent motion.

Must read:
- strong top-to-bottom depth gradient from surface glow to abyss
- dark seabed or trench-floor read
- particles that feel underwater rather than dusty air
- at least one bioluminescent or caustic accent

Motion contract:
- caustic shimmer, bubbles, or drifting particulate must visibly change across frames

Hard fail:
- reads as generic blue fog rather than underwater depth
- no abyssal gradient
- particles feel like snow or dust instead of underwater matter

### 4. Void Station

Text prompt: A cold metallic room with a single rounded viewport looking into deep space, a visible eclipse beyond the glass, subtle wall panel lines, and pale viewport spill on the deck.

Must read:
- interior room bounds, not open outdoors
- one clear viewport opening
- deep-space starfield and eclipse visible through that opening
- cool spill light tying the window to the room

Motion contract:
- panel scan or viewport spill should advance subtly but visibly across review frames

Hard fail:
- interior bounds collapse and the room reads as open space
- the eclipse or viewport opening is not clearly legible
- room becomes muddy enough that wall/floor separation is lost

### 5. Desert Mirage

Text prompt: A vast desert at brutal noon, with dark dune silhouettes, textured sand, heat shimmer at the horizon, a false mirage pool, and airborne dust in a white-hot atmosphere.

Must read:
- strong dune horizon
- warm sand floor with visible texture
- horizon heat or mirage effect
- intense sun-blasted dryness, not sunset romance

Motion contract:
- heat distortion or dust drift should visibly advance across review frames

Hard fail:
- no dune horizon
- mirage read is absent and the floor is just flat tan noise
- scene reads as warm sky fantasy instead of desert exposure

### 6. Enchanted Grove

Text prompt: A magical forest clearing with dark tree canopy, mossy floor, golden shafts of light, warm green atmosphere, and sparse firefly motes.

Must read:
- forest silhouette or canopy framing the sky
- mossy or grassy forest floor
- visible shafts of warm sunlight
- magical firefly accents without turning into ember spam

Motion contract:
- light shafts, dappled floor light, or motes should visibly drift across frames

Hard fail:
- reads as generic green fog instead of a forest clearing
- no clear light-shaft structure
- fireflies overwhelm the space or vanish completely

### 7. Astral Void

Text prompt: An infinite cosmic void with near-black purple depth, dense stars, one large eclipsed celestial body, a smaller secondary moon, and faint nebular drift.

Must read:
- mostly black cosmic space with clear depth
- bright stars against dark negative space
- one large hero celestial body plus a secondary body
- subtle nebula or galactic band accents

Motion contract:
- nebular or galactic drift should move visibly but remain restrained

Hard fail:
- scene becomes too bright or cloudy to read as deep space
- hero celestial body is missing or lost
- stars are too weak to anchor the image

### 8. Hell Core

Text prompt: The heart of hell: shattered black volcanic ground, glowing lava cracks, a lower hellgate rift, sparse embers, and oppressive red-black heat.

Must read:
- cracked volcanic foundation
- bright lava fissures as the primary focal structure
- infernal glow from below
- ominous static menace rather than chaotic flicker

Motion contract:
- lava pulse, ember drift, or infernal heat shimmer must visibly advance across review frames while staying restrained and non-seizure

Hard fail:
- cracks are not the main read
- motion is absent or so weak that the frame reads static
- too much motion or flashing destroys the oppressive stillness
- scene reads as generic red abstract pattern instead of volcanic fracture

### 9. Sky Ruins

Text prompt: Floating marble ruins at the edge of the world, with broken colonnades, blazing clouds, warm sunlight, drifting cloud banks, and a noble sky-adventure mood.

Must read:
- ruined skyline or colonnade silhouette
- warm marble or stone platform floor
- dramatic sunlit clouds with strong sky depth
- airy, elevated outdoor feeling

Motion contract:
- clouds, sun band, or light drift should visibly advance across frames

Hard fail:
- no clear ruins read
- clouds flatten into generic orange wash
- the scene loses the elevated open-air feeling

### 10. Combat Lab

Text prompt: A sterile sci-fi training room with white structural bounds, a bright cyan floor grid, world-anchored projection planes embedded in the room architecture, a rectangular projection bay or test-field frame, and harsh fluorescent light.

Must read:
- clinical interior room, not abstract cyan space
- strong floor grid and wall scan language
- world-integrated projection planes or luminous wall emitters rooted to the architecture
- one clear rectangular projection bay or test-field frame in direct view
- bright high-tech cleanliness

Motion contract:
- grid scan, projection-plane pulse, or scan bars should visibly advance across frames

Authoring guidance:
- prefer `GRID`, `VEIL/RAIN_WALL`, `LOBE`, and room-anchored projection emitters for the readable motion beat
- treat `PORTAL/RECT` as a static bay or frame, not the main mover
- if a structural cutout is needed, remember `APERTURE` is for bounds/composition, not the hero feature read

Hard fail:
- room stops reading as a lab
- cyan effects bloom so hard that structure disappears
- projection elements read like overlay UI instead of room-anchored light structures
- rectangular projection bay disappears or only survives in reflection

### 11. Frozen Tundra

Text prompt: A wind-cut arctic expanse with a hard glacier ridge on the horizon, polished blue ice underfoot, low cold sunlight, drifting snow, and crisp polar air.

Must read:
- mountain or glacier ridge cleanly separating sky from ground
- visible ice-sheet floor, not just a blue bowl around the sphere
- cold blue-white aerial depth and a low polar glow
- snow as support, not as full-frame whiteout

Motion contract:
- drifting snow and/or ice sheen must visibly move across review frames

Authoring guidance:
- primary motion should come from `FLOW`, `PLANE/WATER`, `LOBE`, or another proven mover that reads across the ice sheet or sky band
- `SCATTER/SNOW` is support only; it should not be the main motion beat
- `BAND` should not be treated as the primary horizon builder or motion carrier for this scene

Hard fail:
- ridge and floor separation disappear
- concentric ringing or bowl-like floor artifacts dominate the frame
- snow becomes static speckle or full-scene washout

### 12. Storm Front

Text prompt: A violent squall line over black water, with a readable sea horizon, dark storm shelf, driven rain, distant lightning, and bruised blue-gray storm light.

Must read:
- clear sea horizon
- dark stormwater floor separate from the sky
- visible rain or squall structure
- lightning as a scene event, not only a sphere reflection

Motion contract:
- rain, water streaks, or squall structure must visibly change across review frames; lightning must survive as a world-space event even if it is not the primary mover

Hard fail:
- horizon is buried
- looping rings or repeated contour patterning dominate walls or floor
- lightning reads only in reflection and not in the world

## Planned Expansion 8

### 13. Crystal Cavern

Text prompt: A radiant ice-crystal cave with faceted walls, luminous mineral veins, cold reflected light, and a clean sense of chamber depth.

Must read:
- cave enclosure
- crystal or faceted structural language
- luminous vein or filament accents
- cold reflective depth, not muddy darkness

Motion contract:
- light crawl or crystal shimmer should visibly advance across frames

Hard fail:
- cave reads as generic blue room
- no faceted crystalline structure

### 14. Moonlit Graveyard

Text prompt: A moonlit graveyard with crooked stones, tunnel-like gothic depth, low mist, a spectral rift, and a cold silver-blue night sky.

Must read:
- cemetery silhouette or grave-marker language
- cold nocturnal moonlight
- tunnel or recession depth
- one eerie supernatural rupture or portal event

Motion contract:
- mist or spectral disturbance should visibly drift across frames

Hard fail:
- reads as generic dark fantasy outdoors
- no graveyard read
- portal overwhelms the scene and erases the cemetery

### 15. Alien Jungle

Text prompt: A humid extraterrestrial jungle with massive alien foliage, bioluminescent spores, toxic atmosphere, and uncanny organic color separation.

Must read:
- dense alien plant silhouette
- humid atmosphere
- bioluminescent accent colors unlike the forest preset
- overtly non-Earth mood

Motion contract:
- spores, fog, or canopy shimmer should visibly move across frames

Hard fail:
- reads as Enchanted Grove recolor
- atmosphere lacks alien strangeness

### 16. Gothic Cathedral

Text prompt: A towering cathedral interior with arches, stained light, incense haze, stone floor depth, and solemn sacred grandeur.

Must read:
- tall sacred interior architecture
- stone nave or aisle floor
- colored stained-light read
- solemn indoor atmosphere

Motion contract:
- dust, incense haze, or stained-light drift should visibly advance across frames

Hard fail:
- interior scale is unclear
- scene reads as generic castle hall

### 17. Toxic Wasteland

Text prompt: A corrupted industrial wasteland with ruined machinery, chemical slicks, sickly haze, hazard glows, and poisoned ground.

Must read:
- industrial silhouette or machinery language
- contaminated ground plane
- toxic color palette distinct from Hell Core and Alien Jungle
- hazardous atmospheric corruption

Motion contract:
- fumes, slick shimmer, or fallout drift should visibly change across frames

Hard fail:
- reads as simple green fog
- industrial identity is absent

### 18. Neon Arcade

Text prompt: A retro-futurist arcade space with glowing cabinets, synthwave color splits, glossy floor reflections, CRT scan flavor, and playful kinetic energy.

Must read:
- interior entertainment space
- arcade or cabinet-like visual rhythm
- glossy reflective floor
- bright nostalgic neon palette distinct from Neon Metropolis

Motion contract:
- scanlines, cabinet glow pulse, or floor reflections should visibly animate

Hard fail:
- reads as second cyberpunk alley
- no arcade interior read

### 19. War Zone

Text prompt: A battle-torn night battlefield with smoke, tracer fire, ruined structures, searchlight or flare illumination, and urgent militarized chaos.

Must read:
- battlefield destruction
- smoke and debris depth
- directional combat lighting or tracer language
- grounded military identity

Motion contract:
- smoke drift, tracer cadence, or flare change should visibly advance across frames

Hard fail:
- reads as generic orange destruction without battlefield structure
- chaos overwhelms all readable forms

### 20. Digital Matrix

Text prompt: An abstract cyberspace chamber of luminous grids, code-like partitions, prism splits, and impossible clean digital depth.

Must read:
- unmistakably synthetic digital space
- crisp partitioning or split-space structure
- disciplined geometric depth
- high-contrast luminous data aesthetic

Motion contract:
- scan, code drift, or partition pulse should visibly advance across frames

Hard fail:
- reads as generic green abstract glow
- structure is too muddy to feel digital

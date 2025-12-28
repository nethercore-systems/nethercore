# Prism Survivors - Game Design Document

## Overview

**Title:** Prism Survivors
**Genre:** Roguelite Wave Survival
**Platform:** Nethercore ZX
**Players:** 1-4 (local/online co-op with rollback netcode)
**Render Mode:** 3 (Specular-Shininess Blinn-Phong)

### Concept
A fast-paced cooperative survival roguelite where players choose from 6 unique hero classes and fight endless waves of crystalline enemies across 4 mystical realms. Collect XP, level up, choose powerful synergizing upgrades, and survive as long as possible.

---

## Hero Classes (6)

| Class | HP | Speed | Damage | Armor | Special | Starting Weapon |
|-------|-----|-------|--------|-------|---------|-----------------|
| **Knight** | 150 | 3.5 | 1.0x | 10% | Tank frontline | Cleave (melee sweep) |
| **Mage** | 80 | 4.0 | 1.3x | 0% | High burst damage | Magic Missile (homing) |
| **Ranger** | 100 | 5.0 | 1.0x | 0% | +30% attack speed | Piercing Arrow |
| **Cleric** | 120 | 3.8 | 1.0x | 0% | +67% pickup range | Holy Nova (radial) |
| **Necromancer** | 90 | 3.6 | 1.2x | 0% | Life steal specialist | Soul Drain (beam) |
| **Paladin** | 180 | 3.0 | 1.0x | 20% | Ultimate tank | Divine Crush (hammer) |

### Visual Design Requirements

Each hero needs:
- **Mesh:** 800-1200 triangles, distinct silhouette
- **Texture:** 256x256 PNG with class-appropriate colors
- **Color Scheme:**
  - Knight: Steel blue/silver armor
  - Mage: Purple robes, arcane glow
  - Ranger: Green/brown, hooded cloak
  - Cleric: White/gold holy vestments
  - Necromancer: Dark purple/green, tattered robes
  - Paladin: Ornate gold armor

---

## Enemy Types (13)

### Basic Enemies (7)

| Enemy | Scale | HP | XP | Speed | Behavior |
|-------|-------|-----|-----|-------|----------|
| Crawler | 0.6 | 15 | 1 | Fast | Spider-like, swarms |
| Skeleton | 0.8 | 25 | 2 | Medium | Undead warrior |
| Wisp | 0.5 | 10 | 2 | Erratic | Floating spirit |
| Golem | 1.2 | 60 | 5 | Slow | Heavy rock creature |
| Shade | 0.7 | 20 | 2 | Fast | Shadow specter |
| Berserker | 1.0 | 40 | 4 | Aggressive | Crazed warrior |
| Arcane Sentinel | 0.9 | 35 | 3 | Medium | Magic construct |

### Elite Enemies (4)

| Enemy | Scale | HP | XP | Visual |
|-------|-------|-----|-----|--------|
| Crystal Knight | 1.3 | 150 | 15 | Purple crystalline armor |
| Void Mage | 1.1 | 100 | 12 | Dark robes, floating runes |
| Golem Titan | 1.6 | 250 | 20 | Massive stone, multiple eyes |
| Specter Lord | 1.2 | 120 | 15 | Crowned ghost with chains |

### Boss Enemies (2)

| Boss | Scale | HP | XP | Description |
|------|-------|-----|-----|-------------|
| Prism Colossus | 2.5 | 1000 | 100 | Giant crystal humanoid |
| Void Dragon | 2.2 | 1500 | 100 | Ethereal dragon, void flames |

### Visual Hierarchy
- **Basic:** Muted colors, 200-600 triangles
- **Elite:** Purple glow accents, 600-1000 triangles
- **Boss:** Red glow, imposing scale, 1000-2000 triangles

---

## Power-Up System (18)

### Weapons (10)

| Weapon | Type | Cooldown | Description |
|--------|------|----------|-------------|
| Cleave | Melee | 0.8s | Wide sweep attack |
| Magic Missile | Ranged | 0.6s | Homing projectiles |
| Piercing Arrow | Ranged | 0.4s | Passes through enemies |
| Holy Nova | AoE | 1.5s | Radial burst heal/damage |
| Soul Drain | Beam | 1.0s | Life steal channeled beam |
| Divine Crush | Melee | 1.2s | Hammer ground slam |
| Fireball | AoE | 2.0s | Explosive projectile |
| Ice Shards | Ranged | 0.7s | Multi-projectile spread |
| Lightning Bolt | Chain | 1.2s | Chains between enemies |
| Shadow Orb | Orbit | 3.0s | Orbiting damage shields |

### Passives (8)

| Passive | Effect |
|---------|--------|
| Might | +20% damage |
| Swiftness | +15% move speed |
| Vitality | +25 max HP, instant heal |
| Haste | +20% attack speed |
| Magnetism | +50% pickup range |
| Armor | -15% damage taken |
| Luck | +25% XP gain |
| Fury | +5% damage per 10% missing HP |

### Synergies (10)

| Combo | Effect |
|-------|--------|
| Fireball + Ice Shards | Steam Explosion: +5 shards, AoE slow |
| Magic Missile + Lightning | Chain Lightning: +50% dmg, chains to 2nd enemy |
| Cleave + Shadow Orb | Whirlwind: +75% range, 3 orbs |
| Holy Nova + Fireball | Holy Fire: Nova ignites (DoT) |
| Ice Shards + Piercing Arrow | Frost Arrow: Slow + extra pierce |
| Soul Drain + Shadow Orb | Void Drain: Orbs heal on hit |
| Divine Crush + Lightning | Divine Bolt: Stun + chain smash |
| Cleave + Fury | Berserk Fury: Cleave scales with missing HP |
| Soul Drain + Vitality | Vampire Touch: Double lifesteal |
| Swiftness + Haste | Speed Demon: +30% proj speed, -20% cooldown |

---

## Stage Environments (4)

### Stage 1: Crystal Cavern
- **Theme:** Underground crystalline caves
- **Palette:** Blues, purples, cyan
- **Ground:** Faceted crystal floor with reflective shards
- **Hazard:** Falling crystal shards (periodic damage zones)
- **Duration:** Waves 1-5

### Stage 2: Enchanted Forest
- **Theme:** Mystical forest with glowing flora
- **Palette:** Greens, teals, gold accents
- **Ground:** Moss-covered stone with glowing mushrooms
- **Hazard:** Poison clouds (DoT zones)
- **Duration:** Waves 6-10

### Stage 3: Volcanic Depths
- **Theme:** Volcanic caverns with lava
- **Palette:** Reds, oranges, yellows
- **Ground:** Cracked lava rock with glowing fissures
- **Hazard:** Lava geysers (burst damage)
- **Duration:** Waves 11-15

### Stage 4: Void Realm
- **Theme:** Abstract void space
- **Palette:** Deep purples, blacks, cosmic blues
- **Ground:** Swirling void energy patterns
- **Hazard:** Void rifts (moving damage zones)
- **Duration:** Waves 16-20 (Final)

---

## Difficulty Modes

| Mode | Enemy HP | Enemy Damage | Spawn Rate | XP Bonus | Elite Wave |
|------|----------|--------------|------------|----------|------------|
| Easy | 0.6x | 0.7x | 0.8x | - | Wave 7+ |
| Normal | 1.0x | 1.0x | 1.0x | - | Wave 5+ |
| Hard | 1.4x | 1.3x | 1.2x | +10% | Wave 4+ |
| Nightmare | 2.0x | 1.6x | 1.5x | +25% | Wave 3+ |

---

## Combo System

Kill enemies in rapid succession to build combo multiplier:

| Combo | Multiplier | Color |
|-------|------------|-------|
| 5+ | 1.2x | Green |
| 10+ | 1.5x | Orange |
| 25+ | 2.0x | Gold |
| 50+ | 2.5x | Purple |

- Combo window: 2 seconds between kills
- Each kill extends window by 0.5 seconds
- Taking damage breaks combo

---

## Co-op Features

### Revive System
- Downed players can be revived by allies
- Revive time: 3 seconds proximity hold
- Revive health: 50% of max HP
- Revive invulnerability: 1 second

### Player Scaling
- Enemy HP scales with player count
- XP gems shared (first to collect)
- Wave difficulty balanced for party size

---

## UI Requirements

### Title Screen
- **Current:** Black background (needs improvement)
- **Required:** Animated prism/crystal background
- Display rotating hero carousel
- Difficulty selection with visual indicators

### Class Selection Screen (NEW)
- 2x3 grid of hero panels
- Per-player selection (1-4 panels)
- Stats display: HP bar, Speed bar, Damage, Armor, Special
- Starting weapon name and icon
- Class description (1-2 lines)
- "READY!" indicator when confirmed
- 3-second countdown when all ready

### In-Game HUD
- Health bars per player (top-left)
- XP bar with level number
- Combo counter (animated on milestone)
- Wave number and timer
- Mini power-up icons (bottom)

---

## Technical Specifications

### Rollback State Size
- Players: ~8KB (4 players x 2KB each)
- Enemies: ~8KB (100 max x 80 bytes)
- Projectiles: ~6KB (80 max x 80 bytes)
- XP Gems: ~1.6KB (200 max x 8 bytes)
- **Total:** ~24KB (well under 100KB limit)

### Asset Budget
- Hero models: 6 x 1000 tris = 6K tris
- Enemy models: 100 x 500 avg = 50K tris
- Arena: 1K tris
- **Total on-screen:** ~60K triangles

### ROM Size Estimate
- Meshes: ~200KB
- Textures: ~3MB
- Audio: ~2MB
- Code: ~1MB
- **Total:** ~6MB

---

## Development Priorities

### Phase 1: Visual Overhaul (Current)
1. Replace placeholder character textures
2. Replace placeholder enemy textures
3. Add arena floor textures per stage
4. Add title screen background

### Phase 2: Character Selection
1. Implement ClassSelect game phase ✓
2. Add selection UI rendering ✓
3. Wire up class-based stats ✓

### Phase 3: Polish
1. Add particle effects for abilities
2. Add damage numbers
3. Add combo celebration effects
4. Add boss intro animations

---

*Document Version: 1.0*
*Last Updated: 2025-01-28*

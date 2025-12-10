# Network Condition Simulator Specification

## Overview

The Network Condition Simulator allows developers to test their game's netcode under various adverse network conditions without needing actual bad connections. This is critical for testing GGRS rollback behavior, visual rollback artifacts, and ensuring games remain playable on poor connections.

This enables:
- **Rollback testing**: See how rollback looks with different ping/jitter
- **Edge case testing**: Simulate packet loss, connection drops
- **Performance tuning**: Find the breaking point of your netcode
- **Accessibility testing**: Ensure game works in regions with poor internet

## Architecture

### Network Condition Model

```rust
/// Simulated network conditions
pub struct NetworkConditions {
    /// Base latency in milliseconds (one-way)
    pub latency_ms: u32,

    /// Latency variation (jitter) in milliseconds
    pub jitter_ms: u32,

    /// Packet loss percentage (0-100)
    pub packet_loss_percent: f32,

    /// Packet duplication percentage (0-100)
    pub packet_duplicate_percent: f32,

    /// Packet reordering percentage (0-100)
    pub packet_reorder_percent: f32,

    /// Bandwidth limit in bytes per second (0 = unlimited)
    pub bandwidth_limit_bps: u32,

    /// Connection spike frequency (spikes per minute, 0 = none)
    pub spike_frequency: f32,

    /// Connection spike duration in milliseconds
    pub spike_duration_ms: u32,

    /// Connection spike additional latency
    pub spike_latency_ms: u32,
}

impl NetworkConditions {
    /// No simulation - pass-through
    pub fn none() -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            packet_loss_percent: 0.0,
            packet_duplicate_percent: 0.0,
            packet_reorder_percent: 0.0,
            bandwidth_limit_bps: 0,
            spike_frequency: 0.0,
            spike_duration_ms: 0,
            spike_latency_ms: 0,
        }
    }

    /// Good connection (low ping, stable)
    pub fn good() -> Self {
        Self {
            latency_ms: 20,
            jitter_ms: 5,
            packet_loss_percent: 0.1,
            ..Self::none()
        }
    }

    /// Average connection
    pub fn average() -> Self {
        Self {
            latency_ms: 50,
            jitter_ms: 15,
            packet_loss_percent: 1.0,
            ..Self::none()
        }
    }

    /// Poor connection (high ping, unstable)
    pub fn poor() -> Self {
        Self {
            latency_ms: 100,
            jitter_ms: 30,
            packet_loss_percent: 3.0,
            packet_reorder_percent: 2.0,
            ..Self::none()
        }
    }

    /// Terrible connection (very high ping, lots of issues)
    pub fn terrible() -> Self {
        Self {
            latency_ms: 200,
            jitter_ms: 50,
            packet_loss_percent: 5.0,
            packet_duplicate_percent: 2.0,
            packet_reorder_percent: 5.0,
            spike_frequency: 2.0,
            spike_duration_ms: 500,
            spike_latency_ms: 300,
        }
    }

    /// Mobile network (variable, spiky)
    pub fn mobile_4g() -> Self {
        Self {
            latency_ms: 60,
            jitter_ms: 40,
            packet_loss_percent: 2.0,
            spike_frequency: 5.0,
            spike_duration_ms: 200,
            spike_latency_ms: 150,
            ..Self::none()
        }
    }

    /// Satellite connection (high latency, stable)
    pub fn satellite() -> Self {
        Self {
            latency_ms: 600,
            jitter_ms: 20,
            packet_loss_percent: 0.5,
            ..Self::none()
        }
    }

    /// WiFi with interference
    pub fn wifi_congested() -> Self {
        Self {
            latency_ms: 30,
            jitter_ms: 25,
            packet_loss_percent: 3.0,
            packet_reorder_percent: 5.0,
            spike_frequency: 10.0,
            spike_duration_ms: 100,
            spike_latency_ms: 100,
            ..Self::none()
        }
    }
}
```

### Simulation Layer

The simulator sits between GGRS and the actual network transport:

```
┌─────────────────────────────────────────────────────────────┐
│                      GGRS Session                           │
├─────────────────────────────────────────────────────────────┤
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            Network Condition Simulator              │   │
│  │                                                     │   │
│  │  Outgoing packets:                                 │   │
│  │  1. Apply latency (delay queue)                    │   │
│  │  2. Apply jitter (randomize delay)                 │   │
│  │  3. Apply packet loss (drop randomly)             │   │
│  │  4. Apply duplication (send twice)                │   │
│  │  5. Apply reordering (shuffle queue)              │   │
│  │  6. Apply bandwidth limit (throttle)              │   │
│  │                                                     │   │
│  │  Incoming packets: same process                    │   │
│  │                                                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
├─────────────────────────────────────────────────────────────┤
│                 matchbox_socket (WebRTC)                   │
└─────────────────────────────────────────────────────────────┘
```

### Packet Delay Queue

```rust
/// Packet waiting to be delivered
struct DelayedPacket {
    /// Packet data
    data: Vec<u8>,

    /// When to deliver (instant + latency + jitter)
    deliver_at: Instant,

    /// Original sender
    from: SocketAddr,
}

/// Manages packet delays
struct PacketDelayQueue {
    /// Pending packets sorted by delivery time
    queue: BinaryHeap<DelayedPacket>,

    /// Random generator for jitter
    rng: SmallRng,
}

impl PacketDelayQueue {
    fn enqueue(&mut self, packet: Vec<u8>, from: SocketAddr, conditions: &NetworkConditions) {
        let base_delay = Duration::from_millis(conditions.latency_ms as u64);
        let jitter = Duration::from_millis(
            self.rng.gen_range(0..=conditions.jitter_ms) as u64
        );

        // Check for spike
        let spike_delay = if self.is_in_spike(conditions) {
            Duration::from_millis(conditions.spike_latency_ms as u64)
        } else {
            Duration::ZERO
        };

        let deliver_at = Instant::now() + base_delay + jitter + spike_delay;

        self.queue.push(DelayedPacket { data: packet, deliver_at, from });
    }

    fn poll(&mut self) -> Option<(Vec<u8>, SocketAddr)> {
        if let Some(pkt) = self.queue.peek() {
            if pkt.deliver_at <= Instant::now() {
                let pkt = self.queue.pop().unwrap();
                return Some((pkt.data, pkt.from));
            }
        }
        None
    }
}
```

## UI Design

### Network Simulator Panel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Network Simulator                                                      [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Preset: [Custom      ▼]  [None] [Good] [Average] [Poor] [Terrible]         │
├─────────────────────────────────────────────────────────────────────────────┤
│ Latency                                                                     │
│ Base:     [━━━━━━━●━━━━━━] 50 ms     Jitter: [━━━●━━━━━━━━━] 15 ms        │
├─────────────────────────────────────────────────────────────────────────────┤
│ Packet Loss                                                                 │
│ Loss:     [━●━━━━━━━━━━━━] 1.0%      Duplicate: [●━━━━━━━━━━] 0.0%         │
│ Reorder:  [●━━━━━━━━━━━━━] 0.0%                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Bandwidth                                                                   │
│ Limit:    [━━━━━━━━━━━━━●] Unlimited                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│ Spikes                                                                      │
│ Frequency: [●━━━━━━━━━━━━] 0/min     Duration: [━━━━●━━━━━━━] 200 ms       │
│ Extra latency: [━━━━━━●━━] 150 ms                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│ Direction: [◉ Both] [○ Send only] [○ Receive only]                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ Statistics                                                                  │
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ Packets sent:     1,234   │ Packets received:    1,198               │  │
│ │ Packets dropped:     12   │ Packets duplicated:      5               │  │
│ │ Avg latency:       52ms   │ Max latency:          87ms               │  │
│ │ Current rollback:   2f    │ Max rollback:          8f                │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ [▶ Enable Simulation]  [Reset Stats]  [Save Preset]                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Live Graph

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Network Graph                                                          [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Latency (ms)                                                                │
│ 200├─────────────────────────────────────────────────────────────────────  │
│    │                    ╱╲                                                  │
│ 100├──────────────────╱──╲───────────────────────────────────────────────  │
│    │    ╱╲  ╱╲  ╱╲  ╱╲    ╲╱╲  ╱╲  ╱╲                                      │
│  50├──╱──╲╱──╲╱──╲╱────────────╲╱──╲╱──╲────────────────────────────────  │
│   0└─────────────────────────────────────────────────────────────────────  │
│    -10s                                                              now    │
├─────────────────────────────────────────────────────────────────────────────┤
│ Rollback Frames                                                             │
│  10├─────────────────────────────────────────────────────────────────────  │
│    │                    █                                                   │
│   5├────────█───────────█─────────────────────────────────────────────────  │
│    │  █  █  █  █  █  █  █  █  █                                            │
│   0└─────────────────────────────────────────────────────────────────────  │
│    -10s                                                              now    │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Hotkeys

| Key | Action |
|-----|--------|
| Ctrl+N | Toggle network simulator panel |
| Ctrl+1 | Set "None" preset (disable simulation) |
| Ctrl+2 | Set "Good" preset |
| Ctrl+3 | Set "Average" preset |
| Ctrl+4 | Set "Poor" preset |
| Ctrl+5 | Set "Terrible" preset |
| Ctrl+0 | Toggle simulation on/off |

## Presets System

### Built-in Presets

| Preset | Latency | Jitter | Loss | Description |
|--------|---------|--------|------|-------------|
| None | 0ms | 0ms | 0% | No simulation |
| Good | 20ms | 5ms | 0.1% | Wired connection, same region |
| Average | 50ms | 15ms | 1% | Typical broadband |
| Poor | 100ms | 30ms | 3% | Cross-continent or poor ISP |
| Terrible | 200ms | 50ms | 5% | Very bad connection |
| Mobile 4G | 60ms | 40ms | 2% | Mobile with spikes |
| Satellite | 600ms | 20ms | 0.5% | High latency, stable |
| WiFi Congested | 30ms | 25ms | 3% | Interference issues |

### Custom Presets

Users can save custom presets:

```toml
# ~/.emberware/network_presets/australia_to_usa.toml
[preset]
name = "Australia to USA"
description = "Simulates AU->US connection"

[conditions]
latency_ms = 180
jitter_ms = 25
packet_loss_percent = 1.5
packet_duplicate_percent = 0.5
packet_reorder_percent = 1.0
bandwidth_limit_bps = 0
spike_frequency = 1.0
spike_duration_ms = 300
spike_latency_ms = 100
```

## Local Testing Mode

### Same-Machine Multiplayer

For testing netcode without a second computer:
1. Launch two instances of the emulator
2. Connect via localhost
3. Apply network simulation to one or both instances
4. See rollback behavior as if on separate machines

```
┌──────────────────┐    simulated network    ┌──────────────────┐
│   Instance 1     │ ◄─────────────────────► │   Instance 2     │
│   (Player 1)     │    latency + jitter     │   (Player 2)     │
│   localhost:5000 │    packet loss          │   localhost:5001 │
└──────────────────┘                         └──────────────────┘
```

### Asymmetric Simulation

Different conditions for each direction:
- Player 1 has good upload, bad download
- Player 2 has bad upload, good download

```rust
pub struct AsymmetricConditions {
    pub send: NetworkConditions,
    pub receive: NetworkConditions,
}
```

## Integration with GGRS

### Rollback Metrics

The simulator integrates with GGRS to show:
- Current rollback depth (frames)
- Maximum rollback this session
- Rollback frequency graph
- Input delay recommendation

### Auto-Tuning Mode

Experimental: Automatically adjust input delay based on observed conditions:

```rust
pub struct AutoTuneConfig {
    /// Enable auto-tuning
    pub enabled: bool,

    /// Target rollback frames (try to stay below this)
    pub target_rollback_frames: u32,

    /// Minimum input delay
    pub min_input_delay: u32,

    /// Maximum input delay
    pub max_input_delay: u32,

    /// How quickly to adjust (0.0 - 1.0)
    pub adjustment_rate: f32,
}
```

## FFI API

### Query Network Status (Game → Host)

```rust
// Get current simulated round-trip time
extern "C" fn net_sim_get_rtt() -> u32;

// Get current packet loss rate (0-100)
extern "C" fn net_sim_get_loss_percent() -> f32;

// Check if simulation is enabled
extern "C" fn net_sim_is_enabled() -> i32;

// Get current rollback frame count
extern "C" fn net_get_rollback_frames() -> u32;
```

### Control (Game → Host, Optional)

```rust
// Request minimum input delay (GGRS setting)
extern "C" fn net_request_input_delay(frames: u32);

// Get recommended input delay based on conditions
extern "C" fn net_get_recommended_delay() -> u32;
```

## Testing Scenarios

### Scenario 1: High Latency
Test how game feels with 200ms+ ping:
- Is input delay acceptable?
- Do animations still look smooth?
- Is gameplay still fun?

### Scenario 2: Packet Loss
Test rollback visual artifacts:
- How often does rollback cause visible "teleporting"?
- Are sound effects handled correctly on rollback?
- Does UI stay consistent?

### Scenario 3: Spikes
Test recovery from connection hiccups:
- Does game recover gracefully?
- How bad is the visual pop after a spike?
- Does game desync?

### Scenario 4: Asymmetric
Test when one player has worse connection:
- Is advantage fair?
- Does the good-connection player suffer?

## Pending Questions

### Q1: Where does simulation run?
- A) On both instances (each simulates its own)
- B) On network layer (affects both directions together)
- C) Configurable per-instance

**Recommendation**: Option A - each instance controls its own simulation.

### Q2: Persist simulation settings?
- A) Reset on restart
- B) Remember last settings
- C) Per-game settings

**Recommendation**: Option B.

### Q3: Affect local testing?
Should simulation work in local play (no network)?
- A) No - only for netplay
- B) Yes - simulate even for local splitscreen

**Recommendation**: Option A - local play should be responsive.

### Q4: Bandwidth simulation accuracy?
How accurate should bandwidth limiting be?
- A) Simple byte count throttle
- B) Full TCP/UDP congestion simulation
- C) Just show warning, don't actually limit

**Recommendation**: Option A - good enough for testing.

### Q5: Integration with replays?
Should network conditions be recorded in replays?
- A) No - replays are deterministic inputs
- B) Yes - record actual latencies experienced
- C) Metadata only (for reference)

**Recommendation**: Option C.

## Pros

1. **Test without real network**: No need for two machines or bad ISP
2. **Reproducible**: Same conditions every time
3. **Presets**: Quick switching between scenarios
4. **Metrics**: See rollback stats in real-time
5. **Local testing**: Test netcode on single machine
6. **Edge cases**: Test conditions that are hard to get naturally

## Cons

1. **Not perfect simulation**: Real networks are more complex
2. **Added complexity**: More code in network path
3. **Performance**: Packet queuing adds some overhead
4. **False confidence**: Game might work in simulator but fail in reality
5. **Only for GGRS**: Specific to rollback netcode

## Implementation Complexity

**Estimated effort:** Medium

**Key components:**
1. NetworkConditions data structures - 0.5 days
2. Packet delay queue - 1 day
3. Integration with matchbox_socket - 2 days
4. Packet loss/duplication/reorder - 1 day
5. Bandwidth limiting - 1 day
6. Spike simulation - 0.5 days
7. UI panel - 2 days
8. Presets system - 0.5 days
9. Statistics tracking - 0.5 days
10. Testing - 1 day

**Total:** ~10 days

## Console-Agnostic Design

Network simulation lives in core alongside GGRS:
- `NetworkSimulator` wraps the transport layer
- Configuration stored in `GameSession`
- UI via egui in debug panel
- Works identically for all consoles

## Way Forward: Implementation Guide

This section provides concrete implementation steps based on the current Emberware codebase architecture.

### Step 1: Add Network Simulation Types

**File: `core/src/net/simulator.rs` (new file)**

```rust
//! Network condition simulation for testing rollback netcode

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

/// Simulated network conditions
#[derive(Clone)]
pub struct NetworkConditions {
    pub latency_ms: u32,
    pub jitter_ms: u32,
    pub packet_loss_percent: f32,
    pub packet_duplicate_percent: f32,
    pub packet_reorder_percent: f32,
    pub spike_frequency: f32,
    pub spike_duration_ms: u32,
    pub spike_latency_ms: u32,
}

impl NetworkConditions {
    pub fn none() -> Self {
        Self {
            latency_ms: 0, jitter_ms: 0, packet_loss_percent: 0.0,
            packet_duplicate_percent: 0.0, packet_reorder_percent: 0.0,
            spike_frequency: 0.0, spike_duration_ms: 0, spike_latency_ms: 0,
        }
    }

    pub fn good() -> Self { Self { latency_ms: 20, jitter_ms: 5, packet_loss_percent: 0.1, ..Self::none() } }
    pub fn average() -> Self { Self { latency_ms: 50, jitter_ms: 15, packet_loss_percent: 1.0, ..Self::none() } }
    pub fn poor() -> Self { Self { latency_ms: 100, jitter_ms: 30, packet_loss_percent: 3.0, ..Self::none() } }
    pub fn terrible() -> Self { Self { latency_ms: 200, jitter_ms: 50, packet_loss_percent: 5.0, ..Self::none() } }
}

/// Packet waiting for delivery
struct DelayedPacket {
    data: Vec<u8>,
    deliver_at: Instant,
}

/// Network simulator wrapping a transport
pub struct NetworkSimulator {
    conditions: NetworkConditions,
    enabled: bool,
    outbound_queue: VecDeque<DelayedPacket>,
    inbound_queue: VecDeque<DelayedPacket>,
    rng: SmallRng,
    last_spike_time: Instant,
    in_spike: bool,
    stats: NetworkStats,
}

/// Network statistics
#[derive(Default, Clone)]
pub struct NetworkStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_dropped: u64,
    pub packets_duplicated: u64,
    pub avg_latency_ms: f32,
    pub max_latency_ms: u32,
    pub current_rollback_frames: u32,
    pub max_rollback_frames: u32,
}

impl NetworkSimulator {
    pub fn new() -> Self {
        Self {
            conditions: NetworkConditions::none(),
            enabled: false,
            outbound_queue: VecDeque::new(),
            inbound_queue: VecDeque::new(),
            rng: SmallRng::from_entropy(),
            last_spike_time: Instant::now(),
            in_spike: false,
            stats: NetworkStats::default(),
        }
    }

    pub fn set_conditions(&mut self, conditions: NetworkConditions) {
        self.conditions = conditions;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Deliver all queued packets immediately
            self.outbound_queue.clear();
            self.inbound_queue.clear();
        }
    }

    /// Queue a packet for simulated delivery
    pub fn send(&mut self, data: Vec<u8>) -> Option<Vec<u8>> {
        if !self.enabled {
            return Some(data);
        }

        self.stats.packets_sent += 1;

        // Packet loss check
        if self.rng.gen_range(0.0..100.0) < self.conditions.packet_loss_percent {
            self.stats.packets_dropped += 1;
            return None;
        }

        // Calculate delay
        let mut delay_ms = self.conditions.latency_ms;
        delay_ms += self.rng.gen_range(0..=self.conditions.jitter_ms);

        // Spike check
        if self.check_spike() {
            delay_ms += self.conditions.spike_latency_ms;
        }

        let deliver_at = Instant::now() + Duration::from_millis(delay_ms as u64);

        // Duplicate check
        if self.rng.gen_range(0.0..100.0) < self.conditions.packet_duplicate_percent {
            self.stats.packets_duplicated += 1;
            self.outbound_queue.push_back(DelayedPacket {
                data: data.clone(),
                deliver_at: deliver_at + Duration::from_millis(self.rng.gen_range(0..50)),
            });
        }

        self.outbound_queue.push_back(DelayedPacket { data, deliver_at });

        None // Packet queued, not delivered immediately
    }

    /// Get packets ready for delivery
    pub fn receive(&mut self) -> Vec<Vec<u8>> {
        let now = Instant::now();
        let mut ready = Vec::new();

        while let Some(pkt) = self.outbound_queue.front() {
            if pkt.deliver_at <= now {
                ready.push(self.outbound_queue.pop_front().unwrap().data);
                self.stats.packets_received += 1;
            } else {
                break;
            }
        }

        ready
    }

    fn check_spike(&mut self) -> bool {
        if self.conditions.spike_frequency <= 0.0 {
            return false;
        }

        let spike_interval = Duration::from_secs_f32(60.0 / self.conditions.spike_frequency);
        let spike_duration = Duration::from_millis(self.conditions.spike_duration_ms as u64);

        if self.in_spike {
            if self.last_spike_time.elapsed() > spike_duration {
                self.in_spike = false;
            }
            return self.in_spike;
        }

        if self.last_spike_time.elapsed() > spike_interval {
            self.in_spike = true;
            self.last_spike_time = Instant::now();
            return true;
        }

        false
    }

    pub fn stats(&self) -> &NetworkStats {
        &self.stats
    }

    pub fn conditions(&self) -> &NetworkConditions {
        &self.conditions
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
```

### Step 2: Integrate with Rollback Session

**File: `core/src/rollback/session.rs`**

Add network simulator to the rollback session:

```rust
use crate::net::simulator::{NetworkSimulator, NetworkConditions};

pub struct RollbackSession<C: Console> {
    // ... existing fields ...
    pub network_simulator: NetworkSimulator,
}

impl<C: Console> RollbackSession<C> {
    /// Wrap outgoing packet through simulator
    pub fn send_packet(&mut self, data: Vec<u8>) -> Option<Vec<u8>> {
        self.network_simulator.send(data)
    }

    /// Get packets ready from simulator
    pub fn poll_packets(&mut self) -> Vec<Vec<u8>> {
        self.network_simulator.receive()
    }

    /// Update rollback stats in simulator
    pub fn update_rollback_stats(&mut self, rollback_frames: u32) {
        let stats = &mut self.network_simulator.stats;
        stats.current_rollback_frames = rollback_frames;
        stats.max_rollback_frames = stats.max_rollback_frames.max(rollback_frames);
    }
}
```

### Step 3: Add UI Panel

**File: `core/src/app/network_ui.rs` (new file)**

```rust
use crate::net::simulator::{NetworkSimulator, NetworkConditions, NetworkStats};
use egui::{Ui, Slider};

pub struct NetworkSimulatorUI {
    visible: bool,
    custom_conditions: NetworkConditions,
}

impl NetworkSimulatorUI {
    pub fn new() -> Self {
        Self {
            visible: false,
            custom_conditions: NetworkConditions::average(),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn show(&mut self, ctx: &egui::Context, simulator: &mut NetworkSimulator) {
        if !self.visible {
            return;
        }

        egui::Window::new("Network Simulator")
            .default_size([350.0, 400.0])
            .show(ctx, |ui| {
                // Enable toggle
                let mut enabled = simulator.is_enabled();
                if ui.checkbox(&mut enabled, "Enable Simulation").changed() {
                    simulator.set_enabled(enabled);
                }

                ui.separator();

                // Presets
                ui.horizontal(|ui| {
                    ui.label("Preset:");
                    if ui.button("None").clicked() {
                        self.custom_conditions = NetworkConditions::none();
                        simulator.set_conditions(self.custom_conditions.clone());
                    }
                    if ui.button("Good").clicked() {
                        self.custom_conditions = NetworkConditions::good();
                        simulator.set_conditions(self.custom_conditions.clone());
                    }
                    if ui.button("Average").clicked() {
                        self.custom_conditions = NetworkConditions::average();
                        simulator.set_conditions(self.custom_conditions.clone());
                    }
                    if ui.button("Poor").clicked() {
                        self.custom_conditions = NetworkConditions::poor();
                        simulator.set_conditions(self.custom_conditions.clone());
                    }
                    if ui.button("Terrible").clicked() {
                        self.custom_conditions = NetworkConditions::terrible();
                        simulator.set_conditions(self.custom_conditions.clone());
                    }
                });

                ui.separator();

                // Custom sliders
                ui.heading("Latency");
                let mut changed = false;
                changed |= ui.add(Slider::new(&mut self.custom_conditions.latency_ms, 0..=500).text("Base (ms)")).changed();
                changed |= ui.add(Slider::new(&mut self.custom_conditions.jitter_ms, 0..=100).text("Jitter (ms)")).changed();

                ui.heading("Packet Loss");
                changed |= ui.add(Slider::new(&mut self.custom_conditions.packet_loss_percent, 0.0..=20.0).text("Loss %")).changed();
                changed |= ui.add(Slider::new(&mut self.custom_conditions.packet_duplicate_percent, 0.0..=10.0).text("Duplicate %")).changed();

                ui.heading("Spikes");
                changed |= ui.add(Slider::new(&mut self.custom_conditions.spike_frequency, 0.0..=20.0).text("Frequency (/min)")).changed();
                changed |= ui.add(Slider::new(&mut self.custom_conditions.spike_duration_ms, 0..=1000).text("Duration (ms)")).changed();
                changed |= ui.add(Slider::new(&mut self.custom_conditions.spike_latency_ms, 0..=500).text("Extra latency (ms)")).changed();

                if changed {
                    simulator.set_conditions(self.custom_conditions.clone());
                }

                ui.separator();

                // Statistics
                ui.heading("Statistics");
                let stats = simulator.stats();
                ui.label(format!("Packets sent: {}", stats.packets_sent));
                ui.label(format!("Packets received: {}", stats.packets_received));
                ui.label(format!("Packets dropped: {}", stats.packets_dropped));
                ui.label(format!("Packets duplicated: {}", stats.packets_duplicated));
                ui.label(format!("Current rollback: {} frames", stats.current_rollback_frames));
                ui.label(format!("Max rollback: {} frames", stats.max_rollback_frames));
            });
    }
}
```

### Step 4: Register FFI Functions

**File: `core/src/ffi.rs`**

```rust
// Add to register_common_ffi
linker.func_wrap("env", "net_sim_get_rtt", net_sim_get_rtt)?;
linker.func_wrap("env", "net_sim_get_loss_percent", net_sim_get_loss_percent)?;
linker.func_wrap("env", "net_sim_is_enabled", net_sim_is_enabled)?;
linker.func_wrap("env", "net_get_rollback_frames", net_get_rollback_frames)?;

fn net_sim_get_rtt<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> u32 {
    caller.data().game.net_sim_rtt_ms
}

fn net_sim_get_loss_percent<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> f32 {
    caller.data().game.net_sim_loss_percent
}

fn net_sim_is_enabled<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> i32 {
    if caller.data().game.net_sim_enabled { 1 } else { 0 }
}

fn net_get_rollback_frames<I: ConsoleInput, S>(caller: Caller<'_, GameStateWithConsole<I, S>>) -> u32 {
    caller.data().game.current_rollback_frames
}
```

### Step 5: Wire into GameSession

**File: `core/src/app/session.rs`**

```rust
use crate::app::network_ui::NetworkSimulatorUI;

impl<C: Console> GameSession<C> {
    pub fn render_network_ui(&mut self, ctx: &egui::Context) {
        if let Some(rollback) = &mut self.rollback_session {
            self.network_ui.show(ctx, &mut rollback.network_simulator);
        }
    }

    pub fn handle_network_hotkey(&mut self, key: &winit::keyboard::Key) -> bool {
        use winit::keyboard::{Key, NamedKey};
        match key {
            Key::Named(NamedKey::F7) => {
                self.network_ui.toggle();
                true
            }
            _ => false,
        }
    }
}
```

### File Checklist

| File | Changes |
|------|---------|
| `core/src/net/simulator.rs` | New file: NetworkSimulator, NetworkConditions, NetworkStats |
| `core/src/net/mod.rs` | Export simulator module |
| `core/src/rollback/session.rs` | Add NetworkSimulator integration |
| `core/src/app/network_ui.rs` | New file: NetworkSimulatorUI |
| `core/src/app/session.rs` | Add network UI rendering and hotkeys |
| `core/src/wasm/state.rs` | Add net_sim fields to GameState |
| `core/src/ffi.rs` | Add network FFI functions |
| `emberware-z/src/app/mod.rs` | Wire F7 hotkey and UI rendering |
| `core/Cargo.toml` | Add `rand` crate if not present |

### Test Cases

1. **Latency simulation**: Set 100ms latency, verify input delay increases
2. **Packet loss**: Set 5% loss, verify occasional rollbacks
3. **Jitter**: Set high jitter, verify variable latency
4. **Spike simulation**: Set spikes, verify periodic latency spikes
5. **Preset switching**: Switch presets, verify conditions update
6. **Statistics**: Play game, verify stats accumulate correctly
7. **Enable/disable**: Toggle simulation, verify immediate effect
8. **Local play**: Verify simulation disabled for non-netplay

## Future Enhancements

1. **Network recording**: Record actual network conditions for replay
2. **Chaos mode**: Randomly vary conditions during play
3. **Region presets**: Auto-configure for specific geographic routes
4. **A/B comparison**: Split-screen with different conditions
5. **Stress testing**: Automated test suite with various conditions
6. **Spectator mode**: Watch match with artificial latency

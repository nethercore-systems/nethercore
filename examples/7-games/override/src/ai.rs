//! Runner AI for single-player mode
//!
//! When player_count < 4, AI takes control of uncontrolled runners.
//! Uses BFS pathfinding to navigate toward goals:
//! 1. If cores uncollected: navigate to nearest uncollected core
//! 2. If all cores collected: navigate to extraction point
//!
//! AI avoids active traps and respects wall collision.

use crate::{
    FACILITY_WIDTH, FACILITY_HEIGHT, MAX_RUNNERS, TILE_SIZE,
    TileType, Tile, Core, Trap, Runner, RunnerState,
    fixed_to_int,
};

/// Maximum BFS queue size (facility is 40x22 = 880 tiles)
const BFS_QUEUE_SIZE: usize = 1024;

/// AI state for a single runner
#[derive(Clone, Copy)]
pub struct RunnerAI {
    /// Current target tile X
    pub target_x: u8,
    /// Current target tile Y
    pub target_y: u8,
    /// Next waypoint X (immediate next step)
    pub waypoint_x: u8,
    /// Next waypoint Y
    pub waypoint_y: u8,
    /// Ticks since last path recalculation
    pub path_age: u16,
    /// Whether AI is currently blocked
    pub blocked: bool,
}

impl RunnerAI {
    pub const fn new() -> Self {
        Self {
            target_x: 0,
            target_y: 0,
            waypoint_x: 0,
            waypoint_y: 0,
            path_age: 0,
            blocked: false,
        }
    }
}

/// Input values for a runner (returned by AI)
#[derive(Clone, Copy, Default)]
pub struct RunnerInput {
    pub move_x: i32,
    pub move_y: i32,
    pub sprint: bool,
    pub crouch: bool,
}

/// BFS queue entry
#[derive(Clone, Copy)]
struct BfsNode {
    x: u8,
    y: u8,
    parent_x: u8,
    parent_y: u8,
}

/// Check if a tile is walkable for pathfinding
fn is_walkable(
    tiles: &[[Tile; FACILITY_WIDTH]; FACILITY_HEIGHT],
    traps: &[Trap],
    x: usize,
    y: usize,
) -> bool {
    if x >= FACILITY_WIDTH || y >= FACILITY_HEIGHT {
        return false;
    }

    let tile = &tiles[y][x];

    // Walls are never walkable
    if tile.tile_type == TileType::Wall {
        return false;
    }

    // Check for active traps at this location
    for trap in traps.iter() {
        if trap.x as usize == x && trap.y as usize == y && trap.active {
            return false;
        }
    }

    true
}

/// Find path using BFS from start to any goal tile
/// Returns the first step direction, or None if no path found
fn find_path(
    tiles: &[[Tile; FACILITY_WIDTH]; FACILITY_HEIGHT],
    traps: &[Trap],
    start_x: usize,
    start_y: usize,
    goal_x: usize,
    goal_y: usize,
) -> Option<(u8, u8)> {
    // Already at goal?
    if start_x == goal_x && start_y == goal_y {
        return Some((start_x as u8, start_y as u8));
    }

    // BFS queue (using array for no_std)
    let mut queue: [BfsNode; BFS_QUEUE_SIZE] = [BfsNode {
        x: 0,
        y: 0,
        parent_x: 255,
        parent_y: 255,
    }; BFS_QUEUE_SIZE];
    let mut head: usize = 0;
    let mut tail: usize = 0;

    // Visited array
    let mut visited = [[false; FACILITY_WIDTH]; FACILITY_HEIGHT];

    // Parent tracking for path reconstruction
    let mut parent = [[(255u8, 255u8); FACILITY_WIDTH]; FACILITY_HEIGHT];

    // Start BFS
    queue[tail] = BfsNode {
        x: start_x as u8,
        y: start_y as u8,
        parent_x: 255,
        parent_y: 255,
    };
    tail += 1;
    visited[start_y][start_x] = true;

    // Direction offsets (4-directional movement)
    const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

    while head < tail && tail < BFS_QUEUE_SIZE {
        let current = queue[head];
        head += 1;

        let cx = current.x as usize;
        let cy = current.y as usize;

        // Check if we reached the goal
        if cx == goal_x && cy == goal_y {
            // Backtrack to find first step
            let mut px = goal_x;
            let mut py = goal_y;

            loop {
                let (ppx, ppy) = parent[py][px];
                if ppx == 255 || ppy == 255 {
                    // This is the start
                    return Some((px as u8, py as u8));
                }
                if ppx as usize == start_x && ppy as usize == start_y {
                    // Current position is the first step
                    return Some((px as u8, py as u8));
                }
                px = ppx as usize;
                py = ppy as usize;
            }
        }

        // Explore neighbors
        for (dx, dy) in DIRS.iter() {
            let nx = (cx as i32 + dx) as usize;
            let ny = (cy as i32 + dy) as usize;

            if nx < FACILITY_WIDTH && ny < FACILITY_HEIGHT
               && !visited[ny][nx]
               && is_walkable(tiles, traps, nx, ny)
            {
                visited[ny][nx] = true;
                parent[ny][nx] = (cx as u8, cy as u8);

                queue[tail] = BfsNode {
                    x: nx as u8,
                    y: ny as u8,
                    parent_x: cx as u8,
                    parent_y: cy as u8,
                };
                tail += 1;
            }
        }
    }

    // No path found
    None
}

/// Find the nearest uncollected core
fn find_nearest_core(
    cores: &[Core],
    runner_tile_x: usize,
    runner_tile_y: usize,
) -> Option<(u8, u8)> {
    let mut nearest: Option<(u8, u8)> = None;
    let mut nearest_dist = i32::MAX;

    for core in cores.iter() {
        if !core.collected {
            let dx = core.x as i32 - runner_tile_x as i32;
            let dy = core.y as i32 - runner_tile_y as i32;
            let dist = dx * dx + dy * dy;

            if dist < nearest_dist {
                nearest_dist = dist;
                nearest = Some((core.x, core.y));
            }
        }
    }

    nearest
}

/// Update AI for a runner and return input
pub fn update_runner_ai(
    ai: &mut RunnerAI,
    runner: &Runner,
    tiles: &[[Tile; FACILITY_WIDTH]; FACILITY_HEIGHT],
    traps: &[Trap],
    cores: &[Core],
    cores_collected: u8,
    extract_x: u8,
    extract_y: u8,
) -> RunnerInput {
    let mut input = RunnerInput::default();

    // Dead runners don't need AI
    if runner.state == RunnerState::Dead {
        return input;
    }

    // Get runner's current tile position
    let runner_tile_x = (fixed_to_int(runner.x) / TILE_SIZE) as usize;
    let runner_tile_y = (fixed_to_int(runner.y) / TILE_SIZE) as usize;

    // Increment path age
    ai.path_age = ai.path_age.saturating_add(1);

    // Recalculate path periodically or if blocked
    let needs_repath = ai.path_age > 30 || ai.blocked;

    if needs_repath {
        ai.path_age = 0;
        ai.blocked = false;

        // Determine goal
        let goal = if cores_collected >= 3 {
            // All cores collected, head to extraction
            Some((extract_x, extract_y))
        } else {
            // Find nearest uncollected core
            find_nearest_core(cores, runner_tile_x, runner_tile_y)
        };

        if let Some((goal_x, goal_y)) = goal {
            ai.target_x = goal_x;
            ai.target_y = goal_y;

            // Find path to goal
            if let Some((waypoint_x, waypoint_y)) = find_path(
                tiles,
                traps,
                runner_tile_x,
                runner_tile_y,
                goal_x as usize,
                goal_y as usize,
            ) {
                ai.waypoint_x = waypoint_x;
                ai.waypoint_y = waypoint_y;
            } else {
                // No path found, mark as blocked
                ai.blocked = true;
            }
        }
    }

    // Calculate movement direction toward waypoint
    let waypoint_px = ai.waypoint_x as i32 * TILE_SIZE + TILE_SIZE / 2;
    let waypoint_py = ai.waypoint_y as i32 * TILE_SIZE + TILE_SIZE / 2;
    let runner_px = fixed_to_int(runner.x);
    let runner_py = fixed_to_int(runner.y);

    let dx = waypoint_px - runner_px;
    let dy = waypoint_py - runner_py;

    // Determine movement direction (prioritize larger delta)
    const THRESHOLD: i32 = 2;

    if dx.abs() > dy.abs() {
        input.move_x = if dx > THRESHOLD { 1 } else if dx < -THRESHOLD { -1 } else { 0 };
        input.move_y = 0;
    } else if dy.abs() > THRESHOLD {
        input.move_x = 0;
        input.move_y = if dy > THRESHOLD { 1 } else if dy < -THRESHOLD { -1 } else { 0 };
    }

    // Sprint when far from target, crouch when near traps
    let dist_to_target = dx.abs() + dy.abs();
    input.sprint = dist_to_target > TILE_SIZE * 4;

    // Check for nearby active traps and crouch
    for trap in traps.iter() {
        if trap.active {
            let trap_dx = (trap.x as i32 * TILE_SIZE) - runner_px;
            let trap_dy = (trap.y as i32 * TILE_SIZE) - runner_py;
            if trap_dx.abs() < TILE_SIZE * 2 && trap_dy.abs() < TILE_SIZE * 2 {
                input.crouch = true;
                input.sprint = false;
                break;
            }
        }
    }

    // Mark as blocked if we're not moving but should be
    if input.move_x == 0 && input.move_y == 0 && !ai.blocked {
        // Check if we've reached the waypoint
        let at_waypoint = runner_tile_x == ai.waypoint_x as usize
                       && runner_tile_y == ai.waypoint_y as usize;

        if at_waypoint {
            // Recalculate path next frame
            ai.path_age = 100;
        } else {
            ai.blocked = true;
        }
    }

    input
}

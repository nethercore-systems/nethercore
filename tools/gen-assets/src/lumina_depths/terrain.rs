//! Terrain and effect generators for LUMINA DEPTHS
//!
//! Rocks, vent chimneys, seafloor patches, and bubble effects.
//! All meshes use organic shaping with subdivision and smooth normals.

use super::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

// === HELPER: Create smooth organic mesh from parts ===
fn smooth_combine(parts: &[&UnpackedMesh]) -> UnpackedMesh {
    let mut result = combine(parts);
    result.apply(SmoothNormals { weld_threshold: 0.01 });
    result
}

/// Boulder - large weathered rock with natural erosion (~350 tris)
pub fn generate_rock_boulder(output_dir: &Path) {
    // Main boulder shape with organic irregularity
    let mut rock: UnpackedMesh = generate_sphere(0.3, 14, 10);
    rock.apply(Transform::scale(1.25, 0.75, 1.0));
    rock.apply(Subdivide { iterations: 1 });

    // Apply natural rock-like deformation
    for pos in &mut rock.positions {
        let x = pos[0];
        let y = pos[1];
        let z = pos[2];

        // Large-scale warping (major features)
        let warp1 = (x * 8.0).sin() * (z * 7.0).cos() * 0.04;
        let warp2 = (y * 10.0).sin() * (x * 9.0).cos() * 0.03;
        let warp3 = (z * 6.0).sin() * (y * 8.0).cos() * 0.035;

        pos[0] += warp1;
        pos[1] += warp2;
        pos[2] += warp3;

        // Flattened bottom (resting surface)
        if y < -0.1 {
            pos[1] = pos[1].max(-0.18);
        }

        // Erosion patterns - subtle surface detail
        let erosion = ((x * 20.0).sin() * (z * 22.0).cos() * (y * 18.0).sin()) * 0.015;
        let r = (x * x + y * y + z * z).sqrt();
        if r > 0.01 {
            pos[0] += erosion * (x / r);
            pos[1] += erosion * (y / r);
            pos[2] += erosion * (z / r);
        }
    }

    // Smaller embedded rocks for detail
    let mut embed1: UnpackedMesh = generate_sphere(0.1, 8, 6);
    embed1.apply(Subdivide { iterations: 1 });
    for pos in &mut embed1.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 5.0).sin() * 0.02;
        pos[0] += wobble;
        pos[2] += wobble * 0.8;
    }
    embed1.apply(Transform::translate(0.22, 0.08, 0.12));

    let mut embed2: UnpackedMesh = generate_sphere(0.08, 6, 5);
    for pos in &mut embed2.positions {
        let noise = ((pos[0] * 15.0).sin() * (pos[2] * 13.0).cos()) * 0.015;
        pos[1] += noise;
    }
    embed2.apply(Transform::translate(-0.18, 0.02, 0.2));

    let mut embed3: UnpackedMesh = generate_sphere(0.06, 6, 4);
    embed3.apply(Transform::scale(1.2, 0.8, 1.0));
    embed3.apply(Transform::translate(0.12, -0.08, -0.22));

    write_mesh(
        &smooth_combine(&[&rock, &embed1, &embed2, &embed3]),
        "rock_boulder",
        output_dir,
    );
}

/// Rock pillar - tall volcanic formation with natural layering (~300 tris)
pub fn generate_rock_pillar(output_dir: &Path) {
    // Main column with organic taper
    let mut pillar: UnpackedMesh = generate_cylinder(0.16, 0.12, 0.85, 12);
    pillar.apply(Subdivide { iterations: 1 });

    // Natural rock formation - slightly twisted and irregular
    for pos in &mut pillar.positions {
        let y = pos[1];
        let normalized_y = (y + 0.425) / 0.85;

        // Slight twist along height
        let twist_angle = normalized_y * 0.15;
        let old_x = pos[0];
        let old_z = pos[2];
        pos[0] = old_x * twist_angle.cos() - old_z * twist_angle.sin();
        pos[2] = old_x * twist_angle.sin() + old_z * twist_angle.cos();

        // Irregular bulges at different heights
        let bulge = (y * 8.0).sin() * 0.02 + (y * 15.0).cos() * 0.015;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += bulge * (pos[0] / r);
            pos[2] += bulge * (pos[2] / r);
        }

        // Vertical erosion channels
        let angle = old_x.atan2(old_z);
        let channel = (angle * 6.0).sin().abs() * 0.015;
        pos[0] -= channel * (pos[0] / r.max(0.01));
        pos[2] -= channel * (pos[2] / r.max(0.01));
    }
    pillar.apply(Transform::translate(0.0, 0.425, 0.0));

    // Weathered cap
    let mut top: UnpackedMesh = generate_sphere(0.2, 12, 8);
    top.apply(Transform::scale(1.0, 0.5, 1.0));
    top.apply(Subdivide { iterations: 1 });

    for pos in &mut top.positions {
        // Irregular eroded top
        let x = pos[0];
        let z = pos[2];
        let erosion = ((x * 12.0).sin() * (z * 14.0).cos()) * 0.025;
        pos[1] += erosion;
        // Edge weathering
        let edge_dist = (x * x + z * z).sqrt();
        if edge_dist > 0.12 {
            pos[1] -= (edge_dist - 0.12) * 0.3;
        }
    }
    top.apply(Transform::translate(0.0, 0.88, 0.0));

    // Wider spreading base
    let mut base: UnpackedMesh = generate_cylinder(0.24, 0.18, 0.18, 12);
    base.apply(Subdivide { iterations: 1 });
    for pos in &mut base.positions {
        let angle = pos[0].atan2(pos[2]);
        let lobe = (angle * 4.0).sin() * 0.03 + (angle * 7.0).cos() * 0.02;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += lobe * (pos[0] / r);
            pos[2] += lobe * (pos[2] / r);
        }
    }
    base.apply(Transform::translate(0.0, 0.09, 0.0));

    // Natural ledges/ridges
    let mut ledge1: UnpackedMesh = generate_torus(0.19, 0.035, 14, 6);
    ledge1.apply(Subdivide { iterations: 1 });
    for pos in &mut ledge1.positions {
        let angle = pos[0].atan2(pos[2]);
        let variation = (angle * 3.0).sin() * 0.015;
        pos[1] += variation;
    }
    ledge1.apply(Transform::translate(0.0, 0.32, 0.0));

    let mut ledge2: UnpackedMesh = generate_torus(0.16, 0.025, 12, 5);
    for pos in &mut ledge2.positions {
        let angle = pos[0].atan2(pos[2]);
        pos[1] += (angle * 4.0).sin() * 0.01;
    }
    ledge2.apply(Transform::translate(0.0, 0.58, 0.0));

    write_mesh(
        &smooth_combine(&[&pillar, &top, &base, &ledge1, &ledge2]),
        "rock_pillar",
        output_dir,
    );
}

/// Hydrothermal vent chimney - organic mineral structure (~280 tris)
pub fn generate_vent_chimney(output_dir: &Path) {
    // Main chimney stack with organic growth patterns
    let mut stack: UnpackedMesh = generate_cylinder(0.14, 0.1, 0.55, 12);
    stack.apply(Subdivide { iterations: 1 });

    // Organic chimney shape - mineral buildup patterns
    for pos in &mut stack.positions {
        let y = pos[1];
        let normalized_y = (y + 0.275) / 0.55;

        // Mineral buildup creates irregular surface
        let angle = pos[0].atan2(pos[2]);
        let buildup = (angle * 5.0).sin() * 0.025 * (1.0 - normalized_y * 0.5);
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += buildup * (pos[0] / r);
            pos[2] += buildup * (pos[2] / r);
        }

        // Vertical striations
        let striation = (angle * 12.0).sin() * 0.008;
        pos[0] += striation * (pos[0] / r.max(0.01));
        pos[2] += striation * (pos[2] / r.max(0.01));

        // Slight lean
        pos[0] += normalized_y * 0.03;
    }
    stack.apply(Transform::translate(0.0, 0.275, 0.0));

    // Flared vent opening with mineral rim
    let mut top: UnpackedMesh = generate_torus(0.16, 0.06, 14, 8);
    top.apply(Subdivide { iterations: 1 });
    for pos in &mut top.positions {
        // Irregular mineral rim
        let angle = pos[0].atan2(pos[2]);
        let rim_var = (angle * 6.0).sin() * 0.02 + (angle * 11.0).cos() * 0.015;
        pos[1] += rim_var;
        let lobe = (angle * 4.0).sin() * 0.02;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += lobe * (pos[0] / r);
            pos[2] += lobe * (pos[2] / r);
        }
    }
    top.apply(Transform::translate(0.03, 0.55, 0.0));

    // Dark opening cavity
    let mut opening: UnpackedMesh = generate_cylinder(0.1, 0.08, 0.12, 10);
    opening.apply(Subdivide { iterations: 1 });
    for pos in &mut opening.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 5.0).sin() * 0.01;
        pos[0] += wobble;
        pos[2] += wobble;
    }
    opening.apply(Transform::translate(0.03, 0.56, 0.0));

    // Mineral deposits - organic growths on sides
    let mut deposits = Vec::new();
    let deposit_configs = [
        (0.14, 0.22, 0.02, 0.07),
        (-0.12, 0.38, 0.1, 0.055),
        (0.1, 0.42, -0.12, 0.045),
        (-0.08, 0.18, -0.08, 0.05),
        (0.06, 0.48, 0.08, 0.04),
    ];

    for (x, y, z, size) in deposit_configs {
        let mut dep: UnpackedMesh = generate_sphere(size, 8, 6);
        dep.apply(Subdivide { iterations: 1 });
        // Organic blob shape
        for pos in &mut dep.positions {
            let angle = pos[0].atan2(pos[2]);
            let blob = (angle * 4.0).sin() * size * 0.2;
            pos[0] += blob;
            pos[2] += blob * 0.7;
        }
        dep.apply(Transform::translate(x, y, z));
        deposits.push(dep);
    }

    // Base mound - accumulated minerals
    let mut base: UnpackedMesh = generate_sphere(0.22, 12, 8);
    base.apply(Transform::scale(1.0, 0.28, 1.0));
    base.apply(Subdivide { iterations: 1 });
    for pos in &mut base.positions {
        let x = pos[0];
        let z = pos[2];
        // Irregular mound shape
        let mound_var = ((x * 10.0).sin() * (z * 12.0).cos()) * 0.02;
        pos[1] += mound_var;
        // Radial lobes
        let angle = x.atan2(z);
        let lobe = (angle * 5.0).sin() * 0.025;
        let r = (x * x + z * z).sqrt();
        if r > 0.01 {
            pos[0] += lobe * (x / r);
            pos[2] += lobe * (z / r);
        }
    }
    base.apply(Transform::translate(0.0, 0.03, 0.0));

    let deposit_refs: Vec<&UnpackedMesh> = deposits.iter().collect();
    let mut parts = vec![&stack, &top, &opening, &base];
    parts.extend(deposit_refs);

    write_mesh(&smooth_combine(&parts), "vent_chimney", output_dir);
}

/// Seafloor patch - undulating organic ground surface (~200 tris)
pub fn generate_seafloor_patch(output_dir: &Path) {
    // Base plane with natural undulation
    let mut floor: UnpackedMesh = generate_plane(2.2, 2.2, 12, 12);
    floor.apply(Transform::rotate_x(-90.0));

    // Apply natural seafloor undulation
    for pos in &mut floor.positions {
        let x = pos[0];
        let z = pos[2];

        // Rolling hills/dunes
        let dune1 = (x * 3.0).sin() * (z * 2.5).cos() * 0.08;
        let dune2 = (x * 5.0 + 1.0).sin() * (z * 4.0 + 0.5).cos() * 0.04;

        // Small ripples
        let ripple = (x * 12.0).sin() * (z * 10.0).cos() * 0.015;

        pos[1] = dune1 + dune2 + ripple;

        // Edge fade (blends into environment)
        let edge_dist = (x.abs().max(z.abs()) - 0.8).max(0.0);
        pos[1] *= 1.0 - edge_dist * 0.5;
    }

    // Organic mounds (sediment accumulation)
    let mut mound1: UnpackedMesh = generate_sphere(0.35, 10, 8);
    mound1.apply(Transform::scale(1.6, 0.18, 1.3));
    mound1.apply(Subdivide { iterations: 1 });
    for pos in &mut mound1.positions {
        let x = pos[0];
        let z = pos[2];
        let wobble = ((x * 8.0).sin() * (z * 7.0).cos()) * 0.02;
        pos[1] += wobble;
    }
    mound1.apply(Transform::translate(0.45, 0.05, 0.35));

    let mut mound2: UnpackedMesh = generate_sphere(0.25, 8, 6);
    mound2.apply(Transform::scale(1.4, 0.12, 1.1));
    for pos in &mut mound2.positions {
        let x = pos[0];
        let z = pos[2];
        pos[1] += ((x * 10.0).sin() * (z * 9.0).cos()) * 0.015;
    }
    mound2.apply(Transform::translate(-0.55, 0.03, -0.45));

    let mut mound3: UnpackedMesh = generate_sphere(0.2, 6, 5);
    mound3.apply(Transform::scale(1.3, 0.1, 1.2));
    mound3.apply(Transform::translate(-0.3, 0.02, 0.5));

    write_mesh(
        &smooth_combine(&[&floor, &mound1, &mound2, &mound3]),
        "seafloor_patch",
        output_dir,
    );
}

/// Bubble cluster - group of organic rising bubbles (~80 tris)
pub fn generate_bubble_cluster(output_dir: &Path) {
    // Various sized bubbles with slight deformation (water pressure)
    let bubble_data = [
        (0.0, 0.0, 0.0, 0.035, 1.05f32),
        (0.045, 0.06, 0.025, 0.025, 1.08),
        (-0.035, 0.1, -0.015, 0.028, 1.04),
        (0.025, 0.14, 0.035, 0.018, 1.1),
        (-0.025, 0.18, -0.025, 0.022, 1.06),
        (0.015, 0.22, 0.01, 0.015, 1.12),
        (-0.01, 0.25, 0.02, 0.012, 1.08),
        (0.03, 0.08, -0.02, 0.02, 1.05),
    ];

    let mut bubbles = Vec::new();
    for (x, y, z, radius, squash) in bubble_data {
        let mut bubble: UnpackedMesh = generate_sphere(radius, 10, 8);
        // Bubbles are slightly oblate when rising
        bubble.apply(Transform::scale(1.0, squash, 1.0));

        // Subtle surface wobble
        for pos in &mut bubble.positions {
            let angle = pos[0].atan2(pos[2]);
            let wobble = (angle * 3.0).sin() * radius * 0.08;
            pos[0] += wobble;
            pos[2] += wobble * 0.6;
        }

        bubble.apply(Transform::translate(x, y, z));
        bubbles.push(bubble);
    }

    let bubble_refs: Vec<&UnpackedMesh> = bubbles.iter().collect();
    write_mesh(&smooth_combine(&bubble_refs), "bubble_cluster", output_dir);
}

/// Small scattered rocks - detail rocks for seafloor (~150 tris)
pub fn generate_rock_scatter(output_dir: &Path) {
    let mut rocks = Vec::new();

    let rock_configs = [
        (0.0, 0.0, 0.0, 0.06, 0.8f32, 1.2f32),
        (0.12, 0.0, 0.08, 0.045, 1.1, 0.9),
        (-0.1, 0.0, 0.05, 0.05, 0.9, 1.0),
        (0.05, 0.0, -0.1, 0.035, 1.0, 1.1),
        (-0.08, 0.0, -0.06, 0.04, 0.85, 1.05),
    ];

    for (x, _, z, size, squash_y, stretch_x) in rock_configs {
        let mut rock: UnpackedMesh = generate_sphere(size, 8, 6);
        rock.apply(Transform::scale(stretch_x, squash_y, 1.0));
        rock.apply(Subdivide { iterations: 1 });

        // Irregular rock surface
        for pos in &mut rock.positions {
            let px = pos[0];
            let py = pos[1];
            let pz = pos[2];
            let noise = ((px * 20.0).sin() * (pz * 18.0).cos() * (py * 22.0).sin()) * size * 0.15;
            let r = (px * px + py * py + pz * pz).sqrt();
            if r > 0.001 {
                pos[0] += noise * (px / r);
                pos[1] += noise * (py / r);
                pos[2] += noise * (pz / r);
            }
            // Flat bottom
            if pos[1] < -size * 0.3 {
                pos[1] = -size * 0.35;
            }
        }

        rock.apply(Transform::translate(x, size * squash_y * 0.4, z));
        rocks.push(rock);
    }

    let rock_refs: Vec<&UnpackedMesh> = rocks.iter().collect();
    write_mesh(&smooth_combine(&rock_refs), "rock_scatter", output_dir);
}

/// Underwater cave entrance - rocky archway (~400 tris)
pub fn generate_cave_entrance(output_dir: &Path) {
    // Left rock pillar
    let mut left: UnpackedMesh = generate_cylinder(0.2, 0.15, 0.6, 10);
    left.apply(Subdivide { iterations: 1 });
    for pos in &mut left.positions {
        let y = pos[1];
        let angle = pos[0].atan2(pos[2]);
        let bulge = (y * 6.0).sin() * 0.03 + (angle * 5.0).sin() * 0.02;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += bulge * (pos[0] / r);
            pos[2] += bulge * (pos[2] / r);
        }
    }
    left.apply(Transform::translate(-0.25, 0.3, 0.0));

    // Right rock pillar
    let mut right: UnpackedMesh = generate_cylinder(0.18, 0.14, 0.55, 10);
    right.apply(Subdivide { iterations: 1 });
    for pos in &mut right.positions {
        let y = pos[1];
        let angle = pos[0].atan2(pos[2]);
        let bulge = (y * 7.0).sin() * 0.025 + (angle * 4.0).sin() * 0.02;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += bulge * (pos[0] / r);
            pos[2] += bulge * (pos[2] / r);
        }
    }
    right.apply(Transform::translate(0.28, 0.275, 0.0));

    // Arch connecting top
    let mut arch: UnpackedMesh = generate_torus(0.3, 0.12, 12, 8);
    arch.apply(Subdivide { iterations: 1 });
    // Keep only top half
    for pos in &mut arch.positions {
        if pos[1] < 0.0 {
            pos[1] = 0.0;
            pos[0] *= 0.5;
            pos[2] *= 0.5;
        }
        // Irregular surface
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 6.0).sin() * 0.025;
        pos[1] += wobble;
    }
    arch.apply(Transform::scale(1.0, 0.7, 0.8));
    arch.apply(Transform::translate(0.0, 0.55, 0.0));

    // Base rocks
    let mut base_left: UnpackedMesh = generate_sphere(0.15, 8, 6);
    base_left.apply(Transform::scale(1.4, 0.4, 1.2));
    base_left.apply(Transform::translate(-0.3, 0.03, 0.0));

    let mut base_right: UnpackedMesh = generate_sphere(0.12, 8, 6);
    base_right.apply(Transform::scale(1.3, 0.35, 1.1));
    base_right.apply(Transform::translate(0.32, 0.02, 0.0));

    write_mesh(
        &smooth_combine(&[&left, &right, &arch, &base_left, &base_right]),
        "cave_entrance",
        output_dir,
    );
}

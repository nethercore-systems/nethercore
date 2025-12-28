//! Flora generators for LUMINA DEPTHS
//!
//! Coral, kelp, anemones, and sea grass for underwater environments.
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

/// Brain coral - organic dome with meandering ridges (~300 tris)
pub fn generate_coral_brain(output_dir: &Path) {
    // Main dome with organic bulging
    let mut dome: UnpackedMesh = generate_sphere(0.2, 16, 12);
    dome.apply(Transform::scale(1.0, 0.65, 1.0));
    dome.apply(Subdivide { iterations: 1 });

    // Apply organic warping - brain coral has meandering ridges
    for pos in &mut dome.positions {
        let x = pos[0];
        let z = pos[2];
        // Create ridge pattern using sine waves
        let ridge_pattern = (x * 25.0).sin() * 0.02 + (z * 25.0).cos() * 0.015;
        let height_factor = 1.0 - (pos[1] / 0.15).abs().min(1.0);
        pos[1] += ridge_pattern * height_factor;

        // Add subtle organic irregularity
        let noise = ((x * 15.0).sin() * (z * 17.0).cos()) * 0.01;
        pos[0] += noise;
        pos[2] += noise * 0.8;
    }

    // Base attachment - organic holdfast
    let mut base: UnpackedMesh = generate_cylinder(0.1, 0.08, 0.06, 10);
    base.apply(Subdivide { iterations: 1 });
    base.apply(Transform::translate(0.0, -0.1, 0.0));

    // Warp base for organic look
    for pos in &mut base.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 3.0).sin() * 0.015;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += wobble * (pos[0] / r);
            pos[2] += wobble * (pos[2] / r);
        }
    }

    write_mesh(&smooth_combine(&[&dome, &base]), "coral_brain", output_dir);
}

/// Fan coral - delicate branching fan structure (~250 tris)
pub fn generate_coral_fan(output_dir: &Path) {
    // Create fan shape with organic curves
    let mut fan: UnpackedMesh = generate_sphere(0.22, 14, 10);
    fan.apply(Transform::scale(1.0, 1.3, 0.08));
    fan.apply(Subdivide { iterations: 1 });

    // Shape into fan with wavy edges
    for pos in &mut fan.positions {
        let y = pos[1];
        let x = pos[0];

        // Taper bottom
        if y < 0.0 {
            let taper = 1.0 + y * 2.0;
            pos[0] *= taper.max(0.1);
        }

        // Wavy edge pattern
        let wave = (y * 20.0).sin() * 0.02 * y.abs();
        pos[2] += wave;

        // Organic fenestrations (holes pattern suggestion via thinning)
        let fenestrate = ((x * 15.0).sin() * (y * 12.0).cos()).abs();
        pos[2] *= 1.0 - fenestrate * 0.3;
    }
    fan.apply(Transform::translate(0.0, 0.18, 0.0));

    // Curved stem
    let mut stem: UnpackedMesh = generate_cylinder(0.025, 0.02, 0.12, 8);
    stem.apply(Subdivide { iterations: 1 });

    // Add slight curve to stem
    for pos in &mut stem.positions {
        let y = pos[1];
        let curve = y * y * 0.15;
        pos[2] += curve;
    }

    // Organic holdfast base
    let mut base: UnpackedMesh = generate_sphere(0.05, 8, 6);
    base.apply(Transform::scale(1.8, 0.4, 1.8));
    base.apply(Transform::translate(0.0, -0.06, 0.0));

    // Irregular base shape
    for pos in &mut base.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 4.0).sin() * 0.01 + (angle * 7.0).cos() * 0.008;
        pos[0] += wobble;
        pos[2] += wobble;
    }

    write_mesh(&smooth_combine(&[&fan, &stem, &base]), "coral_fan", output_dir);
}

/// Branching coral - organic tree-like structure (~350 tris)
pub fn generate_coral_branch(output_dir: &Path) {
    // Main trunk with organic taper
    let mut trunk: UnpackedMesh = generate_cylinder(0.035, 0.025, 0.22, 8);
    trunk.apply(Subdivide { iterations: 1 });
    trunk.apply(Transform::translate(0.0, 0.11, 0.0));

    // Organic curve to trunk
    for pos in &mut trunk.positions {
        let y = pos[1];
        let lean = y * 0.08;
        pos[0] += lean;
        // Subtle thickness variation
        let bulge = (y * 15.0).sin() * 0.003;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.001 {
            pos[0] += bulge * (pos[0] / r);
            pos[2] += bulge * (pos[2] / r);
        }
    }

    // Create organic branches
    let mut branches = Vec::new();
    let branch_configs = [
        (0.06, 0.19, 0.02, 0.14, -25.0f32, 15.0f32),
        (-0.04, 0.16, 0.03, 0.12, 30.0, -10.0),
        (0.02, 0.17, -0.04, 0.11, -20.0, -25.0),
        (-0.02, 0.21, 0.01, 0.09, 35.0, 20.0),
    ];

    for (ox, oy, oz, length, angle_z, angle_x) in branch_configs {
        let mut branch: UnpackedMesh = generate_cylinder(0.018, 0.01, length, 6);
        branch.apply(Subdivide { iterations: 1 });

        // Organic curve
        for pos in &mut branch.positions {
            let y = pos[1];
            let curve = y * y * 0.2;
            pos[0] += curve;
        }

        branch.apply(Transform::rotate_z(angle_z));
        branch.apply(Transform::rotate_x(angle_x));
        branch.apply(Transform::translate(ox, oy, oz));
        branches.push(branch);
    }

    // Secondary twigs with polyp tips
    let mut twigs = Vec::new();
    let twig_configs = [
        (0.12, 0.26, 0.03, -40.0f32),
        (-0.08, 0.23, 0.04, 45.0),
        (0.04, 0.25, -0.06, -35.0),
        (0.0, 0.28, 0.0, 0.0),
    ];

    for (x, y, z, angle) in twig_configs {
        let mut twig: UnpackedMesh = generate_cylinder(0.008, 0.004, 0.05, 5);
        twig.apply(Transform::rotate_z(angle));
        twig.apply(Transform::translate(x, y, z));
        twigs.push(twig);

        // Polyp tip (small organic bulb)
        let mut tip: UnpackedMesh = generate_sphere(0.012, 6, 4);
        tip.apply(Transform::scale(1.0, 1.3, 1.0));
        let tip_offset = 0.03;
        tip.apply(Transform::translate(
            x + angle.to_radians().sin() * tip_offset,
            y + tip_offset,
            z,
        ));
        twigs.push(tip);
    }

    // Organic base
    let mut base: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.03, 8);
    base.apply(Subdivide { iterations: 1 });
    for pos in &mut base.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 5.0).sin() * 0.01;
        pos[0] += wobble;
        pos[2] += wobble;
    }

    let branch_refs: Vec<&UnpackedMesh> = branches.iter().collect();
    let twig_refs: Vec<&UnpackedMesh> = twigs.iter().collect();
    let mut parts = vec![&trunk, &base];
    parts.extend(branch_refs);
    parts.extend(twig_refs);

    write_mesh(&smooth_combine(&parts), "coral_branch", output_dir);
}

/// Kelp stalk - tall swaying plant with realistic blades (~200 tris)
pub fn generate_kelp(output_dir: &Path) {
    // Long stipe (stem) with natural curve
    let mut stipe: UnpackedMesh = generate_cylinder(0.018, 0.012, 0.65, 8);
    stipe.apply(Subdivide { iterations: 2 });

    // Apply natural S-curve like real kelp
    for pos in &mut stipe.positions {
        let y = pos[1];
        let normalized_y = y / 0.65;
        // S-curve sway
        let sway = (normalized_y * 3.14159).sin() * 0.06 * normalized_y;
        pos[0] += sway;
        // Secondary wobble
        let wobble = (y * 12.0).sin() * 0.01;
        pos[2] += wobble;
    }
    stipe.apply(Transform::translate(0.0, 0.325, 0.0));

    // Create organic kelp blades (leaves)
    let mut blades = Vec::new();
    for i in 0..6 {
        let y = 0.12 + (i as f32) * 0.1;
        let angle = (i as f32) * 72.0;
        let blade_length = 0.12 + (i as f32) * 0.02;

        // Each blade is a stretched, curved shape
        let mut blade: UnpackedMesh = generate_sphere(0.04, 8, 6);
        blade.apply(Transform::scale(blade_length / 0.04, 0.15, 1.2));
        blade.apply(Subdivide { iterations: 1 });

        // Organic undulation and curl
        for pos in &mut blade.positions {
            let x = pos[0];
            // Blade curls at edges
            let curl = (x / blade_length).powi(2) * 0.03;
            pos[1] += curl;
            // Wavy undulation
            let wave = (x * 25.0).sin() * 0.008;
            pos[1] += wave;
            // Taper toward tip
            if x > 0.0 {
                let taper = 1.0 - (x / blade_length).min(1.0) * 0.5;
                pos[2] *= taper;
            }
        }

        blade.apply(Transform::rotate_y(angle));
        blade.apply(Transform::rotate_z(35.0 + (i as f32) * 5.0));

        // Apply stipe curve offset
        let normalized_y = y / 0.65;
        let stipe_offset = (normalized_y * 3.14159).sin() * 0.06 * normalized_y;
        blade.apply(Transform::translate(stipe_offset, y, 0.0));
        blades.push(blade);
    }

    // Organic holdfast (root structure)
    let mut holdfast: UnpackedMesh = generate_sphere(0.05, 10, 6);
    holdfast.apply(Transform::scale(1.6, 0.35, 1.6));
    holdfast.apply(Subdivide { iterations: 1 });

    // Irregular holdfast shape
    for pos in &mut holdfast.positions {
        let angle = pos[0].atan2(pos[2]);
        let lobe = (angle * 5.0).sin() * 0.015 + (angle * 8.0).cos() * 0.01;
        pos[0] += lobe;
        pos[2] += lobe;
    }

    // Float bladder at top (pneumatocyst)
    let mut bladder: UnpackedMesh = generate_sphere(0.03, 8, 6);
    bladder.apply(Transform::scale(1.0, 1.4, 1.0));
    bladder.apply(Transform::translate(0.06, 0.68, 0.0));

    let blade_refs: Vec<&UnpackedMesh> = blades.iter().collect();
    let mut parts = vec![&stipe, &holdfast, &bladder];
    parts.extend(blade_refs);

    write_mesh(&smooth_combine(&parts), "kelp", output_dir);
}

/// Sea anemone - organic column with flowing tentacles (~280 tris)
pub fn generate_anemone(output_dir: &Path) {
    // Column (body) with organic shape
    let mut column: UnpackedMesh = generate_cylinder(0.065, 0.055, 0.12, 12);
    column.apply(Subdivide { iterations: 1 });

    // Organic column shape - slight hourglass
    for pos in &mut column.positions {
        let y = pos[1];
        let normalized_y = (y / 0.12 - 0.5).abs();
        let pinch = 1.0 - normalized_y * 0.15;
        pos[0] *= pinch;
        pos[2] *= pinch;

        // Subtle vertical ridges
        let angle = pos[0].atan2(pos[2]);
        let ridge = (angle * 8.0).sin() * 0.005;
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r > 0.01 {
            pos[0] += ridge * (pos[0] / r);
            pos[2] += ridge * (pos[2] / r);
        }
    }
    column.apply(Transform::translate(0.0, 0.06, 0.0));

    // Oral disc - fleshy top
    let mut disc: UnpackedMesh = generate_sphere(0.075, 12, 6);
    disc.apply(Transform::scale(1.0, 0.25, 1.0));
    disc.apply(Subdivide { iterations: 1 });

    // Central mouth depression
    for pos in &mut disc.positions {
        let r = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if r < 0.03 {
            pos[1] -= (1.0 - r / 0.03) * 0.02;
        }
    }
    disc.apply(Transform::translate(0.0, 0.12, 0.0));

    // Organic flowing tentacles
    let mut tentacles = Vec::new();
    for i in 0..16 {
        let angle = (i as f32) * 22.5;
        let angle_rad = angle.to_radians();
        let radius = if i % 2 == 0 { 0.055 } else { 0.045 };
        let tent_length = if i % 2 == 0 { 0.1 } else { 0.07 };

        let mut tent: UnpackedMesh = generate_cylinder(0.008, 0.003, tent_length, 6);
        tent.apply(Subdivide { iterations: 1 });

        // Organic curve and taper
        for pos in &mut tent.positions {
            let y = pos[1];
            let normalized_y = y / tent_length;
            // Graceful outward curve
            let curve = normalized_y * normalized_y * 0.04;
            pos[0] += curve;
            // Slight wave
            let wave = (y * 30.0).sin() * 0.003 * normalized_y;
            pos[2] += wave;
        }

        tent.apply(Transform::rotate_z(-30.0 - (i % 3) as f32 * 10.0));
        tent.apply(Transform::rotate_y(angle));
        tent.apply(Transform::translate(
            angle_rad.cos() * radius,
            0.14,
            angle_rad.sin() * radius,
        ));
        tentacles.push(tent);
    }

    // Pedal disc base
    let mut base: UnpackedMesh = generate_cylinder(0.085, 0.085, 0.02, 12);
    base.apply(Subdivide { iterations: 1 });
    for pos in &mut base.positions {
        let angle = pos[0].atan2(pos[2]);
        let wobble = (angle * 6.0).sin() * 0.008;
        pos[0] += wobble;
        pos[2] += wobble;
    }

    let tent_refs: Vec<&UnpackedMesh> = tentacles.iter().collect();
    let mut parts = vec![&column, &disc, &base];
    parts.extend(tent_refs);

    write_mesh(&smooth_combine(&parts), "anemone", output_dir);
}

/// Sea grass - cluster of organic flowing blades (~120 tris)
pub fn generate_sea_grass(output_dir: &Path) {
    // Cluster of flowing grass blades
    let mut blades = Vec::new();
    let blade_configs: [(f32, f32, f32, f32); 7] = [
        (0.0, 0.0, 0.0, 0.14),
        (0.025, 0.0, 0.018, 0.12),
        (-0.02, 0.0, 0.022, 0.11),
        (0.018, 0.0, -0.02, 0.13),
        (-0.022, 0.0, -0.015, 0.10),
        (0.01, 0.0, 0.03, 0.09),
        (-0.015, 0.0, -0.025, 0.115),
    ];

    for (x, _, z, height) in blade_configs {
        // Each blade is a thin curved shape
        let mut blade: UnpackedMesh = generate_cube(0.004, height, 0.018);
        blade.apply(Subdivide { iterations: 2 });

        // Organic curve and taper
        for pos in &mut blade.positions {
            let y = pos[1] + height / 2.0;
            let normalized_y = y / height;

            // Graceful curve
            let curve = normalized_y * normalized_y * 0.06 * (1.0 + x * 3.0);
            pos[0] += curve;

            // Slight twist
            let twist = normalized_y * 0.3;
            let old_x = pos[0];
            let old_z = pos[2];
            pos[0] = old_x * twist.cos() - old_z * twist.sin() + old_x * (1.0 - twist.cos().abs());
            pos[2] = old_x * twist.sin() + old_z * twist.cos();

            // Taper toward tip
            let taper = 1.0 - normalized_y * 0.4;
            pos[2] *= taper;

            // Subtle wave
            let wave = (y * 25.0).sin() * 0.004;
            pos[0] += wave;
        }

        blade.apply(Transform::rotate_z(x * 50.0));
        blade.apply(Transform::translate(x, height / 2.0, z));
        blades.push(blade);
    }

    // Small root cluster
    let mut roots: UnpackedMesh = generate_sphere(0.025, 6, 4);
    roots.apply(Transform::scale(1.5, 0.3, 1.5));
    for pos in &mut roots.positions {
        let angle = pos[0].atan2(pos[2]);
        let bump = (angle * 6.0).sin() * 0.005;
        pos[0] += bump;
        pos[2] += bump;
    }

    let blade_refs: Vec<&UnpackedMesh> = blades.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = blade_refs;
    parts.push(&roots);

    write_mesh(&smooth_combine(&parts), "sea_grass", output_dir);
}

/// Giant kelp forest piece - multiple stalks for dense forests (~400 tris)
pub fn generate_kelp_forest(output_dir: &Path) {
    let mut all_parts = Vec::new();

    // Generate 3 kelp stalks at different positions
    let stalk_positions = [
        (0.0, 0.0, 0.0, 1.0f32),
        (0.15, 0.0, 0.1, 0.8),
        (-0.12, 0.0, 0.08, 0.9),
    ];

    for (ox, _, oz, scale) in stalk_positions {
        let height = 0.5 * scale;

        // Stipe
        let mut stipe: UnpackedMesh = generate_cylinder(0.015 * scale, 0.01 * scale, height, 6);
        stipe.apply(Subdivide { iterations: 1 });

        for pos in &mut stipe.positions {
            let y = pos[1];
            let sway = (y / height * 3.14159).sin() * 0.04 * scale;
            pos[0] += sway;
        }
        stipe.apply(Transform::translate(ox, height / 2.0, oz));
        all_parts.push(stipe);

        // A few blades per stalk
        for i in 0..3 {
            let y = 0.15 * scale + (i as f32) * 0.12 * scale;
            let mut blade: UnpackedMesh = generate_sphere(0.025 * scale, 6, 4);
            blade.apply(Transform::scale(2.5, 0.2, 1.0));
            blade.apply(Transform::rotate_z(30.0 + i as f32 * 15.0));
            blade.apply(Transform::rotate_y((i as f32) * 90.0));

            let stipe_sway = (y / height * 3.14159).sin() * 0.04 * scale;
            blade.apply(Transform::translate(ox + stipe_sway, y, oz));
            all_parts.push(blade);
        }
    }

    // Shared holdfast
    let mut holdfast: UnpackedMesh = generate_sphere(0.08, 10, 6);
    holdfast.apply(Transform::scale(1.5, 0.25, 1.5));
    all_parts.push(holdfast);

    let part_refs: Vec<&UnpackedMesh> = all_parts.iter().collect();
    write_mesh(&smooth_combine(&part_refs), "kelp_forest", output_dir);
}

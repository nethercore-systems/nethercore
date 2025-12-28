//! Sea creature generators for LUMINA DEPTHS
//!
//! All creatures follow specs from SHOWCASE_3.md:
//! - Zone 1 (Sunlit): Reef fish, sea turtle, manta ray
//! - Zone 2 (Twilight): Moon jelly, lanternfish, siphonophore
//! - Zone 3 (Midnight): Anglerfish, gulper eel, dumbo octopus
//! - Zone 4 (Vents): Tube worms, vent shrimp
//! - Epic: Blue whale
//!
//! ## Organic Mesh Philosophy
//! These generators create natural, organic-looking creatures using:
//! - Subdivision for smooth surfaces
//! - SmoothNormals for proper shading across seams
//! - Organic deformation of base primitives
//! - Careful blending where parts join

use super::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

// === HELPER: Create smooth organic mesh from parts ===
fn smooth_combine(parts: &[&UnpackedMesh]) -> UnpackedMesh {
    let mut result = combine(parts);
    // Apply smooth normals to blend seams between parts
    result.apply(SmoothNormals { weld_threshold: 0.01 });
    result
}

// === ZONE 1: SUNLIT WATERS ===

/// Reef fish - sleek tropical fish with flowing fins (~150 tris)
pub fn generate_reef_fish(output_dir: &Path) {
    // Main body - teardrop shape using deformed sphere
    let mut body: UnpackedMesh = generate_sphere(0.08, 12, 8);
    body.apply(Transform::scale(1.8, 1.0, 0.5)); // Elongated, laterally compressed
    body.apply(Subdivide { iterations: 1 });

    // Taper the rear by scaling vertices (manual organic shaping)
    for pos in &mut body.positions {
        let x = pos[0];
        if x < 0.0 {
            // Taper toward tail
            let taper = 1.0 - (-x / 0.15).min(1.0) * 0.6;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Tail fin - graceful forked shape
    let mut tail_upper: UnpackedMesh = generate_sphere(0.025, 6, 4);
    tail_upper.apply(Transform::scale(2.5, 1.8, 0.15));
    tail_upper.apply(Transform::rotate_z(25.0));
    tail_upper.apply(Transform::translate(-0.16, 0.02, 0.0));

    let mut tail_lower: UnpackedMesh = generate_sphere(0.025, 6, 4);
    tail_lower.apply(Transform::scale(2.5, 1.8, 0.15));
    tail_lower.apply(Transform::rotate_z(-25.0));
    tail_lower.apply(Transform::translate(-0.16, -0.02, 0.0));

    // Dorsal fin - elegant sail-like shape
    let mut dorsal: UnpackedMesh = generate_sphere(0.02, 6, 4);
    dorsal.apply(Transform::scale(2.0, 2.5, 0.12));
    dorsal.apply(Transform::translate(0.02, 0.065, 0.0));

    // Pectoral fins (side fins)
    let mut pec_l: UnpackedMesh = generate_sphere(0.015, 5, 3);
    pec_l.apply(Transform::scale(1.5, 0.8, 2.0));
    pec_l.apply(Transform::rotate_x(-20.0));
    pec_l.apply(Transform::translate(0.03, -0.01, -0.04));

    let mut pec_r: UnpackedMesh = generate_sphere(0.015, 5, 3);
    pec_r.apply(Transform::scale(1.5, 0.8, 2.0));
    pec_r.apply(Transform::rotate_x(20.0));
    pec_r.apply(Transform::translate(0.03, -0.01, 0.04));

    // Eyes - slightly bulging with realistic placement
    let mut eye_l: UnpackedMesh = generate_sphere(0.012, 6, 5);
    eye_l.apply(Transform::translate(0.08, 0.015, -0.028));

    let mut eye_r: UnpackedMesh = generate_sphere(0.012, 6, 5);
    eye_r.apply(Transform::translate(0.08, 0.015, 0.028));

    // Mouth area - slight protrusion
    let mut snout: UnpackedMesh = generate_sphere(0.025, 6, 4);
    snout.apply(Transform::scale(1.2, 0.7, 0.6));
    snout.apply(Transform::translate(0.12, -0.005, 0.0));

    let mesh = smooth_combine(&[
        &body, &tail_upper, &tail_lower, &dorsal,
        &pec_l, &pec_r, &eye_l, &eye_r, &snout
    ]);
    write_mesh(&mesh, "reef_fish", output_dir);
}

/// Sea turtle - elegant swimmer with detailed shell (~350 tris)
pub fn generate_sea_turtle(output_dir: &Path) {
    // Shell (carapace) - organic domed shape
    let mut shell: UnpackedMesh = generate_sphere(0.2, 14, 10);
    shell.apply(Transform::scale(1.3, 0.55, 1.1));
    shell.apply(Subdivide { iterations: 1 });

    // Sculpt shell edges to be more natural
    for pos in &mut shell.positions {
        // Add slight scalloping at edges
        let dist_from_center = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        if dist_from_center > 0.15 && pos[1] > 0.0 {
            let angle = pos[2].atan2(pos[0]);
            let scallop = 1.0 + (angle * 5.0).sin() * 0.03;
            pos[0] *= scallop;
            pos[2] *= scallop;
        }
    }
    shell.apply(Transform::translate(0.0, 0.04, 0.0));

    // Plastron (underside) - flatter organic shape
    let mut plastron: UnpackedMesh = generate_sphere(0.18, 12, 6);
    plastron.apply(Transform::scale(1.25, 0.18, 1.05));
    plastron.apply(Transform::translate(0.0, -0.03, 0.0));

    // Head - teardrop shaped with neck
    let mut neck: UnpackedMesh = generate_capsule(0.035, 0.08, 6, 4);
    neck.apply(Transform::rotate_z(80.0));
    neck.apply(Transform::translate(0.2, 0.0, 0.0));

    let mut head: UnpackedMesh = generate_sphere(0.055, 8, 6);
    head.apply(Transform::scale(1.3, 0.9, 0.85));
    head.apply(Subdivide { iterations: 1 });
    head.apply(Transform::translate(0.28, 0.01, 0.0));

    // Beak-like snout
    let mut snout: UnpackedMesh = generate_sphere(0.025, 6, 4);
    snout.apply(Transform::scale(1.4, 0.7, 0.8));
    snout.apply(Transform::translate(0.32, -0.005, 0.0));

    // Eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.015, 5, 4);
    eye_l.apply(Transform::translate(0.26, 0.025, -0.035));

    let mut eye_r: UnpackedMesh = generate_sphere(0.015, 5, 4);
    eye_r.apply(Transform::translate(0.26, 0.025, 0.035));

    // Front flippers - paddle-shaped, organic
    let mut flipper_fl: UnpackedMesh = generate_sphere(0.04, 8, 5);
    flipper_fl.apply(Transform::scale(3.0, 0.25, 1.3));
    flipper_fl.apply(Transform::rotate_z(-35.0));
    flipper_fl.apply(Transform::rotate_y(-15.0));
    flipper_fl.apply(Transform::translate(0.08, -0.04, -0.22));

    let mut flipper_fr: UnpackedMesh = generate_sphere(0.04, 8, 5);
    flipper_fr.apply(Transform::scale(3.0, 0.25, 1.3));
    flipper_fr.apply(Transform::rotate_z(-35.0));
    flipper_fr.apply(Transform::rotate_y(15.0));
    flipper_fr.apply(Transform::translate(0.08, -0.04, 0.22));

    // Rear flippers - smaller, more paddle-like
    let mut flipper_rl: UnpackedMesh = generate_sphere(0.025, 6, 4);
    flipper_rl.apply(Transform::scale(2.2, 0.25, 1.0));
    flipper_rl.apply(Transform::rotate_z(35.0));
    flipper_rl.apply(Transform::rotate_y(-20.0));
    flipper_rl.apply(Transform::translate(-0.2, -0.035, -0.14));

    let mut flipper_rr: UnpackedMesh = generate_sphere(0.025, 6, 4);
    flipper_rr.apply(Transform::scale(2.2, 0.25, 1.0));
    flipper_rr.apply(Transform::rotate_z(35.0));
    flipper_rr.apply(Transform::rotate_y(20.0));
    flipper_rr.apply(Transform::translate(-0.2, -0.035, 0.14));

    // Short tail
    let mut tail: UnpackedMesh = generate_sphere(0.02, 5, 3);
    tail.apply(Transform::scale(2.5, 0.6, 0.7));
    tail.apply(Transform::translate(-0.27, -0.01, 0.0));

    let mesh = smooth_combine(&[
        &shell, &plastron, &neck, &head, &snout, &eye_l, &eye_r,
        &flipper_fl, &flipper_fr, &flipper_rl, &flipper_rr, &tail
    ]);
    write_mesh(&mesh, "sea_turtle", output_dir);
}

/// Manta ray - graceful glider with flowing wings (~400 tris)
pub fn generate_manta_ray(output_dir: &Path) {
    // Main body/wings - single flowing organic shape
    let mut body: UnpackedMesh = generate_sphere(0.35, 16, 10);
    body.apply(Transform::scale(1.1, 0.12, 2.0));
    body.apply(Subdivide { iterations: 1 });

    // Sculpt wing shape - taper toward tips, add curvature
    for pos in &mut body.positions {
        let z_dist = pos[2].abs();
        // Taper wings toward tips
        if z_dist > 0.3 {
            let taper = 1.0 - ((z_dist - 0.3) / 0.4).min(1.0) * 0.7;
            pos[0] *= taper;
            pos[1] *= taper * 0.5;
        }
        // Curve wing tips upward
        if z_dist > 0.4 {
            let lift = ((z_dist - 0.4) / 0.3).min(1.0) * 0.04;
            pos[1] += lift;
        }
        // Taper toward tail
        if pos[0] < -0.15 {
            let taper = 1.0 - ((-pos[0] - 0.15) / 0.2).min(1.0) * 0.6;
            pos[2] *= taper;
        }
    }

    // Central body ridge
    let mut ridge: UnpackedMesh = generate_sphere(0.08, 8, 6);
    ridge.apply(Transform::scale(2.5, 0.6, 0.5));
    ridge.apply(Transform::translate(0.05, 0.02, 0.0));

    // Head - slightly bulbous
    let mut head: UnpackedMesh = generate_sphere(0.08, 10, 8);
    head.apply(Transform::scale(1.2, 0.6, 1.0));
    head.apply(Subdivide { iterations: 1 });
    head.apply(Transform::translate(0.32, 0.0, 0.0));

    // Cephalic fins - organic curved horns for filter feeding
    let mut horn_l: UnpackedMesh = generate_capsule(0.015, 0.08, 6, 4);
    horn_l.apply(Transform::rotate_z(-50.0));
    horn_l.apply(Transform::rotate_y(-25.0));
    horn_l.apply(Transform::translate(0.35, 0.02, -0.06));

    let mut horn_r: UnpackedMesh = generate_capsule(0.015, 0.08, 6, 4);
    horn_r.apply(Transform::rotate_z(-50.0));
    horn_r.apply(Transform::rotate_y(25.0));
    horn_r.apply(Transform::translate(0.35, 0.02, 0.06));

    // Horn tips (curled inward)
    let mut horn_tip_l: UnpackedMesh = generate_sphere(0.012, 5, 4);
    horn_tip_l.apply(Transform::translate(0.4, 0.06, -0.1));

    let mut horn_tip_r: UnpackedMesh = generate_sphere(0.012, 5, 4);
    horn_tip_r.apply(Transform::translate(0.4, 0.06, 0.1));

    // Eyes (on sides of head)
    let mut eye_l: UnpackedMesh = generate_sphere(0.015, 5, 4);
    eye_l.apply(Transform::translate(0.28, 0.02, -0.08));

    let mut eye_r: UnpackedMesh = generate_sphere(0.015, 5, 4);
    eye_r.apply(Transform::translate(0.28, 0.02, 0.08));

    // Whip-like tail - tapered organic shape
    let mut tail_base: UnpackedMesh = generate_capsule(0.02, 0.15, 6, 4);
    tail_base.apply(Transform::rotate_z(90.0));
    tail_base.apply(Transform::translate(-0.4, 0.0, 0.0));

    let mut tail_mid: UnpackedMesh = generate_capsule(0.012, 0.15, 5, 3);
    tail_mid.apply(Transform::rotate_z(95.0));
    tail_mid.apply(Transform::translate(-0.55, -0.01, 0.0));

    let mut tail_tip: UnpackedMesh = generate_capsule(0.006, 0.12, 4, 2);
    tail_tip.apply(Transform::rotate_z(100.0));
    tail_tip.apply(Transform::translate(-0.7, -0.03, 0.0));

    // Gill slits (subtle indentations represented as small forms)
    let mut gill_l: UnpackedMesh = generate_sphere(0.015, 4, 3);
    gill_l.apply(Transform::scale(0.3, 0.8, 1.5));
    gill_l.apply(Transform::translate(0.15, -0.02, -0.12));

    let mut gill_r: UnpackedMesh = generate_sphere(0.015, 4, 3);
    gill_r.apply(Transform::scale(0.3, 0.8, 1.5));
    gill_r.apply(Transform::translate(0.15, -0.02, 0.12));

    let mesh = smooth_combine(&[
        &body, &ridge, &head, &horn_l, &horn_r, &horn_tip_l, &horn_tip_r,
        &eye_l, &eye_r, &tail_base, &tail_mid, &tail_tip, &gill_l, &gill_r
    ]);
    write_mesh(&mesh, "manta_ray", output_dir);
}

// === ZONE 2: TWILIGHT REALM ===

/// Moon jellyfish - ethereal translucent dome (~250 tris)
pub fn generate_moon_jelly(output_dir: &Path) {
    // Bell - smooth organic dome with subtle pulsing shape
    let mut bell: UnpackedMesh = generate_sphere(0.15, 16, 12);
    bell.apply(Transform::scale(1.0, 0.55, 1.0));
    bell.apply(Subdivide { iterations: 1 });

    // Sculpt bell to have natural undulating edge
    for pos in &mut bell.positions {
        let dist = (pos[0] * pos[0] + pos[2] * pos[2]).sqrt();
        // Only affect lower rim
        if pos[1] < 0.02 {
            let angle = pos[2].atan2(pos[0]);
            // Undulating rim
            let wave = 1.0 + (angle * 8.0).sin() * 0.04;
            pos[0] *= wave;
            pos[2] *= wave;
            // Curl inward slightly
            if dist > 0.1 {
                pos[1] -= 0.01;
            }
        }
    }
    bell.apply(Transform::translate(0.0, 0.04, 0.0));

    // Four-lobed gonad pattern (horseshoe shapes visible through bell)
    let mut gonads = Vec::new();
    for i in 0..4 {
        let angle = (i as f32 * 90.0 + 45.0).to_radians();
        let mut gonad: UnpackedMesh = generate_torus(0.04, 0.012, 8, 4);
        gonad.apply(Transform::scale(1.0, 0.4, 1.0));
        gonad.apply(Transform::translate(angle.cos() * 0.05, 0.02, angle.sin() * 0.05));
        gonads.push(gonad);
    }

    // Oral arms - 4 delicate flowing appendages
    let mut arms = Vec::new();
    for i in 0..4 {
        let angle = (i as f32 * 90.0).to_radians();
        let x_base = angle.cos() * 0.04;
        let z_base = angle.sin() * 0.04;

        // Main arm - flowing organic shape
        let mut arm: UnpackedMesh = generate_capsule(0.012, 0.1, 6, 4);
        // Add slight wave to arm
        for pos in &mut arm.positions {
            let wave = (pos[1] * 15.0).sin() * 0.01;
            pos[0] += wave;
        }
        arm.apply(Transform::translate(x_base, -0.08, z_base));
        arms.push(arm);

        // Frilly edge detail
        let mut frill: UnpackedMesh = generate_sphere(0.015, 5, 3);
        frill.apply(Transform::scale(0.5, 1.5, 1.2));
        frill.apply(Transform::translate(x_base, -0.14, z_base));
        arms.push(frill);
    }

    // Marginal tentacles - delicate ring of short tendrils
    let mut margin: UnpackedMesh = generate_torus(0.145, 0.006, 24, 4);
    margin.apply(Transform::translate(0.0, -0.01, 0.0));

    // Individual marginal tentacles (8 thin strands)
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let mut tendril: UnpackedMesh = generate_capsule(0.004, 0.04, 4, 2);
        tendril.apply(Transform::translate(angle.cos() * 0.14, -0.04, angle.sin() * 0.14));
        arms.push(tendril);
    }

    let gonad_refs: Vec<&UnpackedMesh> = gonads.iter().collect();
    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![&bell, &margin];
    parts.extend(gonad_refs);
    parts.extend(arm_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "moon_jelly", output_dir);
}

/// Lanternfish - small bioluminescent fish with oversized eyes (~120 tris)
pub fn generate_lanternfish(output_dir: &Path) {
    // Sleek torpedo body with depth adaptation
    let mut body: UnpackedMesh = generate_sphere(0.025, 10, 8);
    body.apply(Transform::scale(2.0, 1.0, 0.9));
    body.apply(Subdivide { iterations: 1 });

    // Taper toward tail
    for pos in &mut body.positions {
        if pos[0] < 0.0 {
            let taper = 1.0 - (-pos[0] / 0.05).min(1.0) * 0.5;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Large tubular eyes (characteristic of deep-sea fish)
    let mut eye_l: UnpackedMesh = generate_sphere(0.012, 8, 6);
    eye_l.apply(Transform::scale(1.0, 1.3, 1.0));
    eye_l.apply(Transform::translate(0.035, 0.012, -0.015));

    let mut eye_r: UnpackedMesh = generate_sphere(0.012, 8, 6);
    eye_r.apply(Transform::scale(1.0, 1.3, 1.0));
    eye_r.apply(Transform::translate(0.035, 0.012, 0.015));

    // Snout
    let mut snout: UnpackedMesh = generate_sphere(0.008, 5, 4);
    snout.apply(Transform::scale(1.5, 0.8, 0.9));
    snout.apply(Transform::translate(0.05, -0.002, 0.0));

    // Forked tail fin
    let mut tail_upper: UnpackedMesh = generate_sphere(0.008, 4, 3);
    tail_upper.apply(Transform::scale(2.0, 1.5, 0.2));
    tail_upper.apply(Transform::rotate_z(20.0));
    tail_upper.apply(Transform::translate(-0.055, 0.008, 0.0));

    let mut tail_lower: UnpackedMesh = generate_sphere(0.008, 4, 3);
    tail_lower.apply(Transform::scale(2.0, 1.5, 0.2));
    tail_lower.apply(Transform::rotate_z(-20.0));
    tail_lower.apply(Transform::translate(-0.055, -0.008, 0.0));

    // Dorsal fin
    let mut dorsal: UnpackedMesh = generate_sphere(0.006, 4, 3);
    dorsal.apply(Transform::scale(1.5, 2.0, 0.15));
    dorsal.apply(Transform::translate(-0.01, 0.025, 0.0));

    // Adipose fin (small fatty fin near tail)
    let mut adipose: UnpackedMesh = generate_sphere(0.004, 3, 2);
    adipose.apply(Transform::scale(1.0, 1.5, 0.5));
    adipose.apply(Transform::translate(-0.035, 0.015, 0.0));

    // Photophores - rows of bioluminescent spots along body
    let mut photophores = Vec::new();
    // Lateral line of photophores
    for i in 0..5 {
        let x = 0.03 - (i as f32 * 0.015);
        let mut photo: UnpackedMesh = generate_sphere(0.003, 4, 3);
        photo.apply(Transform::translate(x, -0.012, 0.0));
        photophores.push(photo);
    }
    // Ventral photophores
    for i in 0..3 {
        let x = 0.02 - (i as f32 * 0.015);
        let mut photo: UnpackedMesh = generate_sphere(0.0025, 4, 3);
        photo.apply(Transform::translate(x, -0.018, 0.0));
        photophores.push(photo);
    }

    let photo_refs: Vec<&UnpackedMesh> = photophores.iter().collect();
    let mut parts = vec![
        &body, &eye_l, &eye_r, &snout, &tail_upper, &tail_lower,
        &dorsal, &adipose
    ];
    parts.extend(photo_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "lanternfish", output_dir);
}

/// Siphonophore - colonial organism chain (~300 tris)
pub fn generate_siphonophore(output_dir: &Path) {
    // Float (pneumatophore) - gas-filled float at top
    let mut float: UnpackedMesh = generate_sphere(0.045, 10, 8);
    float.apply(Transform::scale(1.0, 1.5, 1.0));
    float.apply(Subdivide { iterations: 1 });
    float.apply(Transform::translate(0.0, 0.08, 0.0));

    // Nectosome - swimming bells below float
    let mut bells = Vec::new();
    for i in 0..3 {
        let mut bell: UnpackedMesh = generate_sphere(0.025, 8, 6);
        bell.apply(Transform::scale(1.0, 0.7, 1.0));
        bell.apply(Transform::translate(0.0, 0.04 - (i as f32 * 0.03), 0.0));
        bells.push(bell);
    }

    // Siphosome - chain of specialized zooids
    let mut zooids = Vec::new();
    let segment_count = 10;

    for i in 0..segment_count {
        let y_offset = -(i as f32) * 0.055;
        let size = 0.035 - (i as f32 * 0.002);

        // Main zooid body - organic bell shape
        let mut zooid: UnpackedMesh = generate_sphere(size, 8, 6);
        zooid.apply(Transform::scale(1.0, 0.8, 1.0));

        // Add slight curve to chain
        let x_offset = (i as f32 * 0.15).sin() * 0.02;
        let z_offset = (i as f32 * 0.2).cos() * 0.015;
        zooid.apply(Transform::translate(x_offset, y_offset, z_offset));
        zooids.push(zooid);

        // Gastrozooids (feeding polyps) - tentacle-like
        if i % 2 == 0 {
            let mut tentacle: UnpackedMesh = generate_capsule(0.004, 0.06, 5, 3);
            for pos in &mut tentacle.positions {
                // Add wave
                let wave = (pos[1] * 20.0).sin() * 0.005;
                pos[0] += wave;
            }
            tentacle.apply(Transform::translate(x_offset, y_offset - 0.04, z_offset));
            zooids.push(tentacle);
        }

        // Dactylozooids (defensive polyps)
        if i % 3 == 0 && i > 0 {
            let mut dactyl: UnpackedMesh = generate_capsule(0.003, 0.04, 4, 2);
            dactyl.apply(Transform::rotate_z(30.0));
            dactyl.apply(Transform::translate(x_offset + 0.02, y_offset - 0.01, z_offset));
            zooids.push(dactyl);
        }

        // Connecting stem between zooids
        if i < segment_count - 1 {
            let mut stem: UnpackedMesh = generate_capsule(0.004, 0.025, 4, 2);
            stem.apply(Transform::translate(
                x_offset + ((i + 1) as f32 * 0.15).sin() * 0.01,
                y_offset - 0.04,
                z_offset
            ));
            zooids.push(stem);
        }
    }

    // Long trailing tentacles at end
    for i in 0..3 {
        let mut trail: UnpackedMesh = generate_capsule(0.002, 0.08, 4, 2);
        let angle = (i as f32 * 120.0).to_radians();
        for pos in &mut trail.positions {
            // Gentle wave
            let wave = (pos[1] * 10.0).sin() * 0.008;
            pos[0] += wave;
        }
        trail.apply(Transform::translate(angle.cos() * 0.02, -0.6, angle.sin() * 0.02));
        zooids.push(trail);
    }

    let bell_refs: Vec<&UnpackedMesh> = bells.iter().collect();
    let zooid_refs: Vec<&UnpackedMesh> = zooids.iter().collect();
    let mut parts = vec![&float];
    parts.extend(bell_refs);
    parts.extend(zooid_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "siphonophore", output_dir);
}

// === ZONE 3: MIDNIGHT ABYSS ===

/// Anglerfish - menacing deep-sea predator with lure (~350 tris)
pub fn generate_anglerfish(output_dir: &Path) {
    // Bulbous, grotesque body - organic bloated shape
    let mut body: UnpackedMesh = generate_sphere(0.14, 14, 10);
    body.apply(Transform::scale(1.0, 0.85, 0.75));
    body.apply(Subdivide { iterations: 1 });

    // Sculpt body for more organic, bloated look
    for pos in &mut body.positions {
        // Bulge in middle
        let y_factor = 1.0 + (1.0 - (pos[1].abs() / 0.12).min(1.0)) * 0.15;
        pos[0] *= y_factor;
        pos[2] *= y_factor * 0.9;
    }

    // Massive jaw - hinged open appearance
    let mut upper_jaw: UnpackedMesh = generate_sphere(0.1, 10, 6);
    upper_jaw.apply(Transform::scale(1.4, 0.5, 1.1));
    upper_jaw.apply(Transform::translate(0.14, 0.02, 0.0));

    let mut lower_jaw: UnpackedMesh = generate_sphere(0.09, 10, 6);
    lower_jaw.apply(Transform::scale(1.5, 0.4, 1.0));
    lower_jaw.apply(Transform::translate(0.16, -0.06, 0.0));

    // Fearsome teeth - curved, needle-like
    let mut teeth: Vec<UnpackedMesh> = Vec::new();
    // Upper teeth
    for i in 0..8 {
        let angle = (i as f32 - 3.5) * 18.0;
        let angle_rad = angle.to_radians();
        let size = 0.025 - (i as f32 - 4.0).abs() * 0.003;

        let mut tooth: UnpackedMesh = generate_capsule(0.004, size, 4, 2);
        tooth.apply(Transform::rotate_z(35.0 + angle_rad.sin() * 10.0));
        tooth.apply(Transform::translate(
            0.2 + angle_rad.cos() * 0.02,
            -0.01,
            angle_rad.sin() * 0.08
        ));
        teeth.push(tooth);
    }
    // Lower teeth (angled up)
    for i in 0..6 {
        let angle = (i as f32 - 2.5) * 22.0;
        let angle_rad = angle.to_radians();

        let mut tooth: UnpackedMesh = generate_capsule(0.003, 0.02, 4, 2);
        tooth.apply(Transform::rotate_z(-30.0));
        tooth.apply(Transform::translate(
            0.22 + angle_rad.cos() * 0.015,
            -0.08,
            angle_rad.sin() * 0.065
        ));
        teeth.push(tooth);
    }

    // Illicium (fishing rod) - organic curved spine
    let mut rod: UnpackedMesh = generate_capsule(0.006, 0.12, 6, 4);
    for pos in &mut rod.positions {
        // Curve forward
        let curve = pos[1] * pos[1] * 2.0;
        pos[0] += curve;
    }
    rod.apply(Transform::rotate_z(-50.0));
    rod.apply(Transform::translate(0.06, 0.11, 0.0));

    // Esca (bioluminescent lure) - organic glowing bulb
    let mut lure: UnpackedMesh = generate_sphere(0.022, 8, 6);
    lure.apply(Transform::scale(1.2, 0.9, 1.0));
    lure.apply(Transform::translate(0.17, 0.19, 0.0));

    // Lure filaments
    for i in 0..4 {
        let angle = (i as f32 * 90.0).to_radians();
        let mut filament: UnpackedMesh = generate_capsule(0.002, 0.015, 3, 2);
        filament.apply(Transform::translate(
            0.17 + angle.cos() * 0.02,
            0.19 + angle.sin() * 0.02,
            0.0
        ));
        teeth.push(filament);
    }

    // Pectoral fins - small, paddle-like
    let mut fin_l: UnpackedMesh = generate_sphere(0.025, 6, 4);
    fin_l.apply(Transform::scale(0.4, 1.2, 1.5));
    fin_l.apply(Transform::rotate_y(20.0));
    fin_l.apply(Transform::translate(-0.04, -0.02, -0.11));

    let mut fin_r: UnpackedMesh = generate_sphere(0.025, 6, 4);
    fin_r.apply(Transform::scale(0.4, 1.2, 1.5));
    fin_r.apply(Transform::rotate_y(-20.0));
    fin_r.apply(Transform::translate(-0.04, -0.02, 0.11));

    // Dorsal fin
    let mut dorsal: UnpackedMesh = generate_sphere(0.015, 5, 3);
    dorsal.apply(Transform::scale(1.0, 2.0, 0.3));
    dorsal.apply(Transform::translate(-0.08, 0.1, 0.0));

    // Small beady eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.012, 6, 5);
    eye_l.apply(Transform::translate(0.08, 0.06, -0.07));

    let mut eye_r: UnpackedMesh = generate_sphere(0.012, 6, 5);
    eye_r.apply(Transform::translate(0.08, 0.06, 0.07));

    // Tail
    let mut tail: UnpackedMesh = generate_sphere(0.025, 6, 4);
    tail.apply(Transform::scale(2.5, 0.8, 0.4));
    tail.apply(Transform::translate(-0.18, 0.0, 0.0));

    let teeth_refs: Vec<&UnpackedMesh> = teeth.iter().collect();
    let mut parts = vec![
        &body, &upper_jaw, &lower_jaw, &rod, &lure,
        &fin_l, &fin_r, &dorsal, &eye_l, &eye_r, &tail
    ];
    parts.extend(teeth_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "anglerfish", output_dir);
}

/// Gulper eel - bizarre deep-sea eel with massive mouth (~280 tris)
pub fn generate_gulper_eel(output_dir: &Path) {
    // Huge expandable pouch-like mouth
    let mut mouth: UnpackedMesh = generate_sphere(0.12, 12, 10);
    mouth.apply(Transform::scale(1.3, 0.75, 1.1));
    mouth.apply(Subdivide { iterations: 1 });

    // Sculpt mouth to be more sac-like
    for pos in &mut mouth.positions {
        // Stretch forward and down
        if pos[0] > 0.0 {
            pos[1] -= pos[0] * 0.3;
        }
        // Thin at the back
        if pos[0] < -0.05 {
            let taper = 1.0 - ((-pos[0] - 0.05) / 0.1).min(1.0) * 0.7;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Neck/throat transition
    let mut throat: UnpackedMesh = generate_capsule(0.04, 0.1, 8, 4);
    throat.apply(Transform::rotate_z(90.0));
    throat.apply(Transform::translate(-0.15, 0.0, 0.0));

    // Long snake-like body - tapered organic tube
    let mut body_segments = Vec::new();
    let segments = 8;
    for i in 0..segments {
        let x = -0.25 - (i as f32 * 0.08);
        let radius = 0.035 - (i as f32 * 0.003);

        let mut seg: UnpackedMesh = generate_sphere(radius, 8, 6);
        seg.apply(Transform::scale(1.5, 1.0, 1.0));

        // Add subtle S-curve
        let y_offset = (i as f32 * 0.4).sin() * 0.02;
        seg.apply(Transform::translate(x, y_offset, 0.0));
        body_segments.push(seg);
    }

    // Whip-like tail - extremely thin
    let mut tail_segments = Vec::new();
    for i in 0..6 {
        let x = -0.9 - (i as f32 * 0.06);
        let radius = 0.008 - (i as f32 * 0.001);

        let mut seg: UnpackedMesh = generate_sphere(radius.max(0.002), 5, 3);
        seg.apply(Transform::scale(2.0, 1.0, 1.0));

        let y_offset = ((i as f32 + 8.0) * 0.4).sin() * 0.015;
        seg.apply(Transform::translate(x, y_offset, 0.0));
        tail_segments.push(seg);
    }

    // Bioluminescent tail tip - pink/red glow organ
    let mut tail_tip: UnpackedMesh = generate_sphere(0.015, 6, 5);
    tail_tip.apply(Transform::translate(-1.26, 0.01, 0.0));

    // Tiny vestigial eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.006, 4, 3);
    eye_l.apply(Transform::translate(0.08, 0.05, -0.05));

    let mut eye_r: UnpackedMesh = generate_sphere(0.006, 4, 3);
    eye_r.apply(Transform::translate(0.08, 0.05, 0.05));

    // Small pectoral fins
    let mut fin_l: UnpackedMesh = generate_sphere(0.015, 5, 3);
    fin_l.apply(Transform::scale(0.5, 1.5, 1.2));
    fin_l.apply(Transform::translate(-0.1, 0.02, -0.05));

    let mut fin_r: UnpackedMesh = generate_sphere(0.015, 5, 3);
    fin_r.apply(Transform::scale(0.5, 1.5, 1.2));
    fin_r.apply(Transform::translate(-0.1, 0.02, 0.05));

    let body_refs: Vec<&UnpackedMesh> = body_segments.iter().collect();
    let tail_refs: Vec<&UnpackedMesh> = tail_segments.iter().collect();
    let mut parts = vec![&mouth, &throat, &tail_tip, &eye_l, &eye_r, &fin_l, &fin_r];
    parts.extend(body_refs);
    parts.extend(tail_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "gulper_eel", output_dir);
}

/// Dumbo octopus - adorable deep-sea octopus (~300 tris)
pub fn generate_dumbo_octopus(output_dir: &Path) {
    // Soft, round mantle - gelatinous appearance
    let mut mantle: UnpackedMesh = generate_sphere(0.12, 14, 10);
    mantle.apply(Transform::scale(1.0, 1.2, 0.95));
    mantle.apply(Subdivide { iterations: 1 });

    // Sculpt mantle for softer, organic shape
    for pos in &mut mantle.positions {
        // Slight forward bulge
        if pos[0] > 0.0 && pos[1] > 0.0 {
            pos[0] *= 1.1;
        }
    }

    // Large ear-like fins - characteristic feature
    let mut ear_l: UnpackedMesh = generate_sphere(0.055, 10, 6);
    ear_l.apply(Transform::scale(0.25, 1.0, 1.3));
    ear_l.apply(Subdivide { iterations: 1 });
    ear_l.apply(Transform::rotate_x(-10.0));
    ear_l.apply(Transform::translate(0.0, 0.09, -0.11));

    let mut ear_r: UnpackedMesh = generate_sphere(0.055, 10, 6);
    ear_r.apply(Transform::scale(0.25, 1.0, 1.3));
    ear_r.apply(Subdivide { iterations: 1 });
    ear_r.apply(Transform::rotate_x(10.0));
    ear_r.apply(Transform::translate(0.0, 0.09, 0.11));

    // Large, expressive eyes - forward-facing
    let mut eye_l: UnpackedMesh = generate_sphere(0.025, 8, 6);
    eye_l.apply(Transform::translate(0.07, 0.05, -0.055));

    let mut eye_r: UnpackedMesh = generate_sphere(0.025, 8, 6);
    eye_r.apply(Transform::translate(0.07, 0.05, 0.055));

    // 8 webbed arms - with cirri and connected by web
    let mut arms = Vec::new();

    // Arm web (umbrella between arms)
    let mut web: UnpackedMesh = generate_sphere(0.09, 12, 6);
    web.apply(Transform::scale(1.0, 0.3, 1.0));
    web.apply(Transform::translate(0.02, -0.08, 0.0));
    arms.push(web);

    // Individual arms with suckers
    for i in 0..8 {
        let angle = (i as f32) * 45.0;
        let angle_rad = angle.to_radians();
        let arm_length = 0.12;

        // Main arm - tapered organic shape
        let mut arm: UnpackedMesh = generate_capsule(0.015, arm_length, 6, 4);

        // Taper arm
        for pos in &mut arm.positions {
            let t = (pos[1] + arm_length / 2.0) / arm_length;
            let taper = 1.0 - t * 0.6;
            pos[0] *= taper;
            pos[2] *= taper;
        }

        // Position arm radiating outward
        arm.apply(Transform::rotate_z(40.0));
        arm.apply(Transform::rotate_y(angle));
        arm.apply(Transform::translate(
            angle_rad.cos() * 0.06,
            -0.1,
            angle_rad.sin() * 0.06
        ));
        arms.push(arm);

        // Add 3 small suckers along each arm
        for j in 0..3 {
            let sucker_t = 0.3 + j as f32 * 0.25;
            let mut sucker: UnpackedMesh = generate_sphere(0.004, 4, 3);
            sucker.apply(Transform::scale(1.0, 0.5, 1.0));

            let arm_x = angle_rad.cos() * (0.06 + sucker_t * 0.08);
            let arm_y = -0.1 - sucker_t * 0.06;
            let arm_z = angle_rad.sin() * (0.06 + sucker_t * 0.08);
            sucker.apply(Transform::translate(arm_x, arm_y - 0.01, arm_z));
            arms.push(sucker);
        }
    }

    // Siphon (funnel)
    let mut siphon: UnpackedMesh = generate_capsule(0.012, 0.03, 5, 3);
    siphon.apply(Transform::rotate_z(-30.0));
    siphon.apply(Transform::translate(0.08, -0.03, 0.0));

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![&mantle, &ear_l, &ear_r, &eye_l, &eye_r, &siphon];
    parts.extend(arm_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "dumbo_octopus", output_dir);
}

// === ZONE 4: HYDROTHERMAL VENTS ===

/// Tube worms - clustered riftia with feathery gills (~250 tris)
pub fn generate_tube_worms(output_dir: &Path) {
    let mut worms = Vec::new();

    // Cluster of tube worms with varying heights and positions
    let worm_configs: [(f32, f32, f32, f32); 7] = [
        (0.0, 0.0, 0.18, 0.018),      // Center, tall
        (0.055, 0.025, 0.15, 0.016),  // Front right
        (-0.05, 0.035, 0.16, 0.015),  // Front left
        (0.035, -0.045, 0.12, 0.014), // Back right
        (-0.04, -0.04, 0.14, 0.013),  // Back left
        (0.02, 0.055, 0.1, 0.012),    // Far front
        (-0.02, -0.06, 0.08, 0.011),  // Far back
    ];

    for (x, z, height, radius) in worm_configs {
        // White chitinous tube - organic, slightly irregular
        let mut tube: UnpackedMesh = generate_capsule(radius, height, 8, 5);

        // Add slight organic waviness to tube
        for pos in &mut tube.positions {
            let wave = (pos[1] * 25.0).sin() * 0.002;
            pos[0] += wave;
            pos[2] += (pos[1] * 20.0 + 1.0).cos() * 0.001;
        }
        tube.apply(Transform::translate(x, height / 2.0 + 0.01, z));
        worms.push(tube);

        // Collar/lip at tube opening
        let mut collar: UnpackedMesh = generate_torus(radius * 1.3, radius * 0.25, 8, 4);
        collar.apply(Transform::translate(x, height + 0.01, z));
        worms.push(collar);

        // Red feathery plume (branchiae) - multiple filaments
        let plume_filaments = 6;
        for i in 0..plume_filaments {
            let angle = (i as f32 * 360.0 / plume_filaments as f32).to_radians();
            let fil_radius = radius * 0.4;

            let mut filament: UnpackedMesh = generate_capsule(0.003, 0.025, 4, 2);

            // Curve filaments outward
            for pos in &mut filament.positions {
                let curve = (pos[1] + 0.0125) * 0.5;
                pos[0] += curve * angle.cos();
                pos[2] += curve * angle.sin();
            }

            filament.apply(Transform::translate(
                x + angle.cos() * fil_radius,
                height + 0.025,
                z + angle.sin() * fil_radius
            ));
            worms.push(filament);
        }

        // Central plume core
        let mut core: UnpackedMesh = generate_sphere(radius * 0.8, 6, 4);
        core.apply(Transform::scale(1.0, 1.5, 1.0));
        core.apply(Transform::translate(x, height + 0.02, z));
        worms.push(core);
    }

    let worm_refs: Vec<&UnpackedMesh> = worms.iter().collect();
    let mesh = smooth_combine(&worm_refs);
    write_mesh(&mesh, "tube_worms", output_dir);
}

/// Vent shrimp - eyeless chemosynthetic crustacean (~120 tris)
pub fn generate_vent_shrimp(output_dir: &Path) {
    // Segmented carapace - organic curved shape
    let mut carapace: UnpackedMesh = generate_sphere(0.015, 10, 6);
    carapace.apply(Transform::scale(2.0, 0.8, 1.0));
    carapace.apply(Subdivide { iterations: 1 });

    // Taper toward tail
    for pos in &mut carapace.positions {
        if pos[0] < 0.0 {
            let taper = 1.0 - (-pos[0] / 0.03).min(1.0) * 0.3;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Abdominal segments
    let mut segments = Vec::new();
    for i in 0..4 {
        let x = -0.03 - (i as f32 * 0.012);
        let size = 0.008 - (i as f32 * 0.001);

        let mut seg: UnpackedMesh = generate_sphere(size, 6, 4);
        seg.apply(Transform::scale(1.2, 0.9, 1.0));
        seg.apply(Transform::translate(x, -0.002, 0.0));
        segments.push(seg);
    }

    // Head region (no visible eyes - vent shrimp are blind)
    let mut head: UnpackedMesh = generate_sphere(0.01, 8, 5);
    head.apply(Transform::scale(1.3, 0.9, 0.95));
    head.apply(Transform::translate(0.028, 0.003, 0.0));

    // Rostrum (pointed snout)
    let mut rostrum: UnpackedMesh = generate_capsule(0.003, 0.012, 4, 2);
    rostrum.apply(Transform::rotate_z(75.0));
    rostrum.apply(Transform::translate(0.038, 0.006, 0.0));

    // Long antennae - sensory organs for finding vents
    let mut antenna_l: UnpackedMesh = generate_capsule(0.0015, 0.04, 4, 2);
    for pos in &mut antenna_l.positions {
        // Gentle curve
        let curve = (pos[1] + 0.02) * 0.3;
        pos[0] += curve;
    }
    antenna_l.apply(Transform::rotate_z(-40.0));
    antenna_l.apply(Transform::translate(0.035, 0.01, -0.006));

    let mut antenna_r: UnpackedMesh = generate_capsule(0.0015, 0.04, 4, 2);
    for pos in &mut antenna_r.positions {
        let curve = (pos[1] + 0.02) * 0.3;
        pos[0] += curve;
    }
    antenna_r.apply(Transform::rotate_z(-40.0));
    antenna_r.apply(Transform::translate(0.035, 0.01, 0.006));

    // Walking legs (5 pairs, but we'll do 3 visible)
    let mut legs = Vec::new();
    for i in 0..3 {
        let x = 0.01 - (i as f32 * 0.01);

        let mut leg_l: UnpackedMesh = generate_capsule(0.002, 0.02, 3, 2);
        leg_l.apply(Transform::rotate_z(70.0));
        leg_l.apply(Transform::translate(x, -0.008, -0.012));
        legs.push(leg_l);

        let mut leg_r: UnpackedMesh = generate_capsule(0.002, 0.02, 3, 2);
        leg_r.apply(Transform::rotate_z(70.0));
        leg_r.apply(Transform::translate(x, -0.008, 0.012));
        legs.push(leg_r);
    }

    // Tail fan (telson and uropods)
    let mut telson: UnpackedMesh = generate_sphere(0.008, 6, 4);
    telson.apply(Transform::scale(1.8, 0.3, 1.2));
    telson.apply(Transform::translate(-0.055, -0.002, 0.0));

    let mut uropod_l: UnpackedMesh = generate_sphere(0.006, 4, 3);
    uropod_l.apply(Transform::scale(1.5, 0.25, 1.0));
    uropod_l.apply(Transform::rotate_y(-25.0));
    uropod_l.apply(Transform::translate(-0.058, -0.003, -0.008));

    let mut uropod_r: UnpackedMesh = generate_sphere(0.006, 4, 3);
    uropod_r.apply(Transform::scale(1.5, 0.25, 1.0));
    uropod_r.apply(Transform::rotate_y(25.0));
    uropod_r.apply(Transform::translate(-0.058, -0.003, 0.008));

    let seg_refs: Vec<&UnpackedMesh> = segments.iter().collect();
    let leg_refs: Vec<&UnpackedMesh> = legs.iter().collect();
    let mut parts = vec![
        &carapace, &head, &rostrum, &antenna_l, &antenna_r,
        &telson, &uropod_l, &uropod_r
    ];
    parts.extend(seg_refs);
    parts.extend(leg_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "vent_shrimp", output_dir);
}

// === ADDITIONAL ZONE 1 ===

/// Coral crab - colorful reef crab with large claws (~180 tris)
pub fn generate_coral_crab(output_dir: &Path) {
    // Rounded carapace - organic shell shape
    let mut carapace: UnpackedMesh = generate_sphere(0.045, 10, 8);
    carapace.apply(Transform::scale(1.4, 0.6, 1.2));
    carapace.apply(Subdivide { iterations: 1 });

    // Sculpt carapace for natural shape
    for pos in &mut carapace.positions {
        // Flatten bottom
        if pos[1] < 0.0 {
            pos[1] *= 0.3;
        }
        // Slight frontal taper
        if pos[0] > 0.02 {
            let taper = 1.0 - ((pos[0] - 0.02) / 0.04).min(1.0) * 0.2;
            pos[2] *= taper;
        }
    }
    carapace.apply(Transform::translate(0.0, 0.025, 0.0));

    // Eye stalks - organic curved shape
    let mut stalk_l: UnpackedMesh = generate_capsule(0.005, 0.025, 5, 3);
    stalk_l.apply(Transform::rotate_z(-15.0));
    stalk_l.apply(Transform::translate(0.035, 0.04, -0.018));

    let mut stalk_r: UnpackedMesh = generate_capsule(0.005, 0.025, 5, 3);
    stalk_r.apply(Transform::rotate_z(-15.0));
    stalk_r.apply(Transform::translate(0.035, 0.04, 0.018));

    // Eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.008, 6, 5);
    eye_l.apply(Transform::translate(0.038, 0.062, -0.02));

    let mut eye_r: UnpackedMesh = generate_sphere(0.008, 6, 5);
    eye_r.apply(Transform::translate(0.038, 0.062, 0.02));

    // Large chelipeds (claws) - asymmetric like many crabs
    // Left claw (larger crusher)
    let mut claw_l_merus: UnpackedMesh = generate_capsule(0.015, 0.035, 5, 3);
    claw_l_merus.apply(Transform::rotate_z(-50.0));
    claw_l_merus.apply(Transform::translate(0.03, 0.015, -0.055));

    let mut claw_l_propodus: UnpackedMesh = generate_sphere(0.025, 8, 5);
    claw_l_propodus.apply(Transform::scale(1.6, 0.7, 0.8));
    claw_l_propodus.apply(Transform::translate(0.055, 0.035, -0.055));

    let mut claw_l_dactyl: UnpackedMesh = generate_capsule(0.006, 0.018, 4, 2);
    claw_l_dactyl.apply(Transform::rotate_z(20.0));
    claw_l_dactyl.apply(Transform::translate(0.075, 0.04, -0.055));

    // Right claw (smaller cutter)
    let mut claw_r_merus: UnpackedMesh = generate_capsule(0.012, 0.03, 5, 3);
    claw_r_merus.apply(Transform::rotate_z(-50.0));
    claw_r_merus.apply(Transform::translate(0.03, 0.015, 0.055));

    let mut claw_r_propodus: UnpackedMesh = generate_sphere(0.018, 6, 4);
    claw_r_propodus.apply(Transform::scale(1.5, 0.65, 0.75));
    claw_r_propodus.apply(Transform::translate(0.048, 0.032, 0.055));

    let mut claw_r_dactyl: UnpackedMesh = generate_capsule(0.004, 0.015, 4, 2);
    claw_r_dactyl.apply(Transform::rotate_z(15.0));
    claw_r_dactyl.apply(Transform::translate(0.065, 0.036, 0.055));

    // Walking legs (4 pairs)
    let mut legs = Vec::new();
    for i in 0..4 {
        let z_offset = -0.025 + (i as f32 * 0.018);
        let x_base = -0.01 - (i as f32 * 0.008);

        // Left leg
        let mut leg_l: UnpackedMesh = generate_capsule(0.005, 0.035, 4, 2);
        leg_l.apply(Transform::rotate_z(65.0));
        leg_l.apply(Transform::rotate_y(-20.0 - i as f32 * 5.0));
        leg_l.apply(Transform::translate(x_base, 0.005, z_offset - 0.045));
        legs.push(leg_l);

        // Leg tip (dactyl)
        let mut tip_l: UnpackedMesh = generate_capsule(0.003, 0.015, 3, 2);
        tip_l.apply(Transform::rotate_z(45.0));
        tip_l.apply(Transform::translate(x_base - 0.02, -0.015, z_offset - 0.06));
        legs.push(tip_l);

        // Right leg
        let mut leg_r: UnpackedMesh = generate_capsule(0.005, 0.035, 4, 2);
        leg_r.apply(Transform::rotate_z(65.0));
        leg_r.apply(Transform::rotate_y(20.0 + i as f32 * 5.0));
        leg_r.apply(Transform::translate(x_base, 0.005, z_offset + 0.045));
        legs.push(leg_r);

        let mut tip_r: UnpackedMesh = generate_capsule(0.003, 0.015, 3, 2);
        tip_r.apply(Transform::rotate_z(45.0));
        tip_r.apply(Transform::translate(x_base - 0.02, -0.015, z_offset + 0.06));
        legs.push(tip_r);
    }

    // Antennae
    let mut antenna_l: UnpackedMesh = generate_capsule(0.002, 0.02, 3, 2);
    antenna_l.apply(Transform::rotate_z(-30.0));
    antenna_l.apply(Transform::translate(0.05, 0.035, -0.01));

    let mut antenna_r: UnpackedMesh = generate_capsule(0.002, 0.02, 3, 2);
    antenna_r.apply(Transform::rotate_z(-30.0));
    antenna_r.apply(Transform::translate(0.05, 0.035, 0.01));

    let leg_refs: Vec<&UnpackedMesh> = legs.iter().collect();
    let mut parts = vec![
        &carapace, &stalk_l, &stalk_r, &eye_l, &eye_r,
        &claw_l_merus, &claw_l_propodus, &claw_l_dactyl,
        &claw_r_merus, &claw_r_propodus, &claw_r_dactyl,
        &antenna_l, &antenna_r
    ];
    parts.extend(leg_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "coral_crab", output_dir);
}

// === ADDITIONAL ZONE 2 ===

/// Giant squid - massive cephalopod with huge eyes (~500 tris)
pub fn generate_giant_squid(output_dir: &Path) {
    // Torpedo-shaped mantle - streamlined and organic
    let mut mantle: UnpackedMesh = generate_sphere(0.18, 14, 10);
    mantle.apply(Transform::scale(2.2, 0.75, 0.7));
    mantle.apply(Subdivide { iterations: 1 });

    // Taper mantle toward rear
    for pos in &mut mantle.positions {
        if pos[0] < 0.0 {
            let t = (-pos[0] / 0.35).min(1.0);
            let taper = 1.0 - t * 0.4;
            pos[1] *= taper;
            pos[2] *= taper;
        }
        // Slight bulge in front third
        if pos[0] > 0.15 {
            let bulge = 1.0 + ((pos[0] - 0.15) / 0.2).min(1.0) * 0.1;
            pos[1] *= bulge;
            pos[2] *= bulge;
        }
    }

    // Triangular fins at rear - elegant shape
    let mut fin_l: UnpackedMesh = generate_sphere(0.1, 10, 6);
    fin_l.apply(Transform::scale(0.4, 1.2, 1.8));
    fin_l.apply(Subdivide { iterations: 1 });
    fin_l.apply(Transform::translate(-0.32, 0.0, -0.12));

    let mut fin_r: UnpackedMesh = generate_sphere(0.1, 10, 6);
    fin_r.apply(Transform::scale(0.4, 1.2, 1.8));
    fin_r.apply(Subdivide { iterations: 1 });
    fin_r.apply(Transform::translate(-0.32, 0.0, 0.12));

    // Head/arms base - transition from mantle
    let mut head: UnpackedMesh = generate_sphere(0.12, 12, 8);
    head.apply(Transform::scale(1.1, 0.9, 0.85));
    head.apply(Transform::translate(0.38, 0.0, 0.0));

    // Massive eyes - largest in animal kingdom
    let mut eye_l: UnpackedMesh = generate_sphere(0.055, 10, 8);
    eye_l.apply(Transform::translate(0.35, 0.04, -0.1));

    let mut eye_r: UnpackedMesh = generate_sphere(0.055, 10, 8);
    eye_r.apply(Transform::translate(0.35, 0.04, 0.1));

    // Beak area
    let mut beak: UnpackedMesh = generate_sphere(0.03, 6, 5);
    beak.apply(Transform::scale(1.0, 0.6, 0.8));
    beak.apply(Transform::translate(0.48, -0.02, 0.0));

    // 8 arms - tapered with suckers
    let mut arms = Vec::new();
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let arm_length = 0.28;

        let mut arm: UnpackedMesh = generate_capsule(0.025, arm_length, 8, 5);

        // Taper and curve arm
        for pos in &mut arm.positions {
            let t = (pos[1] + arm_length / 2.0) / arm_length;
            let taper = 1.0 - t * 0.7;
            pos[0] *= taper;
            pos[2] *= taper;
            // Slight outward curve
            pos[0] += t * t * 0.03 * angle.cos();
            pos[2] += t * t * 0.03 * angle.sin();
        }

        arm.apply(Transform::rotate_z(-25.0));
        arm.apply(Transform::rotate_y(angle.to_degrees()));
        arm.apply(Transform::translate(
            0.5 + angle.cos() * 0.04,
            angle.sin() * 0.06 - 0.02,
            angle.cos() * 0.06
        ));
        arms.push(arm);

        // Suckers along arm (simplified as bumps)
        for j in 0..4 {
            let sucker_t = 0.2 + j as f32 * 0.2;
            let mut sucker: UnpackedMesh = generate_sphere(0.008, 4, 3);
            sucker.apply(Transform::scale(1.0, 0.4, 1.0));
            let arm_x = 0.5 + angle.cos() * 0.04 + sucker_t * 0.15;
            let arm_y = angle.sin() * 0.06 - 0.02 - sucker_t * 0.1;
            sucker.apply(Transform::translate(arm_x, arm_y - 0.015, angle.cos() * 0.06));
            arms.push(sucker);
        }
    }

    // 2 long feeding tentacles
    let mut tentacle_parts = Vec::new();

    for side in [-1.0_f32, 1.0] {
        // Tentacle stalk - long and thin
        let mut stalk: UnpackedMesh = generate_capsule(0.015, 0.5, 8, 4);
        for pos in &mut stalk.positions {
            // Taper and curve
            let t = (pos[1] + 0.25) / 0.5;
            pos[0] *= 1.0 - t * 0.5;
            pos[2] *= 1.0 - t * 0.5;
            // Gentle curve outward
            pos[2] += t * t * 0.05 * side;
        }
        stalk.apply(Transform::rotate_z(-20.0));
        stalk.apply(Transform::translate(0.52, -0.02, side * 0.04));
        tentacle_parts.push(stalk);

        // Club (manus) at end - paddle-shaped
        let mut club: UnpackedMesh = generate_sphere(0.04, 8, 5);
        club.apply(Transform::scale(2.0, 0.5, 0.9));
        club.apply(Transform::translate(0.95, -0.2 + side * 0.02, side * 0.08));
        tentacle_parts.push(club);
    }

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let tent_refs: Vec<&UnpackedMesh> = tentacle_parts.iter().collect();
    let mut parts = vec![
        &mantle, &fin_l, &fin_r, &head, &eye_l, &eye_r, &beak
    ];
    parts.extend(arm_refs);
    parts.extend(tent_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "giant_squid", output_dir);
}

// === ADDITIONAL ZONE 3 ===

/// Vampire squid - living fossil with webbed arms (~350 tris)
pub fn generate_vampire_squid(output_dir: &Path) {
    // Round, gelatinous mantle
    let mut mantle: UnpackedMesh = generate_sphere(0.1, 12, 10);
    mantle.apply(Transform::scale(1.15, 1.0, 0.95));
    mantle.apply(Subdivide { iterations: 1 });

    // Large ear-like fins (different position than dumbo)
    let mut fin_l: UnpackedMesh = generate_sphere(0.045, 8, 5);
    fin_l.apply(Transform::scale(0.25, 1.0, 1.3));
    fin_l.apply(Transform::rotate_x(-5.0));
    fin_l.apply(Transform::translate(-0.04, 0.07, -0.09));

    let mut fin_r: UnpackedMesh = generate_sphere(0.045, 8, 5);
    fin_r.apply(Transform::scale(0.25, 1.0, 1.3));
    fin_r.apply(Transform::rotate_x(5.0));
    fin_r.apply(Transform::translate(-0.04, 0.07, 0.09));

    // Enormous eyes (largest eye-to-body ratio)
    let mut eye_l: UnpackedMesh = generate_sphere(0.032, 8, 6);
    eye_l.apply(Transform::translate(0.07, 0.025, -0.055));

    let mut eye_r: UnpackedMesh = generate_sphere(0.032, 8, 6);
    eye_r.apply(Transform::translate(0.07, 0.025, 0.055));

    // Web between arms - umbrella-like membrane
    let mut web: UnpackedMesh = generate_sphere(0.11, 14, 8);
    web.apply(Transform::scale(1.0, 0.25, 1.0));
    // Sculpt web to be more umbrella-like
    for pos in &mut web.positions {
        if pos[1] > 0.0 {
            pos[1] *= 0.3; // Flatten top
        }
        // Scalloped edge
        let angle = pos[2].atan2(pos[0]);
        let scallop = 1.0 + (angle * 8.0).sin().abs() * 0.1;
        if pos[1] < -0.01 {
            pos[0] *= scallop;
            pos[2] *= scallop;
        }
    }
    web.apply(Transform::translate(0.08, -0.06, 0.0));

    // 8 arms with cirri (fleshy spines)
    let mut arms = Vec::new();
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let arm_length = 0.16;

        // Main arm
        let mut arm: UnpackedMesh = generate_capsule(0.012, arm_length, 6, 4);

        // Taper and add subtle curve
        for pos in &mut arm.positions {
            let t = (pos[1] + arm_length / 2.0) / arm_length;
            let taper = 1.0 - t * 0.6;
            pos[0] *= taper;
            pos[2] *= taper;
        }

        arm.apply(Transform::rotate_z(35.0));
        arm.apply(Transform::rotate_y(angle.to_degrees()));
        arm.apply(Transform::translate(
            0.1 + angle.cos() * 0.05,
            -0.06,
            angle.sin() * 0.05
        ));
        arms.push(arm);

        // Cirri (fleshy spines) along arm - unique to vampire squid
        for j in 0..3 {
            let cirrus_t = 0.3 + j as f32 * 0.25;
            let mut cirrus: UnpackedMesh = generate_capsule(0.003, 0.025, 3, 2);
            cirrus.apply(Transform::rotate_z(70.0));
            cirrus.apply(Transform::rotate_y(angle.to_degrees() + 10.0));

            let arm_x = 0.1 + angle.cos() * 0.05 + cirrus_t * 0.1;
            let arm_y = -0.06 - cirrus_t * 0.08;
            cirrus.apply(Transform::translate(arm_x, arm_y, angle.sin() * 0.05));
            arms.push(cirrus);
        }
    }

    // Two velar filaments (retractable sensory tentacles)
    let mut filament_l: UnpackedMesh = generate_capsule(0.003, 0.12, 5, 3);
    for pos in &mut filament_l.positions {
        let curve = (pos[1] + 0.06) * 0.15;
        pos[0] += curve;
    }
    filament_l.apply(Transform::rotate_z(-20.0));
    filament_l.apply(Transform::translate(0.12, -0.04, -0.03));

    let mut filament_r: UnpackedMesh = generate_capsule(0.003, 0.12, 5, 3);
    for pos in &mut filament_r.positions {
        let curve = (pos[1] + 0.06) * 0.15;
        pos[0] += curve;
    }
    filament_r.apply(Transform::rotate_z(-20.0));
    filament_r.apply(Transform::translate(0.12, -0.04, 0.03));

    // Photophores at arm tips
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let mut photo: UnpackedMesh = generate_sphere(0.006, 4, 3);
        photo.apply(Transform::translate(
            0.1 + angle.cos() * 0.05 + 0.12,
            -0.06 - 0.1,
            angle.sin() * 0.05
        ));
        arms.push(photo);
    }

    // Mantle photophores
    let mut photo_top: UnpackedMesh = generate_sphere(0.007, 4, 3);
    photo_top.apply(Transform::translate(-0.02, 0.085, 0.0));

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![
        &mantle, &fin_l, &fin_r, &eye_l, &eye_r, &web,
        &filament_l, &filament_r, &photo_top
    ];
    parts.extend(arm_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "vampire_squid", output_dir);
}

// === ADDITIONAL ZONE 4 ===

/// Ghost fish - ethereal translucent deep-sea fish (~100 tris)
pub fn generate_ghost_fish(output_dir: &Path) {
    // Elongated, delicate body
    let mut body: UnpackedMesh = generate_sphere(0.025, 10, 8);
    body.apply(Transform::scale(2.5, 0.9, 0.45));
    body.apply(Subdivide { iterations: 1 });

    // Taper toward both ends
    for pos in &mut body.positions {
        let dist = pos[0].abs();
        if dist > 0.03 {
            let taper = 1.0 - ((dist - 0.03) / 0.04).min(1.0) * 0.4;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Translucent head region
    let mut head: UnpackedMesh = generate_sphere(0.015, 6, 5);
    head.apply(Transform::scale(1.3, 1.0, 0.9));
    head.apply(Transform::translate(0.055, 0.003, 0.0));

    // Large, ghostly eyes (adapted to extreme darkness)
    let mut eye_l: UnpackedMesh = generate_sphere(0.009, 6, 4);
    eye_l.apply(Transform::translate(0.055, 0.012, -0.012));

    let mut eye_r: UnpackedMesh = generate_sphere(0.009, 6, 4);
    eye_r.apply(Transform::translate(0.055, 0.012, 0.012));

    // Delicate dorsal fin - almost transparent
    let mut dorsal: UnpackedMesh = generate_sphere(0.015, 6, 4);
    dorsal.apply(Transform::scale(2.5, 1.8, 0.08));
    dorsal.apply(Transform::translate(0.0, 0.022, 0.0));

    // Ethereal tail fin
    let mut tail: UnpackedMesh = generate_sphere(0.012, 5, 3);
    tail.apply(Transform::scale(1.5, 2.0, 0.1));
    tail.apply(Transform::translate(-0.065, 0.0, 0.0));

    // Pectoral fins - wispy
    let mut pec_l: UnpackedMesh = generate_sphere(0.008, 4, 3);
    pec_l.apply(Transform::scale(1.0, 1.5, 2.0));
    pec_l.apply(Transform::translate(0.02, 0.0, -0.018));

    let mut pec_r: UnpackedMesh = generate_sphere(0.008, 4, 3);
    pec_r.apply(Transform::scale(1.0, 1.5, 2.0));
    pec_r.apply(Transform::translate(0.02, 0.0, 0.018));

    // Anal fin
    let mut anal: UnpackedMesh = generate_sphere(0.01, 4, 3);
    anal.apply(Transform::scale(1.8, 1.2, 0.08));
    anal.apply(Transform::translate(-0.03, -0.015, 0.0));

    let mesh = smooth_combine(&[
        &body, &head, &eye_l, &eye_r, &dorsal, &tail,
        &pec_l, &pec_r, &anal
    ]);
    write_mesh(&mesh, "ghost_fish", output_dir);
}

/// Vent octopus - pale chemosynthetic octopus (~280 tris)
pub fn generate_vent_octopus(output_dir: &Path) {
    // Compact, flattened mantle
    let mut mantle: UnpackedMesh = generate_sphere(0.08, 12, 8);
    mantle.apply(Transform::scale(1.1, 0.75, 0.95));
    mantle.apply(Subdivide { iterations: 1 });

    // Head/arm base transition
    let mut head: UnpackedMesh = generate_sphere(0.04, 8, 6);
    head.apply(Transform::scale(1.2, 0.9, 0.95));
    head.apply(Transform::translate(0.065, 0.015, 0.0));

    // Eyes - adapted to darkness near vents
    let mut eye_l: UnpackedMesh = generate_sphere(0.014, 6, 5);
    eye_l.apply(Transform::translate(0.075, 0.03, -0.028));

    let mut eye_r: UnpackedMesh = generate_sphere(0.014, 6, 5);
    eye_r.apply(Transform::translate(0.075, 0.03, 0.028));

    // 8 long, slender arms
    let mut arms = Vec::new();
    for i in 0..8 {
        let angle = (i as f32 * 45.0).to_radians();
        let arm_length = 0.18;

        let mut arm: UnpackedMesh = generate_capsule(0.012, arm_length, 6, 4);

        // Taper arm significantly
        for pos in &mut arm.positions {
            let t = (pos[1] + arm_length / 2.0) / arm_length;
            let taper = 1.0 - t * 0.75;
            pos[0] *= taper;
            pos[2] *= taper;
            // Add slight curl
            let curl = t * t * 0.02;
            pos[0] += curl * angle.cos();
            pos[2] += curl * angle.sin();
        }

        arm.apply(Transform::rotate_z(40.0));
        arm.apply(Transform::rotate_y(angle.to_degrees()));
        arm.apply(Transform::translate(
            0.055 + angle.cos() * 0.035,
            -0.04,
            angle.sin() * 0.035
        ));
        arms.push(arm);

        // Suckers (small)
        for j in 0..4 {
            let sucker_t = 0.2 + j as f32 * 0.2;
            let mut sucker: UnpackedMesh = generate_sphere(0.004, 4, 3);
            sucker.apply(Transform::scale(1.0, 0.4, 1.0));

            let arm_x = 0.055 + angle.cos() * 0.035 + sucker_t * 0.1;
            let arm_y = -0.04 - sucker_t * 0.08;
            let arm_z = angle.sin() * (0.035 + sucker_t * 0.05);
            sucker.apply(Transform::translate(arm_x, arm_y - 0.008, arm_z));
            arms.push(sucker);
        }

        // Bioluminescent arm tip
        let mut tip: UnpackedMesh = generate_sphere(0.005, 4, 3);
        tip.apply(Transform::translate(
            0.055 + angle.cos() * 0.035 + 0.13,
            -0.04 - 0.12,
            angle.sin() * 0.07
        ));
        arms.push(tip);
    }

    // Siphon
    let mut siphon: UnpackedMesh = generate_capsule(0.01, 0.025, 5, 3);
    siphon.apply(Transform::rotate_z(-25.0));
    siphon.apply(Transform::translate(0.07, -0.02, 0.0));

    // Inter-arm web (shallow)
    let mut web: UnpackedMesh = generate_sphere(0.06, 10, 5);
    web.apply(Transform::scale(1.0, 0.2, 1.0));
    web.apply(Transform::translate(0.055, -0.04, 0.0));

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![&mantle, &head, &eye_l, &eye_r, &siphon, &web];
    parts.extend(arm_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "vent_octopus", output_dir);
}

// === EPIC ENCOUNTERS ===

/// Blue whale - majestic ocean giant (~900 tris)
pub fn generate_blue_whale(output_dir: &Path) {
    // Massive streamlined body - organic torpedo shape
    let mut body: UnpackedMesh = generate_sphere(0.45, 18, 12);
    body.apply(Transform::scale(3.5, 0.7, 0.65));
    body.apply(Subdivide { iterations: 1 });

    // Sculpt body for whale proportions
    for pos in &mut body.positions {
        // Taper toward tail
        if pos[0] < 0.0 {
            let t = (-pos[0] / 1.5).min(1.0);
            let taper = 1.0 - t * 0.6;
            pos[1] *= taper;
            pos[2] *= taper;
        }
        // Slight head bulge
        if pos[0] > 0.8 {
            let bulge = 1.0 + ((pos[0] - 0.8) / 0.5).min(1.0) * 0.15;
            pos[1] *= bulge;
            pos[2] *= bulge;
        }
    }

    // Broad, U-shaped head
    let mut head: UnpackedMesh = generate_sphere(0.38, 14, 10);
    head.apply(Transform::scale(1.3, 0.85, 0.95));
    head.apply(Subdivide { iterations: 1 });
    head.apply(Transform::translate(1.2, 0.05, 0.0));

    // Rostrum (upper jaw ridge)
    let mut rostrum: UnpackedMesh = generate_sphere(0.12, 8, 5);
    rostrum.apply(Transform::scale(2.5, 0.4, 0.7));
    rostrum.apply(Transform::translate(1.55, 0.12, 0.0));

    // Splashguard around blowhole
    let mut splashguard: UnpackedMesh = generate_sphere(0.06, 6, 4);
    splashguard.apply(Transform::scale(1.5, 0.8, 1.0));
    splashguard.apply(Transform::translate(1.1, 0.3, 0.0));

    // Long pectoral flippers
    let mut flipper_l: UnpackedMesh = generate_sphere(0.1, 10, 6);
    flipper_l.apply(Transform::scale(3.5, 0.25, 1.2));
    flipper_l.apply(Subdivide { iterations: 1 });
    flipper_l.apply(Transform::rotate_z(-15.0));
    flipper_l.apply(Transform::rotate_y(-10.0));
    flipper_l.apply(Transform::translate(0.4, -0.12, -0.4));

    let mut flipper_r: UnpackedMesh = generate_sphere(0.1, 10, 6);
    flipper_r.apply(Transform::scale(3.5, 0.25, 1.2));
    flipper_r.apply(Subdivide { iterations: 1 });
    flipper_r.apply(Transform::rotate_z(-15.0));
    flipper_r.apply(Transform::rotate_y(10.0));
    flipper_r.apply(Transform::translate(0.4, -0.12, 0.4));

    // Small dorsal fin (distinctive blue whale feature)
    let mut dorsal: UnpackedMesh = generate_sphere(0.06, 6, 4);
    dorsal.apply(Transform::scale(1.5, 1.2, 0.4));
    dorsal.apply(Transform::translate(-0.9, 0.28, 0.0));

    // Peduncle (tail stock)
    let mut peduncle: UnpackedMesh = generate_capsule(0.1, 0.35, 8, 5);
    peduncle.apply(Transform::rotate_z(90.0));
    peduncle.apply(Transform::translate(-1.35, 0.0, 0.0));

    // Massive tail flukes
    let mut fluke_l: UnpackedMesh = generate_sphere(0.25, 10, 6);
    fluke_l.apply(Transform::scale(1.8, 0.12, 1.3));
    fluke_l.apply(Subdivide { iterations: 1 });
    fluke_l.apply(Transform::rotate_y(25.0));
    fluke_l.apply(Transform::translate(-1.6, 0.0, -0.22));

    let mut fluke_r: UnpackedMesh = generate_sphere(0.25, 10, 6);
    fluke_r.apply(Transform::scale(1.8, 0.12, 1.3));
    fluke_r.apply(Subdivide { iterations: 1 });
    fluke_r.apply(Transform::rotate_y(-25.0));
    fluke_r.apply(Transform::translate(-1.6, 0.0, 0.22));

    // Notch between flukes
    let mut notch: UnpackedMesh = generate_sphere(0.03, 5, 3);
    notch.apply(Transform::scale(1.5, 0.5, 0.8));
    notch.apply(Transform::translate(-1.7, 0.0, 0.0));

    // Eyes (small relative to body)
    let mut eye_l: UnpackedMesh = generate_sphere(0.04, 6, 5);
    eye_l.apply(Transform::translate(1.0, 0.12, -0.32));

    let mut eye_r: UnpackedMesh = generate_sphere(0.04, 6, 5);
    eye_r.apply(Transform::translate(1.0, 0.12, 0.32));

    // Throat grooves/pleats (ventral)
    let mut grooves = Vec::new();
    for i in 0..5 {
        let z = -0.15 + (i as f32 * 0.075);
        let mut groove: UnpackedMesh = generate_capsule(0.015, 0.9, 6, 3);
        groove.apply(Transform::rotate_z(90.0));
        groove.apply(Transform::translate(0.5, -0.2, z));
        grooves.push(groove);
    }

    let groove_refs: Vec<&UnpackedMesh> = grooves.iter().collect();
    let mut parts = vec![
        &body, &head, &rostrum, &splashguard,
        &flipper_l, &flipper_r, &dorsal, &peduncle,
        &fluke_l, &fluke_r, &notch, &eye_l, &eye_r
    ];
    parts.extend(groove_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "blue_whale", output_dir);
}

/// Sperm whale - deep-diving leviathan with massive head (~800 tris)
pub fn generate_sperm_whale(output_dir: &Path) {
    // Distinctive rectangular head (spermaceti organ) - 1/3 of body
    let mut head: UnpackedMesh = generate_sphere(0.4, 14, 10);
    head.apply(Transform::scale(1.5, 0.9, 0.85));
    head.apply(Subdivide { iterations: 1 });

    // Sculpt head for distinctive boxy shape
    for pos in &mut head.positions {
        // Flatten top
        if pos[1] > 0.2 {
            pos[1] = 0.2 + (pos[1] - 0.2) * 0.4;
        }
        // Square off front
        if pos[0] > 0.3 {
            let t = ((pos[0] - 0.3) / 0.3).min(1.0);
            pos[1] *= 1.0 - t * 0.2;
            pos[2] *= 1.0 - t * 0.15;
        }
    }
    head.apply(Transform::translate(0.85, 0.05, 0.0));

    // Blunt snout
    let mut snout: UnpackedMesh = generate_sphere(0.28, 10, 8);
    snout.apply(Transform::scale(0.8, 0.7, 0.75));
    snout.apply(Transform::translate(1.35, 0.0, 0.0));

    // Streamlined body - tapers significantly toward tail
    let mut body: UnpackedMesh = generate_sphere(0.3, 14, 10);
    body.apply(Transform::scale(2.5, 0.75, 0.7));
    body.apply(Subdivide { iterations: 1 });

    // Taper body
    for pos in &mut body.positions {
        if pos[0] < 0.0 {
            let t = (-pos[0] / 0.7).min(1.0);
            let taper = 1.0 - t * 0.65;
            pos[1] *= taper;
            pos[2] *= taper;
        }
    }

    // Dorsal hump (not a true fin)
    let mut hump: UnpackedMesh = generate_sphere(0.08, 6, 4);
    hump.apply(Transform::scale(2.0, 1.0, 0.6));
    hump.apply(Transform::translate(-0.3, 0.22, 0.0));

    // Series of knuckles behind dorsal hump
    let mut knuckles = Vec::new();
    for i in 0..4 {
        let mut knuckle: UnpackedMesh = generate_sphere(0.04 - i as f32 * 0.008, 5, 3);
        knuckle.apply(Transform::translate(-0.5 - i as f32 * 0.12, 0.18 - i as f32 * 0.02, 0.0));
        knuckles.push(knuckle);
    }

    // Peduncle
    let mut peduncle: UnpackedMesh = generate_capsule(0.08, 0.35, 8, 5);
    peduncle.apply(Transform::rotate_z(90.0));
    peduncle.apply(Transform::translate(-1.1, 0.0, 0.0));

    // Tail flukes - triangular
    let mut fluke_l: UnpackedMesh = generate_sphere(0.22, 8, 5);
    fluke_l.apply(Transform::scale(1.6, 0.1, 1.2));
    fluke_l.apply(Transform::rotate_y(28.0));
    fluke_l.apply(Transform::translate(-1.4, 0.0, -0.18));

    let mut fluke_r: UnpackedMesh = generate_sphere(0.22, 8, 5);
    fluke_r.apply(Transform::scale(1.6, 0.1, 1.2));
    fluke_r.apply(Transform::rotate_y(-28.0));
    fluke_r.apply(Transform::translate(-1.4, 0.0, 0.18));

    // Small pectoral flippers
    let mut flipper_l: UnpackedMesh = generate_sphere(0.06, 8, 5);
    flipper_l.apply(Transform::scale(2.5, 0.25, 1.0));
    flipper_l.apply(Transform::rotate_z(-20.0));
    flipper_l.apply(Transform::translate(0.35, -0.12, -0.28));

    let mut flipper_r: UnpackedMesh = generate_sphere(0.06, 8, 5);
    flipper_r.apply(Transform::scale(2.5, 0.25, 1.0));
    flipper_r.apply(Transform::rotate_z(-20.0));
    flipper_r.apply(Transform::translate(0.35, -0.12, 0.28));

    // Lower jaw - narrow, underslung
    let mut jaw: UnpackedMesh = generate_capsule(0.06, 0.45, 8, 4);
    jaw.apply(Transform::rotate_z(90.0));
    jaw.apply(Transform::translate(1.05, -0.2, 0.0));

    // Jaw tip
    let mut jaw_tip: UnpackedMesh = generate_sphere(0.05, 5, 4);
    jaw_tip.apply(Transform::scale(1.2, 0.6, 0.8));
    jaw_tip.apply(Transform::translate(1.32, -0.22, 0.0));

    // Small eye (relative to head)
    let mut eye_l: UnpackedMesh = generate_sphere(0.03, 5, 4);
    eye_l.apply(Transform::translate(0.6, 0.08, -0.32));

    let mut eye_r: UnpackedMesh = generate_sphere(0.03, 5, 4);
    eye_r.apply(Transform::translate(0.6, 0.08, 0.32));

    // Blowhole (offset to left - distinctive feature)
    let mut blowhole: UnpackedMesh = generate_sphere(0.025, 5, 4);
    blowhole.apply(Transform::scale(1.5, 0.5, 1.0));
    blowhole.apply(Transform::translate(1.25, 0.2, -0.08));

    // Wrinkled skin texture (subtle ridges)
    let mut wrinkles = Vec::new();
    for i in 0..3 {
        let mut wrinkle: UnpackedMesh = generate_capsule(0.02, 0.4, 5, 3);
        wrinkle.apply(Transform::rotate_z(85.0 + i as f32 * 3.0));
        wrinkle.apply(Transform::translate(0.8 - i as f32 * 0.25, 0.22, 0.0));
        wrinkles.push(wrinkle);
    }

    let knuckle_refs: Vec<&UnpackedMesh> = knuckles.iter().collect();
    let wrinkle_refs: Vec<&UnpackedMesh> = wrinkles.iter().collect();
    let mut parts = vec![
        &head, &snout, &body, &hump, &peduncle,
        &fluke_l, &fluke_r, &flipper_l, &flipper_r,
        &jaw, &jaw_tip, &eye_l, &eye_r, &blowhole
    ];
    parts.extend(knuckle_refs);
    parts.extend(wrinkle_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "sperm_whale", output_dir);
}

/// Giant isopod - armored deep-sea scavenger (~400 tris)
pub fn generate_giant_isopod(output_dir: &Path) {
    // Domed, segmented body
    let mut main_body: UnpackedMesh = generate_sphere(0.07, 14, 10);
    main_body.apply(Transform::scale(1.6, 0.5, 1.0));
    main_body.apply(Subdivide { iterations: 1 });

    // Sculpt body for segmented appearance
    for pos in &mut main_body.positions {
        // Add segment ridges
        let ridge = 1.0 + ((pos[0] * 25.0).sin().abs()) * 0.05;
        if pos[1] > 0.0 {
            pos[1] *= ridge;
        }
        // Flatten bottom
        if pos[1] < 0.0 {
            pos[1] *= 0.4;
        }
        // Taper toward tail
        if pos[0] < -0.05 {
            let t = ((-pos[0] - 0.05) / 0.08).min(1.0);
            pos[2] *= 1.0 - t * 0.3;
        }
    }

    // Head shield (cephalon) - armored plate
    let mut head: UnpackedMesh = generate_sphere(0.045, 10, 8);
    head.apply(Transform::scale(1.3, 0.55, 1.0));
    head.apply(Subdivide { iterations: 1 });
    head.apply(Transform::translate(0.1, 0.005, 0.0));

    // Large sessile compound eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.015, 8, 6);
    eye_l.apply(Transform::scale(1.0, 0.7, 1.2));
    eye_l.apply(Transform::translate(0.12, 0.02, -0.035));

    let mut eye_r: UnpackedMesh = generate_sphere(0.015, 8, 6);
    eye_r.apply(Transform::scale(1.0, 0.7, 1.2));
    eye_r.apply(Transform::translate(0.12, 0.02, 0.035));

    // Two pairs of antennae
    let mut appendages = Vec::new();

    // First antennae (shorter)
    let mut ant1_l: UnpackedMesh = generate_capsule(0.003, 0.04, 5, 3);
    for pos in &mut ant1_l.positions {
        let curve = (pos[1] + 0.02) * 0.2;
        pos[0] += curve;
    }
    ant1_l.apply(Transform::rotate_z(-35.0));
    ant1_l.apply(Transform::rotate_y(-25.0));
    ant1_l.apply(Transform::translate(0.14, 0.015, -0.015));
    appendages.push(ant1_l);

    let mut ant1_r: UnpackedMesh = generate_capsule(0.003, 0.04, 5, 3);
    for pos in &mut ant1_r.positions {
        let curve = (pos[1] + 0.02) * 0.2;
        pos[0] += curve;
    }
    ant1_r.apply(Transform::rotate_z(-35.0));
    ant1_r.apply(Transform::rotate_y(25.0));
    ant1_r.apply(Transform::translate(0.14, 0.015, 0.015));
    appendages.push(ant1_r);

    // Second antennae (longer)
    let mut ant2_l: UnpackedMesh = generate_capsule(0.0025, 0.06, 5, 3);
    for pos in &mut ant2_l.positions {
        let curve = (pos[1] + 0.03) * 0.25;
        pos[0] += curve;
    }
    ant2_l.apply(Transform::rotate_z(-45.0));
    ant2_l.apply(Transform::rotate_y(-35.0));
    ant2_l.apply(Transform::translate(0.135, 0.01, -0.02));
    appendages.push(ant2_l);

    let mut ant2_r: UnpackedMesh = generate_capsule(0.0025, 0.06, 5, 3);
    for pos in &mut ant2_r.positions {
        let curve = (pos[1] + 0.03) * 0.25;
        pos[0] += curve;
    }
    ant2_r.apply(Transform::rotate_z(-45.0));
    ant2_r.apply(Transform::rotate_y(35.0));
    ant2_r.apply(Transform::translate(0.135, 0.01, 0.02));
    appendages.push(ant2_r);

    // 7 pairs of walking legs (pereopods) - jointed
    for i in 0..7 {
        let x_pos = 0.05 - (i as f32 * 0.018);
        let leg_length = 0.035 + (i as f32 * 0.002);

        // Coxa (base segment)
        let mut coxa_l: UnpackedMesh = generate_capsule(0.006, 0.015, 4, 2);
        coxa_l.apply(Transform::rotate_z(75.0));
        coxa_l.apply(Transform::translate(x_pos, -0.008, -0.06));
        appendages.push(coxa_l);

        // Leg shaft
        let mut leg_l: UnpackedMesh = generate_capsule(0.004, leg_length, 4, 2);
        leg_l.apply(Transform::rotate_z(55.0));
        leg_l.apply(Transform::rotate_y(-15.0 - i as f32 * 3.0));
        leg_l.apply(Transform::translate(x_pos, -0.015, -0.07));
        appendages.push(leg_l);

        // Dactyl (claw tip)
        let mut dactyl_l: UnpackedMesh = generate_capsule(0.0025, 0.012, 3, 2);
        dactyl_l.apply(Transform::rotate_z(35.0));
        dactyl_l.apply(Transform::translate(x_pos - 0.015, -0.03, -0.08));
        appendages.push(dactyl_l);

        // Right side
        let mut coxa_r: UnpackedMesh = generate_capsule(0.006, 0.015, 4, 2);
        coxa_r.apply(Transform::rotate_z(75.0));
        coxa_r.apply(Transform::translate(x_pos, -0.008, 0.06));
        appendages.push(coxa_r);

        let mut leg_r: UnpackedMesh = generate_capsule(0.004, leg_length, 4, 2);
        leg_r.apply(Transform::rotate_z(55.0));
        leg_r.apply(Transform::rotate_y(15.0 + i as f32 * 3.0));
        leg_r.apply(Transform::translate(x_pos, -0.015, 0.07));
        appendages.push(leg_r);

        let mut dactyl_r: UnpackedMesh = generate_capsule(0.0025, 0.012, 3, 2);
        dactyl_r.apply(Transform::rotate_z(35.0));
        dactyl_r.apply(Transform::translate(x_pos - 0.015, -0.03, 0.08));
        appendages.push(dactyl_r);
    }

    // Pleon (tail section) - 5 segments fused
    let mut pleon: UnpackedMesh = generate_sphere(0.035, 8, 6);
    pleon.apply(Transform::scale(1.8, 0.45, 1.1));
    pleon.apply(Transform::translate(-0.1, 0.0, 0.0));

    // Telson (tail plate) - broad and rounded
    let mut telson: UnpackedMesh = generate_sphere(0.035, 8, 5);
    telson.apply(Transform::scale(1.2, 0.35, 1.3));
    telson.apply(Transform::translate(-0.14, -0.005, 0.0));

    // Uropods (tail paddles)
    let mut uropod_l: UnpackedMesh = generate_sphere(0.02, 6, 4);
    uropod_l.apply(Transform::scale(1.8, 0.25, 1.0));
    uropod_l.apply(Transform::rotate_y(-20.0));
    uropod_l.apply(Transform::translate(-0.155, -0.008, -0.035));

    let mut uropod_r: UnpackedMesh = generate_sphere(0.02, 6, 4);
    uropod_r.apply(Transform::scale(1.8, 0.25, 1.0));
    uropod_r.apply(Transform::rotate_y(20.0));
    uropod_r.apply(Transform::translate(-0.155, -0.008, 0.035));

    let appendage_refs: Vec<&UnpackedMesh> = appendages.iter().collect();
    let mut parts = vec![
        &main_body, &head, &eye_l, &eye_r,
        &pleon, &telson, &uropod_l, &uropod_r
    ];
    parts.extend(appendage_refs);

    let mesh = smooth_combine(&parts);
    write_mesh(&mesh, "giant_isopod", output_dir);
}

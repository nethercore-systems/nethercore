//! Submersible (player vessel) generator
//!
//! The player controls a small research submersible - their window into the deep.
//! Detailed mechanical design with:
//! - Streamlined hull with panel lines
//! - Large glass observation dome with reinforced frame
//! - Dual headlight array (emissive)
//! - Twin rear thrusters with gimbals
//! - Maneuvering thrusters (attitude control)
//! - Sensor suite (antenna, sonar dome)
//! - Manipulator arm mounts
//! - External equipment pods
//! - Ballast tanks with detail
//! - Stabilizer fins
//! - ~600-700 tris

use super::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

// Helper for smooth mechanical assembly
fn smooth_combine(parts: &[&UnpackedMesh]) -> UnpackedMesh {
    let mut result = combine(parts);
    result.apply(SmoothNormals { weld_threshold: 0.008 });
    result
}

pub fn generate_submersible(output_dir: &Path) {
    // === MAIN HULL ===
    // Streamlined capsule shape with refined proportions
    let mut hull: UnpackedMesh = generate_capsule(0.28, 0.55, 16, 8);
    hull.apply(Transform::rotate_z(90.0));
    hull.apply(Subdivide { iterations: 1 });

    // Shape hull for hydrodynamic profile
    for pos in &mut hull.positions {
        let x = pos[0];

        // Slight taper toward rear
        if x < -0.1 {
            let taper = 1.0 - ((-x - 0.1) / 0.4).min(1.0) * 0.12;
            pos[1] *= taper;
            pos[2] *= taper;
        }

        // Subtle forward bulge (equipment bay)
        if x > 0.1 && x < 0.35 {
            let bulge = 1.0 + (1.0 - ((x - 0.225) / 0.125).abs()) * 0.05;
            pos[1] *= bulge;
        }

        // Flatten bottom slightly (landing skid area)
        if pos[1] < -0.15 {
            pos[1] = pos[1].max(-0.18);
        }
    }
    hull.apply(Transform::scale(1.0, 0.82, 0.82));

    // === OBSERVATION DOME ===
    // Large glass bubble with reinforced frame
    let mut dome: UnpackedMesh = generate_sphere(0.24, 16, 12);
    dome.apply(Transform::scale(0.75, 0.8, 0.88));
    dome.apply(Subdivide { iterations: 1 });
    dome.apply(Transform::translate(0.38, 0.05, 0.0));

    // Dome frame (reinforcement ring)
    let mut dome_frame: UnpackedMesh = generate_torus(0.19, 0.018, 16, 6);
    dome_frame.apply(Transform::rotate_z(90.0));
    dome_frame.apply(Transform::translate(0.32, 0.05, 0.0));

    // Secondary frame (horizontal)
    let mut dome_band: UnpackedMesh = generate_torus(0.165, 0.012, 14, 4);
    dome_band.apply(Transform::translate(0.38, 0.05, 0.0));

    // === HEADLIGHT ARRAY ===
    // Dual headlights with housing
    let mut light_housing_l: UnpackedMesh = generate_cylinder(0.055, 0.05, 0.065, 10);
    light_housing_l.apply(Transform::rotate_z(90.0));
    light_housing_l.apply(Subdivide { iterations: 1 });
    light_housing_l.apply(Transform::translate(0.42, -0.06, -0.08));

    let mut light_housing_r: UnpackedMesh = generate_cylinder(0.055, 0.05, 0.065, 10);
    light_housing_r.apply(Transform::rotate_z(90.0));
    light_housing_r.apply(Subdivide { iterations: 1 });
    light_housing_r.apply(Transform::translate(0.42, -0.06, 0.08));

    // Headlight lenses (emissive when on)
    let mut lens_l: UnpackedMesh = generate_sphere(0.042, 10, 8);
    lens_l.apply(Transform::scale(0.6, 1.0, 1.0));
    lens_l.apply(Transform::translate(0.455, -0.06, -0.08));

    let mut lens_r: UnpackedMesh = generate_sphere(0.042, 10, 8);
    lens_r.apply(Transform::scale(0.6, 1.0, 1.0));
    lens_r.apply(Transform::translate(0.455, -0.06, 0.08));

    // Light bezels
    let mut bezel_l: UnpackedMesh = generate_torus(0.04, 0.008, 10, 4);
    bezel_l.apply(Transform::rotate_z(90.0));
    bezel_l.apply(Transform::translate(0.456, -0.06, -0.08));

    let mut bezel_r: UnpackedMesh = generate_torus(0.04, 0.008, 10, 4);
    bezel_r.apply(Transform::rotate_z(90.0));
    bezel_r.apply(Transform::translate(0.456, -0.06, 0.08));

    // === MAIN THRUSTERS ===
    // Twin rear propulsion units with nacelles
    let mut nacelle_l: UnpackedMesh = generate_capsule(0.065, 0.18, 10, 5);
    nacelle_l.apply(Transform::rotate_z(90.0));
    nacelle_l.apply(Subdivide { iterations: 1 });
    nacelle_l.apply(Transform::translate(-0.42, 0.0, -0.16));

    let mut nacelle_r: UnpackedMesh = generate_capsule(0.065, 0.18, 10, 5);
    nacelle_r.apply(Transform::rotate_z(90.0));
    nacelle_r.apply(Subdivide { iterations: 1 });
    nacelle_r.apply(Transform::translate(-0.42, 0.0, 0.16));

    // Thruster nozzles (emissive glow when moving)
    let mut nozzle_l: UnpackedMesh = generate_torus(0.055, 0.018, 12, 6);
    nozzle_l.apply(Transform::rotate_z(90.0));
    nozzle_l.apply(Transform::translate(-0.52, 0.0, -0.16));

    let mut nozzle_r: UnpackedMesh = generate_torus(0.055, 0.018, 12, 6);
    nozzle_r.apply(Transform::rotate_z(90.0));
    nozzle_r.apply(Transform::translate(-0.52, 0.0, 0.16));

    // Inner nozzle cones
    let mut cone_l: UnpackedMesh = generate_cylinder(0.045, 0.025, 0.04, 8);
    cone_l.apply(Transform::rotate_z(90.0));
    cone_l.apply(Transform::translate(-0.54, 0.0, -0.16));

    let mut cone_r: UnpackedMesh = generate_cylinder(0.045, 0.025, 0.04, 8);
    cone_r.apply(Transform::rotate_z(90.0));
    cone_r.apply(Transform::translate(-0.54, 0.0, 0.16));

    // Thruster pylons (connecting to hull)
    let mut pylon_l: UnpackedMesh = generate_cube(0.06, 0.035, 0.045);
    pylon_l.apply(Subdivide { iterations: 1 });
    pylon_l.apply(Transform::translate(-0.35, 0.0, -0.12));

    let mut pylon_r: UnpackedMesh = generate_cube(0.06, 0.035, 0.045);
    pylon_r.apply(Subdivide { iterations: 1 });
    pylon_r.apply(Transform::translate(-0.35, 0.0, 0.12));

    // === MANEUVERING THRUSTERS ===
    // Small attitude control jets
    let mut thruster_top: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.03, 6);
    thruster_top.apply(Transform::translate(0.0, 0.2, 0.0));

    let mut thruster_bottom: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.03, 6);
    thruster_bottom.apply(Transform::rotate_x(180.0));
    thruster_bottom.apply(Transform::translate(0.0, -0.18, 0.0));

    // === SENSOR SUITE ===
    // Top antenna mast with communication array
    let mut antenna_mast: UnpackedMesh = generate_cylinder(0.012, 0.008, 0.16, 6);
    antenna_mast.apply(Subdivide { iterations: 1 });
    antenna_mast.apply(Transform::translate(0.05, 0.28, 0.0));

    // Antenna dish (communication/GPS)
    let mut antenna_dish: UnpackedMesh = generate_sphere(0.028, 8, 6);
    antenna_dish.apply(Transform::scale(1.2, 0.4, 1.2));
    antenna_dish.apply(Transform::translate(0.05, 0.36, 0.0));

    // Sonar dome (forward-looking)
    let mut sonar: UnpackedMesh = generate_sphere(0.045, 10, 8);
    sonar.apply(Transform::scale(1.0, 0.6, 1.0));
    sonar.apply(Transform::translate(0.32, -0.12, 0.0));

    // Side sensor pods
    let mut sensor_l: UnpackedMesh = generate_cylinder(0.018, 0.018, 0.04, 6);
    sensor_l.apply(Transform::rotate_z(90.0));
    sensor_l.apply(Transform::translate(0.25, 0.0, -0.2));

    let mut sensor_r: UnpackedMesh = generate_cylinder(0.018, 0.018, 0.04, 6);
    sensor_r.apply(Transform::rotate_z(90.0));
    sensor_r.apply(Transform::translate(0.25, 0.0, 0.2));

    // === MANIPULATOR ARM MOUNTS ===
    // Robotic arm base (retracted position)
    let mut arm_mount_l: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.05, 8);
    arm_mount_l.apply(Transform::rotate_x(45.0));
    arm_mount_l.apply(Transform::translate(0.2, -0.1, -0.15));

    let mut arm_mount_r: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.05, 8);
    arm_mount_r.apply(Transform::rotate_x(-45.0));
    arm_mount_r.apply(Transform::translate(0.2, -0.1, 0.15));

    // Arm joint spheres
    let mut arm_joint_l: UnpackedMesh = generate_sphere(0.022, 6, 4);
    arm_joint_l.apply(Transform::translate(0.22, -0.13, -0.17));

    let mut arm_joint_r: UnpackedMesh = generate_sphere(0.022, 6, 4);
    arm_joint_r.apply(Transform::translate(0.22, -0.13, 0.17));

    // === STABILIZER FINS ===
    // Vertical stabilizer (dorsal)
    let mut fin_dorsal: UnpackedMesh = generate_cube(0.1, 0.075, 0.008);
    fin_dorsal.apply(Subdivide { iterations: 1 });
    // Taper the fin
    for pos in &mut fin_dorsal.positions {
        let x = pos[0];
        if x < 0.0 {
            let taper = 1.0 + x * 1.2;
            pos[1] *= taper.max(0.4);
        }
    }
    fin_dorsal.apply(Transform::translate(-0.28, 0.2, 0.0));

    // Horizontal stabilizers (port/starboard)
    let mut fin_l: UnpackedMesh = generate_cube(0.09, 0.008, 0.065);
    fin_l.apply(Subdivide { iterations: 1 });
    for pos in &mut fin_l.positions {
        let x = pos[0];
        if x < 0.0 {
            let taper = 1.0 + x * 1.0;
            pos[2] *= taper.max(0.5);
        }
    }
    fin_l.apply(Transform::translate(-0.28, 0.0, -0.2));

    let mut fin_r: UnpackedMesh = generate_cube(0.09, 0.008, 0.065);
    fin_r.apply(Subdivide { iterations: 1 });
    for pos in &mut fin_r.positions {
        let x = pos[0];
        if x < 0.0 {
            let taper = 1.0 + x * 1.0;
            pos[2] *= taper.max(0.5);
        }
    }
    fin_r.apply(Transform::translate(-0.28, 0.0, 0.2));

    // === BALLAST TANKS ===
    // Side-mounted ballast/buoyancy pods
    let mut ballast_l: UnpackedMesh = generate_capsule(0.045, 0.22, 10, 5);
    ballast_l.apply(Transform::rotate_z(90.0));
    ballast_l.apply(Subdivide { iterations: 1 });
    ballast_l.apply(Transform::translate(-0.02, -0.13, -0.2));

    let mut ballast_r: UnpackedMesh = generate_capsule(0.045, 0.22, 10, 5);
    ballast_r.apply(Transform::rotate_z(90.0));
    ballast_r.apply(Subdivide { iterations: 1 });
    ballast_r.apply(Transform::translate(-0.02, -0.13, 0.2));

    // Ballast tank end caps (detail)
    let mut cap_lf: UnpackedMesh = generate_sphere(0.035, 6, 4);
    cap_lf.apply(Transform::scale(0.5, 1.0, 1.0));
    cap_lf.apply(Transform::translate(0.1, -0.13, -0.2));

    let mut cap_rf: UnpackedMesh = generate_sphere(0.035, 6, 4);
    cap_rf.apply(Transform::scale(0.5, 1.0, 1.0));
    cap_rf.apply(Transform::translate(0.1, -0.13, 0.2));

    // === EQUIPMENT PODS ===
    // Sample container pod (belly)
    let mut sample_pod: UnpackedMesh = generate_capsule(0.035, 0.12, 8, 4);
    sample_pod.apply(Transform::rotate_z(90.0));
    sample_pod.apply(Transform::translate(0.08, -0.16, 0.0));

    // === LANDING SKIDS ===
    // Simple landing struts
    let mut skid_l: UnpackedMesh = generate_cylinder(0.012, 0.012, 0.15, 6);
    skid_l.apply(Transform::rotate_z(90.0));
    skid_l.apply(Transform::translate(0.0, -0.19, -0.1));

    let mut skid_r: UnpackedMesh = generate_cylinder(0.012, 0.012, 0.15, 6);
    skid_r.apply(Transform::rotate_z(90.0));
    skid_r.apply(Transform::translate(0.0, -0.19, 0.1));

    // Skid pads
    let mut pad_lf: UnpackedMesh = generate_sphere(0.018, 6, 4);
    pad_lf.apply(Transform::scale(1.0, 0.5, 1.0));
    pad_lf.apply(Transform::translate(0.08, -0.19, -0.1));

    let mut pad_lr: UnpackedMesh = generate_sphere(0.018, 6, 4);
    pad_lr.apply(Transform::scale(1.0, 0.5, 1.0));
    pad_lr.apply(Transform::translate(-0.08, -0.19, -0.1));

    let mut pad_rf: UnpackedMesh = generate_sphere(0.018, 6, 4);
    pad_rf.apply(Transform::scale(1.0, 0.5, 1.0));
    pad_rf.apply(Transform::translate(0.08, -0.19, 0.1));

    let mut pad_rr: UnpackedMesh = generate_sphere(0.018, 6, 4);
    pad_rr.apply(Transform::scale(1.0, 0.5, 1.0));
    pad_rr.apply(Transform::translate(-0.08, -0.19, 0.1));

    // === VIEWPORT/PORTHOLE ===
    // Small side viewing ports
    let mut porthole_l: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.015, 8);
    porthole_l.apply(Transform::rotate_x(90.0));
    porthole_l.apply(Transform::translate(0.15, 0.02, -0.19));

    let mut porthole_r: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.015, 8);
    porthole_r.apply(Transform::rotate_x(-90.0));
    porthole_r.apply(Transform::translate(0.15, 0.02, 0.19));

    // Porthole frames
    let mut port_frame_l: UnpackedMesh = generate_torus(0.028, 0.006, 10, 4);
    port_frame_l.apply(Transform::rotate_x(90.0));
    port_frame_l.apply(Transform::translate(0.15, 0.02, -0.2));

    let mut port_frame_r: UnpackedMesh = generate_torus(0.028, 0.006, 10, 4);
    port_frame_r.apply(Transform::rotate_x(-90.0));
    port_frame_r.apply(Transform::translate(0.15, 0.02, 0.2));

    // Combine all parts with smooth normals for polished look
    write_mesh(
        &smooth_combine(&[
            // Main body
            &hull,
            // Observation dome
            &dome,
            &dome_frame,
            &dome_band,
            // Headlights
            &light_housing_l,
            &light_housing_r,
            &lens_l,
            &lens_r,
            &bezel_l,
            &bezel_r,
            // Main thrusters
            &nacelle_l,
            &nacelle_r,
            &nozzle_l,
            &nozzle_r,
            &cone_l,
            &cone_r,
            &pylon_l,
            &pylon_r,
            // Maneuvering
            &thruster_top,
            &thruster_bottom,
            // Sensors
            &antenna_mast,
            &antenna_dish,
            &sonar,
            &sensor_l,
            &sensor_r,
            // Manipulator arms
            &arm_mount_l,
            &arm_mount_r,
            &arm_joint_l,
            &arm_joint_r,
            // Fins
            &fin_dorsal,
            &fin_l,
            &fin_r,
            // Ballast
            &ballast_l,
            &ballast_r,
            &cap_lf,
            &cap_rf,
            // Equipment
            &sample_pod,
            // Landing gear
            &skid_l,
            &skid_r,
            &pad_lf,
            &pad_lr,
            &pad_rf,
            &pad_rr,
            // Portholes
            &porthole_l,
            &porthole_r,
            &port_frame_l,
            &port_frame_r,
        ]),
        "submersible",
        output_dir,
    );
}

//! Simple test to verify OBJ export and modifiers work

use proc_gen::mesh::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    println!("Testing proc-gen crate...\n");

    // Test 1: Basic OBJ export
    println!("1. Testing basic OBJ export...");
    let sphere: UnpackedMesh = generate_sphere(1.0, 16, 8);
    write_obj(&sphere, Path::new("test_sphere.obj"), "sphere")?;
    println!("   ✓ Exported sphere: {} vertices, {} triangles",
        sphere.vertex_count(), sphere.triangle_count());

    // Test 2: Transform modifier
    println!("\n2. Testing Transform modifier...");
    let mut cube: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
    Transform::scale(2.0, 1.0, 1.0).apply(&mut cube);
    write_obj(&cube, Path::new("test_cube_scaled.obj"), "cube_scaled")?;
    println!("   ✓ Scaled cube exported");

    // Test 3: Mirror modifier
    println!("\n3. Testing Mirror modifier...");
    let mut capsule: UnpackedMesh = generate_capsule(0.4, 0.6, 8, 4);
    Mirror {
        axis: Axis::X,
        merge_threshold: 0.001,
        flip_u: false,
        flip_v: false,
    }.apply(&mut capsule);
    write_obj(&capsule, Path::new("test_capsule_mirrored.obj"), "capsule_mirrored")?;
    println!("   ✓ Mirrored capsule: {} vertices, {} triangles",
        capsule.vertex_count(), capsule.triangle_count());

    // Test 4: SmoothNormals vs FlatNormals
    println!("\n4. Testing SmoothNormals...");
    let mut sphere_smooth: UnpackedMesh = generate_sphere(1.0, 8, 4);
    SmoothNormals::default().apply(&mut sphere_smooth);
    write_obj(&sphere_smooth, Path::new("test_sphere_smooth.obj"), "sphere_smooth")?;
    println!("   ✓ Smooth sphere exported");

    println!("\n5. Testing FlatNormals...");
    let mut sphere_flat: UnpackedMesh = generate_sphere(1.0, 8, 4);
    FlatNormals.apply(&mut sphere_flat);
    write_obj(&sphere_flat, Path::new("test_sphere_flat.obj"), "sphere_flat")?;
    println!("   ✓ Flat sphere: {} vertices (was {})",
        sphere_flat.vertex_count(),
        generate_sphere::<UnpackedMesh>(1.0, 8, 4).vertex_count());

    // Test 5: Combine meshes
    println!("\n6. Testing combine...");
    let c1: UnpackedMesh = generate_cube(0.5, 0.5, 0.5);
    let c2: UnpackedMesh = generate_cube(0.3, 0.3, 0.3);
    let c3: UnpackedMesh = generate_cube(0.2, 0.2, 0.2);
    let combined = combine(&[&c1, &c2, &c3]);
    write_obj(&combined, Path::new("test_combined.obj"), "combined")?;
    println!("   ✓ Combined 3 cubes: {} vertices, {} triangles",
        combined.vertex_count(), combined.triangle_count());

    // Test 6: Complex modifier chain
    println!("\n7. Testing modifier chain...");
    let mut mesh: UnpackedMesh = generate_torus(1.0, 0.3, 16, 8);
    Transform::scale(1.0, 1.2, 1.0).apply(&mut mesh);
    Mirror { axis: Axis::Y, ..Default::default() }.apply(&mut mesh);
    SmoothNormals::default().apply(&mut mesh);
    write_obj(&mesh, Path::new("test_complex.obj"), "complex")?;
    println!("   ✓ Complex mesh: {} vertices, {} triangles",
        mesh.vertex_count(), mesh.triangle_count());

    println!("\n✓ All tests completed successfully!");
    println!("\nGenerated OBJ files:");
    println!("  - test_sphere.obj");
    println!("  - test_cube_scaled.obj");
    println!("  - test_capsule_mirrored.obj");
    println!("  - test_sphere_smooth.obj");
    println!("  - test_sphere_flat.obj");
    println!("  - test_combined.obj");
    println!("  - test_complex.obj");

    Ok(())
}

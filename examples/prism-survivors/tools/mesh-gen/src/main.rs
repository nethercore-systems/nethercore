//! PRISM SURVIVORS - Procedural Asset Generator
//!
//! Generates all meshes, textures, and audio for the game.
//! Run with: cargo run -p mesh-gen
//!
//! Output goes to: assets/meshes/, assets/textures/, assets/audio/

mod obj;
mod png_writer;

use std::path::Path;

fn main() {
    println!("=== PRISM SURVIVORS Asset Generator ===\n");

    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let meshes_dir = base_path.join("assets/meshes");
    let textures_dir = base_path.join("assets/textures");
    let _audio_dir = base_path.join("assets/audio");

    // Ensure directories exist
    std::fs::create_dir_all(&meshes_dir).unwrap();
    std::fs::create_dir_all(&textures_dir).unwrap();

    println!("Generating meshes to: {}", meshes_dir.display());
    println!("Generating textures to: {}\n", textures_dir.display());

    // === TEST ASSETS ===
    generate_test_cube(&meshes_dir);
    generate_test_texture(&textures_dir);

    // === HEROES ===
    // TODO: generate_knight(&meshes_dir);
    // TODO: generate_mage(&meshes_dir);
    // TODO: generate_ranger(&meshes_dir);
    // TODO: generate_cleric(&meshes_dir);

    // === ENEMIES ===
    // TODO: generate_golem(&meshes_dir);
    // TODO: generate_crawler(&meshes_dir);
    // TODO: generate_wisp(&meshes_dir);
    // TODO: generate_skeleton(&meshes_dir);

    println!("\n=== Generation Complete ===");
}

/// Generate a simple test cube to verify the pipeline works
fn generate_test_cube(output_dir: &Path) {
    use obj::ObjWriter;

    println!("Generating: test_cube.obj");

    let mut obj = ObjWriter::new();

    // Simple cube vertices (8 corners)
    let size = 0.5;
    let verts = [
        // Front face
        [-size, -size,  size],
        [ size, -size,  size],
        [ size,  size,  size],
        [-size,  size,  size],
        // Back face
        [-size, -size, -size],
        [ size, -size, -size],
        [ size,  size, -size],
        [-size,  size, -size],
    ];

    // Add vertices
    for v in &verts {
        obj.vertex(v[0], v[1], v[2]);
    }

    // Add UVs (simple planar mapping)
    obj.uv(0.0, 0.0);
    obj.uv(1.0, 0.0);
    obj.uv(1.0, 1.0);
    obj.uv(0.0, 1.0);

    // Add normals for each face direction
    obj.normal(0.0, 0.0, 1.0);   // Front
    obj.normal(0.0, 0.0, -1.0);  // Back
    obj.normal(1.0, 0.0, 0.0);   // Right
    obj.normal(-1.0, 0.0, 0.0);  // Left
    obj.normal(0.0, 1.0, 0.0);   // Top
    obj.normal(0.0, -1.0, 0.0);  // Bottom

    // Faces (1-indexed in OBJ format)
    // Front face (normal 1)
    obj.face_vtn(1, 1, 1);
    obj.face_vtn(2, 2, 1);
    obj.face_vtn(3, 3, 1);
    obj.face_vtn(1, 1, 1);
    obj.face_vtn(3, 3, 1);
    obj.face_vtn(4, 4, 1);

    // Back face (normal 2)
    obj.face_vtn(6, 1, 2);
    obj.face_vtn(5, 2, 2);
    obj.face_vtn(8, 3, 2);
    obj.face_vtn(6, 1, 2);
    obj.face_vtn(8, 3, 2);
    obj.face_vtn(7, 4, 2);

    // Right face (normal 3)
    obj.face_vtn(2, 1, 3);
    obj.face_vtn(6, 2, 3);
    obj.face_vtn(7, 3, 3);
    obj.face_vtn(2, 1, 3);
    obj.face_vtn(7, 3, 3);
    obj.face_vtn(3, 4, 3);

    // Left face (normal 4)
    obj.face_vtn(5, 1, 4);
    obj.face_vtn(1, 2, 4);
    obj.face_vtn(4, 3, 4);
    obj.face_vtn(5, 1, 4);
    obj.face_vtn(4, 3, 4);
    obj.face_vtn(8, 4, 4);

    // Top face (normal 5)
    obj.face_vtn(4, 1, 5);
    obj.face_vtn(3, 2, 5);
    obj.face_vtn(7, 3, 5);
    obj.face_vtn(4, 1, 5);
    obj.face_vtn(7, 3, 5);
    obj.face_vtn(8, 4, 5);

    // Bottom face (normal 6)
    obj.face_vtn(5, 1, 6);
    obj.face_vtn(6, 2, 6);
    obj.face_vtn(2, 3, 6);
    obj.face_vtn(5, 1, 6);
    obj.face_vtn(2, 3, 6);
    obj.face_vtn(1, 4, 6);

    let path = output_dir.join("test_cube.obj");
    obj.write_to_file(&path).expect("Failed to write OBJ file");
    println!("  -> Written: {}", path.display());
}

/// Generate a simple test texture (checkerboard pattern)
fn generate_test_texture(output_dir: &Path) {
    use png_writer::write_png;

    println!("Generating: test_checker.png");

    let size = 64;
    let mut pixels = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let checker = ((x / 8) + (y / 8)) % 2 == 0;
            let color = if checker { 200 } else { 50 };
            pixels[idx] = color;     // R
            pixels[idx + 1] = color; // G
            pixels[idx + 2] = color; // B
            pixels[idx + 3] = 255;   // A
        }
    }

    let path = output_dir.join("test_checker.png");
    write_png(&path, size as u32, size as u32, &pixels).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());
}

//! Integration test for matrix packing system
//!
//! Tests the full flow of matrix packing across multiple frames to ensure
//! indices stay synchronized between model matrices and draw commands.

use emberware_z::graphics::MvpIndex;
use emberware_z::graphics::VirtualRenderPass;
use emberware_z::state::ZFFIState;
use glam::Mat4;

/// Simulate a single frame of the render pipeline
#[test]
fn test_matrix_packing_single_frame() {
    let mut z_state = ZFFIState::default();

    // Populate view/proj matrices (what app.rs does)
    z_state.view_matrices.push(Mat4::IDENTITY);
    z_state.proj_matrices.push(Mat4::IDENTITY);
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    // Simulate game adding model matrices and commands
    let model_a = Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
    let model_b = Mat4::from_translation(glam::Vec3::new(2.0, 0.0, 0.0));

    let idx_a = z_state.add_model_matrix(model_a);
    let idx_b = z_state.add_model_matrix(model_b);

    assert_eq!(idx_a, 0, "First model matrix should be at index 0");
    assert_eq!(idx_b, 1, "Second model matrix should be at index 1");

    // Record commands
    z_state.current_model_idx = idx_a;
    let mvp_a = z_state.pack_current_mvp();
    z_state.current_model_idx = idx_b;
    let mvp_b = z_state.pack_current_mvp();

    // Verify MVP indices
    let (model_idx, view_idx, proj_idx) = mvp_a.unpack();
    assert_eq!(model_idx, 0);
    assert_eq!(view_idx, 0);
    assert_eq!(proj_idx, 0);

    let (model_idx, view_idx, proj_idx) = mvp_b.unpack();
    assert_eq!(model_idx, 1);
    assert_eq!(view_idx, 0);
    assert_eq!(proj_idx, 0);

    // Verify model matrices are at correct indices
    assert_eq!(z_state.model_matrices[0], model_a);
    assert_eq!(z_state.model_matrices[1], model_b);
}

/// Simulate multiple frames to test clearing and index reuse
#[test]
fn test_matrix_packing_multiple_frames() {
    let mut z_state = ZFFIState::default();

    // --- FRAME 1 ---
    println!("=== Frame 1 ===");

    // Populate view/proj
    z_state.view_matrices.push(Mat4::IDENTITY);
    z_state.proj_matrices.push(Mat4::IDENTITY);
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    // Add model matrices
    let model1_a = Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
    let model1_b = Mat4::from_translation(glam::Vec3::new(2.0, 0.0, 0.0));

    let idx1_a = z_state.add_model_matrix(model1_a);
    let idx1_b = z_state.add_model_matrix(model1_b);

    println!(
        "Frame 1 - Added matrices at indices: {}, {}",
        idx1_a, idx1_b
    );
    assert_eq!(idx1_a, 0, "Frame 1: First matrix should be at index 0");
    assert_eq!(idx1_b, 1, "Frame 1: Second matrix should be at index 1");
    assert_eq!(z_state.model_matrices.len(), 2);

    // Simulate clear_frame
    z_state.clear_frame();
    println!(
        "Frame 1 - After clear: model_matrices.len() = {}",
        z_state.model_matrices.len()
    );
    assert_eq!(
        z_state.model_matrices.len(),
        0,
        "Model matrices should be cleared"
    );
    assert_eq!(
        z_state.current_model_idx, 0,
        "current_model_idx should be reset"
    );

    // --- FRAME 2 ---
    println!("\n=== Frame 2 ===");

    // Re-populate view/proj (app.rs updates in place)
    z_state.view_matrices[0] = Mat4::IDENTITY;
    z_state.proj_matrices[0] = Mat4::IDENTITY;
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    // Add NEW model matrices (should start at index 0 again)
    let model2_a = Mat4::from_translation(glam::Vec3::new(10.0, 0.0, 0.0));
    let model2_b = Mat4::from_translation(glam::Vec3::new(20.0, 0.0, 0.0));

    let idx2_a = z_state.add_model_matrix(model2_a);
    let idx2_b = z_state.add_model_matrix(model2_b);

    println!(
        "Frame 2 - Added matrices at indices: {}, {}",
        idx2_a, idx2_b
    );
    assert_eq!(
        idx2_a, 0,
        "Frame 2: First matrix should be at index 0 (reused)"
    );
    assert_eq!(
        idx2_b, 1,
        "Frame 2: Second matrix should be at index 1 (reused)"
    );
    assert_eq!(z_state.model_matrices.len(), 2);

    // Verify the matrices are the NEW ones, not the old ones
    assert_eq!(
        z_state.model_matrices[0], model2_a,
        "Should have new matrix, not old"
    );
    assert_eq!(
        z_state.model_matrices[1], model2_b,
        "Should have new matrix, not old"
    );
    assert_ne!(
        z_state.model_matrices[0], model1_a,
        "Should NOT have old matrix"
    );
}

/// Test deferred command expansion (billboards) adds matrices at correct indices
#[test]
fn test_deferred_command_matrix_indices() {
    let mut z_state = ZFFIState::default();

    // Populate view/proj
    z_state.view_matrices.push(Mat4::IDENTITY);
    z_state.proj_matrices.push(Mat4::IDENTITY);
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    // Game adds two regular draw commands
    let model_a = Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
    let model_b = Mat4::from_translation(glam::Vec3::new(2.0, 0.0, 0.0));
    let idx_a = z_state.add_model_matrix(model_a);
    let idx_b = z_state.add_model_matrix(model_b);

    assert_eq!(idx_a, 0);
    assert_eq!(idx_b, 1);
    assert_eq!(z_state.model_matrices.len(), 2);

    // Simulate deferred command expansion (what process_draw_commands does)
    let billboard_transform = Mat4::from_translation(glam::Vec3::new(5.0, 5.0, 5.0));
    let billboard_idx = z_state.add_model_matrix(billboard_transform);
    let billboard_mvp = MvpIndex::new(
        billboard_idx,
        z_state.current_view_idx,
        z_state.current_proj_idx,
    );

    println!("Billboard added at index: {}", billboard_idx);
    assert_eq!(
        billboard_idx, 2,
        "Billboard should be at index 2 (after A and B)"
    );
    assert_eq!(z_state.model_matrices.len(), 3);

    // Verify all matrices are at correct indices
    assert_eq!(z_state.model_matrices[0], model_a);
    assert_eq!(z_state.model_matrices[1], model_b);
    assert_eq!(z_state.model_matrices[2], billboard_transform);

    // Verify MVP index unpacks correctly
    let (model_idx, view_idx, proj_idx) = billboard_mvp.unpack();
    assert_eq!(model_idx, 2);
    assert_eq!(view_idx, 0);
    assert_eq!(proj_idx, 0);
}

/// Test render pass swap doesn't corrupt indices
#[test]
fn test_render_pass_swap_preserves_indices() {
    let mut z_state = ZFFIState::default();
    let mut graphics_command_buffer = VirtualRenderPass::new();

    // Populate view/proj
    z_state.view_matrices.push(Mat4::IDENTITY);
    z_state.proj_matrices.push(Mat4::IDENTITY);
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    // Add matrices and record commands
    let model_a = Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
    let idx_a = z_state.add_model_matrix(model_a);
    let mvp_a = MvpIndex::new(idx_a, 0, 0);

    z_state.render_pass.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        mvp_a,
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    println!("Before swap:");
    println!(
        "  z_state.render_pass commands: {}",
        z_state.render_pass.commands().len()
    );
    println!(
        "  graphics_command_buffer commands: {}",
        graphics_command_buffer.commands().len()
    );
    println!("  z_state.model_matrices: {}", z_state.model_matrices.len());

    // Simulate process_draw_commands swap
    std::mem::swap(&mut graphics_command_buffer, &mut z_state.render_pass);

    println!("\nAfter swap:");
    println!(
        "  z_state.render_pass commands: {}",
        z_state.render_pass.commands().len()
    );
    println!(
        "  graphics_command_buffer commands: {}",
        graphics_command_buffer.commands().len()
    );
    println!("  z_state.model_matrices: {}", z_state.model_matrices.len());

    // After swap, graphics_command_buffer should have the command
    assert_eq!(graphics_command_buffer.commands().len(), 1);
    assert_eq!(graphics_command_buffer.commands()[0].mvp_index, mvp_a);

    // Model matrices should still be accessible
    assert_eq!(z_state.model_matrices.len(), 1);
    assert_eq!(z_state.model_matrices[0], model_a);

    // Verify MVP index still references correct matrix
    let (model_idx, _, _) = graphics_command_buffer.commands()[0].mvp_index.unpack();
    assert_eq!(model_idx, 0);
    assert_eq!(z_state.model_matrices[model_idx as usize], model_a);
}

/// Full integration test simulating the actual render pipeline across multiple frames
#[test]
fn test_full_pipeline_multiple_frames() {
    let mut z_state = ZFFIState::default();
    let mut graphics_command_buffer = VirtualRenderPass::new();

    // === FRAME 1 ===
    println!("\n=== FRAME 1 ===");

    // Step 1: Populate view/proj (what execute_draw_commands_new does)
    let view1 = Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -5.0));
    let proj1 = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);

    z_state.view_matrices.push(view1);
    z_state.proj_matrices.push(proj1);
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    println!("Frame 1: Populated view/proj");

    // Step 2: Game adds commands (simulating draw calls)
    let model1_a = Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
    let model1_b = Mat4::from_translation(glam::Vec3::new(2.0, 0.0, 0.0));

    let idx1_a = z_state.add_model_matrix(model1_a);
    let idx1_b = z_state.add_model_matrix(model1_b);

    println!(
        "Frame 1: Added model matrices at indices {} and {}",
        idx1_a, idx1_b
    );

    z_state.render_pass.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        MvpIndex::new(idx1_a, 0, 0),
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    z_state.render_pass.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        MvpIndex::new(idx1_b, 0, 0),
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    println!("Frame 1: Recorded 2 draw commands");

    // Step 3: Swap render pass (what process_draw_commands does)
    std::mem::swap(&mut graphics_command_buffer, &mut z_state.render_pass);
    println!(
        "Frame 1: Swapped render pass - graphics_command_buffer now has {} commands",
        graphics_command_buffer.commands().len()
    );

    // Step 4: Expand deferred command (simulate billboard)
    let billboard1 = Mat4::from_translation(glam::Vec3::new(5.0, 5.0, 5.0));
    let billboard1_idx = z_state.add_model_matrix(billboard1);
    println!("Frame 1: Added billboard at index {}", billboard1_idx);

    // Simulate adding billboard command directly to graphics_command_buffer
    graphics_command_buffer.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        MvpIndex::new(billboard1_idx, 0, 0),
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    println!(
        "Frame 1: After deferred expansion - {} commands, {} model matrices",
        graphics_command_buffer.commands().len(),
        z_state.model_matrices.len()
    );

    // Step 5: Verify indices are correct before rendering
    assert_eq!(graphics_command_buffer.commands().len(), 3);
    assert_eq!(z_state.model_matrices.len(), 3);

    let cmd0_mvp = graphics_command_buffer.commands()[0].mvp_index.unpack();
    let cmd1_mvp = graphics_command_buffer.commands()[1].mvp_index.unpack();
    let cmd2_mvp = graphics_command_buffer.commands()[2].mvp_index.unpack();

    assert_eq!(
        cmd0_mvp.0, 0,
        "First command should reference model matrix 0"
    );
    assert_eq!(
        cmd1_mvp.0, 1,
        "Second command should reference model matrix 1"
    );
    assert_eq!(cmd2_mvp.0, 2, "Billboard should reference model matrix 2");

    assert_eq!(z_state.model_matrices[0], model1_a);
    assert_eq!(z_state.model_matrices[1], model1_b);
    assert_eq!(z_state.model_matrices[2], billboard1);

    // Step 6: Clear frame (what happens after rendering)
    z_state.clear_frame();
    println!(
        "Frame 1: Cleared - model_matrices.len() = {}",
        z_state.model_matrices.len()
    );

    // === FRAME 2 ===
    println!("\n=== FRAME 2 ===");

    // Step 1: Update view/proj in place
    let view2 = Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -10.0)); // Different camera
    let proj2 = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);

    z_state.view_matrices[0] = view2;
    z_state.proj_matrices[0] = proj2;
    z_state.current_view_idx = 0;
    z_state.current_proj_idx = 0;

    println!("Frame 2: Updated view/proj");

    // Step 2: Game adds NEW commands with DIFFERENT transforms
    let model2_a = Mat4::from_translation(glam::Vec3::new(10.0, 0.0, 0.0)); // Very different
    let model2_b = Mat4::from_translation(glam::Vec3::new(20.0, 0.0, 0.0));

    let idx2_a = z_state.add_model_matrix(model2_a);
    let idx2_b = z_state.add_model_matrix(model2_b);

    println!(
        "Frame 2: Added model matrices at indices {} and {} (should be 0 and 1)",
        idx2_a, idx2_b
    );

    assert_eq!(idx2_a, 0, "Frame 2: Indices should restart at 0");
    assert_eq!(idx2_b, 1, "Frame 2: Indices should restart at 0");

    z_state.render_pass.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        MvpIndex::new(idx2_a, 0, 0),
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    println!("Frame 2: Recorded draw command");

    // Step 3: Swap (should get Frame 2 commands, old buffer gets Frame 1 commands)
    std::mem::swap(&mut graphics_command_buffer, &mut z_state.render_pass);
    println!(
        "Frame 2: After swap - graphics_command_buffer has {} commands",
        graphics_command_buffer.commands().len()
    );
    println!(
        "Frame 2: z_state.render_pass (old graphics_command_buffer) has {} commands",
        z_state.render_pass.commands().len()
    );

    // The old commands from Frame 1 should now be in z_state.render_pass
    assert_eq!(
        z_state.render_pass.commands().len(),
        3,
        "Old Frame 1 commands should be in z_state.render_pass"
    );
    assert_eq!(
        graphics_command_buffer.commands().len(),
        1,
        "Frame 2 commands should be in graphics_command_buffer"
    );

    // Step 4: Add Frame 2 billboard
    let billboard2 = Mat4::from_translation(glam::Vec3::new(50.0, 50.0, 50.0)); // Very different
    let billboard2_idx = z_state.add_model_matrix(billboard2);
    println!("Frame 2: Added billboard at index {}", billboard2_idx);

    assert_eq!(billboard2_idx, 2, "Billboard should be at index 2");

    graphics_command_buffer.record_triangles(
        0,
        &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0],
        MvpIndex::new(billboard2_idx, 0, 0),
        0xFFFFFFFF,
        true,
        emberware_z::graphics::CullMode::Back,
        emberware_z::graphics::BlendMode::None,
        [emberware_z::graphics::TextureHandle::INVALID; 4],
        [emberware_z::graphics::MatcapBlendMode::Multiply; 4],
    );

    println!(
        "Frame 2: After deferred expansion - {} commands, {} model matrices",
        graphics_command_buffer.commands().len(),
        z_state.model_matrices.len()
    );

    // Step 5: Verify Frame 2 indices
    assert_eq!(graphics_command_buffer.commands().len(), 2);
    assert_eq!(z_state.model_matrices.len(), 3);

    let cmd2_0_mvp = graphics_command_buffer.commands()[0].mvp_index.unpack();
    let cmd2_1_mvp = graphics_command_buffer.commands()[1].mvp_index.unpack();

    assert_eq!(
        cmd2_0_mvp.0, 0,
        "Frame 2: First command should reference model matrix 0"
    );
    assert_eq!(
        cmd2_1_mvp.0, 2,
        "Frame 2: Billboard should reference model matrix 2"
    );

    // CRITICAL CHECK: Verify matrices are the NEW ones from Frame 2, not stale from Frame 1
    assert_eq!(
        z_state.model_matrices[0], model2_a,
        "Should have Frame 2 model_a, not Frame 1!"
    );
    assert_eq!(
        z_state.model_matrices[2], billboard2,
        "Should have Frame 2 billboard, not Frame 1!"
    );

    assert_ne!(
        z_state.model_matrices[0], model1_a,
        "Should NOT have Frame 1 model_a!"
    );

    println!("Frame 2: All checks passed!");
}

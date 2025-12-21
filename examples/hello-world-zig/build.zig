const std = @import("std");

pub fn build(b: *std.Build) void {
    // Target WebAssembly freestanding
    const target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });

    // Build the game executable
    const exe = b.addExecutable(.{
        .name = "game",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = .ReleaseSmall,
    });

    // Configure for Emberware
    exe.entry = .disabled; // No _start, we use init/update/render
    exe.rdynamic = true; // Export public functions

    // Install the artifact
    b.installArtifact(exe);

    // Add a run step (requires ember CLI)
    const run_cmd = b.addSystemCommand(&.{ "ember", "run", "zig-out/bin/game.wasm" });
    run_cmd.step.dependOn(b.getInstallStep());

    const run_step = b.step("run", "Build and run in Emberware player");
    run_step.dependOn(&run_cmd.step);
}

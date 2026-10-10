const std = @import("std");

// Pinned/tested with Zig 0.13.0. Import the generated SDK as a module;
// a relative @import from src/ would escape Zig's module root.
pub fn build(b: *std.Build) void {
    const target = b.resolveTargetQuery(.{ .cpu_arch = .wasm32, .os_tag = .freestanding });
    const exe = b.addExecutable(.{
        .name = "game",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = .ReleaseSmall,
    });
    exe.root_module.addImport("zx", b.createModule(.{
        .root_source_file = b.path("../../../include/zx.zig"),
    }));
    exe.entry = .disabled;
    exe.rdynamic = true;
    b.installArtifact(exe);
}

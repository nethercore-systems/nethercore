// Include pregenerated shaders from build.rs.
//
// This provides: PREGENERATED_SHADERS, get_pregenerated_shader(), ENVIRONMENT_SHADER, QUAD_SHADER.
include!(concat!(env!("OUT_DIR"), "/generated_shaders.rs"));

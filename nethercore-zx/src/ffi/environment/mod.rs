//! Environment system FFI functions (Multi-Environment v4)
//!
//! Functions for setting procedural environment rendering parameters.
//! Environments support 8 modes with layering (base + overlay) and blend modes.

mod config;
mod draw;

use anyhow::Result;
use wasmtime::{FuncType, Linker, ValType};

use super::ZXGameContext;

// Re-export functions for registration
pub(crate) use config::{
    env_cells, env_gradient, env_lines, env_nebula, env_rings, env_room, env_silhouette, env_veil,
};
pub(crate) use draw::{draw_env, env_blend, matcap_set};

/// Register environment system FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // wasmtime::Linker::func_wrap has a hard limit on parameter arity (16).
    // Our environment configuration functions have grown beyond that.
    // Use func_new to avoid the arity cap while keeping the wasm ABI stable.

    linker.func_new(
        "env",
        "env_gradient",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_gradient(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_f32(),
                params[6].unwrap_f32(),
                params[7].unwrap_f32(),
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
                params[13].unwrap_i32() as u32,
                params[14].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_cells",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_cells(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_i32() as u32,
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
                params[13].unwrap_i32() as u32,
                params[14].unwrap_f32(),
                params[15].unwrap_f32(),
                params[16].unwrap_f32(),
                params[17].unwrap_i32() as u32,
                params[18].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_lines",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_lines(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_f32(),
                params[5].unwrap_f32(),
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
                params[13].unwrap_i32() as u32,
                params[14].unwrap_i32() as u32,
                params[15].unwrap_f32(),
                params[16].unwrap_f32(),
                params[17].unwrap_f32(),
                params[18].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_silhouette",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_silhouette(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_i32() as u32,
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_nebula",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_nebula(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_i32() as u32,
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
                params[13].unwrap_i32() as u32,
                params[14].unwrap_f32(),
                params[15].unwrap_f32(),
                params[16].unwrap_f32(),
                params[17].unwrap_i32() as u32,
                params[18].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_room",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_room(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_f32(),
                params[5].unwrap_i32() as u32,
                params[6].unwrap_f32(),
                params[7].unwrap_f32(),
                params[8].unwrap_f32(),
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_f32(),
                params[13].unwrap_i32(),
                params[14].unwrap_i32(),
                params[15].unwrap_i32(),
                params[16].unwrap_i32() as u32,
                params[17].unwrap_i32() as u32,
                params[18].unwrap_i32() as u32,
                params[19].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_veil",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_veil(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_i32() as u32,
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_i32() as u32,
                params[9].unwrap_i32() as u32,
                params[10].unwrap_i32() as u32,
                params[11].unwrap_i32() as u32,
                params[12].unwrap_i32() as u32,
                params[13].unwrap_f32(),
                params[14].unwrap_f32(),
                params[15].unwrap_f32(),
                params[16].unwrap_i32() as u32,
                params[17].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_new(
        "env",
        "env_rings",
        FuncType::new(
            linker.engine(),
            [
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::F32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ],
            [],
        ),
        |caller, params, _results| {
            env_rings(
                caller,
                params[0].unwrap_i32() as u32,
                params[1].unwrap_i32() as u32,
                params[2].unwrap_i32() as u32,
                params[3].unwrap_i32() as u32,
                params[4].unwrap_i32() as u32,
                params[5].unwrap_i32() as u32,
                params[6].unwrap_i32() as u32,
                params[7].unwrap_i32() as u32,
                params[8].unwrap_f32(),
                params[9].unwrap_f32(),
                params[10].unwrap_f32(),
                params[11].unwrap_f32(),
                params[12].unwrap_i32() as u32,
                params[13].unwrap_i32() as u32,
                params[14].unwrap_i32() as u32,
                params[15].unwrap_i32() as u32,
                params[16].unwrap_i32() as u32,
                params[17].unwrap_i32() as u32,
            );
            Ok(())
        },
    )?;

    linker.func_wrap("env", "env_blend", env_blend)?;
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    linker.func_wrap("env", "draw_env", draw_env)?;
    Ok(())
}

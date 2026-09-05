//! Replay system support for Nethercore ZX
//!
//! Provides the `ZxInputLayout` implementation of the `InputLayout` trait
//! for encoding/decoding ZX input in replay scripts.

use std::borrow::Cow;
use std::collections::{HashMap as StdHashMap, HashSet};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use wasmtime::Linker;

/// Maximum value for stick axis conversion (-128 to 127 -> -1.0 to 1.0)
const STICK_SCALE: f32 = 127.0;

/// Maximum value for trigger conversion (0-255 -> 0.0 to 1.0)
const TRIGGER_SCALE: f32 = 255.0;

/// Threshold below which analog values are considered zero
const ANALOG_DEADZONE: f32 = 0.01;

use nethercore_core::app::{LoadedRom, RomLoader};
use nethercore_core::debug::registry::{RegisteredAction, RegisteredValue};
use nethercore_core::debug::types::{
    ActionParamType, ActionParamValue as RuntimeActionValue, DebugValue,
};
use nethercore_core::ffi::register_common_ffi;
use nethercore_core::replay::{
    ActionParamValue, CompiledAction, CompiledScript, DebugActionInfo, DebugActionParamInfo,
    DebugValueData, DebugVariableInfo, ExecutionReport, HeadlessBackend, HeadlessConfig,
    HeadlessRunner, InputLayout, StructuredInput,
};
use nethercore_core::wasm::{GameInstance, WasmEngine, WasmGameContext};
use nethercore_core::{AudioGenerator, Console};

use crate::audio::ZXAudioGenerator;
use crate::console::{Button, NethercoreZX, ZInput};
use crate::player::ZXRomLoader;
use crate::state::{ZRollbackState, ZXFFIState};

type ZxGame = GameInstance<ZInput, ZXFFIState, ZRollbackState>;

/// Load and execute a replay script against a real ZX ROM without graphics or an audio device.
pub fn run_headless(
    rom_path: &Path,
    script_path: &Path,
    config: HeadlessConfig,
) -> Result<ExecutionReport> {
    let loaded = ZXRomLoader::load_rom(rom_path)
        .with_context(|| format!("Failed to load ROM: {}", rom_path.display()))?;
    let script = nethercore_core::replay::ReplayScript::from_file(script_path)
        .with_context(|| format!("Failed to parse script: {}", script_path.display()))?;
    let compiled = nethercore_core::replay::Compiler::new(&ZxInputLayout)
        .compile(&script)
        .context("Failed to compile replay script")?;

    execute_headless(loaded, compiled, config)
}

/// Execute an already loaded ROM and compiled script. Exposed for focused runtime tests.
pub fn execute_headless(
    loaded: LoadedRom<NethercoreZX>,
    script: CompiledScript,
    config: HeadlessConfig,
) -> Result<ExecutionReport> {
    anyhow::ensure!(
        script.console.eq_ignore_ascii_case("zx"),
        "script console must be 'zx'"
    );

    anyhow::ensure!(
        (1..=4).contains(&script.player_count),
        "players must be 1..=4"
    );
    anyhow::ensure!(script.frame_count > 0, "replay must contain frames");
    anyhow::ensure!(
        script.screenshot_frames.is_empty(),
        "headless replay cannot capture screenshots; use rendered input-only playback"
    );
    let engine = WasmEngine::new_interruptible()?;
    let module = engine.load_module(&loaded.code)?;
    WasmEngine::validate_module_memory(&module, NethercoreZX::specs().ram_limit)?;
    anyhow::ensure!(
        matches!(module.get_export("update"), Some(wasmtime::ExternType::Func(f))
        if f.params().len() == 0 && f.results().len() == 0),
        "replay ROM must export update() with no arguments or results"
    );
    let mut linker: Linker<WasmGameContext<ZInput, ZXFFIState, ZRollbackState>> =
        Linker::new(engine.engine());
    register_common_ffi(&mut linker)?;
    loaded.console.register_ffi(&mut linker)?;

    let mut game =
        GameInstance::with_ram_limit(&engine, &module, &linker, NethercoreZX::specs().ram_limit)?;
    game.store_mut().set_epoch_deadline(1);
    game.configure_session(script.player_count.into(), player_mask(script.player_count));
    game.state_mut().seed_rng(script.seed);
    let initial_tick_rate =
        NethercoreZX::specs().tick_rates[NethercoreZX::specs().default_tick_rate];
    game.state_mut().delta_time = 1.0 / initial_tick_rate as f32;
    loaded
        .console
        .initialize_ffi_state(game.console_state_mut());

    let (cancel_timeout, timeout_thread) =
        timeout_watchdog(engine.engine().clone(), config.timeout_secs);
    let init_result = game.init();
    if let Err(error) = init_result {
        let _ = cancel_timeout.send(());
        let _ = timeout_thread.join();
        return Err(error).context("Replay game initialization failed");
    }
    game.store_mut()
        .data_mut()
        .debug_registry
        .finalize_registration();

    let tick_rate_index = game.console_state().init_config.tick_rate_index as usize;
    let tick_rate = *NethercoreZX::specs()
        .tick_rates
        .get(tick_rate_index)
        .context("game selected an invalid tick rate")?;
    let (bindings, variables) = debug_bindings(&game)?;
    let actions = action_metadata(&game);
    let mut backend = ZxHeadlessBackend {
        game,
        tick_rate,
        player_count: script.player_count as usize,
        bindings,
    };
    let mut runner = HeadlessRunner::new(script, config);
    runner.register_debug_variables(variables);
    runner.register_debug_actions(actions);
    let report = runner.execute_with_backend(&mut backend)?;

    let _ = cancel_timeout.send(());
    let _ = timeout_thread.join();
    Ok(report)
}

fn timeout_watchdog(
    engine: wasmtime::Engine,
    timeout_secs: u64,
) -> (mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        if receiver
            .recv_timeout(Duration::from_secs(timeout_secs))
            .is_err()
        {
            engine.increment_epoch();
        }
    });
    (sender, thread)
}

fn player_mask(players: u8) -> u32 {
    (1_u32 << players) - 1
}

struct DebugBinding {
    value: RegisteredValue,
    names: Vec<String>,
}

struct ZxHeadlessBackend {
    game: ZxGame,
    tick_rate: u32,
    player_count: usize,
    bindings: Vec<DebugBinding>,
}

impl HeadlessBackend for ZxHeadlessBackend {
    fn apply_inputs(&mut self, inputs: Option<&Vec<Vec<u8>>>) -> Result<()> {
        let inputs = inputs.context("script has no inputs for current frame")?;
        anyhow::ensure!(
            inputs.len() == self.player_count,
            "expected {} player inputs, got {}",
            self.player_count,
            inputs.len()
        );
        for (player, bytes) in inputs.iter().enumerate() {
            anyhow::ensure!(
                bytes.len() == 8,
                "player {} input must be 8 bytes",
                player + 1
            );
            self.game
                .set_input(player, ZxInputLayout::bytes_to_zinput(bytes));
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        self.game
            .update(1.0 / self.tick_rate as f32)
            .context("game update failed")?;
        let (state, rollback) = self.game.ffi_and_rollback_mut();
        crate::audio::advance_audio_positions(
            &mut rollback.audio,
            &mut rollback.tracker,
            &mut state.tracker_engine,
            &state.sounds,
            self.tick_rate,
            ZXAudioGenerator::default_sample_rate(),
        );
        Ok(())
    }

    fn read_debug_values(&mut self) -> Result<hashbrown::HashMap<String, DebugValueData>> {
        let memory = self
            .game
            .state()
            .memory
            .context("game does not export memory")?;
        let data = memory.data(self.game.store());
        let mut output = hashbrown::HashMap::new();
        let registry = &self.game.store().data().debug_registry;

        for binding in &self.bindings {
            let start = binding.value.wasm_ptr as usize;
            let end = start
                .checked_add(binding.value.value_type.byte_size())
                .context("debug value address overflow")?;
            let bytes = data.get(start..end).with_context(|| {
                format!(
                    "debug value '{}' is outside WASM memory",
                    binding.value.full_path
                )
            })?;
            let value =
                replay_value(registry.read_value_from_slice(bytes, binding.value.value_type));
            for name in &binding.names {
                output.insert(name.clone(), value.clone());
            }
        }
        Ok(output)
    }

    fn execute_actions(&mut self, actions: &[&CompiledAction]) -> Result<()> {
        for requested in actions {
            let registered = resolve_action(&self.game, &requested.name)?;
            let arguments = action_arguments(&registered, requested)?;
            self.game.call_action(&registered.func_name, &arguments)?;
        }
        Ok(())
    }
}

fn debug_bindings(game: &ZxGame) -> Result<(Vec<DebugBinding>, Vec<DebugVariableInfo>)> {
    let registry = &game.store().data().debug_registry;
    let mut canonical_paths = HashSet::new();
    let mut leaf_counts = StdHashMap::new();
    for value in &registry.values {
        anyhow::ensure!(
            canonical_paths.insert(canonical_name(&value.full_path)),
            "debug paths collide after normalization: {}",
            value.full_path
        );
        *leaf_counts
            .entry(canonical_name(&value.name))
            .or_insert(0_usize) += 1;
    }

    let mut bindings = Vec::with_capacity(registry.values.len());
    let mut variables = Vec::with_capacity(registry.values.len());
    for value in &registry.values {
        let canonical = canonical_name(&value.full_path);
        anyhow::ensure!(
            canonical.len() > 1,
            "debug path has no usable name: {}",
            value.full_path
        );
        let leaf = canonical_name(&value.name);
        let mut names = vec![canonical.clone()];
        if leaf != canonical && leaf_counts[&leaf] == 1 && !canonical_paths.contains(&leaf) {
            names.push(leaf);
        }
        let aliases = names.iter().skip(1).cloned().collect();
        bindings.push(DebugBinding {
            value: value.clone(),
            names,
        });
        variables.push(DebugVariableInfo {
            name: canonical,
            type_name: value.value_type.type_name().to_string(),
            description: value.name.clone(),
            full_path: Some(value.full_path.clone()),
            aliases,
        });
    }
    Ok((bindings, variables))
}

fn canonical_name(path: &str) -> String {
    let mut name = String::from("$");
    let mut separator = false;
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch.to_ascii_lowercase());
            separator = false;
        } else if !separator && name.len() > 1 {
            name.push('_');
            separator = true;
        }
    }
    while name.ends_with('_') {
        name.pop();
    }
    name
}

fn replay_value(value: DebugValue) -> DebugValueData {
    match value {
        DebugValue::I8(value) => DebugValueData::I32(value.into()),
        DebugValue::I16(value) => DebugValueData::I32(value.into()),
        DebugValue::I32(value) => DebugValueData::I32(value),
        DebugValue::U8(value) => DebugValueData::U32(value.into()),
        DebugValue::U16(value) => DebugValueData::U32(value.into()),
        DebugValue::U32(value) | DebugValue::Color(value) => DebugValueData::U32(value),
        DebugValue::F32(value) => DebugValueData::F32(value),
        DebugValue::Bool(value) => DebugValueData::Bool(value),
        DebugValue::Vec2 { x, y } => DebugValueData::Vec2 { x, y },
        DebugValue::Vec3 { x, y, z } => DebugValueData::Vec3 { x, y, z },
        DebugValue::Rect { x, y, w, h } => DebugValueData::Rect { x, y, w, h },
        fixed @ (DebugValue::FixedI16Q8(_)
        | DebugValue::FixedI32Q16(_)
        | DebugValue::FixedI32Q8(_)
        | DebugValue::FixedI32Q24(_)) => DebugValueData::F32(fixed.as_f32()),
    }
}

fn resolve_action(game: &ZxGame, requested: &str) -> Result<RegisteredAction> {
    let normalized = canonical_name(requested);
    let matches = game
        .store()
        .data()
        .debug_registry
        .actions
        .iter()
        .filter(|action| {
            action.name.eq_ignore_ascii_case(requested)
                || action.full_path.eq_ignore_ascii_case(requested)
                || canonical_name(&action.full_path) == normalized
        })
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [action] => Ok(action.clone()),
        [] => anyhow::bail!("unknown debug action '{requested}'"),
        _ => anyhow::bail!("debug action '{requested}' is ambiguous; use its full path"),
    }
}

fn action_arguments(
    action: &RegisteredAction,
    requested: &CompiledAction,
) -> Result<Vec<RuntimeActionValue>> {
    let expected = action
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<HashSet<_>>();
    let mut unknown = requested
        .params
        .keys()
        .filter(|name| !expected.contains(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    anyhow::ensure!(
        unknown.is_empty(),
        "unknown parameters for '{}': {}",
        action.full_path,
        unknown.join(", ")
    );

    action
        .params
        .iter()
        .map(|param| {
            let supplied = requested.params.get(&param.name);
            match (param.param_type, supplied) {
                (ActionParamType::I32, Some(ActionParamValue::Int(value))) => {
                    Ok(RuntimeActionValue::I32(*value))
                }
                (ActionParamType::F32, Some(ActionParamValue::Float(value))) => {
                    Ok(RuntimeActionValue::F32(*value))
                }
                (ActionParamType::F32, Some(ActionParamValue::Int(value))) => {
                    Ok(RuntimeActionValue::F32(*value as f32))
                }
                (_, Some(_)) => anyhow::bail!("parameter '{}' has the wrong type", param.name),
                (_, None) => Ok(param.default_value),
            }
        })
        .collect()
}

fn action_metadata(game: &ZxGame) -> Vec<DebugActionInfo> {
    game.store()
        .data()
        .debug_registry
        .actions
        .iter()
        .map(|action| DebugActionInfo {
            name: action.name.clone(),
            full_path: action.full_path.clone(),
            parameters: action
                .params
                .iter()
                .map(|param| DebugActionParamInfo {
                    name: param.name.clone(),
                    type_name: match param.param_type {
                        ActionParamType::I32 => "i32",
                        ActionParamType::F32 => "f32",
                    }
                    .to_string(),
                    default: match param.default_value {
                        RuntimeActionValue::I32(value) => serde_json::json!(value),
                        RuntimeActionValue::F32(value) => serde_json::json!(value),
                    },
                })
                .collect(),
        })
        .collect()
}

/// Input layout for Nethercore ZX console
///
/// Handles encoding/decoding between structured input (symbolic button names,
/// analog values) and raw ZX input bytes.
///
/// # Input Format (8 bytes)
///
/// | Offset | Size | Field           | Description                    |
/// |--------|------|-----------------|--------------------------------|
/// | 0      | 2    | buttons         | u16 bitmask (little-endian)    |
/// | 2      | 1    | left_stick_x    | i8 (-128 to 127)               |
/// | 3      | 1    | left_stick_y    | i8 (-128 to 127)               |
/// | 4      | 1    | right_stick_x   | i8 (-128 to 127)               |
/// | 5      | 1    | right_stick_y   | i8 (-128 to 127)               |
/// | 6      | 1    | left_trigger    | u8 (0 to 255)                  |
/// | 7      | 1    | right_trigger   | u8 (0 to 255)                  |
///
/// # Button Mapping
///
/// | Bit | Button | Script Name |
/// |-----|--------|-------------|
/// | 0   | Up     | up          |
/// | 1   | Down   | down        |
/// | 2   | Left   | left        |
/// | 3   | Right  | right       |
/// | 4   | A      | a           |
/// | 5   | B      | b           |
/// | 6   | X      | x           |
/// | 7   | Y      | y           |
/// | 8   | LB     | l / lb      |
/// | 9   | RB     | r / rb      |
/// | 10  | L3     | l3          |
/// | 11  | R3     | r3          |
/// | 12  | Start  | start       |
/// | 13  | Select | select      |
#[derive(Debug, Clone, Copy, Default)]
pub struct ZxInputLayout;

impl ZxInputLayout {
    /// Create a new ZX input layout
    pub fn new() -> Self {
        Self
    }

    /// Convert a ZInput struct to raw bytes
    pub fn zinput_to_bytes(input: &ZInput) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = (input.buttons & 0xFF) as u8;
        bytes[1] = ((input.buttons >> 8) & 0xFF) as u8;
        bytes[2] = input.left_stick_x as u8;
        bytes[3] = input.left_stick_y as u8;
        bytes[4] = input.right_stick_x as u8;
        bytes[5] = input.right_stick_y as u8;
        bytes[6] = input.left_trigger;
        bytes[7] = input.right_trigger;
        bytes
    }

    /// Convert raw bytes to a ZInput struct
    pub fn bytes_to_zinput(bytes: &[u8]) -> ZInput {
        let mut input = ZInput::default();

        if bytes.len() >= 2 {
            input.buttons = u16::from_le_bytes([bytes[0], bytes[1]]);
        }
        if bytes.len() >= 4 {
            input.left_stick_x = bytes[2] as i8;
            input.left_stick_y = bytes[3] as i8;
        }
        if bytes.len() >= 6 {
            input.right_stick_x = bytes[4] as i8;
            input.right_stick_y = bytes[5] as i8;
        }
        if bytes.len() >= 7 {
            input.left_trigger = bytes[6];
        }
        if bytes.len() >= 8 {
            input.right_trigger = bytes[7];
        }

        input
    }
}

impl InputLayout for ZxInputLayout {
    fn encode_input(&self, input: &StructuredInput) -> Vec<u8> {
        let mut buttons: u16 = 0;

        for button in &input.buttons {
            match button.to_lowercase().as_str() {
                "up" => buttons |= Button::Up.mask(),
                "down" => buttons |= Button::Down.mask(),
                "left" => buttons |= Button::Left.mask(),
                "right" => buttons |= Button::Right.mask(),
                "a" => buttons |= Button::A.mask(),
                "b" => buttons |= Button::B.mask(),
                "x" => buttons |= Button::X.mask(),
                "y" => buttons |= Button::Y.mask(),
                "l" | "lb" | "l1" => buttons |= Button::LeftBumper.mask(),
                "r" | "rb" | "r1" => buttons |= Button::RightBumper.mask(),
                "l3" | "ls" => buttons |= Button::LeftStick.mask(),
                "r3" | "rs" => buttons |= Button::RightStick.mask(),
                "start" => buttons |= Button::Start.mask(),
                "select" | "back" => buttons |= Button::Select.mask(),
                _ => {}
            }
        }

        let mut bytes = vec![0u8; 8];
        bytes[0] = (buttons & 0xFF) as u8;
        bytes[1] = ((buttons >> 8) & 0xFF) as u8;

        // Left stick: convert -1.0..1.0 to -128..127
        if let Some([x, _]) = input.lstick {
            bytes[2] = (x.clamp(-1.0, 1.0) * STICK_SCALE) as i8 as u8;
        }
        if let Some([_, y]) = input.lstick {
            bytes[3] = (y.clamp(-1.0, 1.0) * STICK_SCALE) as i8 as u8;
        }

        // Right stick: convert -1.0..1.0 to -128..127
        if let Some([x, _]) = input.rstick {
            bytes[4] = (x.clamp(-1.0, 1.0) * STICK_SCALE) as i8 as u8;
        }
        if let Some([_, y]) = input.rstick {
            bytes[5] = (y.clamp(-1.0, 1.0) * STICK_SCALE) as i8 as u8;
        }

        // Triggers: convert 0.0..1.0 to 0..255
        if let Some(lt) = input.lt {
            bytes[6] = (lt.clamp(0.0, 1.0) * TRIGGER_SCALE) as u8;
        }
        if let Some(rt) = input.rt {
            bytes[7] = (rt.clamp(0.0, 1.0) * TRIGGER_SCALE) as u8;
        }

        bytes
    }

    fn decode_input(&self, bytes: &[u8]) -> StructuredInput {
        let mut input = StructuredInput::default();

        if bytes.len() >= 2 {
            let buttons = u16::from_le_bytes([bytes[0], bytes[1]]);

            if buttons & Button::Up.mask() != 0 {
                input.buttons.push(Cow::Borrowed("up"));
            }
            if buttons & Button::Down.mask() != 0 {
                input.buttons.push(Cow::Borrowed("down"));
            }
            if buttons & Button::Left.mask() != 0 {
                input.buttons.push(Cow::Borrowed("left"));
            }
            if buttons & Button::Right.mask() != 0 {
                input.buttons.push(Cow::Borrowed("right"));
            }
            if buttons & Button::A.mask() != 0 {
                input.buttons.push(Cow::Borrowed("a"));
            }
            if buttons & Button::B.mask() != 0 {
                input.buttons.push(Cow::Borrowed("b"));
            }
            if buttons & Button::X.mask() != 0 {
                input.buttons.push(Cow::Borrowed("x"));
            }
            if buttons & Button::Y.mask() != 0 {
                input.buttons.push(Cow::Borrowed("y"));
            }
            if buttons & Button::LeftBumper.mask() != 0 {
                input.buttons.push(Cow::Borrowed("l"));
            }
            if buttons & Button::RightBumper.mask() != 0 {
                input.buttons.push(Cow::Borrowed("r"));
            }
            if buttons & Button::LeftStick.mask() != 0 {
                input.buttons.push(Cow::Borrowed("l3"));
            }
            if buttons & Button::RightStick.mask() != 0 {
                input.buttons.push(Cow::Borrowed("r3"));
            }
            if buttons & Button::Start.mask() != 0 {
                input.buttons.push(Cow::Borrowed("start"));
            }
            if buttons & Button::Select.mask() != 0 {
                input.buttons.push(Cow::Borrowed("select"));
            }
        }

        // Left stick: convert -128..127 to -1.0..1.0
        if bytes.len() >= 4 {
            let lx = bytes[2] as i8 as f32 / STICK_SCALE;
            let ly = bytes[3] as i8 as f32 / STICK_SCALE;
            if lx.abs() > ANALOG_DEADZONE || ly.abs() > ANALOG_DEADZONE {
                input.lstick = Some([lx, ly]);
            }
        }

        // Right stick: convert -128..127 to -1.0..1.0
        if bytes.len() >= 6 {
            let rx = bytes[4] as i8 as f32 / STICK_SCALE;
            let ry = bytes[5] as i8 as f32 / STICK_SCALE;
            if rx.abs() > ANALOG_DEADZONE || ry.abs() > ANALOG_DEADZONE {
                input.rstick = Some([rx, ry]);
            }
        }

        // Triggers: convert 0..255 to 0.0..1.0
        if bytes.len() >= 7 && bytes[6] > 0 {
            input.lt = Some(bytes[6] as f32 / TRIGGER_SCALE);
        }
        if bytes.len() >= 8 && bytes[7] > 0 {
            input.rt = Some(bytes[7] as f32 / TRIGGER_SCALE);
        }

        input
    }

    fn input_size(&self) -> usize {
        8
    }

    fn console_id(&self) -> u8 {
        1 // ZX console ID
    }

    fn button_names(&self) -> &[&str] {
        &[
            "up", "down", "left", "right", "a", "b", "x", "y", "l", "r", "l3", "r3", "start",
            "select",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashMap;
    use nethercore_core::app::LoadedRom;
    use nethercore_core::replay::{
        ActionParamValue, CompareOp, CompiledAction, CompiledAssertValue, CompiledAssertion,
        CompiledScript, HeadlessConfig, InputSequence,
    };

    use crate::console::NethercoreZX;

    #[test]
    fn test_button_encoding() {
        let layout = ZxInputLayout;

        // Test single button
        let input = StructuredInput {
            buttons: vec![Cow::Borrowed("a")],
            ..Default::default()
        };
        let bytes = layout.encode_input(&input);
        assert_eq!(bytes[0] & 0x10, 0x10); // A button is bit 4

        // Test multiple buttons
        let input = StructuredInput {
            buttons: vec![Cow::Borrowed("up"), Cow::Borrowed("a")],
            ..Default::default()
        };
        let bytes = layout.encode_input(&input);
        assert_eq!(bytes[0] & 0x11, 0x11); // Up (bit 0) + A (bit 4)
    }

    #[test]
    fn test_button_decoding() {
        let layout = ZxInputLayout;

        // A button pressed
        let bytes = [0x10, 0x00, 0, 0, 0, 0, 0, 0];
        let input = layout.decode_input(&bytes);
        assert_eq!(input.buttons, vec![Cow::Borrowed("a")]);

        // Up + Start
        let bytes = [0x01, 0x10, 0, 0, 0, 0, 0, 0];
        let input = layout.decode_input(&bytes);
        assert!(input.buttons.contains(&Cow::Borrowed("up")));
        assert!(input.buttons.contains(&Cow::Borrowed("start")));
    }

    #[test]
    fn test_analog_roundtrip() {
        let layout = ZxInputLayout;

        let input = StructuredInput {
            buttons: Vec::new(),
            lstick: Some([0.5, -0.5]),
            rstick: Some([-1.0, 1.0]),
            lt: Some(0.75),
            rt: Some(0.25),
        };

        let bytes = layout.encode_input(&input);
        let decoded = layout.decode_input(&bytes);

        // Check lstick (with some tolerance for quantization)
        let lstick = decoded.lstick.unwrap();
        assert!((lstick[0] - 0.5).abs() < 0.02);
        assert!((lstick[1] - (-0.5)).abs() < 0.02);

        // Check rstick
        let rstick = decoded.rstick.unwrap();
        assert!((rstick[0] - (-1.0)).abs() < 0.02);
        assert!((rstick[1] - 1.0).abs() < 0.02);

        // Check triggers
        assert!((decoded.lt.unwrap() - 0.75).abs() < 0.01);
        assert!((decoded.rt.unwrap() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_zinput_conversion() {
        let input = ZInput {
            buttons: 0x0011, // Up + A
            left_stick_x: 64,
            left_stick_y: -64,
            right_stick_x: 0,
            right_stick_y: 0,
            left_trigger: 128,
            right_trigger: 0,
        };

        let bytes = ZxInputLayout::zinput_to_bytes(&input);
        let decoded = ZxInputLayout::bytes_to_zinput(&bytes);

        assert_eq!(decoded.buttons, input.buttons);
        assert_eq!(decoded.left_stick_x, input.left_stick_x);
        assert_eq!(decoded.left_stick_y, input.left_stick_y);
        assert_eq!(decoded.left_trigger, input.left_trigger);
    }

    #[test]
    fn headless_wasm_applies_seed_players_inputs_actions_and_snapshots() {
        let wasm = wat::parse_str(
            r#"
(module
  (import "env" "random" (func $random (result i32)))
  (import "env" "player_count" (func $player_count (result i32)))
  (import "env" "delta_time" (func $delta_time (result f32)))
  (import "env" "buttons_held" (func $buttons_held (param i32) (result i32)))
  (import "env" "debug_group_begin" (func $group_begin (param i32 i32)))
  (import "env" "debug_group_end" (func $group_end))
  (import "env" "debug_register_i32" (func $register_i32 (param i32 i32 i32)))
  (import "env" "debug_register_f32" (func $register_f32 (param i32 i32 i32)))
  (import "env" "debug_action_begin" (func $action_begin (param i32 i32 i32 i32)))
  (import "env" "debug_action_param_i32" (func $action_param_i32 (param i32 i32 i32)))
  (import "env" "debug_action_end" (func $action_end))
  (memory (export "memory") 1)
  (data (i32.const 0) "Match")
  (data (i32.const 8) "seed_sample")
  (data (i32.const 24) "players")
  (data (i32.const 32) "tick_seconds")
  (data (i32.const 48) "input_sum")
  (data (i32.const 64) "score_p1")
  (data (i32.const 80) "Add Score")
  (data (i32.const 96) "add_score")
  (data (i32.const 112) "amount")
  (func (export "init")
    (i32.store (i32.const 128) (call $random))
    (i32.store (i32.const 132) (call $player_count))
    (f32.store (i32.const 136) (call $delta_time))
    (call $group_begin (i32.const 0) (i32.const 5))
    (call $register_i32 (i32.const 8) (i32.const 11) (i32.const 128))
    (call $register_i32 (i32.const 24) (i32.const 7) (i32.const 132))
    (call $register_f32 (i32.const 32) (i32.const 12) (i32.const 136))
    (call $register_i32 (i32.const 48) (i32.const 9) (i32.const 140))
    (call $register_i32 (i32.const 64) (i32.const 8) (i32.const 144))
    (call $action_begin (i32.const 80) (i32.const 9) (i32.const 96) (i32.const 9))
    (call $action_param_i32 (i32.const 112) (i32.const 6) (i32.const 1))
    (call $action_end)
    (call $group_end))
  (func (export "update")
    (i32.store (i32.const 140)
      (i32.add (call $buttons_held (i32.const 0))
               (call $buttons_held (i32.const 1)))))
  (func (export "add_score") (param $amount i32)
    (i32.store (i32.const 144)
      (i32.add (i32.load (i32.const 144)) (local.get $amount)))))
"#,
        )
        .unwrap();

        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![
            vec![1, 0, 0, 0, 0, 0, 0, 0],
            vec![2, 0, 0, 0, 0, 0, 0, 0],
        ]);
        let mut params = HashMap::new();
        params.insert("amount".to_string(), ActionParamValue::Int(7));
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0x1234_5678_9abc_def0,
            player_count: 2,
            input_size: 8,
            inputs,
            screenshot_frames: vec![],
            snap_frames: vec![0],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$match_score_p1 == 7".to_string(),
                variable: "$match_score_p1".to_string(),
                operator: CompareOp::Eq,
                value: CompiledAssertValue::Number(7.0),
            }],
            actions: vec![CompiledAction {
                frame: 0,
                name: "Add Score".to_string(),
                params,
            }],
            frame_count: 1,
        };
        let rom = LoadedRom {
            code: wasm,
            console: NethercoreZX::new(),
            game_name: "fixture".to_string(),
            game_id: "fixture".to_string(),
        };

        let report = execute_headless(rom, script, HeadlessConfig::default()).unwrap();

        assert_eq!(report.summary.status, "PASSED");
        let values = &report.snapshots[0].post;
        assert_eq!(values["$match_players"].as_f64(), Some(2.0));
        assert_eq!(values["$match_input_sum"].as_f64(), Some(3.0));
        assert_eq!(values["$match_score_p1"].as_f64(), Some(7.0));
        assert!(values["$match_tick_seconds"].as_f64().unwrap() > 0.0);
        let mut expected_rng = nethercore_core::wasm::GameState::<ZInput>::new();
        expected_rng.seed_rng(0x1234_5678_9abc_def0);
        assert_eq!(
            values["$match_seed_sample"].as_f64(),
            Some(expected_rng.random() as i32 as f64)
        );
        assert_eq!(
            report.registered_actions.unwrap()[0].parameters[0].name,
            "amount"
        );
    }

    #[test]
    fn previous_tick_values_and_aliases_are_unambiguous() {
        let wat = r#"(module
          (import "env" "debug_register_i32" (func $reg (param i32 i32 i32)))
          (import "env" "debug_group_begin" (func $group (param i32 i32)))
          (import "env" "debug_group_end" (func $end))
          (memory (export "memory") 1)
          (data (i32.const 0) "a/b")
          (data (i32.const 8) "a_b")
          (data (i32.const 16) "other")
          (func (export "init")
            (call $reg (i32.const 0) (i32.const 3) (i32.const 128))
            (call $group (i32.const 16) (i32.const 5))
            (call $reg (i32.const 8) (i32.const 3) (i32.const 132))
            (call $end)
            (i32.store (i32.const 132) (i32.const 99)))
          (func (export "update")
            (i32.store (i32.const 128) (i32.add (i32.load (i32.const 128)) (i32.const 1)))))"#;
        let script = nethercore_core::replay::ReplayScript::from_toml(
            r#"
console = "zx"
[[frames]]
f = 2
snap = true
assert = "$a_b > $prev_a_b"
"#,
        )
        .unwrap();
        let compiled = nethercore_core::replay::Compiler::new(&ZxInputLayout)
            .compile(&script)
            .unwrap();
        let mut rom = minimal_rom("");
        rom.code = wat::parse_str(wat).unwrap();
        let report = execute_headless(
            rom,
            nethercore_core::replay::Compiler::new(&ZxInputLayout)
                .compile(&script)
                .unwrap(),
            HeadlessConfig::default(),
        )
        .unwrap();
        assert!(report.succeeded(), "{report:?}");
        assert_eq!(report.snapshots[0].post["$a_b"].as_f64(), Some(3.0));
        assert_eq!(report.snapshots[0].post["$other_a_b"].as_f64(), Some(99.0));
        let mut rom = minimal_rom("");
        rom.code = wat::parse_str(
            wat.replace("(call $group (i32.const 16) (i32.const 5))", "")
                .replace("(call $end)", ""),
        )
        .unwrap();
        let error = execute_headless(rom, compiled, HeadlessConfig::default()).unwrap_err();
        assert!(error.to_string().contains("collide"));
    }

    #[test]
    fn malformed_inputs_and_unsupported_captures_fail_closed() {
        for body in [
            "p1 = 'aa'",
            "p1 = [0]",
            "p1 = {lt = 2.0}",
            "params = {x = 1}",
        ] {
            let text = format!("console = 'zx'\n[[frames]]\nf = 0\n{body}");
            let result = nethercore_core::replay::ReplayScript::from_toml(&text)
                .map_err(|e| e.to_string())
                .and_then(|s| {
                    nethercore_core::replay::Compiler::new(&ZxInputLayout)
                        .compile(&s)
                        .map_err(|e| e.to_string())
                });
            assert!(result.is_err(), "accepted {body}");
        }
        let mut script = one_frame_script();
        script.screenshot_frames.push(0);
        assert!(execute_headless(minimal_rom(""), script, HeadlessConfig::default()).is_err());
    }

    fn minimal_rom(update_body: &str) -> LoadedRom<NethercoreZX> {
        let wat = format!(
            r#"(module
  (memory (export "memory") 1)
  (func (export "init"))
  (func (export "update") {update_body}))"#
        );
        LoadedRom {
            code: wat::parse_str(wat).unwrap(),
            console: NethercoreZX::new(),
            game_name: "fixture".to_string(),
            game_id: "fixture".to_string(),
        }
    }

    fn one_frame_script() -> CompiledScript {
        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![vec![0; 8]]);
        CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 1,
            player_count: 1,
            input_size: 8,
            inputs,
            screenshot_frames: vec![],
            snap_frames: vec![],
            assertions: vec![],
            actions: vec![],
            frame_count: 1,
        }
    }

    #[test]
    fn float_assertions_use_registered_precision() {
        let mut rom = minimal_rom("");
        rom.code = wat::parse_str(
            r#"(module
            (import "env" "debug_register_f32" (func $reg (param i32 i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "value")
            (func (export "init")
                (f32.store (i32.const 32) (f32.const 0.1))
                (call $reg (i32.const 0) (i32.const 5) (i32.const 32)))
            (func (export "update")))"#,
        )
        .unwrap();
        let mut script = one_frame_script();
        script.assertions.push(CompiledAssertion {
            frame: 0,
            condition: "$value == 0.1".into(),
            variable: "$value".into(),
            operator: CompareOp::Eq,
            value: CompiledAssertValue::Number(0.1),
        });
        let report = execute_headless(rom, script, HeadlessConfig::default()).unwrap();
        assert!(report.succeeded(), "{report:?}");
    }

    #[test]
    fn unknown_action_is_an_error_report() {
        let mut script = one_frame_script();
        script.actions.push(CompiledAction {
            frame: 0,
            name: "Missing Action".to_string(),
            params: HashMap::new(),
        });

        let report = execute_headless(minimal_rom(""), script, HeadlessConfig::default()).unwrap();

        assert_eq!(report.summary.status, "ERROR");
        assert!(
            report
                .error
                .unwrap()
                .message
                .contains("unknown debug action")
        );
    }

    #[test]
    fn unknown_assertion_variable_fails() {
        let mut script = one_frame_script();
        script.assertions.push(CompiledAssertion {
            frame: 0,
            condition: "$missing == 1".to_string(),
            variable: "$missing".to_string(),
            operator: CompareOp::Eq,
            value: CompiledAssertValue::Number(1.0),
        });

        let report = execute_headless(minimal_rom(""), script, HeadlessConfig::default()).unwrap();

        assert_eq!(report.summary.status, "FAILED");
        assert_eq!(report.assertions[0].actual, None);
    }

    #[test]
    fn wasm_timeout_is_an_error_report() {
        let report = execute_headless(
            minimal_rom("(loop $forever (br $forever))"),
            one_frame_script(),
            HeadlessConfig {
                timeout_secs: 0,
                ..HeadlessConfig::default()
            },
        )
        .unwrap();

        assert_eq!(report.summary.status, "ERROR");
        assert_eq!(report.error.unwrap().context, "update");
    }
}

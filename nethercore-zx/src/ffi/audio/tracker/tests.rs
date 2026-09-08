use super::*;
use std::sync::Arc;
use wasmtime::{Engine, Module, Store};

// Original source: one mono instrument and one two-slot stereo instrument.
fn source() -> Vec<u8> {
    let mut data = vec![0; 336];
    data[..17].copy_from_slice(b"Extended Module: ");
    data[37] = 26;
    data[38..58].fill(b' ');
    let creator = b"FastTracker v2.00";
    data[38..38 + creator.len()].copy_from_slice(creator);
    data[58..60].copy_from_slice(&0x104u16.to_le_bytes());
    data[60..64].copy_from_slice(&276u32.to_le_bytes());
    for (offset, value) in [
        (64, 1u16),
        (68, 1),
        (70, 1),
        (72, 2),
        (74, 1),
        (76, 6),
        (78, 125),
    ] {
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    data.extend(9u32.to_le_bytes());
    data.push(0);
    data.extend(8u16.to_le_bytes());
    data.extend(40u16.to_le_bytes());
    for row in 0..8 {
        data.extend(match row {
            0 => [49, 1, 0, 0, 0],
            2 => [49, 2, 0, 0, 0],
            4 => [61, 2, 0, 0, 0],
            _ => [0; 5],
        });
    }
    for count in [1u16, 2] {
        let mut instrument = vec![0; 263];
        instrument[..4].copy_from_slice(&263u32.to_le_bytes());
        instrument[27..29].copy_from_slice(&count.to_le_bytes());
        instrument[29..33].copy_from_slice(&40u32.to_le_bytes());
        if count == 2 {
            instrument[93..129].fill(1);
        }
        data.extend(instrument);
        for _ in 0..count {
            let mut sample = vec![0; 40];
            let bytes = if count == 1 { 8u32 } else { 16 };
            sample[..4].copy_from_slice(&bytes.to_le_bytes());
            sample[8..12].copy_from_slice(&bytes.to_le_bytes());
            sample[12] = 64;
            sample[14] = 1 | 16 | if count == 2 { 32 } else { 0 };
            sample[15] = 128;
            sample[18..].fill(b' ');
            data.extend(sample);
        }
        data.extend(vec![0; if count == 1 { 8 } else { 32 }]);
    }
    data
}

#[test]
fn raw_tracker_supplied_handles_real_wasm_and_rollback() {
    let engine = Engine::default();
    let wasm = Module::new(
        &engine,
        r#"(module
      (import "env" "load_tracker_with_samples" (func $load (param i32 i32 i32 i32) (result i32)))
      (import "env" "load_tracker" (func $old (param i32 i32) (result i32)))
      (import "env" "rom_tracker" (func $rom (param i32 i32) (result i32)))
      (import "env" "music_play" (func $play (param i32 f32 i32)))
      (memory (export "memory") 1)
      (func (export "load") (param i32 i32 i32 i32) (result i32)
        local.get 0 local.get 1 local.get 2 local.get 3 call $load)
      (func (export "old") (param i32 i32) (result i32) local.get 0 local.get 1 call $old)
      (func (export "rom") (param i32 i32) (result i32) local.get 0 local.get 1 call $rom)
      (func (export "play") (param i32) local.get 0 f32.const 1 i32.const 0 call $play))"#,
    )
    .unwrap();
    let mut linker = Linker::new(&engine);
    crate::ffi::register_zx_ffi(&mut linker).unwrap();
    let mut ctx = ZXGameContext::default();
    ctx.game.in_init = true;
    ctx.ffi.sounds = vec![
        None,
        Some(Sound {
            data: Arc::new(vec![8192; 256]),
        }),
        Some(Sound {
            data: Arc::new((0..256).flat_map(|_| [8192, -8192]).collect()),
        }),
        Some(Sound {
            data: Arc::new((0..256).flat_map(|_| [-4096, 4096]).collect()),
        }),
    ];
    let mut store = Store::new(&engine, ctx);
    let instance = linker.instantiate(&mut store, &wasm).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let load = instance
        .get_typed_func::<(u32, u32, u32, u32), u32>(&mut store, "load")
        .unwrap();
    let old = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "old")
        .unwrap();
    let play = instance
        .get_typed_func::<u32, ()>(&mut store, "play")
        .unwrap();
    let data = source();
    let len = data.len() as u32;
    memory.write(&mut store, 1024, &data).unwrap();
    memory
        .write(
            &mut store,
            0,
            &[1u32, 2, 3]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert_eq!(old.call(&mut store, (1024, len)).unwrap(), 0);
    for args in [
        (1024, len, 0, 2),
        (1024, len, 0, 16000), // In bounds, but reject count before allocating handles.
        (65535, len, 0, 3),
        (1024, len, 65535, 3),
        (1024, len, 0, u32::MAX),
    ] {
        assert_eq!(load.call(&mut store, args).unwrap(), 0);
    }
    memory.write(&mut store, 8, &999u32.to_le_bytes()).unwrap();
    assert_eq!(load.call(&mut store, (1024, len, 0, 3)).unwrap(), 0);
    memory.write(&mut store, 8, &3u32.to_le_bytes()).unwrap();
    let handle = load.call(&mut store, (1024, len, 0, 3)).unwrap();
    assert!(is_tracker_handle(handle));
    play.call(&mut store, handle).unwrap();
    let ctx = store.data_mut();
    let sounds = ctx.ffi.sounds.clone();
    let state = &mut ctx.rollback.tracker;
    let tracker = &mut ctx.ffi.tracker_engine;
    tracker.sync_to_state(state, &sounds);
    let mut segments = [(0.0, 0.0); 3];
    for i in 0..26000 {
        let pcm = tracker.render_sample_and_advance(state, &sounds, 44100);
        for (slot, at) in [3000, 14000, 25000].into_iter().enumerate() {
            if i == at {
                segments[slot] = pcm;
            }
        }
    }
    assert!(segments[0].0 > 0.01 && segments[0].1 > 0.01, "{segments:?}");
    assert!(
        segments[1].0 > 0.01 && segments[1].1 < -0.01,
        "{segments:?}"
    );
    assert!(
        segments[2].0 < -0.01 && segments[2].1 > 0.01,
        "{segments:?}"
    );
    let snapshot = tracker.snapshot();
    let saved = *state;
    let expected: Vec<_> = (0..2048)
        .map(|_| tracker.render_sample_and_advance(state, &sounds, 44100))
        .collect();
    let final_state = *state;
    for cold in [false, true] {
        let mut receiver = crate::tracker::TrackerEngine::new();
        if cold {
            assert_eq!(
                receiver.load_xm_module(nether_xm::parse_xm(&data).unwrap(), vec![1, 2, 3]),
                handle
            );
            receiver.sync_to_state(&saved, &sounds);
        } else {
            receiver.apply_snapshot(&snapshot);
        }
        let mut position = saved;
        let actual: Vec<_> = (0..2048)
            .map(|_| receiver.render_sample_and_advance(&mut position, &sounds, 44100))
            .collect();
        assert_eq!(actual, expected, "cold={cold}");
        assert_eq!(
            bytemuck::bytes_of(&position),
            bytemuck::bytes_of(&final_state)
        );
    }
    // Explicit silent slots are accepted; the init-only guard is exercised through WASM.
    memory.write(&mut store, 4, &0u32.to_le_bytes()).unwrap();
    assert!(is_tracker_handle(
        load.call(&mut store, (1024, len, 0, 3)).unwrap()
    ));
    store.data_mut().game.in_init = false;
    assert_eq!(load.call(&mut store, (1024, len, 0, 3)).unwrap(), 0);
    store.data_mut().game.in_init = true;
    let mut empty = data[..385].to_vec();
    empty[72..74].fill(0);
    memory.write(&mut store, 4096, &empty).unwrap();
    assert!(is_tracker_handle(
        old.call(&mut store, (4096, empty.len() as u32)).unwrap()
    ));
    // Existing ROM route uses the same module and supplied sample ordering.
    let mut pack = zx_common::ZXDataPack::default();
    pack.trackers.push(zx_common::PackedTracker::new(
        "test",
        TrackerFormat::Xm,
        nether_xm::pack_xm_minimal(&nether_xm::parse_xm(&data).unwrap()).unwrap(),
        vec!["a".into(), "b".into(), "c".into()],
    ));
    store.data_mut().ffi.data_pack = Some(Arc::new(pack));
    for (name, handle) in [("a", 1), ("b", 2), ("c", 3)] {
        store
            .data_mut()
            .ffi
            .sound_id_to_handle
            .insert(name.into(), handle);
    }
    memory.write(&mut store, 128, b"test").unwrap();
    let rom = instance
        .get_typed_func::<(u32, u32), u32>(&mut store, "rom")
        .unwrap();
    let rom_handle = rom.call(&mut store, (128, 4)).unwrap();
    assert!(is_tracker_handle(rom_handle));
    play.call(&mut store, rom_handle).unwrap();
    let ctx = store.data_mut();
    ctx.ffi
        .tracker_engine
        .sync_to_state(&ctx.rollback.tracker, &sounds);
    let mut audible = false;
    for _ in 0..2000 {
        let (l, r) = ctx.ffi.tracker_engine.render_sample_and_advance(
            &mut ctx.rollback.tracker,
            &sounds,
            44100,
        );
        audible |= l > 0.01 && r > 0.01;
    }
    assert!(audible);
}

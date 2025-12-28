//! Procedural asset generation library for Nethercore
//!
//! This library provides tools for procedural generation of meshes, textures, and audio
//! for use in Nethercore games. It builds on the low-level procedural primitives from
//! nethercore-zx and adds high-level modifiers and utilities.
//!
//! # Mesh Example
//! ```no_run
//! use proc_gen::mesh::*;
//!
//! // Generate a base mesh
//! let mut mesh: UnpackedMesh = generate_capsule(0.4, 0.6, 8, 4);
//!
//! // Apply modifiers
//! Transform::scale(1.0, 1.2, 0.8).apply(&mut mesh);
//! Mirror::default().apply(&mut mesh);
//! SmoothNormals::default().apply(&mut mesh);
//!
//! // Export to OBJ
//! write_obj(&mesh, "output.obj".as_ref(), "test")?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Texture Example
//! ```no_run
//! use proc_gen::texture::*;
//! use std::path::Path;
//!
//! // Generate a metal texture
//! let metal_tex = metal(256, 256, [180, 180, 200, 255], 42);
//!
//! // Apply modifiers
//! let mut tex = gradient_v(256, 256, [255, 200, 100, 255], [100, 50, 0, 255]);
//! tex.apply(Contrast { factor: 1.2 })
//!    .apply(Invert);
//!
//! // Export to PNG
//! write_png(&metal_tex, Path::new("metal.png"))?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Audio Example
//! ```no_run
//! use proc_gen::audio::*;
//!
//! // Create a synthesizer
//! let synth = Synth::new(SAMPLE_RATE);
//!
//! // Generate a laser sound effect
//! let laser = synth.laser();
//!
//! // Or create custom sounds
//! let custom = synth.sweep(Waveform::Saw, 800.0, 200.0, 0.2, Envelope::zap());
//!
//! // Convert to PCM i16 for ZX console
//! let pcm = to_pcm_i16(&custom);
//!
//! // Export to WAV for debugging (requires wav-export feature)
//! #[cfg(feature = "wav-export")]
//! write_wav(&pcm, SAMPLE_RATE, std::path::Path::new("laser.wav"))?;
//! # Ok::<(), std::io::Error>(())
//! ```

pub mod audio;
pub mod mesh;
pub mod texture;
pub mod sadl;

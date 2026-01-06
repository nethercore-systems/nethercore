use zx_common::TextureFormat;

pub fn render_mode_name(render_mode: u8) -> &'static str {
    match render_mode {
        0 => "Lambert",
        1 => "Matcap",
        2 => "PBR",
        3 => "Hybrid",
        _ => "Unknown",
    }
}

pub fn select_texture_format(compress_textures: bool) -> TextureFormat {
    if compress_textures {
        println!("  Texture compression: enabled (BC7, 4:1 ratio)");
        TextureFormat::Bc7
    } else {
        println!("  Texture compression: disabled (RGBA8, uncompressed)");
        TextureFormat::Rgba8
    }
}

pub fn warn_compression_mismatch(render_mode: u8, compress_textures: bool) {
    if render_mode > 0 && !compress_textures {
        eprintln!(
            "  WARNING: Detected render_mode {} (Matcap/PBR/Hybrid) but compress_textures=false.",
            render_mode
        );
        eprintln!("      Consider enabling texture compression for better performance:");
        eprintln!("      Add 'compress_textures = true' to [game] section in nether.toml");
    }

    if render_mode == 0 && compress_textures {
        eprintln!(
            "  WARNING: Detected render_mode 0 (Lambert) but compress_textures=true."
        );
        eprintln!("      Lambert mode works best with uncompressed RGBA8 textures.");
        eprintln!("      Consider setting 'compress_textures = false' in nether.toml");
    }
}

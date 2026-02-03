//! WGSL Metadata Parser
//!
//! Extracts `@epu_meta_begin`/`@epu_meta_end` blocks from WGSL files.
//! This module runs at build time (std available) and produces structured
//! data for code generation.
//!
//! # Usage
//!
//! This parser is designed to be used by `build.rs` to read WGSL opcode files
//! and generate Rust code for the epu-inspector game.
//!
//! ```ignore
//! // In build.rs:
//! #[path = "src/wgsl_meta_parser.rs"]
//! mod wgsl_meta_parser;
//!
//! use wgsl_meta_parser::parse_wgsl_meta;
//!
//! let source = std::fs::read_to_string("path/to/opcode.wgsl").unwrap();
//! let meta = parse_wgsl_meta(&source, "path/to/opcode.wgsl");
//! ```

use std::collections::HashMap;

/// Mapping types for field values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKind {
    /// u8 mapped to 0.0..1.0
    U8_01,
    /// u8 mapped via linear interpolation between min..max
    U8Lerp,
    /// u4 (nibble) mapped to 0.0..1.0
    U4_01,
    /// 16-bit octahedral direction encoding
    Dir16Oct,
}

impl MapKind {
    /// Parse a map kind from a string.
    ///
    /// # Panics
    /// Panics if the map kind is unknown.
    pub fn parse(s: &str) -> Self {
        match s {
            "u8_01" => MapKind::U8_01,
            "u8_lerp" => MapKind::U8Lerp,
            "u4_01" => MapKind::U4_01,
            "dir16_oct" => MapKind::Dir16Oct,
            _ => panic!("Unknown field mapping type: '{}'", s),
        }
    }
}

/// Kind of EPU opcode (bounds vs radiance).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeKind {
    /// Bounds opcode (defines regions)
    Bounds,
    /// Radiance opcode (additive feature layer)
    Radiance,
}

impl OpcodeKind {
    /// Parse an opcode kind from a string.
    ///
    /// # Panics
    /// Panics if the kind is unknown.
    pub fn parse(s: &str) -> Self {
        match s {
            "bounds" => OpcodeKind::Bounds,
            "radiance" => OpcodeKind::Radiance,
            _ => panic!("Unknown opcode kind: '{}' (expected 'bounds' or 'radiance')", s),
        }
    }
}

/// Specification for a single field in an EPU opcode.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldSpec {
    /// The raw field name (e.g., "intensity", "param_a", "direction")
    pub field_name: String,
    /// Human-readable label for the UI
    pub label: String,
    /// Optional unit string (e.g., "x", "tangent")
    pub unit: Option<String>,
    /// Mapping type for the field value
    pub map: MapKind,
    /// Minimum value for u8_lerp mapping (None for other mappings)
    pub min: Option<f32>,
    /// Maximum value for u8_lerp mapping (None for other mappings)
    pub max: Option<f32>,
}

/// Parsed metadata for a single EPU opcode.
#[derive(Debug, Clone, PartialEq)]
pub struct OpcodeMeta {
    /// Opcode number (0x00..0x1F)
    pub opcode: u8,
    /// Opcode name (e.g., "PLANE", "RAMP")
    pub name: String,
    /// Opcode kind (bounds or radiance)
    pub kind: OpcodeKind,
    /// Variant names (max 8)
    pub variants: Vec<String>,
    /// Domain names (max 4)
    pub domains: Vec<String>,
    /// Field specifications
    pub fields: Vec<FieldSpec>,
}

/// Parse a WGSL file and extract the EPU metadata block.
///
/// # Panics
/// - If no metadata block is found
/// - If multiple metadata blocks are found
/// - If any field mapping type is unknown
/// - If variant count > 8 or domain count > 4
/// - If opcode number is invalid (> 0x1F)
/// - If metadata block is unterminated (missing `@epu_meta_end`)
/// - If nested `@epu_meta_begin` is found
/// - If `u8_lerp` mapping is used without min/max values
pub fn parse_wgsl_meta(source: &str, file_path: &str) -> OpcodeMeta {
    let blocks = extract_meta_blocks(source, file_path);

    if blocks.is_empty() {
        panic!(
            "Missing metadata block in opcode file: {}\n\
             Expected a block starting with '// @epu_meta_begin' and ending with '// @epu_meta_end'",
            file_path
        );
    }

    if blocks.len() > 1 {
        panic!(
            "Multiple metadata blocks found in opcode file: {}\n\
             Each opcode file must contain exactly one metadata block",
            file_path
        );
    }

    parse_meta_block(&blocks[0], file_path)
}

/// Extract all `@epu_meta_begin`/`@epu_meta_end` blocks from source.
///
/// # Panics
/// - If a nested `@epu_meta_begin` is found (begin inside an existing block)
/// - If the file ends with an unterminated block (missing `@epu_meta_end`)
pub fn extract_meta_blocks(source: &str, file_path: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current_block = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed == "// @epu_meta_begin" {
            if in_block {
                panic!(
                    "Nested @epu_meta_begin found (already inside a metadata block) in file: {}",
                    file_path
                );
            }
            in_block = true;
            current_block.clear();
            continue;
        }

        if trimmed == "// @epu_meta_end" {
            if in_block {
                blocks.push(current_block.clone());
                current_block.clear();
            }
            in_block = false;
            continue;
        }

        if in_block {
            // Remove the leading "// " prefix if present
            let content = if let Some(stripped) = trimmed.strip_prefix("// ") {
                stripped
            } else if let Some(stripped) = trimmed.strip_prefix("//") {
                stripped
            } else {
                trimmed
            };
            current_block.push_str(content);
            current_block.push('\n');
        }
    }

    if in_block {
        panic!(
            "Unterminated @epu_meta_begin block (missing @epu_meta_end) in file: {}",
            file_path
        );
    }

    blocks
}

/// Parse a single metadata block into an OpcodeMeta.
fn parse_meta_block(block: &str, file_path: &str) -> OpcodeMeta {
    let mut opcode: Option<u8> = None;
    let mut name: Option<String> = None;
    let mut kind: Option<OpcodeKind> = None;
    let mut variants: Vec<String> = Vec::new();
    let mut domains: Vec<String> = Vec::new();
    let mut fields: Vec<FieldSpec> = Vec::new();

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("opcode = ") {
            opcode = Some(parse_opcode_number(rest.trim(), file_path));
        } else if let Some(rest) = line.strip_prefix("name = ") {
            name = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("kind = ") {
            kind = Some(OpcodeKind::parse(rest.trim()));
        } else if let Some(rest) = line.strip_prefix("variants = ") {
            variants = parse_string_array(rest.trim());
            if variants.len() > 8 {
                panic!(
                    "Variant count exceeds maximum of 8 in file: {}\n\
                     Found {} variants: {:?}",
                    file_path,
                    variants.len(),
                    variants
                );
            }
        } else if let Some(rest) = line.strip_prefix("domains = ") {
            domains = parse_string_array(rest.trim());
            if domains.len() > 4 {
                panic!(
                    "Domain count exceeds maximum of 4 in file: {}\n\
                     Found {} domains: {:?}",
                    file_path,
                    domains.len(),
                    domains
                );
            }
        } else if let Some(rest) = line.strip_prefix("field ") {
            fields.push(parse_field_spec(rest, file_path));
        }
    }

    // Validate required fields
    let opcode = opcode.unwrap_or_else(|| {
        panic!(
            "Missing 'opcode' field in metadata block in file: {}",
            file_path
        )
    });

    let name = name.unwrap_or_else(|| {
        panic!(
            "Missing 'name' field in metadata block in file: {}",
            file_path
        )
    });

    let kind = kind.unwrap_or_else(|| {
        panic!(
            "Missing 'kind' field in metadata block in file: {}",
            file_path
        )
    });

    OpcodeMeta {
        opcode,
        name,
        kind,
        variants,
        domains,
        fields,
    }
}

/// Parse an opcode number from a string (supports hex "0x0F" and decimal "15").
fn parse_opcode_number(s: &str, file_path: &str) -> u8 {
    let value = if let Some(hex) = s.strip_prefix("0x") {
        u8::from_str_radix(hex, 16).unwrap_or_else(|e| {
            panic!(
                "Invalid hex opcode number '{}' in file: {}: {}",
                s, file_path, e
            )
        })
    } else if let Some(hex) = s.strip_prefix("0X") {
        u8::from_str_radix(hex, 16).unwrap_or_else(|e| {
            panic!(
                "Invalid hex opcode number '{}' in file: {}: {}",
                s, file_path, e
            )
        })
    } else {
        s.parse::<u8>().unwrap_or_else(|e| {
            panic!(
                "Invalid decimal opcode number '{}' in file: {}: {}",
                s, file_path, e
            )
        })
    };

    if value > 0x1F {
        panic!(
            "Invalid opcode number {} (0x{:02X}) in file: {}\n\
             Opcode must be in range 0x00..0x1F (0..31)",
            value, value, file_path
        );
    }

    value
}

/// Parse a string array like `[TILES, HEX, STONE]` or `[]`.
pub fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim();

    // Handle empty array
    if s == "[]" {
        return Vec::new();
    }

    // Remove brackets
    let inner = s
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(s);

    if inner.trim().is_empty() {
        return Vec::new();
    }

    inner
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Parse a field specification like `intensity = { label="contrast", map="u8_01" }`.
pub fn parse_field_spec(s: &str, file_path: &str) -> FieldSpec {
    // Split on first '='
    let (field_name, spec_str) = s
        .split_once('=')
        .unwrap_or_else(|| panic!("Invalid field spec (missing '='): '{}' in {}", s, file_path));

    let field_name = field_name.trim().to_string();
    let spec_str = spec_str.trim();

    // Parse the spec object { ... }
    let inner = spec_str
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or_else(|| {
            panic!(
                "Invalid field spec (expected '{{ ... }}'): '{}' in {}",
                spec_str, file_path
            )
        })
        .trim();

    let props = parse_properties(inner);

    let label = props.get("label").cloned().unwrap_or_else(|| {
        panic!(
            "Missing 'label' in field spec for '{}' in {}",
            field_name, file_path
        )
    });

    let map_str = props.get("map").unwrap_or_else(|| {
        panic!(
            "Missing 'map' in field spec for '{}' in {}",
            field_name, file_path
        )
    });

    let map = MapKind::parse(map_str);

    let unit = props.get("unit").cloned();

    let min = props.get("min").map(|s| {
        s.parse::<f32>().unwrap_or_else(|e| {
            panic!(
                "Invalid 'min' value '{}' for field '{}' in {}: {}",
                s, field_name, file_path, e
            )
        })
    });

    let max = props.get("max").map(|s| {
        s.parse::<f32>().unwrap_or_else(|e| {
            panic!(
                "Invalid 'max' value '{}' for field '{}' in {}: {}",
                s, field_name, file_path, e
            )
        })
    });

    // Validate that u8_lerp mapping has min and max values
    if map == MapKind::U8Lerp && (min.is_none() || max.is_none()) {
        panic!(
            "Field '{}' uses u8_lerp mapping but is missing min/max values in {}",
            field_name, file_path
        );
    }

    FieldSpec {
        field_name,
        label,
        unit,
        map,
        min,
        max,
    }
}

/// Parse key="value" pairs from a comma-separated string.
fn parse_properties(s: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    let mut remaining = s.trim();

    while !remaining.is_empty() {
        // Find the key
        let eq_pos = remaining.find('=');
        if eq_pos.is_none() {
            break;
        }
        let eq_pos = eq_pos.unwrap();

        let key = remaining[..eq_pos].trim().to_string();
        remaining = remaining[eq_pos + 1..].trim();

        // Parse the value
        let value = if remaining.starts_with('"') {
            // Quoted string
            let end_quote = remaining[1..].find('"').map(|p| p + 1);
            if let Some(end) = end_quote {
                let val = remaining[1..end].to_string();
                remaining = remaining[end + 1..].trim();
                // Skip comma if present
                remaining = remaining.strip_prefix(',').unwrap_or(remaining).trim();
                val
            } else {
                // Unterminated quote - take rest
                let val = remaining[1..].to_string();
                remaining = "";
                val
            }
        } else {
            // Unquoted value (number or identifier)
            let end = remaining
                .find(|c: char| c == ',' || c.is_whitespace())
                .unwrap_or(remaining.len());
            let val = remaining[..end].to_string();
            remaining = remaining[end..].trim();
            // Skip comma if present
            remaining = remaining.strip_prefix(',').unwrap_or(remaining).trim();
            val
        };

        props.insert(key, value);
    }

    props
}

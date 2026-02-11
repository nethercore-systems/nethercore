//! Tests for WGSL metadata parser.
//!
//! These tests run with std available, separate from the no_std game library.

use zx_common::wgsl_meta_parser::*;

const VALID_PLANE_META: &str = r#"
// Some header comment
// ============================================================================

// @epu_meta_begin
// opcode = 0x0F
// name = PLANE
// kind = radiance
// variants = [TILES, HEX, STONE, SAND, WATER, GRATING, GRASS, PAVEMENT]
// domains = []
// field intensity = { label="contrast", map="u8_01" }
// field param_a   = { label="pattern_scale", unit="x", map="u8_lerp", min=0.5, max=16.0 }
// field param_b   = { label="gap_width", unit="tangent", map="u8_lerp", min=0.0, max=0.2 }
// field param_c   = { label="roughness", map="u8_01" }
// field param_d   = { label="phase", map="u8_01" }
// field direction = { label="normal", map="dir16_oct" }
// field alpha_a   = { label="coverage", map="u4_01" }
// @epu_meta_end

// Rest of the WGSL file...
fn eval_plane(...) { }
"#;

const VALID_RAMP_META: &str = r#"
// @epu_meta_begin
// opcode = 0x01
// name = RAMP
// kind = bounds
// variants = []
// domains = []
// field intensity = { label="softness", map="u8_lerp", min=0.01, max=0.5 }
// field param_a = { label="wall_r", map="u8_01" }
// field param_b = { label="wall_g", map="u8_01" }
// field param_c = { label="wall_b", map="u8_01" }
// field direction = { label="up", map="dir16_oct" }
// @epu_meta_end
"#;

#[test]
fn test_parse_valid_plane_meta() {
    let meta = parse_wgsl_meta(VALID_PLANE_META, "test_plane.wgsl");

    assert_eq!(meta.opcode, 0x0F);
    assert_eq!(meta.name, "PLANE");
    assert_eq!(meta.kind, OpcodeKind::Radiance);
    assert_eq!(
        meta.variants,
        vec![
            "TILES", "HEX", "STONE", "SAND", "WATER", "GRATING", "GRASS", "PAVEMENT"
        ]
    );
    assert!(meta.domains.is_empty());

    // Check fields
    assert_eq!(meta.fields.len(), 7);

    let intensity = meta
        .fields
        .iter()
        .find(|f| f.field_name == "intensity")
        .unwrap();
    assert_eq!(intensity.label, "contrast");
    assert_eq!(intensity.map, MapKind::U8_01);
    assert!(intensity.unit.is_none());
    assert!(intensity.min.is_none());
    assert!(intensity.max.is_none());

    let param_a = meta
        .fields
        .iter()
        .find(|f| f.field_name == "param_a")
        .unwrap();
    assert_eq!(param_a.label, "pattern_scale");
    assert_eq!(param_a.unit, Some("x".to_string()));
    assert_eq!(param_a.map, MapKind::U8Lerp);
    assert_eq!(param_a.min, Some(0.5));
    assert_eq!(param_a.max, Some(16.0));

    let direction = meta
        .fields
        .iter()
        .find(|f| f.field_name == "direction")
        .unwrap();
    assert_eq!(direction.label, "normal");
    assert_eq!(direction.map, MapKind::Dir16Oct);

    let alpha_a = meta
        .fields
        .iter()
        .find(|f| f.field_name == "alpha_a")
        .unwrap();
    assert_eq!(alpha_a.label, "coverage");
    assert_eq!(alpha_a.map, MapKind::U4_01);
}

#[test]
fn test_parse_valid_ramp_meta() {
    let meta = parse_wgsl_meta(VALID_RAMP_META, "test_ramp.wgsl");

    assert_eq!(meta.opcode, 0x01);
    assert_eq!(meta.name, "RAMP");
    assert_eq!(meta.kind, OpcodeKind::Bounds);
    assert!(meta.variants.is_empty());
    assert!(meta.domains.is_empty());

    let intensity = meta
        .fields
        .iter()
        .find(|f| f.field_name == "intensity")
        .unwrap();
    assert_eq!(intensity.label, "softness");
    assert_eq!(intensity.map, MapKind::U8Lerp);
    assert_eq!(intensity.min, Some(0.01));
    assert_eq!(intensity.max, Some(0.5));
}

#[test]
fn test_parse_decimal_opcode() {
    let source = r#"
// @epu_meta_begin
// opcode = 15
// name = PLANE
// kind = radiance
// variants = []
// domains = []
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.opcode, 15);
}

#[test]
#[should_panic(expected = "Missing metadata block")]
fn test_missing_meta_block() {
    let source = "// Just a regular WGSL file\nfn foo() {}";
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Multiple metadata blocks")]
fn test_multiple_meta_blocks() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end

// @epu_meta_begin
// opcode = 0x02
// name = BAR
// kind = radiance
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Unknown field mapping type")]
fn test_unknown_map_kind() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field intensity = { label="test", map="unknown_map" }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Variant count exceeds maximum of 8")]
fn test_too_many_variants() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = [A, B, C, D, E, F, G, H, I]
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Domain count exceeds maximum of 4")]
fn test_too_many_domains() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = [A, B, C, D, E]
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Invalid opcode number")]
fn test_invalid_opcode_too_large() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x20
// name = FOO
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Invalid opcode number")]
fn test_invalid_opcode_decimal_too_large() {
    let source = r#"
// @epu_meta_begin
// opcode = 32
// name = FOO
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Missing 'opcode' field")]
fn test_missing_opcode() {
    let source = r#"
// @epu_meta_begin
// name = FOO
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Missing 'name' field")]
fn test_missing_name() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Missing 'kind' field")]
fn test_missing_kind() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Unknown opcode kind")]
fn test_invalid_kind() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = unknown
// variants = []
// domains = []
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
fn test_parse_string_array_empty() {
    assert!(parse_string_array("[]").is_empty());
    assert!(parse_string_array("[ ]").is_empty());
}

#[test]
fn test_parse_string_array_single() {
    assert_eq!(parse_string_array("[FOO]"), vec!["FOO"]);
}

#[test]
fn test_parse_string_array_multiple() {
    assert_eq!(parse_string_array("[A, B, C]"), vec!["A", "B", "C"]);
}

#[test]
fn test_parse_string_array_whitespace() {
    assert_eq!(parse_string_array("[ A , B , C ]"), vec!["A", "B", "C"]);
}

#[test]
fn test_extract_meta_blocks_empty() {
    assert!(extract_meta_blocks("no meta here", "test.wgsl").is_empty());
}

#[test]
fn test_extract_meta_blocks_single() {
    let source = r#"
// @epu_meta_begin
// hello
// world
// @epu_meta_end
"#;
    let blocks = extract_meta_blocks(source, "test.wgsl");
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("hello"));
    assert!(blocks[0].contains("world"));
}

#[test]
fn test_field_spec_without_unit() {
    let spec = parse_field_spec("intensity = { label=\"test\", map=\"u8_01\" }", "test.wgsl");
    assert_eq!(spec.field_name, "intensity");
    assert_eq!(spec.label, "test");
    assert_eq!(spec.map, MapKind::U8_01);
    assert!(spec.unit.is_none());
    assert!(spec.min.is_none());
    assert!(spec.max.is_none());
}

#[test]
fn test_field_spec_with_all_props() {
    let spec = parse_field_spec(
        "param_a = { label=\"scale\", unit=\"x\", map=\"u8_lerp\", min=0.5, max=16.0 }",
        "test.wgsl",
    );
    assert_eq!(spec.field_name, "param_a");
    assert_eq!(spec.label, "scale");
    assert_eq!(spec.unit, Some("x".to_string()));
    assert_eq!(spec.map, MapKind::U8Lerp);
    assert_eq!(spec.min, Some(0.5));
    assert_eq!(spec.max, Some(16.0));
}

#[test]
fn test_map_kind_parse_all_types() {
    assert_eq!(MapKind::parse("u8_01"), MapKind::U8_01);
    assert_eq!(MapKind::parse("u8_lerp"), MapKind::U8Lerp);
    assert_eq!(MapKind::parse("u4_01"), MapKind::U4_01);
    assert_eq!(MapKind::parse("dir16_oct"), MapKind::Dir16Oct);
}

#[test]
fn test_opcode_kind_parse() {
    assert_eq!(OpcodeKind::parse("bounds"), OpcodeKind::Bounds);
    assert_eq!(OpcodeKind::parse("radiance"), OpcodeKind::Radiance);
}

#[test]
fn test_exactly_8_variants_is_ok() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = [A, B, C, D, E, F, G, H]
// domains = []
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.variants.len(), 8);
}

#[test]
fn test_exactly_4_domains_is_ok() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = [A, B, C, D]
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.domains.len(), 4);
}

#[test]
fn test_opcode_boundary_values() {
    // Test minimum opcode (0)
    let source = r#"
// @epu_meta_begin
// opcode = 0x00
// name = NOP
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.opcode, 0);

    // Test maximum opcode (31 = 0x1F)
    let source = r#"
// @epu_meta_begin
// opcode = 0x1F
// name = MAX
// kind = bounds
// variants = []
// domains = []
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.opcode, 0x1F);
}

#[test]
fn test_negative_min_max_values() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { label="offset", map="u8_lerp", min=-1.0, max=1.0 }
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    let param_a = meta
        .fields
        .iter()
        .find(|f| f.field_name == "param_a")
        .unwrap();
    assert_eq!(param_a.min, Some(-1.0));
    assert_eq!(param_a.max, Some(1.0));
}

#[test]
fn test_preserves_field_order() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field intensity = { label="a", map="u8_01" }
// field param_a = { label="b", map="u8_01" }
// field param_b = { label="c", map="u8_01" }
// field param_c = { label="d", map="u8_01" }
// @epu_meta_end
"#;
    let meta = parse_wgsl_meta(source, "test.wgsl");
    assert_eq!(meta.fields[0].field_name, "intensity");
    assert_eq!(meta.fields[1].field_name, "param_a");
    assert_eq!(meta.fields[2].field_name, "param_b");
    assert_eq!(meta.fields[3].field_name, "param_c");
}

#[test]
#[should_panic(expected = "Missing 'label'")]
fn test_field_missing_label() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { map="u8_01" }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Missing 'map'")]
fn test_field_missing_map() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { label="test" }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "uses u8_lerp mapping but is missing min/max values")]
fn test_u8_lerp_missing_min_max() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { label="test", map="u8_lerp" }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "uses u8_lerp mapping but is missing min/max values")]
fn test_u8_lerp_missing_max() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { label="test", map="u8_lerp", min=0.0 }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "uses u8_lerp mapping but is missing min/max values")]
fn test_u8_lerp_missing_min() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
// field param_a = { label="test", map="u8_lerp", max=1.0 }
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Unterminated @epu_meta_begin block")]
fn test_unterminated_meta_block() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// kind = bounds
// variants = []
// domains = []
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

#[test]
#[should_panic(expected = "Nested @epu_meta_begin found")]
fn test_nested_meta_begin() {
    let source = r#"
// @epu_meta_begin
// opcode = 0x01
// name = FOO
// @epu_meta_begin
// opcode = 0x02
// name = BAR
// @epu_meta_end
"#;
    parse_wgsl_meta(source, "test.wgsl");
}

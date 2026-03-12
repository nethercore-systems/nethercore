//! Capability and lint guidance for the EPU debug UI.
//!
//! These rules are intentionally lightweight and static. They surface the
//! practical authoring constraints we have learned from showcase iteration
//! without requiring shader metadata changes or a larger schema refactor.

#[derive(Clone, Debug, Default)]
pub struct CapabilityReport {
    pub best_for: Vec<&'static str>,
    pub cautions: Vec<&'static str>,
    pub warnings: Vec<&'static str>,
}

impl CapabilityReport {
    pub fn is_empty(&self) -> bool {
        self.best_for.is_empty() && self.cautions.is_empty() && self.warnings.is_empty()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

#[derive(Clone, Copy, Debug)]
struct CapabilityRule {
    opcode: Option<u8>,
    variant_id: Option<u8>,
    domain_id: Option<u8>,
    best_for: &'static [&'static str],
    cautions: &'static [&'static str],
    warnings: &'static [&'static str],
}

const fn rule(
    opcode: Option<u8>,
    variant_id: Option<u8>,
    domain_id: Option<u8>,
    best_for: &'static [&'static str],
    cautions: &'static [&'static str],
    warnings: &'static [&'static str],
) -> CapabilityRule {
    CapabilityRule {
        opcode,
        variant_id,
        domain_id,
        best_for,
        cautions,
        warnings,
    }
}

const RULES: &[CapabilityRule] = &[
    rule(
        Some(0x04),
        None,
        None,
        &["Hard scene organization, region retagging, large structural cuts"],
        &["Use it to define spatial read first, then let later features carry motion/detail"],
        &[],
    ),
    rule(
        Some(0x04),
        Some(6),
        None,
        &["TIER is good for stepped horizons, terraces, shelves, and layered far-field structure"],
        &["Best when later feature layers supply atmosphere, motion, or material read"],
        &[],
    ),
    rule(
        Some(0x04),
        Some(7),
        None,
        &["FACE is good for one dominant structural wall or far-field plane"],
        &["Use it to avoid repeated wall/floor striping when one face should dominate"],
        &[],
    ),
    rule(
        Some(0x07),
        None,
        None,
        &["Structural masks, arches, frames, and composition cuts"],
        &["Treat it as a space shaper first"],
        &["Not a readable world-feature carrier on its own"],
    ),
    rule(
        Some(0x09),
        None,
        None,
        &["Scaffolds, scan volumes, projected room structure"],
        &["Best when another layer supplies atmosphere or glow"],
        &[],
    ),
    rule(
        Some(0x0A),
        None,
        None,
        &["Sparkle, particulate glints, ambient speckle"],
        &["Use it as seasoning rather than the main scene read"],
        &["Seed-driven scatter; not a smooth primary motion carrier"],
    ),
    rule(
        Some(0x0B),
        None,
        None,
        &["Primary drift, lane motion, flowing energy, scan travel"],
        &["Strongest when paired with a simple structural carrier"],
        &[],
    ),
    rule(
        Some(0x0C),
        None,
        None,
        &["Etched filaments, crack lines, bolt silhouettes"],
        &["Reads better as an accent than a whole scene carrier"],
        &[],
    ),
    rule(
        Some(0x0C),
        Some(0),
        None,
        &[],
        &[],
        &["LIGHTNING is a static strike shape, not a phase-driven bolt animation"],
    ),
    rule(
        Some(0x0D),
        None,
        None,
        &["Sheets, curtains, haze walls, weather volumes"],
        &["Works best when the world-space silhouette is already legible"],
        &[],
    ),
    rule(
        Some(0x0D),
        Some(3),
        None,
        &["RAIN_WALL is a strong world-space weather carrier"],
        &[],
        &[],
    ),
    rule(
        Some(0x0F),
        None,
        None,
        &["Ground planes, surface beds, broad material carriers"],
        &["Use variant choice to lock the scene read early"],
        &[],
    ),
    rule(
        Some(0x0F),
        Some(4),
        None,
        &["WATER is a strong reflective surface and horizon bed"],
        &[],
        &[],
    ),
    rule(
        Some(0x11),
        None,
        None,
        &["Luminous volumes, rifts, gates, local projection anchors"],
        &["Portal shapes read locally unless another layer expands the space"],
        &[],
    ),
    rule(
        Some(0x11),
        Some(1),
        None,
        &["RECT is useful for rigid projection bays and framed volumes"],
        &["Keep supporting layers around it so it reads as world geometry"],
        &["RECT is a static local frame, not a strong standalone mover"],
    ),
    rule(
        Some(0x11),
        Some(3),
        None,
        &["VORTEX is a strong phase-driven mover for energy wells and pull"],
        &[],
        &[],
    ),
    rule(
        Some(0x12),
        None,
        None,
        &["Ambient glow volumes, focal pools, reflection/IBL shaping"],
        &["Excels at mood and room energy more than hard geometry"],
        &[],
    ),
    rule(
        Some(0x13),
        None,
        None,
        &["Accent rims, sweep trims, horizon highlights"],
        &["Use it to reinforce a scene read that already exists"],
        &["Not a general horizon scroller or broad travel carrier"],
    ),
    rule(
        Some(0x14),
        None,
        None,
        &["Subtle texture breakup, fog/body variation, anti-flat-fill support"],
        &["Best as a support carrier under a stronger structural or motion layer"],
        &["Not a structural bounds replacement or a hero motif by itself"],
    ),
    rule(
        Some(0x15),
        None,
        None,
        &["Broad transport masses, fog banks, spindrift, squall motion"],
        &["Use it as the main mover when the scene needs direct-view mass transport"],
        &["Do not ask it to replace bounds structure or rigid local projection geometry"],
    ),
    rule(
        Some(0x15),
        Some(1),
        None,
        &["SPINDRIFT is good for suspended snow, ash drift, and cold wind transport"],
        &["Pair it with a strong floor or ridge carrier so the place read survives"],
        &[],
    ),
    rule(
        Some(0x15),
        Some(2),
        None,
        &["SQUALL is good for storm fronts, dense haze walls, and broad weather masses"],
        &["Best when the water or floor read is already established underneath it"],
        &[],
    ),
    rule(
        Some(0x15),
        Some(4),
        None,
        &[
            "BANK is good for forceful wall-attached fronts, dense storm shelves, and one dominant moving weather body",
        ],
        &[
            "Use it when the scene needs the storm mass itself to read in direct view, not just supporting haze or streaks",
        ],
        &[
            "Still depends on bounds for horizon structure; it will not replace shelf/floor organization by itself",
        ],
    ),
    rule(
        Some(0x15),
        Some(5),
        None,
        &[
            "FRONT is good for one dominant storm shelf, scene-owning wall bodies, and broad weather fronts that must read directly",
        ],
        &[
            "Use it when the body itself must own the wall belt; keep transport and lightning subordinate",
        ],
        &[
            "Still needs bounds support for the overall horizon/floor split and is too heavy-handed for subtle haze",
        ],
    ),
    rule(
        Some(0x16),
        None,
        None,
        &[
            "Broad material identity, frozen sheets, crusted floors, and non-liquid surface response",
        ],
        &[
            "Use it to distinguish a surface from WATER without turning the opcode into a literal scene noun",
        ],
        &["Not a replacement for far-field bounds structure or direct-view weather transport"],
    ),
    rule(
        Some(0x16),
        Some(0),
        None,
        &["GLAZE is good for slick frozen sheens and broad polished beds"],
        &["Pair it with a subordinate breakup layer if the scene risks going too flat"],
        &[],
    ),
    rule(
        Some(0x16),
        Some(1),
        None,
        &["CRUST is good for broken plate fields, frosted seams, and uneven surface beds"],
        &["Works best as the structural counterweight to a smoother base surface layer"],
        &[],
    ),
    rule(
        None,
        None,
        Some(0),
        &["Domain 0 is usually the clearest starting point for world-space carriers"],
        &[],
        &[],
    ),
    rule(
        None,
        None,
        Some(1),
        &["AXIS_CYL favors wrapped shafts, columns, and cylindrical sweeps"],
        &["Less dependable for flat horizon or trench-floor motion"],
        &[],
    ),
    rule(
        None,
        None,
        Some(2),
        &["AXIS_POLAR favors radial and orbital reads"],
        &["Radial space usually reads weaker for planar travel"],
        &[],
    ),
    rule(
        None,
        None,
        Some(3),
        &["TANGENT_LOCAL anchors motion to a local surface frame"],
        &["Weaker for far-field skyline, ocean, and weather reads"],
        &[],
    ),
];

fn push_unique(target: &mut Vec<&'static str>, values: &[&'static str]) {
    for value in values {
        if !target.contains(value) {
            target.push(*value);
        }
    }
}

fn merge_report(target: &mut CapabilityReport, source: CapabilityReport) {
    push_unique(&mut target.best_for, source.best_for.as_slice());
    push_unique(&mut target.cautions, source.cautions.as_slice());
    push_unique(&mut target.warnings, source.warnings.as_slice());
}

fn collect_matching<F>(opcode: u8, variant_id: u8, domain_id: u8, predicate: F) -> CapabilityReport
where
    F: Fn(&CapabilityRule) -> bool,
{
    let mut report = CapabilityReport::default();

    for entry in RULES {
        let opcode_matches = entry.opcode.is_none() || entry.opcode == Some(opcode);
        let variant_matches = entry.variant_id.is_none() || entry.variant_id == Some(variant_id);
        let domain_matches = entry.domain_id.is_none() || entry.domain_id == Some(domain_id);

        if opcode_matches && variant_matches && domain_matches && predicate(entry) {
            push_unique(&mut report.best_for, entry.best_for);
            push_unique(&mut report.cautions, entry.cautions);
            push_unique(&mut report.warnings, entry.warnings);
        }
    }

    report
}

pub fn base_report(opcode: u8) -> CapabilityReport {
    collect_matching(opcode, 0, 0, |entry| {
        entry.variant_id.is_none() && entry.domain_id.is_none()
    })
}

pub fn variant_report(opcode: u8, variant_id: u8) -> CapabilityReport {
    collect_matching(opcode, variant_id, 0, |entry| {
        entry.variant_id == Some(variant_id) && entry.domain_id.is_none()
    })
}

pub fn domain_report(opcode: u8, domain_id: u8) -> CapabilityReport {
    collect_matching(opcode, 0, domain_id, |entry| {
        entry.domain_id == Some(domain_id) && entry.variant_id.is_none()
    })
}

fn exact_report(opcode: u8, variant_id: u8, domain_id: u8) -> CapabilityReport {
    collect_matching(opcode, variant_id, domain_id, |entry| {
        entry.variant_id == Some(variant_id) && entry.domain_id == Some(domain_id)
    })
}

pub fn report_for(opcode: u8, variant_id: u8, domain_id: u8) -> CapabilityReport {
    let mut report = base_report(opcode);
    merge_report(&mut report, variant_report(opcode, variant_id));
    merge_report(&mut report, domain_report(opcode, domain_id));
    merge_report(&mut report, exact_report(opcode, variant_id, domain_id));
    report
}

fn tone_color(kind: &str) -> egui::Color32 {
    match kind {
        "Best for" => egui::Color32::from_rgb(140, 220, 160),
        "Caution" => egui::Color32::from_rgb(230, 210, 120),
        "Warning" => egui::Color32::from_rgb(255, 140, 140),
        _ => egui::Color32::LIGHT_GRAY,
    }
}

fn render_row(ui: &mut egui::Ui, label: &str, values: &[&str]) {
    if values.is_empty() {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(format!("{}:", label)).color(tone_color(label)));
        ui.label(values.join(" | "));
    });
}

pub fn render_report(ui: &mut egui::Ui, report: &CapabilityReport) {
    render_row(ui, "Best for", &report.best_for);
    render_row(ui, "Caution", &report.cautions);
    render_row(ui, "Warning", &report.warnings);
}

pub fn render_compact_report(
    ui: &mut egui::Ui,
    report: &CapabilityReport,
    max_cautions: usize,
    max_warnings: usize,
) {
    if let Some(primary) = report.best_for.first() {
        ui.label(
            egui::RichText::new(format!("Use: {}", primary))
                .small()
                .color(tone_color("Best for")),
        );
    }

    for caution in report.cautions.iter().take(max_cautions) {
        ui.label(
            egui::RichText::new(format!("Caution: {}", caution))
                .small()
                .color(tone_color("Caution")),
        );
    }

    for warning in report.warnings.iter().take(max_warnings) {
        ui.label(
            egui::RichText::new(format!("Warning: {}", warning))
                .small()
                .color(tone_color("Warning")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{base_report, report_for, variant_report};

    #[test]
    fn aperture_reports_structural_warning() {
        let report = base_report(0x07);
        assert!(
            report
                .warnings
                .contains(&"Not a readable world-feature carrier on its own")
        );
    }

    #[test]
    fn portal_rect_reports_static_frame_warning() {
        let report = variant_report(0x11, 1);
        assert!(
            report
                .warnings
                .contains(&"RECT is a static local frame, not a strong standalone mover")
        );
    }

    #[test]
    fn portal_vortex_reports_positive_motion_guidance() {
        let report = report_for(0x11, 3, 0);
        assert!(
            report
                .best_for
                .contains(&"VORTEX is a strong phase-driven mover for energy wells and pull")
        );
    }

    #[test]
    fn direct3d_domain_guidance_is_merged_into_full_report() {
        let report = report_for(0x0C, 0, 0);
        assert!(
            report.best_for.contains(
                &"Domain 0 is usually the clearest starting point for world-space carriers"
            )
        );
    }

    #[test]
    fn advect_bank_reports_front_body_guidance() {
        let report = report_for(0x15, 4, 0);
        assert!(
            report.best_for.contains(
                &"BANK is good for forceful wall-attached fronts, dense storm shelves, and one dominant moving weather body"
            )
        );
    }
}

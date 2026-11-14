use crate::app::{AppTab, NavCommand};
use crate::theme::AcademicTheme;
use calc_core::{
    CurriculumCatalog, ModuleMastery, ModuleSpec, ProblemGenerator, ScienceDomain,
};
use egui::{Frame, Margin, RichText, Rounding, Ui};
use std::collections::HashMap;

pub fn render_lesson(
    ui: &mut Ui,
    spec: &ModuleSpec,
    domain: ScienceDomain,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    let mut cmd = None;
    let accent = theme.stage_accent(spec.stage);
    let mod_mastery = mastery
        .get(spec.id)
        .map(|m| m.mastery_score)
        .unwrap_or(0.0);

    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new(format!("{} | {}", spec.stage.short_label(), spec.code))
                .monospace()
                .strong()
                .color(accent),
        );
        ui.separator();
        ui.label(format!("Current Mastery: {:.0}%", mod_mastery * 100.0));
        ui.separator();
        ui.label(format!("Active Domain: {}", domain.label()));
    });

    ui.heading(spec.title);
    ui.label(RichText::new(spec.subtitle).italics());
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Launch Learning or Timed Run").clicked() {
            cmd = Some(NavCommand::OpenModule {
                module_id: spec.id.to_string(),
                tab: AppTab::Practice,
            });
        }
        if ui.button("Launch Linked Visual Lab Preset").clicked() {
            cmd = Some(NavCommand::OpenLabPreset(spec.id.to_string()));
        }
    });

    if let Some(domain_note) = domain.context_banner(spec.stage, spec.id) {
        ui.add_space(8.0);
        Frame::none()
            .fill(theme.card_fill())
            .stroke(egui::Stroke::new(1.2, accent))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("[{}] {}", domain.short_tag(), domain_note))
                        .strong()
                        .color(accent),
                );
            });
    }

    ui.add_space(10.0);

    // Conceptual Overview Card
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Conceptual Synthesis").strong().color(accent));
            ui.add_space(4.0);
            ui.label(spec.summary);
        });

    ui.add_space(10.0);

    // Prerequisites & Downstream Bridge
    ui.columns(2, |cols| {
        Frame::none()
            .fill(theme.card_fill())
            .stroke(theme.card_stroke())
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(12.0))
            .show(&mut cols[0], |ui| {
                ui.label(RichText::new("Upstream Prerequisites").strong());
                ui.add_space(4.0);
                if spec.prerequisites.is_empty() {
                    ui.label("Entry foundation module (no prior module prerequisites).");
                } else {
                    for &pid in spec.prerequisites {
                        if let Some(p_mod) = catalog.get(pid) {
                            let p_score = mastery
                                .get(pid)
                                .map(|m| m.mastery_score * 100.0)
                                .unwrap_or(0.0);
                            ui.horizontal(|ui| {
                                if ui
                                    .small_button(format!("{} {}", p_mod.code, p_mod.title))
                                    .clicked()
                                {
                                    cmd = Some(NavCommand::OpenModule {
                                        module_id: p_mod.id.to_string(),
                                        tab: AppTab::Lesson,
                                    });
                                }
                                ui.label(
                                    RichText::new(format!("({:.0}%)", p_score))
                                        .small()
                                        .monospace(),
                                );
                            });
                        }
                    }
                }
            });

        Frame::none()
            .fill(theme.card_fill())
            .stroke(theme.card_stroke())
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(12.0))
            .show(&mut cols[1], |ui| {
                ui.label(
                    RichText::new("Downstream Mathematical & Scientific Bridge")
                        .strong()
                        .color(accent),
                );
                ui.add_space(4.0);
                ui.label(spec.calculus_bridge);
            });
    });

    ui.add_space(10.0);

    // First-Principles Mechanics
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new("First-Principles Notes").strong());
            ui.add_space(4.0);
            for (idx, note) in spec.first_principles_notes.iter().enumerate() {
                ui.label(format!("{}. {}", idx + 1, note));
                ui.add_space(3.0);
            }
        });

    ui.add_space(10.0);

    // Plain-Text Reference Identities
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Core Plain-Text Identities & Relations").strong());
            ui.add_space(4.0);
            for ident in spec.key_identities {
                ui.label(RichText::new(*ident).monospace());
            }
        });

    ui.add_space(10.0);

    // Worked Reference Example with Named Method Steps
    let sample = ProblemGenerator::generate_with_domain(spec.id, 101, domain);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("Worked Example ({}) — {}", domain.short_tag(), sample.title))
                    .strong()
                    .color(accent),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(&sample.prompt).monospace());
            ui.add_space(6.0);
            for st in &sample.steps {
                ui.label(
                    RichText::new(format!("Step {} [{}]:", st.step_number, st.method_name))
                        .strong(),
                );
                ui.label(format!("  {}", st.explanation));
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("Canonical Answer: {}", sample.answer.canonical()))
                    .monospace()
                    .strong()
                    .color(theme.ok_color()),
            );
        });

    cmd
}

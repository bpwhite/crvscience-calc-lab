// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use crate::app::{AppTab, NavCommand};
use crate::math_render;
use crate::theme::AcademicTheme;
use calc_core::{
    CurriculumCatalog, ModuleMastery, ModuleSpec, ProblemGenerator, ScienceDomain,
};
use egui::{Color32, Frame, Margin, RichText, Rounding, Stroke, Ui};
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

    ui.heading(math_render::ui_safe_text(spec.title));
    ui.label(RichText::new(math_render::ui_safe_text(spec.subtitle)).italics());
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
            .stroke(egui::Stroke::new(1.2_f32, accent))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "[{}] {}",
                        domain.short_tag(),
                        math_render::ui_safe_text(&domain_note)
                    ))
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
            ui.label(math_render::ui_safe_text(spec.summary));
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
                                    .small_button(format!(
                                        "{} {}",
                                        p_mod.code,
                                        math_render::ui_safe_text(p_mod.title)
                                    ))
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
                ui.label(math_render::ui_safe_text(spec.calculus_bridge));
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
                ui.label(format!("{}. {}", idx + 1, math_render::ui_safe_text(note)));
                ui.add_space(3.0);
            }
        });

    ui.add_space(10.0);

    // Reference Mathematical Identities & Relations
    let (eq_bg, text_fg) = match theme {
        AcademicTheme::SlateDark => (
            Color32::from_rgb(20, 24, 32),
            Color32::from_rgb(236, 242, 250),
        ),
        AcademicTheme::PaperLight => (
            Color32::from_rgb(248, 247, 242),
            Color32::from_rgb(26, 30, 38),
        ),
    };
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Core Mathematical Identities & Relations").strong());
            ui.add_space(6.0);
            for ident in spec.key_identities {
                let split_idx = ident
                    .find(": ")
                    .filter(|&i| !ident[..i].contains('{') && !ident[..i].contains('('));
                if let Some(i) = split_idx {
                    let prefix = &ident[..=i];
                    let formula = ident[i + 2..].trim();
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(prefix).strong().small().color(accent));
                        let tex = math_render::unicode_math_to_latex(formula);
                        math_render::paint_latex(ui, &tex, 15.0, text_fg);
                    });
                } else {
                    let tex = math_render::unicode_math_to_latex(ident);
                    math_render::paint_latex(ui, &tex, 15.0, text_fg);
                }
                ui.add_space(3.0);
            }
        });

    ui.add_space(10.0);

    // Worked Reference Example with 2D Structured Derivations
    let sample = ProblemGenerator::generate_with_domain(spec.id, 101, domain);
    let [op_r, op_g, op_b] = sample.operation.accent_rgb();
    let op_col = Color32::from_rgb(op_r, op_g, op_b);
    let badge_fill = Color32::from_rgba_unmultiplied(op_r, op_g, op_b, 36);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                Frame::none()
                    .fill(badge_fill)
                    .stroke(Stroke::new(1.1_f32, op_col))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "OPERATION: {}",
                                sample.operation.category_badge()
                            ))
                            .small()
                            .strong()
                            .color(op_col),
                        );
                    });
                ui.label(
                    RichText::new(format!(
                        "Worked Example ({}) — {}",
                        domain.short_tag(),
                        math_render::ui_safe_text(&sample.title)
                    ))
                    .strong()
                    .color(accent),
                );
            });
            ui.add_space(4.0);
            ui.label(math_render::ui_safe_text(&sample.prompt));
            ui.add_space(4.0);
            math_render::paint_latex_boxed(
                ui,
                &sample.target_equation_latex,
                17.5,
                text_fg,
                eq_bg,
                Stroke::new(1.1_f32, op_col.linear_multiply(0.75)),
            );
            ui.add_space(6.0);
            for st in &sample.steps {
                Frame::none()
                    .fill(eq_bg)
                    .stroke(theme.card_stroke())
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "Step {} · {}",
                                st.step_number,
                                math_render::ui_safe_text(&st.rule_name)
                            ))
                            .strong()
                            .color(accent),
                        );
                        ui.label(math_render::ui_safe_text(&st.explanation));
                        ui.add_space(2.0);
                        ui.horizontal_wrapped(|ui| {
                            math_render::paint_latex_boxed(
                                ui,
                                &st.before_latex,
                                14.5,
                                text_fg,
                                theme.card_fill(),
                                theme.card_stroke(),
                            );
                            math_render::paint_transformation_arrow(ui, 15.0, op_col);
                            math_render::paint_latex_boxed(
                                ui,
                                &st.after_latex,
                                14.5,
                                theme.ok_color(),
                                theme.card_fill(),
                                Stroke::new(1.0_f32, theme.ok_color().linear_multiply(0.65)),
                            );
                        });
                    });
                ui.add_space(4.0);
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Final Solution:").strong().color(theme.ok_color()));
                math_render::paint_latex_boxed(
                    ui,
                    &sample.answer_latex,
                    16.5,
                    theme.ok_color(),
                    eq_bg,
                    Stroke::new(1.2_f32, theme.ok_color()),
                );
            });
        });

    cmd
}

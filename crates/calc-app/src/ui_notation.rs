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

use crate::app::NavCommand;
use crate::math_render;
use crate::theme::AcademicTheme;
use calc_core::{
    build_notation_lexicon, generate_notation_flash_question, NotationEntry,
    NotationFlashQuestion, ScienceDomain, Stage,
};
use egui::{
    Align, Color32, Frame, Key, Layout, Margin, RichText, Rounding, ScrollArea, Stroke, TextEdit,
    Ui,
};

pub struct NotationStudioState {
    pub lexicon: Vec<NotationEntry>,
    pub selected_idx: usize,
    pub filter_query: String,
    pub stage_filter: Option<Stage>,
    pub active_flash: NotationFlashQuestion,
    pub selected_choice: Option<usize>,
    pub flash_seed: u64,
    pub flash_streak: u32,
    pub flash_correct_total: u32,
}

impl Default for NotationStudioState {
    fn default() -> Self {
        Self {
            lexicon: build_notation_lexicon(),
            selected_idx: 0,
            filter_query: String::new(),
            stage_filter: None,
            active_flash: generate_notation_flash_question(0),
            selected_choice: None,
            flash_seed: 0,
            flash_streak: 0,
            flash_correct_total: 0,
        }
    }
}

impl NotationStudioState {
    pub fn advance_flash_question(&mut self) {
        self.flash_seed = self.flash_seed.wrapping_add(1);
        self.active_flash = generate_notation_flash_question(self.flash_seed);
        self.selected_choice = None;
    }
}

pub fn render_notation_studio(
    ui: &mut Ui,
    state: &mut NotationStudioState,
    domain: ScienceDomain,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    let mut cmd = None;
    let accent = theme.stage_accent(Stage::Notation);

    ui.horizontal_wrapped(|ui| {
        ui.heading("Notation Studio — Decoding Textbook Symbols into Plain-Text Operations");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button("Launch Symbolic Run on Stage 0 (NOT-001..006)")
                .clicked()
            {
                cmd = Some(NavCommand::StartStageRun(Stage::Notation));
            }
        });
    });
    ui.label(
        "Inspect every subscript, bound, operator, and dialect side-by-side with plain-text syntax and scientific domain meaning, then test rapid recognition below.",
    );
    ui.add_space(6.0);

    // Filter Strip
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search Symbol / Term:");
                ui.add(
                    TextEdit::singleline(&mut state.filter_query)
                        .desired_width(210.0)
                        .hint_text("e.g. nabla, sigma, argmax, Leibniz"),
                );
                ui.separator();
                if ui
                    .selectable_label(state.stage_filter.is_none(), "All Stages")
                    .clicked()
                {
                    state.stage_filter = None;
                }
                for st in [
                    Stage::Notation,
                    Stage::Calculus,
                    Stage::LinearAlgebra,
                    Stage::AdvancedCalculus,
                    Stage::DifferentialEquations,
                    Stage::ProbabilityStats,
                ] {
                    if ui
                        .selectable_label(state.stage_filter == Some(st), st.short_label())
                        .clicked()
                    {
                        state.stage_filter = Some(st);
                    }
                }
            });
        });

    ui.add_space(8.0);

    let q_lower = state.filter_query.trim().to_lowercase();
    let filtered_indices: Vec<usize> = state
        .lexicon
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            let stage_ok = state
                .stage_filter
                .map(|sf| sf == entry.stage)
                .unwrap_or(true);
            let query_ok = q_lower.is_empty()
                || entry.name.to_lowercase().contains(&q_lower)
                || entry.symbol_display.to_lowercase().contains(&q_lower)
                || entry.plain_text_form.to_lowercase().contains(&q_lower)
                || entry.spoken_phrasing.to_lowercase().contains(&q_lower);
            stage_ok && query_ok
        })
        .map(|(i, _)| i)
        .collect();

    if !filtered_indices.is_empty() && !filtered_indices.contains(&state.selected_idx) {
        state.selected_idx = filtered_indices[0];
    }

    ui.columns(2, |cols| {
        // Left Column: Symbol Lexicon Cards + Flash Recognition Trainer
        cols[0].with_layout(Layout::top_down(Align::Min), |ui| {
            Frame::none()
                .fill(theme.card_fill())
                .stroke(theme.card_stroke())
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Symbol & Operator Lexicon ({} entries)",
                            filtered_indices.len()
                        ))
                        .strong(),
                    );
                    ui.separator();
                    ui.push_id("notation_lexicon_scroll", |ui| {
                        ScrollArea::vertical().max_height(265.0).show(ui, |ui| {
                            for &idx in &filtered_indices {
                                let entry = &state.lexicon[idx];
                                let is_sel = idx == state.selected_idx;
                                let st_col = theme.stage_accent(entry.stage);
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(format!("[{}]", entry.stage.short_label()))
                                            .small()
                                            .monospace()
                                            .color(st_col),
                                    );
                                    if ui
                                        .selectable_label(
                                            is_sel,
                                            math_render::ui_safe_text(entry.name),
                                        )
                                        .clicked()
                                    {
                                        state.selected_idx = idx;
                                    }
                                });
                            }
                        });
                    });
                });

            ui.add_space(10.0);

            // Rapid Symbol Recognition Mini-Drill
            Frame::none()
                .fill(theme.card_fill())
                .stroke(egui::Stroke::new(1.3_f32, accent))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new("Rapid Notation Recognition Drill")
                                .strong()
                                .color(accent),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!(
                                "Streak: {}  |  Total Solved: {}",
                                state.flash_streak, state.flash_correct_total
                            ))
                            .monospace()
                            .small(),
                        );
                    });
                    ui.add_space(4.0);
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
                    let flash_tex =
                        math_render::unicode_math_to_latex(&state.active_flash.symbol_banner);
                    math_render::paint_latex_boxed(
                        ui,
                        &flash_tex,
                        17.0,
                        text_fg,
                        eq_bg,
                        theme.card_stroke(),
                    );
                    ui.add_space(4.0);
                    ui.label(math_render::ui_safe_text(&state.active_flash.question));
                    ui.add_space(6.0);

                    for c_idx in 0..4 {
                        let choice_txt = &state.active_flash.choices[c_idx];
                        let is_picked = state.selected_choice == Some(c_idx);
                        let mut btn_rt = RichText::new(format!(
                            "{}. {}",
                            (b'A' + c_idx as u8) as char,
                            math_render::ui_safe_text(choice_txt)
                        ));
                        if let Some(picked) = state.selected_choice {
                            if c_idx == state.active_flash.correct_index {
                                btn_rt = btn_rt.color(theme.ok_color()).strong();
                            } else if picked == c_idx {
                                btn_rt = btn_rt.color(theme.err_color());
                            }
                        }

                        if ui.selectable_label(is_picked, btn_rt).clicked()
                            && state.selected_choice.is_none()
                        {
                            state.selected_choice = Some(c_idx);
                            if c_idx == state.active_flash.correct_index {
                                state.flash_streak += 1;
                                state.flash_correct_total += 1;
                            } else {
                                state.flash_streak = 0;
                            }
                        }
                    }

                    if let Some(picked) = state.selected_choice {
                        ui.add_space(6.0);
                        let ok = picked == state.active_flash.correct_index;
                        ui.label(
                            RichText::new(if ok {
                                "Correct!"
                            } else {
                                "Review Symbol Anatomy:"
                            })
                            .strong()
                            .color(if ok {
                                theme.ok_color()
                            } else {
                                theme.warn_color()
                            }),
                        );
                        ui.label(
                            RichText::new(math_render::ui_safe_text(&state.active_flash.explanation))
                                .small(),
                        );
                        ui.add_space(4.0);
                        let enter_advance = !ui.ctx().wants_keyboard_input()
                            && ui.input(|i| i.key_pressed(Key::Enter));
                        if ui.button("Next Symbol Challenge >> (Enter)").clicked() || enter_advance
                        {
                            state.advance_flash_question();
                        }
                    }
                });
        });

        // Right Column: Visual Anatomy & Plain-Text Decoder Card
        cols[1].with_layout(Layout::top_down(Align::Min), |ui| {
            if let Some(entry) = state.lexicon.get(state.selected_idx) {
                let st_accent = theme.stage_accent(entry.stage);
                Frame::none()
                    .fill(theme.card_fill())
                    .stroke(egui::Stroke::new(1.5_f32, st_accent))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} Notation Anatomy",
                                entry.stage.short_label()
                            ))
                            .small()
                            .strong()
                            .color(st_accent),
                        );
                        ui.heading(math_render::ui_safe_text(entry.name));
                        ui.add_space(6.0);

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
                            .rounding(Rounding::same(5.0))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                let sym_tex =
                                    math_render::unicode_math_to_latex(entry.symbol_display);
                                math_render::paint_latex_boxed(
                                    ui,
                                    &sym_tex,
                                    20.0,
                                    st_accent,
                                    eq_bg,
                                    Stroke::new(1.2_f32, st_accent.linear_multiply(0.7)),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new(format!(
                                        "Plain-Text CalcLab Syntax:  {}",
                                        entry.plain_text_form
                                    ))
                                    .monospace(),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "Read Aloud:  {}",
                                        math_render::ui_safe_text(entry.spoken_phrasing)
                                    ))
                                    .italics(),
                                );
                            });

                        ui.add_space(8.0);
                        ui.label(RichText::new("Structural Anatomy (Piece-by-Piece):").strong());
                        for pt in entry.anatomy_points {
                            ui.label(format!("* {}", math_render::ui_safe_text(pt)));
                        }

                        ui.add_space(8.0);
                        ui.label(RichText::new("Equivalent Notational Dialects:").strong());
                        for eq in entry.dialect_equivalents {
                            let split_idx = eq
                                .find(": ")
                                .filter(|&i| !eq[..i].contains('{') && !eq[..i].contains('('));
                            if let Some(i) = split_idx {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(&eq[..=i])
                                            .small()
                                            .strong()
                                            .color(st_accent),
                                    );
                                    let tex =
                                        math_render::unicode_math_to_latex(eq[i + 2..].trim());
                                    math_render::paint_latex(ui, &tex, 14.5, text_fg);
                                });
                            } else {
                                let tex = math_render::unicode_math_to_latex(eq);
                                math_render::paint_latex(ui, &tex, 14.5, text_fg);
                            }
                        }

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "Domain Interpretation ({}):",
                                domain.label()
                            ))
                            .strong()
                            .color(st_accent),
                        );
                        ui.label(math_render::ui_safe_text(entry.domain_application(domain)));

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Frequent Pitfall to Avoid:")
                                .strong()
                                .color(theme.warn_color()),
                        );
                        ui.label(math_render::ui_safe_text(entry.common_pitfall));
                    });
            }
        });
    });

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notation_studio_advance_flash_question_resets_choice_and_cycles_seed() {
        let mut state = NotationStudioState::default();
        assert_eq!(state.flash_seed, 0);
        assert!(state.selected_choice.is_none());

        state.selected_choice = Some(1);
        state.advance_flash_question();
        assert_eq!(state.flash_seed, 1);
        assert!(state.selected_choice.is_none());
    }
}

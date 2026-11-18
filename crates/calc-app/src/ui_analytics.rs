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

use crate::math_render;
use crate::theme::AcademicTheme;
use calc_core::{
    summarize_stages, AttemptRecord, CurriculumCatalog, LearningRunRecord, ModuleMastery, RunMode,
};
use chrono::Utc;
use egui::{Frame, Grid, Margin, RichText, Rounding, Ui};
use egui_plot::{Bar, BarChart, Plot};
use std::collections::HashMap;

pub enum AnalyticsAction {
    ExportJson,
    ResetProfile,
}

#[derive(Default)]
pub struct AnalyticsViewState {
    pub confirm_reset: bool,
    pub status_banner: Option<String>,
    pub status_is_error: bool,
}

pub fn render_analytics(
    ui: &mut Ui,
    view_state: &mut AnalyticsViewState,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    learning_runs: &[LearningRunRecord],
    recent_attempts: &[AttemptRecord],
    theme: AcademicTheme,
) -> Option<AnalyticsAction> {
    let mut action = None;
    let now = Utc::now();
    let stage_summaries = summarize_stages(catalog, mastery, now);

    ui.horizontal_wrapped(|ui| {
        ui.heading("Run Leaderboard, Mastery Telemetry & Spaced-Repetition Ledger");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !view_state.confirm_reset {
                if ui.button("Reset Profile...").clicked() {
                    view_state.confirm_reset = true;
                }
            } else {
                if ui.button("Cancel").clicked() {
                    view_state.confirm_reset = false;
                }
                if ui
                    .button(RichText::new("Confirm Erase All").color(theme.err_color()))
                    .clicked()
                {
                    view_state.confirm_reset = false;
                    action = Some(AnalyticsAction::ResetProfile);
                }
            }

            if ui.button("Export JSON Snapshot").clicked() {
                action = Some(AnalyticsAction::ExportJson);
            }
        });
    });

    if let Some(msg) = &view_state.status_banner {
        let banner_col = if view_state.status_is_error {
            theme.err_color()
        } else {
            theme.ok_color()
        };
        ui.label(RichText::new(msg).color(banner_col).monospace());
    }

    ui.add_space(6.0);

    // High-Score Summary Across the 4 Game/Learning Modes
    ui.columns(4, |cols| {
        for (idx, mode) in RunMode::ALL.iter().enumerate() {
            let code = mode.short_code();
            let matching: Vec<&LearningRunRecord> =
                learning_runs.iter().filter(|r| r.mode == code).collect();
            let best_score = matching.iter().map(|r| r.score).max().unwrap_or(0);
            let best_streak = matching.iter().map(|r| r.max_streak).max().unwrap_or(0);

            Frame::none()
                .fill(theme.card_fill())
                .stroke(theme.card_stroke())
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(&mut cols[idx], |ui| {
                    ui.label(RichText::new(mode.label()).strong());
                    ui.label(
                        RichText::new(format!("High Score: {} pts", best_score))
                            .monospace()
                            .color(theme.ok_color()),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Runs: {}  |  Max Streak: {}",
                            matching.len(),
                            best_streak
                        ))
                        .small()
                        .monospace(),
                    );
                });
        }
    });

    ui.add_space(8.0);

    // Learning & Timed Run History Table
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Completed Learning & Timed Run History").strong());
            ui.add_space(4.0);
            if learning_runs.is_empty() {
                ui.label(
                    "No completed runs logged yet. Finish a Quick Blitz (90s), Medium (5m), Infinite, or Learning Mode run in the Training Arena to record scores here.",
                );
            } else {
                Grid::new("learning_runs_history_grid")
                    .striped(true)
                    .num_columns(7)
                    .spacing([14.0, 5.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("UTC Time").strong());
                        ui.label(RichText::new("Mode").strong());
                        ui.label(RichText::new("Domain").strong());
                        ui.label(RichText::new("Scope").strong());
                        ui.label(RichText::new("Score").strong());
                        ui.label(RichText::new("Solved").strong());
                        ui.label(RichText::new("Streak / Time").strong());
                        ui.end_row();

                        for run in learning_runs.iter().take(20) {
                            ui.label(
                                RichText::new(
                                    run.finished_at_utc.format("%Y-%m-%d %H:%M").to_string(),
                                )
                                .monospace()
                                .small(),
                            );
                            ui.label(RichText::new(&run.mode).monospace().strong());
                            ui.label(&run.domain);
                            ui.label(RichText::new(&run.stage_scope).small());
                            ui.label(
                                RichText::new(format!("{} pts", run.score))
                                    .monospace()
                                    .color(theme.ok_color()),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{}/{} ({:.0}%)",
                                    run.solved_count,
                                    run.attempted_count,
                                    run.accuracy_pct()
                                ))
                                .monospace(),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "x{} / {:.0}s",
                                    run.max_streak, run.elapsed_secs
                                ))
                                .monospace()
                                .small(),
                            );
                            ui.end_row();
                        }
                    });
            }
        });

    ui.add_space(10.0);

    // Stage Aggregates & 42-Module Mastery Bar Chart
    let total_attempts: u32 = mastery.values().map(|m| m.attempts).sum();
    let total_correct: u32 = mastery.values().map(|m| m.correct_count).sum();
    let overall_acc = if total_attempts == 0 {
        0.0
    } else {
        100.0 * (total_correct as f64) / (total_attempts as f64)
    };

    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "Total Graded Problems: {}  |  Correct: {} ({:.1}%)  |  Stages Tracked: {}  |  Modules: {}",
                    total_attempts,
                    total_correct,
                    overall_acc,
                    stage_summaries.len(),
                    catalog.modules.len()
                ))
                .monospace()
                .strong(),
            );
        });

    ui.add_space(8.0);

    let mut bars = Vec::with_capacity(catalog.modules.len());
    for (idx, spec) in catalog.modules.iter().enumerate() {
        let pct = mastery
            .get(spec.id)
            .map(|m| m.mastery_score * 100.0)
            .unwrap_or(0.0);
        let bar = Bar::new(idx as f64, pct)
            .width(0.72)
            .fill(theme.stage_accent(spec.stage))
            .name(format!("{} {}", spec.code, math_render::ui_safe_text(spec.title)));
        bars.push(bar);
    }

    Plot::new("mastery_bar_chart")
        .height(210.0)
        .include_y(0.0)
        .include_y(100.0)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(BarChart::new(bars).name("Module Mastery %"));
        });

    ui.add_space(10.0);

    // Per-Module Telemetry Table
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(RichText::new("48-Module Mastery & Spaced-Repetition Schedule").strong());
            ui.add_space(4.0);
            Grid::new("module_telemetry_grid")
                .striped(true)
                .num_columns(7)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Code").strong());
                    ui.label(RichText::new("Module").strong());
                    ui.label(RichText::new("Stage").strong());
                    ui.label(RichText::new("Mastery").strong());
                    ui.label(RichText::new("Accuracy").strong());
                    ui.label(RichText::new("Streak").strong());
                    ui.label(RichText::new("Review Status").strong());
                    ui.end_row();

                    for spec in &catalog.modules {
                        let st = mastery.get(spec.id);
                        let score = st.map(|m| m.mastery_score * 100.0).unwrap_or(0.0);
                        let attempts = st.map(|m| m.attempts).unwrap_or(0);
                        let acc = st.map(|m| m.accuracy() * 100.0).unwrap_or(0.0);
                        let streak = st.map(|m| m.streak).unwrap_or(0);
                        let review_str = match st.and_then(|m| m.next_review) {
                            None => "Not started".to_string(),
                            Some(due) if due <= now => "Due now".to_string(),
                            Some(due) => {
                                let hrs = (due - now).num_minutes() as f64 / 60.0;
                                format!("In {:.1} h", hrs)
                            }
                        };

                        ui.label(
                            RichText::new(spec.code)
                                .monospace()
                                .color(theme.stage_accent(spec.stage)),
                        );
                        ui.label(math_render::ui_safe_text(spec.title));
                        ui.label(spec.stage.short_label());
                        ui.label(RichText::new(format!("{:.0}%", score)).monospace());
                        ui.label(
                            RichText::new(if attempts == 0 {
                                "—".to_string()
                            } else {
                                format!("{:.0}% ({})", acc, attempts)
                            })
                            .monospace(),
                        );
                        ui.label(RichText::new(format!("{}", streak)).monospace());
                        ui.label(review_str);
                        ui.end_row();
                    }
                });
        });

    ui.add_space(10.0);

    // Recent Attempt Ledger
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(RichText::new("Recent Individual Problem Ledger").strong());
            ui.add_space(4.0);
            if recent_attempts.is_empty() {
                ui.label("No graded attempts recorded yet.");
            } else {
                Grid::new("recent_attempts_grid")
                    .striped(true)
                    .num_columns(6)
                    .spacing([12.0, 5.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("UTC Time").strong());
                        ui.label(RichText::new("Module").strong());
                        ui.label(RichText::new("Submitted").strong());
                        ui.label(RichText::new("Reference").strong());
                        ui.label(RichText::new("Result").strong());
                        ui.label(RichText::new("Time / Steps").strong());
                        ui.end_row();

                        for rec in recent_attempts.iter().take(20) {
                            ui.label(
                                RichText::new(
                                    rec.timestamp_utc.format("%Y-%m-%d %H:%M").to_string(),
                                )
                                .monospace()
                                .small(),
                            );
                            ui.label(RichText::new(&rec.module_id).monospace().small());
                            ui.label(RichText::new(&rec.user_input).monospace().small());
                            ui.label(RichText::new(&rec.expected_canonical).monospace().small());
                            ui.label(
                                RichText::new(if rec.correct { "PASS" } else { "MISS" })
                                    .strong()
                                    .color(if rec.correct {
                                        theme.ok_color()
                                    } else {
                                        theme.err_color()
                                    }),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "{:.1}s / {}",
                                    rec.solve_secs, rec.hints_used
                                ))
                                .monospace()
                                .small(),
                            );
                            ui.end_row();
                        }
                    });
            }
        });

    action
}

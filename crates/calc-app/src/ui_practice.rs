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
    format_run_scope, pick_next_adaptive_module, plain_input_to_latex, score_attempt,
    AnswerFormat, CurriculumCatalog, Expr, GeneratedProblem, GradeOutcome, LearningRunRecord,
    ModuleMastery, ProblemGenerator, RunMode, ScienceDomain, Stage, StageScopeFilter,
    WalkthroughVisual,
};
use chrono::Utc;
use egui::{
    Align, Color32, ComboBox, Context, Frame, Key, Layout, Margin, ProgressBar, RichText, Rounding,
    Stroke, TextEdit, TextStyle, Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, Points, Polygon, VLine};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::time::Instant;

pub enum PracticeAction {
    RecordAttempt {
        module_id: String,
        title: String,
        user_input: String,
        canonical: String,
        correct: bool,
        hints_used: usize,
        solve_secs: f64,
    },
    RecordLearningRun(LearningRunRecord),
    Navigate(NavCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemSelectionPool {
    FocusedModule,
    AdaptiveFrontier,
    StagePool(StageScopeFilter),
}

pub struct PracticeState {
    pub run_mode: RunMode,
    pub pool: ProblemSelectionPool,
    pub active_problem: GeneratedProblem,
    pub active_domain: ScienceDomain,
    pub queue_reason: &'static str,
    pub answer_input: String,
    pub hints_revealed: usize,
    /// Number of method-tagged visual solution steps currently unlocked in Learning Mode (0..=steps.len())
    pub step_cursor: usize,
    pub last_outcome: Option<GradeOutcome>,
    pub last_points_delta: i64,
    pub outcome_recorded: bool,
    pub focus_answer_input: bool,
    pub last_graded_input: Option<String>,
    pub problem_started_at: Instant,
    pub seed_counter: u64,
    // Active Run / Game Scoreboard State
    pub run_started_at: Instant,
    pub run_finished: bool,
    pub run_saved_to_db: bool,
    pub run_score: i64,
    pub run_solved: u32,
    pub run_attempted: u32,
    pub current_streak: u32,
    pub max_streak: u32,
    pub last_saved_summary: Option<LearningRunRecord>,
    // Symbolic Scratchpad
    pub scratchpad_expr: String,
    pub scratchpad_var: String,
    pub scratchpad_x: f64,
}

impl PracticeState {
    pub fn new(initial_module_id: &str, domain: ScienceDomain) -> Self {
        let seed_counter = 1001;
        let active_problem =
            ProblemGenerator::generate_with_domain(initial_module_id, seed_counter, domain);
        let now = Instant::now();
        Self {
            run_mode: RunMode::LearningWalkthrough,
            pool: ProblemSelectionPool::FocusedModule,
            active_problem,
            active_domain: domain,
            queue_reason: "Focused Module Learning Run",
            answer_input: String::new(),
            hints_revealed: 0,
            step_cursor: 0,
            last_outcome: None,
            last_points_delta: 0,
            outcome_recorded: false,
            focus_answer_input: true,
            last_graded_input: None,
            problem_started_at: now,
            seed_counter,
            run_started_at: now,
            run_finished: false,
            run_saved_to_db: false,
            run_score: 0,
            run_solved: 0,
            run_attempted: 0,
            current_streak: 0,
            max_streak: 0,
            last_saved_summary: None,
            scratchpad_expr: "x^3 - 3*x".to_string(),
            scratchpad_var: "x".to_string(),
            scratchpad_x: 1.5,
        }
    }

    pub fn sync_domain(&mut self, domain: ScienceDomain) {
        if self.active_domain != domain {
            self.active_domain = domain;
            let mid = self.active_problem.module_id.clone();
            self.active_problem =
                ProblemGenerator::generate_with_domain(&mid, self.seed_counter, domain);
            self.step_cursor = 0;
            self.last_outcome = None;
            self.outcome_recorded = false;
            self.last_graded_input = None;
            self.focus_answer_input = true;
        }
    }

    pub fn start_fresh_run(
        &mut self,
        mode: RunMode,
        selected_module_id: &str,
        catalog: &CurriculumCatalog,
        mastery: &HashMap<String, ModuleMastery>,
        free_explore: bool,
    ) {
        self.run_mode = mode;
        self.run_started_at = Instant::now();
        self.run_finished = false;
        self.run_saved_to_db = false;
        self.run_score = 0;
        self.run_solved = 0;
        self.run_attempted = 0;
        self.current_streak = 0;
        self.max_streak = 0;
        self.last_saved_summary = None;
        self.advance_problem(selected_module_id, catalog, mastery, free_explore);
    }

    pub fn load_for_module(&mut self, module_id: &str, reason: &'static str) {
        self.seed_counter = self.seed_counter.wrapping_add(19);
        self.active_problem =
            ProblemGenerator::generate_with_domain(module_id, self.seed_counter, self.active_domain);
        self.queue_reason = reason;
        self.answer_input.clear();
        self.hints_revealed = 0;
        self.step_cursor = 0;
        self.last_outcome = None;
        self.last_points_delta = 0;
        self.outcome_recorded = false;
        self.last_graded_input = None;
        self.focus_answer_input = true;
        self.problem_started_at = Instant::now();
    }

    pub fn on_answer_edited(&mut self) {
        if self.last_graded_input.as_deref() != Some(self.answer_input.as_str()) {
            self.last_outcome = None;
            self.outcome_recorded = false;
        }
    }

    pub fn is_ready_to_advance_on_enter(&self) -> bool {
        self.last_outcome
            .as_ref()
            .is_some_and(|o| o.parse_error.is_none())
            && self.last_graded_input.as_deref() == Some(self.answer_input.as_str())
    }

    pub fn submit_current_answer(&mut self) -> (Option<PracticeAction>, bool) {
        let outcome = self.active_problem.grade(&self.answer_input);
        let mut recorded_action = None;
        let mut auto_advance = false;
        if outcome.parse_error.is_none() {
            self.last_graded_input = Some(self.answer_input.clone());
            if !self.outcome_recorded {
                self.outcome_recorded = true;
                let solve_secs = self.problem_started_at.elapsed().as_secs_f64();
                let s_delta = score_attempt(
                    self.run_mode,
                    self.active_problem.difficulty,
                    outcome.correct,
                    self.current_streak,
                    solve_secs,
                    self.step_cursor,
                );
                self.run_attempted += 1;
                if outcome.correct {
                    self.run_solved += 1;
                    self.run_score += s_delta.points_awarded;
                    self.last_points_delta = s_delta.points_awarded;
                } else {
                    self.last_points_delta = 0;
                    // Unlock first method step automatically on miss in Learning Mode
                    if self.step_cursor == 0 {
                        self.step_cursor = 1;
                    }
                }
                self.current_streak = s_delta.new_streak;
                self.max_streak = self.max_streak.max(self.current_streak);

                recorded_action = Some(PracticeAction::RecordAttempt {
                    module_id: self.active_problem.module_id.clone(),
                    title: self.active_problem.title.clone(),
                    user_input: self.answer_input.trim().to_string(),
                    canonical: outcome.canonical_answer.clone(),
                    correct: outcome.correct,
                    hints_used: self.hints_revealed.max(self.step_cursor),
                    solve_secs,
                });

                if outcome.correct && self.run_mode.is_timed() {
                    auto_advance = true;
                }
            }
        }
        self.last_outcome = Some(outcome);
        (recorded_action, auto_advance)
    }

    pub fn advance_problem(
        &mut self,
        selected_module_id: &str,
        catalog: &CurriculumCatalog,
        mastery: &HashMap<String, ModuleMastery>,
        free_explore: bool,
    ) {
        match self.pool {
            ProblemSelectionPool::FocusedModule => {
                self.load_for_module(selected_module_id, "Focused Module");
            }
            ProblemSelectionPool::AdaptiveFrontier => {
                let next = pick_next_adaptive_module(catalog, mastery, free_explore, Utc::now());
                self.load_for_module(&next.module_id, next.reason);
            }
            ProblemSelectionPool::StagePool(scope) => {
                let candidates: Vec<&str> = match scope {
                    StageScopeFilter::AllStages => {
                        catalog.modules.iter().map(|m| m.id).collect()
                    }
                    StageScopeFilter::Single(st) => {
                        catalog.by_stage(st).into_iter().map(|m| m.id).collect()
                    }
                };
                self.seed_counter = self.seed_counter.wrapping_add(23);
                let mut rng = StdRng::seed_from_u64(self.seed_counter);
                let chosen = if candidates.is_empty() {
                    selected_module_id
                } else {
                    candidates[rng.gen_range(0..candidates.len())]
                };
                self.load_for_module(chosen, "Randomized Stage Pool");
            }
        }
    }

    fn build_run_record(&self) -> LearningRunRecord {
        let scope_filter = match self.pool {
            ProblemSelectionPool::FocusedModule => StageScopeFilter::AllStages,
            ProblemSelectionPool::AdaptiveFrontier => StageScopeFilter::AllStages,
            ProblemSelectionPool::StagePool(sf) => sf,
        };
        let (mut stage_scope, domain_label) = format_run_scope(scope_filter, self.active_domain);
        if self.pool == ProblemSelectionPool::FocusedModule {
            stage_scope = format!("Module: {}", self.active_problem.module_id);
        } else if self.pool == ProblemSelectionPool::AdaptiveFrontier {
            stage_scope = "Adaptive Frontier".to_string();
        }
        let elapsed = match self.run_mode.duration_limit_secs() {
            Some(lim) => self.run_started_at.elapsed().as_secs_f64().min(lim),
            None => self.run_started_at.elapsed().as_secs_f64(),
        };
        LearningRunRecord {
            id: 0,
            finished_at_utc: Utc::now(),
            mode: self.run_mode.short_code().to_string(),
            domain: domain_label,
            stage_scope,
            score: self.run_score,
            solved_count: self.run_solved,
            attempted_count: self.run_attempted,
            max_streak: self.max_streak,
            elapsed_secs: elapsed,
        }
    }
}

fn sample_visual_segments(
    expr_str: &str,
    var: &str,
    x_min: f64,
    x_max: f64,
    y_cap: f64,
) -> Vec<Vec<[f64; 2]>> {
    let Ok(ast) = Expr::parse(expr_str) else {
        return Vec::new();
    };
    if x_max <= x_min {
        return Vec::new();
    }
    let steps = 240usize;
    let jump_thresh = (y_cap * 0.45).max(60.0);
    let mut segments: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut current: Vec<[f64; 2]> = Vec::new();
    for i in 0..=steps {
        let x = x_min + (i as f64 / steps as f64) * (x_max - x_min);
        let y = ast.eval_1d(var, x);
        if y.is_finite() && y.abs() <= y_cap {
            if let Some(&[_px, py]) = current.last() {
                if (y - py).abs() > jump_thresh {
                    if current.len() >= 2 {
                        segments.push(current);
                    }
                    current = Vec::new();
                }
            }
            current.push([x, y]);
        } else if !current.is_empty() {
            if current.len() >= 2 {
                segments.push(current);
            }
            current = Vec::new();
        }
    }
    if current.len() >= 2 {
        segments.push(current);
    }
    segments
}

fn sample_shaded_polygon(
    expr_str: &str,
    var: &str,
    a_raw: f64,
    b_raw: f64,
    y_cap: f64,
) -> Option<Vec<[f64; 2]>> {
    let Ok(ast) = Expr::parse(expr_str) else {
        return None;
    };
    let (a, b) = if a_raw <= b_raw {
        (a_raw, b_raw)
    } else {
        (b_raw, a_raw)
    };
    if (b - a).abs() <= 1e-9 {
        return None;
    }
    let mut pts = Vec::with_capacity(84);
    pts.push([a, 0.0]);
    let mut valid_samples = 0usize;
    for i in 0..=80 {
        let x = a + (i as f64 / 80.0) * (b - a);
        let y = ast.eval_1d(var, x);
        if y.is_finite() {
            pts.push([x, y.clamp(-y_cap, y_cap)]);
            valid_samples += 1;
        }
    }
    pts.push([b, 0.0]);
    if valid_samples >= 2 {
        Some(pts)
    } else {
        None
    }
}

fn adaptive_plot_y_cap(visual: &WalkthroughVisual, var: &str) -> f64 {
    if let (Some([a_raw, b_raw]), Some(prim)) = (visual.shade_interval, visual.primary_expr.as_deref()) {
        if let Ok(ast) = Expr::parse(prim) {
            let (a, b) = if a_raw <= b_raw {
                (a_raw, b_raw)
            } else {
                (b_raw, a_raw)
            };
            let mut max_abs = 0.0f64;
            for i in 0..=40 {
                let x = a + (i as f64 / 40.0) * (b - a);
                let y = ast.eval_1d(var, x);
                if y.is_finite() {
                    max_abs = max_abs.max(y.abs());
                }
            }
            if max_abs > 0.0 {
                return (max_abs * 2.5).clamp(60.0, 500.0);
            }
        }
    }
    500.0
}

pub fn render_practice(
    ctx: &Context,
    ui: &mut Ui,
    state: &mut PracticeState,
    selected_module_id: &str,
    domain: ScienceDomain,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    free_explore: bool,
    theme: AcademicTheme,
) -> Vec<PracticeAction> {
    let mut actions = Vec::new();
    state.sync_domain(domain);

    // Check timed run countdown expiration
    let mut remaining_secs: Option<f64> = None;
    if let Some(limit) = state.run_mode.duration_limit_secs() {
        if !state.run_finished {
            let elapsed = state.run_started_at.elapsed().as_secs_f64();
            let rem = (limit - elapsed).max(0.0);
            remaining_secs = Some(rem);
            if rem <= 0.0 {
                state.run_finished = true;
                if !state.run_saved_to_db {
                    state.run_saved_to_db = true;
                    let rec = state.build_run_record();
                    state.last_saved_summary = Some(rec.clone());
                    actions.push(PracticeAction::RecordLearningRun(rec));
                }
            } else {
                ctx.request_repaint();
            }
        } else {
            remaining_secs = Some(0.0);
        }
    }

    // =========================================================================
    // 1. GAME MODE, POOL & RUN SCOREBOARD HUD
    // =========================================================================
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Run Mode:").strong());
                for mode in RunMode::ALL {
                    if ui
                        .selectable_label(state.run_mode == mode, mode.label())
                        .clicked()
                    {
                        // Save non-empty ongoing run before switching modes
                        if state.run_attempted > 0 && !state.run_saved_to_db {
                            let rec = state.build_run_record();
                            state.last_saved_summary = Some(rec.clone());
                            actions.push(PracticeAction::RecordLearningRun(rec));
                        }
                        if mode.is_timed() && state.pool == ProblemSelectionPool::FocusedModule {
                            state.pool =
                                ProblemSelectionPool::StagePool(StageScopeFilter::AllStages);
                        }
                        state.start_fresh_run(
                            mode,
                            selected_module_id,
                            catalog,
                            mastery,
                            free_explore,
                        );
                    }
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Question Pool:").strong());
                if ui
                    .selectable_label(
                        state.pool == ProblemSelectionPool::FocusedModule,
                        "Focused Module",
                    )
                    .clicked()
                {
                    state.pool = ProblemSelectionPool::FocusedModule;
                    state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                }
                if ui
                    .selectable_label(
                        state.pool == ProblemSelectionPool::AdaptiveFrontier,
                        "Adaptive Frontier + Review",
                    )
                    .clicked()
                {
                    state.pool = ProblemSelectionPool::AdaptiveFrontier;
                    state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                }
                if ui
                    .selectable_label(
                        state.pool == ProblemSelectionPool::StagePool(StageScopeFilter::AllStages),
                        "All 8 Stages Mix",
                    )
                    .clicked()
                {
                    state.pool = ProblemSelectionPool::StagePool(StageScopeFilter::AllStages);
                    state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                }

                ComboBox::from_label("Single Stage Pool")
                    .selected_text(match state.pool {
                        ProblemSelectionPool::StagePool(StageScopeFilter::Single(st)) => {
                            st.short_label()
                        }
                        _ => "Pick Stage...",
                    })
                    .show_ui(ui, |ui| {
                        for st in Stage::ALL {
                            let target = ProblemSelectionPool::StagePool(StageScopeFilter::Single(st));
                            if ui
                                .selectable_label(state.pool == target, st.title())
                                .clicked()
                            {
                                state.pool = target;
                                state.advance_problem(
                                    selected_module_id,
                                    catalog,
                                    mastery,
                                    free_explore,
                                );
                            }
                        }
                    });
            });

            ui.separator();

            // Live Scoreboard & Timer Strip
            ui.horizontal_wrapped(|ui| {
                let mult = (1.0 + 0.25 * ((state.current_streak.saturating_sub(1)).min(8) as f64))
                    .min(3.0);
                ui.label(
                    RichText::new(format!("SCORE: {} pts", state.run_score))
                        .monospace()
                        .strong()
                        .size(16.0)
                        .color(theme.ok_color()),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Streak: {} (Max {}) | {:.2}x Multiplier",
                        state.current_streak, state.max_streak, mult
                    ))
                    .monospace(),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Solved: {} / {}",
                        state.run_solved, state.run_attempted
                    ))
                    .monospace(),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!("Domain: {}", state.active_domain.label()))
                        .small()
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Restart Run").clicked() {
                        if state.run_attempted > 0 && !state.run_saved_to_db {
                            let rec = state.build_run_record();
                            state.last_saved_summary = Some(rec.clone());
                            actions.push(PracticeAction::RecordLearningRun(rec));
                        }
                        let mode = state.run_mode;
                        state.start_fresh_run(
                            mode,
                            selected_module_id,
                            catalog,
                            mastery,
                            free_explore,
                        );
                    }
                    if !state.run_finished && state.run_attempted > 0 {
                        if ui.button("Finish & Save Run").clicked() {
                            state.run_finished = true;
                            state.run_saved_to_db = true;
                            let rec = state.build_run_record();
                            state.last_saved_summary = Some(rec.clone());
                            actions.push(PracticeAction::RecordLearningRun(rec));
                        }
                    }
                });
            });

            if let (Some(rem), Some(lim)) = (remaining_secs, state.run_mode.duration_limit_secs())
            {
                let frac = (rem / lim).clamp(0.0, 1.0) as f32;
                ui.add(
                    ProgressBar::new(frac)
                        .text(format!("Time Remaining: {:.1}s / {:.0}s", rem, lim)),
                );
            }
        });

    ui.add_space(4.0);

    // If a Timed or Manual Run has finished, display the Run Complete Scorecard!
    if state.run_finished {
        Frame::none()
            .fill(theme.card_fill())
            .stroke(egui::Stroke::new(2.0_f32, theme.ok_color()))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(16.0))
            .show(ui, |ui| {
                ui.heading(format!("Run Complete — {}", state.run_mode.label()));
                ui.add_space(6.0);
                if let Some(rec) = &state.last_saved_summary {
                    ui.label(
                        RichText::new(format!(
                            "Final Score: {} pts  |  Solved: {} / {} ({:.1}% accuracy)  |  Best Streak: {}  |  Elapsed: {:.1}s",
                            rec.score,
                            rec.solved_count,
                            rec.attempted_count,
                            rec.accuracy_pct(),
                            rec.max_streak,
                            rec.elapsed_secs
                        ))
                        .monospace()
                        .size(15.5),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Saved to Learning Run Ledger under Domain '{}' ({}).",
                            rec.domain, rec.stage_scope
                        ))
                        .small()
                        .color(theme.ok_color()),
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Start Another Run").clicked() {
                        let mode = state.run_mode;
                        state.start_fresh_run(
                            mode,
                            selected_module_id,
                            catalog,
                            mastery,
                            free_explore,
                        );
                    }
                    if ui.button("View Run Leaderboard & Analytics").clicked() {
                        actions.push(PracticeAction::Navigate(NavCommand::OpenTab(
                            AppTab::Analytics,
                        )));
                    }
                });
            });
        return actions;
    }

    // =========================================================================
    // 2. ACTIVE PROBLEM PROMPT & ANSWER INPUT CARD
    // =========================================================================
    let current_spec = catalog
        .get(&state.active_problem.module_id)
        .or_else(|| catalog.get(selected_module_id));
    let stage_accent = current_spec
        .map(|s| theme.stage_accent(s.stage))
        .unwrap_or_else(|| theme.ok_color());

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
    let op = state.active_problem.operation;
    let [op_r, op_g, op_b] = op.accent_rgb();
    let op_color = Color32::from_rgb(op_r, op_g, op_b);
    let op_badge_bg = Color32::from_rgba_unmultiplied(op_r, op_g, op_b, 38);

    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(12.0, 9.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                Frame::none()
                    .fill(op_badge_bg)
                    .stroke(Stroke::new(1.2_f32, op_color))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(9.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("OPERATION: {}", op.category_badge()))
                                .strong()
                                .color(op_color),
                        );
                    });
                math_render::paint_latex(ui, op.operator_latex(), 15.0, op_color);
                if let Some(spec) = current_spec {
                    let m_score = mastery
                        .get(spec.id)
                        .map(|m| m.mastery_score * 100.0)
                        .unwrap_or(0.0);
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "{} | {} — {}",
                            spec.stage.short_label(),
                            spec.code,
                            math_render::ui_safe_text(spec.title)
                        ))
                        .monospace()
                        .strong()
                        .color(stage_accent),
                    );
                    ui.separator();
                    ui.label(format!("Mastery: {:.0}%", m_score));
                    ui.separator();
                    ui.label(format!("Tier: {}/4", state.active_problem.difficulty));
                }
            });

            if let Some(ctx_banner) = &state.active_problem.domain_context {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "[{}] {}",
                        state.active_problem.domain.short_tag(),
                        math_render::ui_safe_text(ctx_banner)
                    ))
                    .italics()
                    .color(stage_accent),
                );
            }

            ui.add_space(4.0);
            ui.heading(math_render::ui_safe_text(&state.active_problem.title));
            ui.add_space(3.0);
            math_render::paint_mixed_prompt(ui, &state.active_problem.prompt, 15.0, text_fg);
            ui.add_space(4.0);
            math_render::paint_latex_boxed(
                ui,
                &state.active_problem.target_equation_latex,
                19.0,
                text_fg,
                eq_bg,
                Stroke::new(1.2_f32, op_color.linear_multiply(0.75)),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(state.active_problem.answer.format_hint())
                    .small()
                    .weak(),
            );
            ui.add_space(6.0);

            let mut submit_now = false;
            let mut advance_now = false;
            let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
            let ready_to_advance = state.is_ready_to_advance_on_enter();
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Answer:").strong());
                let resp = ui.add(
                    TextEdit::singleline(&mut state.answer_input)
                        .desired_width(360.0)
                        .font(TextStyle::Monospace)
                        .hint_text("Type plain-text expression or value, e.g. 6*x - 2*cos(2*x)"),
                );
                if state.focus_answer_input {
                    resp.request_focus();
                    state.focus_answer_input = false;
                }
                if resp.changed() {
                    state.on_answer_edited();
                }
                if ready_to_advance
                    && !resp.changed()
                    && enter_pressed
                    && (resp.lost_focus() || !ctx.wants_keyboard_input())
                {
                    advance_now = true;
                } else if resp.lost_focus() && enter_pressed {
                    submit_now = true;
                }
                if ui.button("Check Answer").clicked() {
                    submit_now = true;
                }
                let next_label = if state.is_ready_to_advance_on_enter() {
                    "Next Question >> (Enter)"
                } else {
                    "Next Question >>"
                };
                if ui.button(next_label).clicked() {
                    advance_now = true;
                }

                // Live 2D Rendered User-Input Preview inline right beside input controls
                if !state.answer_input.trim().is_empty() {
                    match plain_input_to_latex(&state.answer_input) {
                        Ok(latex) => {
                            ui.separator();
                            ui.label(RichText::new("2D Preview:").small().weak());
                            math_render::paint_latex_boxed(
                                ui,
                                &latex,
                                15.5,
                                text_fg,
                                eq_bg,
                                theme.card_stroke(),
                            );
                        }
                        Err(err) => {
                            ui.label(
                                RichText::new(format!("Syntax check: {}", err))
                                    .small()
                                    .color(theme.warn_color()),
                            );
                        }
                    }
                }
            });

            if advance_now {
                state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                return;
            }

            if submit_now {
                let (maybe_act, auto_advance) = state.submit_current_answer();
                if let Some(act) = maybe_act {
                    actions.push(act);
                }
                if auto_advance {
                    state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                    return;
                }
            }

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let total_hints = state.active_problem.hints.len();
                ui.add_enabled_ui(state.hints_revealed < total_hints, |ui| {
                    if ui
                        .button(format!(
                            "Quick Hint ({}/{})",
                            state.hints_revealed, total_hints
                        ))
                        .clicked()
                    {
                        state.hints_revealed += 1;
                    }
                });

                let total_steps = state.active_problem.steps.len();
                ui.add_enabled_ui(state.step_cursor > 0, |ui| {
                    if ui.button("< Previous Method Step").clicked() {
                        state.step_cursor = state.step_cursor.saturating_sub(1);
                    }
                });
                ui.add_enabled_ui(state.step_cursor < total_steps, |ui| {
                    if ui
                        .button(format!(
                            "Advance Visual Step ({}/{}) >",
                            state.step_cursor, total_steps
                        ))
                        .clicked()
                    {
                        state.step_cursor += 1;
                    }
                });
                if state.step_cursor < total_steps && ui.small_button("Show All Steps").clicked() {
                    state.step_cursor = total_steps;
                }

                if ui.small_button("Concept Lesson").clicked() {
                    actions.push(PracticeAction::Navigate(NavCommand::OpenModule {
                        module_id: state.active_problem.module_id.clone(),
                        tab: AppTab::Lesson,
                    }));
                }
                if ui.small_button("Open in Visual Lab").clicked() {
                    actions.push(PracticeAction::Navigate(NavCommand::OpenLabPreset(
                        state.active_problem.module_id.clone(),
                    )));
                }
            });
        });

    // Hints Banner
    if state.hints_revealed > 0 {
        ui.add_space(8.0);
        Frame::none()
            .fill(theme.card_fill())
            .stroke(theme.card_stroke())
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                for i in 0..state.hints_revealed.min(state.active_problem.hints.len()) {
                    ui.label(format!(
                        "Hint {}: {}",
                        i + 1,
                        math_render::ui_safe_text(&state.active_problem.hints[i])
                    ));
                }
            });
    }

    // Outcome Feedback Banner
    if let Some(outcome) = &state.last_outcome {
        ui.add_space(4.0);
        let col = if outcome.correct {
            theme.ok_color()
        } else if outcome.parse_error.is_some() {
            theme.warn_color()
        } else {
            theme.err_color()
        };
        Frame::none()
            .fill(theme.card_fill())
            .stroke(egui::Stroke::new(1.5_f32, col))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                let headline = if outcome.correct {
                    format!("Correct! +{} pts awarded", state.last_points_delta)
                } else if outcome.parse_error.is_some() {
                    "Syntax Check".to_string()
                } else {
                    "Not Quite — Advance the Method Steps Below to Inspect Each Transformation"
                        .to_string()
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(headline).strong().color(col));
                    ui.separator();
                    ui.label(math_render::ui_safe_text(&outcome.feedback));
                });

                if !outcome.correct && outcome.parse_error.is_none() {
                    if let Some(rem_id) = &state.active_problem.remediate_module_id {
                        if let Some(rem_spec) = catalog.get(rem_id) {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "Upstream prerequisite: {} ({})",
                                        math_render::ui_safe_text(rem_spec.title),
                                        rem_spec.code
                                    ))
                                    .small(),
                                );
                                if ui.small_button("Jump to Prerequisite Lesson").clicked() {
                                    actions.push(PracticeAction::Navigate(
                                        NavCommand::OpenModule {
                                            module_id: rem_spec.id.to_string(),
                                            tab: AppTab::Lesson,
                                        },
                                    ));
                                }
                            });
                        }
                    }
                }
            });
    }

    // =========================================================================
    // 3. STEP-BY-STEP NAMED-METHOD WALKTHROUGH & VISUAL SOLUTION PLOTTER
    // =========================================================================
    if state.step_cursor > 0 || state.run_mode == RunMode::LearningWalkthrough {
        ui.add_space(4.0);
        Frame::none()
            .fill(theme.card_fill())
            .stroke(theme.card_stroke())
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Step-by-Step Visual Method Walkthrough")
                            .strong()
                            .size(15.5)
                            .color(stage_accent),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Showing {} of {} named method steps",
                            state.step_cursor,
                            state.active_problem.steps.len()
                        ))
                        .small()
                        .weak(),
                    );
                });

                ui.add_space(4.0);
                ui.columns(2, |cols| {
                    // Left Column: Sequential Method Cards
                    cols[0].with_layout(Layout::top_down(Align::Min), |ui| {
                        if state.step_cursor == 0 {
                            ui.label(
                                "Click 'Advance Visual Step (0/N) >' above whenever you want to reveal the first named mathematical method without spoiling the final answer.",
                            );
                        } else {
                            let unlocked = state
                                .step_cursor
                                .min(state.active_problem.steps.len());
                            for step in &state.active_problem.steps[..unlocked] {
                                Frame::none()
                                    .fill(theme.card_fill())
                                    .stroke(egui::Stroke::new(1.2_f32, stage_accent))
                                    .rounding(Rounding::same(5.0))
                                    .inner_margin(Margin::symmetric(8.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.horizontal_wrapped(|ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Step {} · {}",
                                                    step.step_number,
                                                    math_render::ui_safe_text(&step.rule_name)
                                                ))
                                                .strong()
                                                .color(stage_accent),
                                            );
                                            if !step.rule_identity_latex.trim().is_empty() {
                                                ui.separator();
                                                math_render::paint_latex(
                                                    ui,
                                                    &step.rule_identity_latex,
                                                    12.5,
                                                    op_color,
                                                );
                                            }
                                        });
                                        ui.label(
                                            RichText::new(math_render::ui_safe_text(
                                                &step.explanation,
                                            ))
                                            .small(),
                                        );
                                        ui.horizontal_wrapped(|ui| {
                                            math_render::paint_latex_boxed(
                                                ui,
                                                &step.before_latex,
                                                14.0,
                                                text_fg,
                                                eq_bg,
                                                theme.card_stroke(),
                                            );
                                            math_render::paint_transformation_arrow(
                                                ui, 14.5, op_color,
                                            );
                                            math_render::paint_latex_boxed(
                                                ui,
                                                &step.after_latex,
                                                14.0,
                                                theme.ok_color(),
                                                eq_bg,
                                                egui::Stroke::new(
                                                    1.1_f32,
                                                    theme.ok_color().linear_multiply(0.65),
                                                ),
                                            );
                                        });
                                    });
                                ui.add_space(3.0);
                            }
                        }
                    });

                    // Right Column: Companion Visual Step Plot + Final Solution Card
                    cols[1].with_layout(Layout::top_down(Align::Min), |ui| {
                        let visual = &state.active_problem.visual;
                        let var = if visual.var.is_empty() {
                            state.active_problem.visual_var.as_str()
                        } else {
                            visual.var.as_str()
                        };
                        let [x_min, x_max] = match visual.x_range {
                            Some([lo, hi]) if hi > lo && lo.is_finite() && hi.is_finite() => {
                                [lo, hi]
                            }
                            _ => [-4.5, 4.5],
                        };
                        let prim_expr = visual
                            .primary_expr
                            .as_deref()
                            .or(state.active_problem.visual_primary_expr.as_deref());
                        let y_cap = adaptive_plot_y_cap(visual, var);
                        let solved_or_all_steps = (state.step_cursor
                            >= state.active_problem.steps.len()
                            && state.step_cursor > 0)
                            || state.last_outcome.as_ref().is_some_and(|o| o.correct);

                        Plot::new("learning_step_visual_plot")
                            .height(225.0)
                            .legend(Legend::default())
                            .show(ui, |plot_ui| {
                                // 1. Shaded Integral Area Polygon over [a, b]
                                if let (Some([a, b]), Some(prim)) =
                                    (visual.shade_interval, prim_expr)
                                {
                                    if let Some(poly_pts) =
                                        sample_shaded_polygon(prim, var, a, b, y_cap)
                                    {
                                        plot_ui.polygon(
                                            Polygon::new(poly_pts)
                                                .fill_color(Color32::from_rgba_unmultiplied(
                                                    88, 166, 236, 56,
                                                ))
                                                .stroke(Stroke::new(
                                                    1.2_f32,
                                                    Color32::from_rgb(88, 166, 236),
                                                ))
                                                .name("Shaded Integral Area"),
                                        );
                                    }
                                }

                                // 2. Primary curve f(x)
                                if let Some(prim) = prim_expr {
                                    let prim_label = visual
                                        .primary_label
                                        .clone()
                                        .unwrap_or_else(|| format!("Reference Curve ({})", var));
                                    let segs =
                                        sample_visual_segments(prim, var, x_min, x_max, y_cap);
                                    for (seg_idx, seg) in segs.into_iter().enumerate() {
                                        let mut line = Line::new(seg)
                                            .color(Color32::from_rgb(88, 166, 236))
                                            .width(2.2_f32);
                                        if seg_idx == 0 {
                                            line = line.name(&prim_label);
                                        }
                                        plot_ui.line(line);
                                    }
                                }

                                // 3. Secondary / step curve (never replace primary curve with flat y = expected!)
                                let setup_secondary = visual
                                    .secondary_label
                                    .as_deref()
                                    .is_some_and(|lbl| {
                                        lbl.contains("branch")
                                            || lbl.contains("Line 2")
                                            || lbl.contains("Component Q")
                                            || lbl.contains("slice")
                                    });
                                let mut drew_secondary = false;
                                if let Some(sec) = &visual.secondary_expr {
                                    if setup_secondary || solved_or_all_steps {
                                        let sec_label = visual
                                            .secondary_label
                                            .clone()
                                            .unwrap_or_else(|| format!("Solution Curve ({})", var));
                                        let segs =
                                            sample_visual_segments(sec, var, x_min, x_max, y_cap);
                                        for (seg_idx, seg) in segs.into_iter().enumerate() {
                                            let mut line = Line::new(seg)
                                                .color(Color32::from_rgb(92, 198, 148))
                                                .width(2.4_f32);
                                            if seg_idx == 0 {
                                                line = line.name(&sec_label);
                                            }
                                            plot_ui.line(line);
                                            drew_secondary = true;
                                        }
                                    }
                                }
                                if !drew_secondary && state.step_cursor > 0 {
                                    let idx = (state.step_cursor - 1)
                                        .min(state.active_problem.steps.len().saturating_sub(1));
                                    if let Some(step) = state.active_problem.steps.get(idx) {
                                        if let Some(step_curve) = &step.visual_plot_expr {
                                            let is_same_as_prim = prim_expr == Some(step_curve.as_str());
                                            let is_flat_numeric_fallback = prim_expr.is_some()
                                                && matches!(
                                                    state.active_problem.answer,
                                                    AnswerFormat::Numeric { .. }
                                                )
                                                && step_curve.trim().parse::<f64>().is_ok();
                                            if !is_same_as_prim && !is_flat_numeric_fallback {
                                                let segs = sample_visual_segments(
                                                    step_curve, var, x_min, x_max, y_cap,
                                                );
                                                let label = format!("Step {} Curve", step.step_number);
                                                for (seg_idx, seg) in segs.into_iter().enumerate() {
                                                    let mut line = Line::new(seg)
                                                        .color(Color32::from_rgb(92, 198, 148))
                                                        .width(2.4_f32);
                                                    if seg_idx == 0 {
                                                        line = line.name(&label);
                                                    }
                                                    plot_ui.line(line);
                                                }
                                            }
                                        }
                                    }
                                }

                                // 4. Tangent line + point of tangency
                                if let (Some([x0, y0]), Some(slope)) =
                                    (visual.tangent_point, visual.tangent_slope)
                                {
                                    if x0.is_finite() && y0.is_finite() && slope.is_finite() {
                                        let half_span = if slope.abs() > 1e-6 {
                                            (25.0 / slope.abs()).min((x_max - x_min) * 0.65)
                                        } else {
                                            x_max - x_min
                                        };
                                        let tx0 = (x0 - half_span).max(x_min);
                                        let tx1 = (x0 + half_span).min(x_max);
                                        let ty0 = y0 + slope * (tx0 - x0);
                                        let ty1 = y0 + slope * (tx1 - x0);
                                        plot_ui.line(
                                            Line::new(vec![[tx0, ty0], [tx1, ty1]])
                                                .color(Color32::from_rgb(232, 168, 72))
                                                .width(2.0_f32)
                                                .name(format!("Tangent (m = {:.3})", slope)),
                                        );
                                        plot_ui.points(
                                            Points::new(vec![[x0, y0]])
                                                .radius(5.0_f32)
                                                .color(Color32::from_rgb(232, 168, 72))
                                                .name(format!("({:.2}, {:.2})", x0, y0)),
                                        );
                                    }
                                }

                                // 5. Highlighted points, vertical/horizontal target lines
                                if !visual.highlight_points.is_empty() {
                                    plot_ui.points(
                                        Points::new(visual.highlight_points.clone())
                                            .radius(5.5_f32)
                                            .color(Color32::from_rgb(232, 168, 72))
                                            .name("Key Point(s)"),
                                    );
                                }
                                if let Some(vx) = visual.vline_x {
                                    if vx.is_finite() {
                                        let mut vl = VLine::new(vx)
                                            .color(Color32::from_rgb(232, 168, 72))
                                            .width(1.6_f32);
                                        if let Some(lbl) = &visual.vline_label {
                                            vl = vl.name(lbl);
                                        }
                                        plot_ui.vline(vl);
                                    }
                                }
                                if let Some(hy) = visual.hline_y {
                                    if hy.is_finite() {
                                        let mut hl = HLine::new(hy)
                                            .color(Color32::from_rgb(206, 126, 218))
                                            .width(1.6_f32);
                                        if let Some(lbl) = &visual.hline_label {
                                            hl = hl.name(lbl);
                                        }
                                        plot_ui.hline(hl);
                                    }
                                }

                                // 6. Fallback BarChart + VLine ONLY when problem has no curve/interval/tangent geometry
                                let has_curve_geometry = prim_expr.is_some()
                                    || visual.shade_interval.is_some()
                                    || visual.tangent_point.is_some();
                                if !has_curve_geometry
                                    && state.step_cursor >= state.active_problem.steps.len()
                                    && state.step_cursor > 0
                                {
                                    if let AnswerFormat::Numeric { expected, .. } =
                                        &state.active_problem.answer
                                    {
                                        if expected.is_finite() && expected.abs() <= 50.0 {
                                            plot_ui.bar_chart(
                                                BarChart::new(vec![Bar::new(0.0, *expected)
                                                    .width(0.5)
                                                    .fill(Color32::from_rgb(232, 168, 72))
                                                    .name("Exact Numeric Target")])
                                                .name("Result Value"),
                                            );
                                            plot_ui.vline(
                                                VLine::new(*expected)
                                                    .color(Color32::from_rgb(232, 168, 72)),
                                            );
                                        }
                                    }
                                }
                            });

                        if solved_or_all_steps {
                            ui.add_space(6.0);
                            let sol_fill = match theme {
                                AcademicTheme::SlateDark => Color32::from_rgb(22, 42, 34),
                                AcademicTheme::PaperLight => Color32::from_rgb(232, 246, 238),
                            };
                            Frame::none()
                                .fill(sol_fill)
                                .stroke(egui::Stroke::new(1.5_f32, theme.ok_color()))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::same(10.0))
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            RichText::new("Final Solution:")
                                                .strong()
                                                .size(16.0)
                                                .color(theme.ok_color()),
                                        );
                                        math_render::paint_latex_boxed(
                                            ui,
                                            &state.active_problem.answer_latex,
                                            18.0,
                                            theme.ok_color(),
                                            eq_bg,
                                            egui::Stroke::new(1.2_f32, theme.ok_color()),
                                        );
                                        ui.label(
                                            RichText::new(format!(
                                                "Plain-Text Input: {}",
                                                state.active_problem.answer.canonical()
                                            ))
                                            .small()
                                            .monospace()
                                            .color(text_fg),
                                        );
                                    });
                                });
                        }
                    });
                });
            });
    }

    ui.add_space(10.0);

    // Collapsible Symbolic Scratchpad
    egui::CollapsingHeader::new("Interactive Symbolic Scratchpad (Derivative & Numeric Probe)")
        .default_open(false)
        .show(ui, |ui| {
            Frame::none()
                .fill(theme.card_fill())
                .stroke(theme.card_stroke())
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("g =");
                        ui.add(
                            TextEdit::singleline(&mut state.scratchpad_expr)
                                .desired_width(260.0)
                                .font(TextStyle::Monospace),
                        );
                        ui.label("var:");
                        ui.add(
                            TextEdit::singleline(&mut state.scratchpad_var)
                                .desired_width(45.0)
                                .font(TextStyle::Monospace),
                        );
                        ui.add(
                            egui::Slider::new(&mut state.scratchpad_x, -5.0..=5.0).text("probe"),
                        );
                    });

                    let var = state.scratchpad_var.trim();
                    let var = if var.is_empty() { "x" } else { var };
                    if let Ok(expr) = Expr::parse(&state.scratchpad_expr) {
                        let simp = expr.simplify();
                        let deriv = simp.differentiate(var);
                        let val = simp.eval_1d(var, state.scratchpad_x);
                        let dval = deriv.eval_1d(var, state.scratchpad_x);
                        ui.label(
                            RichText::new(format!(
                                "d/d{} [{}] = {}  |  At {:.2}: g = {:.4}, g' = {:.4}",
                                var,
                                simp.to_plain_string(),
                                deriv.to_plain_string(),
                                state.scratchpad_x,
                                val,
                                dval
                            ))
                            .monospace(),
                        );
                    }
                });
        });

    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use calc_core::build_curriculum;

    #[test]
    fn practice_state_enter_to_advance_transitions_cleanly() {
        let catalog = build_curriculum();
        let mastery = HashMap::new();
        let mut state = PracticeState::new("calc-integration-tech", ScienceDomain::General);

        assert!(state.focus_answer_input);
        assert!(state.last_graded_input.is_none());
        assert!(!state.is_ready_to_advance_on_enter());

        // Empty submission yields parse_error and must NOT arm Enter-to-advance
        let (act_empty, auto_adv_empty) = state.submit_current_answer();
        assert!(act_empty.is_none());
        assert!(!auto_adv_empty);
        assert!(!state.is_ready_to_advance_on_enter());

        // Valid submission arms Enter-to-advance and records last_graded_input
        let canonical = state.active_problem.answer.canonical();
        state.answer_input = canonical.clone();
        let (act_ok, auto_adv) = state.submit_current_answer();
        assert!(act_ok.is_some());
        assert!(!auto_adv); // LearningWalkthrough waits for user's second Enter
        assert_eq!(state.last_graded_input.as_deref(), Some(canonical.as_str()));
        assert!(state.is_ready_to_advance_on_enter());

        // Editing answer_input clears last_outcome and disarms Enter-to-advance
        state.answer_input.push_str(" + 1");
        state.on_answer_edited();
        assert!(state.last_outcome.is_none());
        assert!(!state.outcome_recorded);
        assert!(!state.is_ready_to_advance_on_enter());

        // Re-grading updated answer re-arms Enter-to-advance
        let (act_regrade, _) = state.submit_current_answer();
        assert!(act_regrade.is_some());
        assert!(state.is_ready_to_advance_on_enter());

        // Advancing resets input, clears grading flags, and requests TextEdit focus
        state.focus_answer_input = false;
        state.advance_problem("calc-integration-tech", &catalog, &mastery, true);
        assert!(state.answer_input.is_empty());
        assert!(state.last_outcome.is_none());
        assert!(state.last_graded_input.is_none());
        assert!(!state.outcome_recorded);
        assert!(state.focus_answer_input);
        assert!(!state.is_ready_to_advance_on_enter());
    }

    #[test]
    fn walkthrough_sampling_and_shaded_polygon_handle_integrals_and_asymptotes() {
        let prob = ProblemGenerator::generate_with_domain(
            "calc-integrals-ftc",
            7,
            ScienceDomain::General,
        );
        let prim = prob
            .visual
            .primary_expr
            .as_deref()
            .expect("FTC problem should carry primary_expr");
        let [a, b] = prob
            .visual
            .shade_interval
            .expect("FTC problem should carry shade_interval");
        let y_cap = adaptive_plot_y_cap(&prob.visual, &prob.visual.var);
        let poly = sample_shaded_polygon(prim, &prob.visual.var, a, b, y_cap)
            .expect("Shaded polygon should sample cleanly");
        assert!(poly.len() >= 82);
        assert_eq!(poly.first().unwrap()[1], 0.0);
        assert_eq!(poly.last().unwrap()[1], 0.0);

        // Discontinuity splitting across vertical asymptote x = 1
        let segs = sample_visual_segments("1 / (x - 1)", "x", -1.0, 3.0, 500.0);
        assert!(
            segs.len() >= 2,
            "Expected pole discontinuity at x=1 to split into >= 2 segments, got {}",
            segs.len()
        );
    }
}

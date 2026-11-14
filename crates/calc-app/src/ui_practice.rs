use crate::app::{AppTab, NavCommand};
use crate::theme::AcademicTheme;
use calc_core::{
    format_run_scope, parse_vector2, pick_next_adaptive_module, score_attempt, AnswerFormat,
    CurriculumCatalog, Expr, GeneratedProblem, GradeOutcome, LearningRunRecord, ModuleMastery,
    ProblemGenerator, RunMode, ScienceDomain, Stage, StageScopeFilter,
};
use chrono::Utc;
use egui::{
    Color32, ComboBox, Context, Frame, Key, Margin, ProgressBar, RichText, Rounding, TextEdit,
    TextStyle, Ui,
};
use egui_plot::{Bar, BarChart, Legend, Line, Plot, VLine};
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
        self.problem_started_at = Instant::now();
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

fn sample_visual_expr(expr_str: &str, var: &str, x_min: f64, x_max: f64) -> Vec<[f64; 2]> {
    let Ok(ast) = Expr::parse(expr_str) else {
        return Vec::new();
    };
    let mut pts = Vec::with_capacity(201);
    for i in 0..=200 {
        let x = x_min + (i as f64 / 200.0) * (x_max - x_min);
        let y = ast.eval_1d(var, x);
        if y.is_finite() && y.abs() <= 45.0 {
            pts.push([x, y]);
        }
    }
    pts
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

    ui.add_space(8.0);

    // If a Timed or Manual Run has finished, display the Run Complete Scorecard!
    if state.run_finished {
        Frame::none()
            .fill(theme.card_fill())
            .stroke(egui::Stroke::new(2.0, theme.ok_color()))
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

    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            if let Some(spec) = current_spec {
                let m_score = mastery
                    .get(spec.id)
                    .map(|m| m.mastery_score * 100.0)
                    .unwrap_or(0.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} | {} — {}",
                            spec.stage.short_label(),
                            spec.code,
                            spec.title
                        ))
                        .monospace()
                        .strong()
                        .color(stage_accent),
                    );
                    ui.separator();
                    ui.label(format!("Mastery: {:.0}%", m_score));
                    ui.separator();
                    ui.label(format!("Tier: {}/4", state.active_problem.difficulty));
                });
            }

            if let Some(ctx_banner) = &state.active_problem.domain_context {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "[{}] {}",
                        state.active_problem.domain.short_tag(),
                        ctx_banner
                    ))
                    .italics()
                    .color(stage_accent),
                );
            }

            ui.add_space(6.0);
            ui.heading(&state.active_problem.title);
            ui.add_space(6.0);
            ui.label(RichText::new(&state.active_problem.prompt).size(16.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new(state.active_problem.answer.format_hint())
                    .small()
                    .weak(),
            );
            ui.add_space(8.0);

            let mut submit_now = false;
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Answer:").strong());
                let resp = ui.add(
                    TextEdit::singleline(&mut state.answer_input)
                        .desired_width(400.0)
                        .font(TextStyle::Monospace)
                        .hint_text("Type plain-text expression or value, e.g. 6*x - 2*cos(2*x)"),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    submit_now = true;
                }
                if ui.button("Check Answer").clicked() {
                    submit_now = true;
                }
                if ui.button("Next Question >>").clicked() {
                    state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                }
            });

            // Live Plain-Text Parser Preview
            if !state.answer_input.trim().is_empty() {
                let preview = match &state.active_problem.answer {
                    AnswerFormat::Vector2Expr { .. } => match parse_vector2(&state.answer_input) {
                        Ok((p, q)) => Ok(format!(
                            "Parsed 2D vector: <{}, {}>",
                            p.to_plain_string(),
                            q.to_plain_string()
                        )),
                        Err(e) => Err(e),
                    },
                    _ => match Expr::parse(&state.answer_input) {
                        Ok(ast) => Ok(format!("Parsed expression: {}", ast.to_plain_string())),
                        Err(e) => Err(e),
                    },
                };
                match preview {
                    Ok(msg) => {
                        ui.label(RichText::new(msg).small().monospace().weak());
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

            if submit_now {
                let outcome = state.active_problem.grade(&state.answer_input);
                if outcome.parse_error.is_none() && !state.outcome_recorded {
                    state.outcome_recorded = true;
                    let solve_secs = state.problem_started_at.elapsed().as_secs_f64();
                    let s_delta = score_attempt(
                        state.run_mode,
                        state.active_problem.difficulty,
                        outcome.correct,
                        state.current_streak,
                        solve_secs,
                        state.step_cursor,
                    );
                    state.run_attempted += 1;
                    if outcome.correct {
                        state.run_solved += 1;
                        state.run_score += s_delta.points_awarded;
                        state.last_points_delta = s_delta.points_awarded;
                    } else {
                        state.last_points_delta = 0;
                        // Unlock first method step automatically on miss in Learning Mode
                        if state.step_cursor == 0 {
                            state.step_cursor = 1;
                        }
                    }
                    state.current_streak = s_delta.new_streak;
                    state.max_streak = state.max_streak.max(state.current_streak);

                    actions.push(PracticeAction::RecordAttempt {
                        module_id: state.active_problem.module_id.clone(),
                        title: state.active_problem.title.clone(),
                        user_input: state.answer_input.trim().to_string(),
                        canonical: outcome.canonical_answer.clone(),
                        correct: outcome.correct,
                        hints_used: state.hints_revealed.max(state.step_cursor),
                        solve_secs,
                    });

                    // In Quick Blitz / Medium Timed modes, auto-advance immediately on correct answer!
                    if outcome.correct && state.run_mode.is_timed() {
                        state.advance_problem(selected_module_id, catalog, mastery, free_explore);
                        return;
                    }
                }
                state.last_outcome = Some(outcome);
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
                    ui.label(format!("Hint {}: {}", i + 1, state.active_problem.hints[i]));
                }
            });
    }

    // Outcome Feedback Banner
    if let Some(outcome) = &state.last_outcome {
        ui.add_space(8.0);
        let col = if outcome.correct {
            theme.ok_color()
        } else if outcome.parse_error.is_some() {
            theme.warn_color()
        } else {
            theme.err_color()
        };
        Frame::none()
            .fill(theme.card_fill())
            .stroke(egui::Stroke::new(1.5, col))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                let headline = if outcome.correct {
                    format!("Correct! +{} pts awarded", state.last_points_delta)
                } else if outcome.parse_error.is_some() {
                    "Syntax Check".to_string()
                } else {
                    "Not Quite — Advance the Method Steps Below to Inspect Each Transformation"
                        .to_string()
                };
                ui.label(RichText::new(headline).strong().color(col));
                ui.label(&outcome.feedback);

                if !outcome.correct && outcome.parse_error.is_none() {
                    if let Some(rem_id) = &state.active_problem.remediate_module_id {
                        if let Some(rem_spec) = catalog.get(rem_id) {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "Upstream prerequisite: {} ({})",
                                        rem_spec.title, rem_spec.code
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
        ui.add_space(10.0);
        Frame::none()
            .fill(theme.card_fill())
            .stroke(theme.card_stroke())
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Step-by-Step Visual Method Walkthrough")
                            .strong()
                            .size(16.0)
                            .color(stage_accent),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "Showing {} of {} named method steps (use 'Advance Visual Step >' above to step through)",
                            state.step_cursor,
                            state.active_problem.steps.len()
                        ))
                        .small()
                        .weak(),
                    );
                });

                ui.add_space(6.0);
                ui.columns(2, |cols| {
                    // Left Column: Sequential Method Cards
                    if state.step_cursor == 0 {
                        cols[0].label(
                            "Click 'Advance Visual Step (0/N) >' above whenever you want to reveal the first named mathematical method without spoiling the final answer.",
                        );
                    } else {
                        let unlocked = state
                            .step_cursor
                            .min(state.active_problem.steps.len());
                        for step in &state.active_problem.steps[..unlocked] {
                            Frame::none()
                                .fill(theme.card_fill())
                                .stroke(egui::Stroke::new(1.2, stage_accent))
                                .rounding(Rounding::same(5.0))
                                .inner_margin(Margin::same(9.0))
                                .show(&mut cols[0], |ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "Step {} — Method: {}",
                                            step.step_number, step.method_name
                                        ))
                                        .strong()
                                        .color(stage_accent),
                                    );
                                    ui.label(&step.explanation);
                                    ui.label(
                                        RichText::new(format!(
                                            "State -> {}",
                                            step.resulting_expr
                                        ))
                                        .monospace()
                                        .strong(),
                                    );
                                });
                            cols[0].add_space(6.0);
                        }
                    }

                    // Right Column: Companion Visual Step Plot
                    let var = &state.active_problem.visual_var;
                    Plot::new("learning_step_visual_plot")
                        .height(245.0)
                        .legend(Legend::default())
                        .show(&mut cols[1], |plot_ui| {
                            if let Some(prim) = &state.active_problem.visual_primary_expr {
                                let pts = sample_visual_expr(prim, var, -4.5, 4.5);
                                if pts.len() >= 2 {
                                    plot_ui.line(
                                        Line::new(pts)
                                            .color(Color32::from_rgb(88, 166, 236))
                                            .width(2.0)
                                            .name(format!("Reference Curve ({})", var)),
                                    );
                                }
                            }

                            if state.step_cursor > 0 {
                                let idx = (state.step_cursor - 1)
                                    .min(state.active_problem.steps.len().saturating_sub(1));
                                if let Some(step) = state.active_problem.steps.get(idx) {
                                    if let Some(step_curve) = &step.visual_plot_expr {
                                        let pts = sample_visual_expr(step_curve, var, -4.5, 4.5);
                                        if pts.len() >= 2 {
                                            plot_ui.line(
                                                Line::new(pts)
                                                    .color(Color32::from_rgb(92, 198, 148))
                                                    .width(2.5)
                                                    .name(format!(
                                                        "Step {} Curve",
                                                        step.step_number
                                                    )),
                                            );
                                        }
                                    }
                                }

                                // For numeric answers unlocked at the final step, display exact magnitude bar/marker
                                if state.step_cursor >= state.active_problem.steps.len() {
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

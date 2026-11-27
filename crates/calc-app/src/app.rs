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
use crate::ui_analytics::{render_analytics, AnalyticsAction, AnalyticsViewState};
use crate::ui_lab::{format_ode_curves_csv, format_pde_curves_csv, render_lab, LabMode, LabState, OdePdeSubmode};
use crate::ui_lesson::render_lesson;
use crate::ui_menu::{render_main_menu, MainMenuState, MainMenuSubView};
use crate::ui_notation::{render_notation_studio, NotationStudioState};
use crate::ui_practice::{render_practice, PracticeAction, PracticeState, ProblemSelectionPool};
use crate::ui_roadmap::render_roadmap;
use calc_core::{
    build_curriculum, evaluate_1d_pde, solve_planar_system_euler_timed, solve_planar_system_rk4_timed,
    AttemptRecord, CurriculumCatalog, Expr, LearningRunRecord, ModuleMastery, ModuleSpec,
    ProfileStore, RunMode, ScienceDomain, Stage, StageScopeFilter,
};
use chrono::Utc;
use egui::{
    CentralPanel, ComboBox, Context, Key, RichText, ScrollArea, SidePanel, TextEdit,
    TopBottomPanel,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    MainMenu,
    Roadmap,
    Notation,
    Lesson,
    Practice,
    Lab,
    Analytics,
}

#[derive(Debug, Clone)]
pub enum NavCommand {
    OpenModule { module_id: String, tab: AppTab },
    OpenLabPreset(String),
    OpenTab(AppTab),
    OpenStagePortal(Stage),
    StartStageRun(Stage),
    LaunchRun {
        mode: RunMode,
        scope: StageScopeFilter,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    OpenTab(AppTab),
    OpenLabMode(LabMode),
    OpenOdeSubmode(OdePdeSubmode),
    OpenModuleLesson(String),
    ApplyOdePreset(&'static str),
    SetOdeIntegrators { show_euler: bool, show_midpoint: bool },
    RunOdeConvergenceBenchmark,
    ExportOdeConvergenceJson,
    ExportOdeConvergenceCsv,
    ExportProgressJson,
    ExportOdeCurvesCsv,
    ExportPdeCurvesCsv,
}

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub category: String,
    pub title: String,
    pub subtitle: String,
    pub action: PaletteAction,
    search_blob: String,
}

pub fn build_palette_entries(catalog: &CurriculumCatalog) -> Vec<PaletteEntry> {
    let mut entries = Vec::new();
    let mut push = |cat: &str, title: String, sub: String, act: PaletteAction| {
        let blob = format!(
            "{} {} {}",
            cat.to_ascii_lowercase(),
            title.to_ascii_lowercase(),
            sub.to_ascii_lowercase()
        );
        entries.push(PaletteEntry {
            category: cat.to_string(),
            title,
            subtitle: sub,
            action: act,
            search_blob: blob,
        });
    };

    for (tab, title, sub) in [
        (AppTab::MainMenu, "Tab: Main Menu Hub (1)", "8 Stage Portals & Learning Overview"),
        (AppTab::Roadmap, "Tab: 48-Module DAG Map (2)", "Prerequisite dependency graph across 8 stages"),
        (AppTab::Notation, "Tab: Notation Studio (3)", "Mathematical symbol lexicon and flash drills"),
        (AppTab::Lesson, "Tab: Concept Lesson (4)", "First-principles notes and worked 2D LaTeX steps"),
        (AppTab::Practice, "Tab: Runs & Practice Arena (5)", "Guided walkthroughs and timed sprints"),
        (AppTab::Lab, "Tab: Interactive Visual Lab (6)", "8 mathematical calculus, ODE, PDE, and stats labs"),
        (AppTab::Analytics, "Tab: Leaderboard & Analytics (7)", "SQLite mastery ledger and progress exports"),
    ] {
        push("Tab", title.to_string(), sub.to_string(), PaletteAction::OpenTab(tab));
    }

    for (mode, title, sub) in [
        (LabMode::AlgebraExplorer, "Lab: Algebra & Difference Quotients", "Plot f(x), g(x), and secant slopes"),
        (LabMode::TrigUnitWave, "Lab: Unit Circle & Trig Wave", "Phase-locked circular projection and sinusoids"),
        (LabMode::SingleVarCalculus, "Lab: Single-Variable Calculus", "Derivatives, Riemann sums, and Taylor polynomials"),
        (LabMode::LinearAlgebraMap, "Lab: Linear Algebra & Eigen-Systems", "2x2 SVD, determinant, and phase classification"),
        (LabMode::EpsilonDelta, "Lab: Epsilon-Delta Real Analysis", "Interactive limit neighborhood verifier"),
        (LabMode::MultivariableField, "Lab: Multivariable Contours & 3D Surface", "Scalar level sets, gradient paths, and 3D mesh"),
        (LabMode::ProbabilityStatsLab, "Lab: Probability, Bayes & OLS Regression", "PDF/CDF intervals, Bayesian diagnostic, and OLS"),
    ] {
        push("Visual Lab", title.to_string(), sub.to_string(), PaletteAction::OpenLabMode(mode));
    }

    push(
        "ODE/PDE Lab",
        "Lab: 2D ODE Systems & Phase Portraits".to_string(),
        "Phase plane quiver, nullclines, equilibria, RK4/Midpoint/Euler".to_string(),
        PaletteAction::OpenOdeSubmode(OdePdeSubmode::OdePhasePortrait),
    );
    push(
        "ODE/PDE Lab",
        "Lab: 1D Spectral Fourier PDEs (Heat, Wave, Advection)".to_string(),
        "Fourier-Galerkin modal synthesis and energy conservation".to_string(),
        PaletteAction::OpenOdeSubmode(OdePdeSubmode::PdeEvolution1D),
    );

    for (preset, title, sub) in [
        ("damped", "ODE Preset: Damped Harmonic Oscillator", "x' = y, y' = -2.0*x - 0.35*y"),
        ("pendulum", "ODE Preset: Nonlinear Damped Pendulum", "x' = y, y' = -2.0*sin(x) - 0.15*y"),
        ("lotka", "ODE Preset: Lotka-Volterra Predator-Prey", "x' = x*(1.2 - 0.8*y), y' = y*(-0.9 + 0.6*x)"),
        ("vanderpol", "ODE Preset: Van der Pol Limit Cycle", "x' = y, y' = 1.1*(1 - x^2)*y - x"),
        ("saddle", "ODE Preset: Hyperbolic Saddle Portrait", "x' = x + 0.5*y, y' = 0.5*x - y"),
    ] {
        push("ODE Preset", title.to_string(), sub.to_string(), PaletteAction::ApplyOdePreset(preset));
    }

    push(
        "Integrator",
        "Integrator: Compare RK4 + Midpoint (RK2) + Explicit Euler".to_string(),
        "Overlay all three ODE integrators in phase plane and time series".to_string(),
        PaletteAction::SetOdeIntegrators { show_euler: true, show_midpoint: true },
    );
    push(
        "Integrator",
        "Integrator: RK4 + Midpoint (RK2) Only".to_string(),
        "Show 4th-order RK4 and 2nd-order Midpoint trajectories".to_string(),
        PaletteAction::SetOdeIntegrators { show_euler: false, show_midpoint: true },
    );
    push(
        "Integrator",
        "Integrator: Classical RK4 Only".to_string(),
        "Display pure 4th-order Runge-Kutta trajectory".to_string(),
        PaletteAction::SetOdeIntegrators { show_euler: false, show_midpoint: false },
    );
    push(
        "Integrator",
        "Benchmark: Run ODE Step-Halving Convergence Comparator".to_string(),
        "Compare empirical convergence orders for Euler, Midpoint (RK2), and RK4".to_string(),
        PaletteAction::RunOdeConvergenceBenchmark,
    );

    push(
        "Export",
        "Export: ODE Convergence Benchmark (.json)".to_string(),
        "Write Euler vs Midpoint/RK2 vs RK4 convergence table to JSON".to_string(),
        PaletteAction::ExportOdeConvergenceJson,
    );
    push(
        "Export",
        "Export: ODE Convergence Benchmark (.csv)".to_string(),
        "Write Euler vs Midpoint/RK2 vs RK4 convergence table to CSV".to_string(),
        PaletteAction::ExportOdeConvergenceCsv,
    );
    push(
        "Export",
        "Export: ODE Trajectory Curves (.csv)".to_string(),
        "Save t, x(t), y(t) RK4/Euler curves to CSV".to_string(),
        PaletteAction::ExportOdeCurvesCsv,
    );
    push(
        "Export",
        "Export: 1D Spectral PDE Curves (.csv)".to_string(),
        "Save spatial u(x, 0) and evolved u(x, t) curves to CSV".to_string(),
        PaletteAction::ExportPdeCurvesCsv,
    );
    push(
        "Export",
        "Export: Progress & Mastery Snapshot (.json) (Ctrl+S)".to_string(),
        "Write SQLite profile mastery and run history to JSON".to_string(),
        PaletteAction::ExportProgressJson,
    );

    for spec in &catalog.modules {
        push(
            "Module",
            format!("{} — {}", spec.code, spec.title),
            format!("{} | {}", spec.stage.title(), spec.subtitle),
            PaletteAction::OpenModuleLesson(spec.id.to_string()),
        );
    }

    entries
}

pub fn search_palette_entries<'a>(
    entries: &'a [PaletteEntry],
    query: &str,
    limit: usize,
) -> Vec<&'a PaletteEntry> {
    let trimmed = query.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return entries.iter().take(limit.max(1)).collect();
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut scored: Vec<(i32, &PaletteEntry)> = Vec::new();
    for entry in entries {
        let mut matched = true;
        let mut score = 0_i32;
        let title_lc = entry.title.to_ascii_lowercase();
        for &tk in &tokens {
            if let Some(pos) = entry.search_blob.find(tk) {
                if title_lc.starts_with(tk) {
                    score += 35;
                } else if pos < 18 {
                    score += 18;
                } else {
                    score += 6;
                }
            } else {
                matched = false;
                break;
            }
        }
        if matched {
            scored.push((score, entry));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored
        .into_iter()
        .take(limit.max(1))
        .map(|(_, e)| e)
        .collect()
}

pub struct CalcLabApp {
    pub catalog: CurriculumCatalog,
    pub store: ProfileStore,
    pub mastery: HashMap<String, ModuleMastery>,
    pub learning_runs: Vec<LearningRunRecord>,
    pub recent_attempts: Vec<AttemptRecord>,
    pub active_tab: AppTab,
    pub selected_module_id: String,
    pub selected_domain: ScienceDomain,
    pub show_sidebar: bool,
    pub free_explore: bool,
    pub theme: AcademicTheme,
    pub theme_dirty: bool,
    pub menu_state: MainMenuState,
    pub notation_studio: NotationStudioState,
    pub practice: PracticeState,
    pub lab: LabState,
    pub analytics_view: AnalyticsViewState,
    pub sidebar_search: String,
    pub focus_sidebar_search: bool,
    pub show_about: bool,
    pub cmd_palette_open: bool,
    pub cmd_query: String,
    pub cmd_selected_idx: usize,
    pub cmd_entries: Vec<PaletteEntry>,
    pub db_error_banner: Option<String>,
}

pub fn module_matches_query(spec: &ModuleSpec, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    spec.code.to_lowercase().contains(&q)
        || spec.title.to_lowercase().contains(&q)
        || spec.subtitle.to_lowercase().contains(&q)
        || spec.id.to_lowercase().contains(&q)
        || spec.stage.title().to_lowercase().contains(&q)
        || spec.summary.to_lowercase().contains(&q)
}

impl CalcLabApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_mode(false)
    }

    pub fn new_with_mode(in_memory: bool) -> Self {
        let catalog = build_curriculum();
        let mut db_error_banner = None;
        let store = if in_memory {
            ProfileStore::open_in_memory().expect("Failed to initialize in-memory ProfileStore")
        } else {
            let db_path = PathBuf::from("calclab_profile.db");
            match ProfileStore::open(&db_path) {
                Ok(s) => s,
                Err(e) => {
                    db_error_banner = Some(format!(
                        "Could not open {} ({}); running with in-memory profile.",
                        db_path.display(),
                        e
                    ));
                    ProfileStore::open_in_memory().expect("Failed to initialize SQLite ProfileStore")
                }
            }
        };

        let mastery = store
            .load_all_mastery(&catalog)
            .unwrap_or_else(|e| {
                db_error_banner = Some(format!("Failed to load mastery ledger: {}", e));
                HashMap::new()
            });
        let learning_runs = store.load_learning_runs(100).unwrap_or_default();
        let recent_attempts = store.load_recent_attempts(100).unwrap_or_default();

        let initial_id = "not-sets-quantifiers".to_string();
        let selected_domain = ScienceDomain::General;
        let practice = PracticeState::new(&initial_id, selected_domain);
        let mut lab = LabState::default();
        if let Some(first) = catalog.get(&initial_id) {
            lab.apply_preset(&first.lab_preset);
        }
        let cmd_entries = build_palette_entries(&catalog);

        Self {
            catalog,
            store,
            mastery,
            learning_runs,
            recent_attempts,
            active_tab: AppTab::MainMenu,
            selected_module_id: initial_id,
            selected_domain,
            show_sidebar: false,
            free_explore: true,
            theme: AcademicTheme::SlateDark,
            theme_dirty: true,
            menu_state: MainMenuState::default(),
            notation_studio: NotationStudioState::default(),
            practice,
            lab,
            analytics_view: AnalyticsViewState::default(),
            sidebar_search: String::new(),
            focus_sidebar_search: false,
            show_about: false,
            cmd_palette_open: false,
            cmd_query: String::new(),
            cmd_selected_idx: 0,
            cmd_entries,
            db_error_banner,
        }
    }

    pub(crate) fn apply_nav_command(&mut self, cmd: NavCommand) {
        match cmd {
            NavCommand::OpenModule { module_id, tab } => {
                self.selected_module_id = module_id.clone();
                self.practice.sync_domain(self.selected_domain);
                if tab == AppTab::Practice {
                    self.practice.pool = ProblemSelectionPool::FocusedModule;
                    self.practice.run_mode = RunMode::LearningWalkthrough;
                    self.practice.run_finished = false;
                    self.practice
                        .load_for_module(&module_id, "Focused Module Drill");
                }
                self.active_tab = tab;
            }
            NavCommand::OpenLabPreset(module_id) => {
                self.selected_module_id = module_id.clone();
                if let Some(spec) = self.catalog.get(&module_id) {
                    self.lab.apply_preset(&spec.lab_preset);
                }
                self.active_tab = AppTab::Lab;
            }
            NavCommand::OpenTab(tab) => {
                if tab == AppTab::MainMenu {
                    self.menu_state.subview = MainMenuSubView::HubOverview;
                }
                self.active_tab = tab;
            }
            NavCommand::OpenStagePortal(stage) => {
                self.menu_state.subview = MainMenuSubView::StageDrillDown(stage);
                self.active_tab = AppTab::MainMenu;
            }
            NavCommand::StartStageRun(stage) => {
                self.practice.sync_domain(self.selected_domain);
                self.practice.pool =
                    ProblemSelectionPool::StagePool(StageScopeFilter::Single(stage));
                self.practice.start_fresh_run(
                    RunMode::LearningWalkthrough,
                    &self.selected_module_id,
                    &self.catalog,
                    &self.mastery,
                    self.free_explore,
                );
                self.active_tab = AppTab::Practice;
            }
            NavCommand::LaunchRun { mode, scope } => {
                self.practice.sync_domain(self.selected_domain);
                self.practice.pool = ProblemSelectionPool::StagePool(scope);
                self.practice.start_fresh_run(
                    mode,
                    &self.selected_module_id,
                    &self.catalog,
                    &self.mastery,
                    self.free_explore,
                );
                self.active_tab = AppTab::Practice;
            }
        }
    }
}

impl CalcLabApp {
    pub(crate) fn execute_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::OpenTab(tab) => {
                self.apply_nav_command(NavCommand::OpenTab(tab));
            }
            PaletteAction::OpenLabMode(mode) => {
                self.lab.mode = mode;
                self.active_tab = AppTab::Lab;
            }
            PaletteAction::OpenOdeSubmode(submode) => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = submode;
                self.active_tab = AppTab::Lab;
            }
            PaletteAction::OpenModuleLesson(module_id) => {
                self.apply_nav_command(NavCommand::OpenModule {
                    module_id,
                    tab: AppTab::Lesson,
                });
            }
            PaletteAction::ApplyOdePreset(preset) => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = OdePdeSubmode::OdePhasePortrait;
                match preset {
                    "pendulum" => {
                        self.lab.ode_rhs_p = "y".to_string();
                        self.lab.ode_rhs_q = "-2.0*sin(x) - 0.15*y".to_string();
                        self.lab.ode_seed_x = 2.4;
                        self.lab.ode_seed_y = 0.2;
                    }
                    "lotka" => {
                        self.lab.ode_rhs_p = "x * (1.2 - 0.8*y)".to_string();
                        self.lab.ode_rhs_q = "y * (-0.9 + 0.6*x)".to_string();
                        self.lab.ode_seed_x = 1.8;
                        self.lab.ode_seed_y = 0.8;
                    }
                    "vanderpol" => {
                        self.lab.ode_rhs_p = "y".to_string();
                        self.lab.ode_rhs_q = "1.1*(1 - x^2)*y - x".to_string();
                        self.lab.ode_seed_x = 0.35;
                        self.lab.ode_seed_y = 0.25;
                    }
                    "saddle" => {
                        self.lab.ode_rhs_p = "x + 0.5*y".to_string();
                        self.lab.ode_rhs_q = "0.5*x - y".to_string();
                        self.lab.ode_seed_x = 0.4;
                        self.lab.ode_seed_y = 2.2;
                    }
                    _ => {
                        self.lab.ode_rhs_p = "y".to_string();
                        self.lab.ode_rhs_q = "-2.0*x - 0.35*y".to_string();
                        self.lab.ode_seed_x = 2.2;
                        self.lab.ode_seed_y = 0.0;
                    }
                }
                self.active_tab = AppTab::Lab;
            }
            PaletteAction::SetOdeIntegrators {
                show_euler,
                show_midpoint,
            } => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = OdePdeSubmode::OdePhasePortrait;
                self.lab.ode_show_euler = show_euler;
                self.lab.ode_show_midpoint = show_midpoint;
                self.active_tab = AppTab::Lab;
            }
            PaletteAction::RunOdeConvergenceBenchmark => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = OdePdeSubmode::OdePhasePortrait;
                self.active_tab = AppTab::Lab;
                match self.lab.run_current_ode_benchmark() {
                    Ok(()) => {
                        self.lab.export_status_msg = Some((
                            false,
                            "Computed Euler vs Midpoint/RK2 vs RK4 convergence orders.".to_string(),
                        ));
                    }
                    Err(e) => {
                        self.lab.export_status_msg = Some((true, format!("Benchmark error: {e}")));
                    }
                }
            }
            PaletteAction::ExportOdeConvergenceJson => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = OdePdeSubmode::OdePhasePortrait;
                self.active_tab = AppTab::Lab;
                self.lab.export_ode_benchmark_dialog(false);
            }
            PaletteAction::ExportOdeConvergenceCsv => {
                self.lab.mode = LabMode::OdePdeLab;
                self.lab.ode_submode = OdePdeSubmode::OdePhasePortrait;
                self.active_tab = AppTab::Lab;
                self.lab.export_ode_benchmark_dialog(true);
            }
            PaletteAction::ExportProgressJson => {
                self.export_progress_snapshot_dialog();
            }
            PaletteAction::ExportOdeCurvesCsv => {
                if let (Ok(pe), Ok(qe)) = (
                    Expr::parse(&self.lab.ode_rhs_p),
                    Expr::parse(&self.lab.ode_rhs_q),
                ) {
                    let dt = self.lab.ode_dt.clamp(0.01, 0.25);
                    let t_max = self.lab.ode_t_max.clamp(2.0, 20.0);
                    let steps = ((t_max / dt).round() as usize).clamp(1, 2000);
                    let rk4 = solve_planar_system_rk4_timed(
                        &pe,
                        &qe,
                        self.lab.ode_seed_x,
                        self.lab.ode_seed_y,
                        dt,
                        steps,
                        20.0,
                    );
                    let euler = if self.lab.ode_show_euler {
                        solve_planar_system_euler_timed(
                            &pe,
                            &qe,
                            self.lab.ode_seed_x,
                            self.lab.ode_seed_y,
                            dt,
                            steps,
                            20.0,
                        )
                    } else {
                        Vec::new()
                    };
                    let csv = format_ode_curves_csv(&rk4, &euler);
                    if let Some(p) = rfd::FileDialog::new()
                        .set_file_name("calclab_ode_curves.csv")
                        .add_filter("CSV Data", &["csv"])
                        .save_file()
                    {
                        let res = match fs::write(&p, csv) {
                            Ok(()) => (false, format!("Exported ODE CSV to {}", p.display())),
                            Err(e) => (true, format!("CSV write error: {e}")),
                        };
                        self.lab.export_status_msg = Some(res);
                    }
                }
            }
            PaletteAction::ExportPdeCurvesCsv => {
                let snap = evaluate_1d_pde(
                    self.lab.pde_kind,
                    self.lab.pde_profile,
                    self.lab.pde_alpha,
                    self.lab.pde_speed,
                    self.lab.pde_time,
                    self.lab.pde_modes,
                    120,
                );
                let csv = format_pde_curves_csv(&snap);
                if let Some(p) = rfd::FileDialog::new()
                    .set_file_name("calclab_pde_curves.csv")
                    .add_filter("CSV Data", &["csv"])
                    .save_file()
                {
                    let res = match fs::write(&p, csv) {
                        Ok(()) => (false, format!("Exported PDE CSV to {}", p.display())),
                        Err(e) => (true, format!("CSV write error: {e}")),
                    };
                    self.lab.export_status_msg = Some(res);
                }
            }
        }
    }

    pub(crate) fn export_progress_snapshot_dialog(&mut self) {
        match self.store.export_json(&self.catalog) {
            Ok(json) => {
                if let Some(out_path) = rfd::FileDialog::new()
                    .set_file_name("calclab_progress_export.json")
                    .add_filter("JSON Snapshot", &["json"])
                    .save_file()
                {
                    match fs::write(&out_path, json) {
                        Ok(_) => {
                            self.analytics_view.status_is_error = false;
                            self.analytics_view.status_banner = Some(format!(
                                "Saved progress & run history snapshot to {}",
                                out_path.display()
                            ));
                        }
                        Err(e) => {
                            self.analytics_view.status_is_error = true;
                            self.analytics_view.status_banner =
                                Some(format!("Write error: {}", e));
                        }
                    }
                } else {
                    self.analytics_view.status_is_error = false;
                    self.analytics_view.status_banner =
                        Some("JSON snapshot export cancelled.".to_string());
                }
            }
            Err(e) => {
                self.analytics_view.status_is_error = true;
                self.analytics_view.status_banner = Some(format!("Export error: {}", e));
            }
        }
    }

    pub(crate) fn render_ui(&mut self, ctx: &Context) {
        if self.theme_dirty {
            self.theme.apply(ctx);
            self.theme_dirty = false;
        }

        let wants_kb = ctx.wants_keyboard_input();
        let (f1_pressed, esc_pressed, save_pressed, find_pressed, palette_pressed, space_pressed, tab_hotkey) =
            ctx.input(|i| {
                let cmd_or_ctrl = i.modifiers.command || i.modifiers.ctrl;
                let no_mods = !i.modifiers.ctrl && !i.modifiers.command && !i.modifiers.alt;
                let tab_key = if no_mods {
                    if i.key_pressed(Key::Num1) {
                        Some(AppTab::MainMenu)
                    } else if i.key_pressed(Key::Num2) {
                        Some(AppTab::Roadmap)
                    } else if i.key_pressed(Key::Num3) {
                        Some(AppTab::Notation)
                    } else if i.key_pressed(Key::Num4) {
                        Some(AppTab::Lesson)
                    } else if i.key_pressed(Key::Num5) {
                        Some(AppTab::Practice)
                    } else if i.key_pressed(Key::Num6) {
                        Some(AppTab::Lab)
                    } else if i.key_pressed(Key::Num7) {
                        Some(AppTab::Analytics)
                    } else {
                        None
                    }
                } else {
                    None
                };
                (
                    i.key_pressed(Key::F1),
                    i.key_pressed(Key::Escape),
                    cmd_or_ctrl && i.key_pressed(Key::S),
                    cmd_or_ctrl && i.key_pressed(Key::F),
                    cmd_or_ctrl && i.key_pressed(Key::K),
                    no_mods && i.key_pressed(Key::Space),
                    tab_key,
                )
            });
        if f1_pressed {
            self.show_about = !self.show_about;
        }
        if palette_pressed {
            self.cmd_palette_open = !self.cmd_palette_open;
            self.cmd_query.clear();
            self.cmd_selected_idx = 0;
        }
        if esc_pressed {
            if self.cmd_palette_open {
                self.cmd_palette_open = false;
            } else if self.show_about {
                self.show_about = false;
            } else if self.analytics_view.confirm_reset {
                self.analytics_view.confirm_reset = false;
            } else if !wants_kb {
                if self.active_tab == AppTab::MainMenu {
                    self.menu_state.subview = MainMenuSubView::HubOverview;
                } else {
                    self.active_tab = AppTab::MainMenu;
                }
            }
        }
        if save_pressed {
            self.export_progress_snapshot_dialog();
        }
        if find_pressed {
            self.show_sidebar = true;
            if self.active_tab == AppTab::MainMenu {
                self.active_tab = AppTab::Roadmap;
            }
            self.focus_sidebar_search = true;
        }
        if !wants_kb {
            if let Some(target_tab) = tab_hotkey {
                if target_tab == AppTab::MainMenu {
                    self.menu_state.subview = MainMenuSubView::HubOverview;
                }
                self.active_tab = target_tab;
            }
            if space_pressed
                && self.active_tab == AppTab::Lab
                && self.lab.mode == crate::ui_lab::LabMode::OdePdeLab
                && self.lab.ode_submode == crate::ui_lab::OdePdeSubmode::PdeEvolution1D
            {
                self.lab.pde_animate = !self.lab.pde_animate;
            }
        }

        // Top Navigation & Scientific Domain Selector Bar
        TopBottomPanel::top("calclab_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.active_tab != AppTab::MainMenu {
                    if ui
                        .button(if self.show_sidebar {
                            "Hide Index"
                        } else {
                            "48-Module Index"
                        })
                        .on_hover_text("Toggle the collapsible 48-module curriculum index sidebar (Ctrl+F to search)")
                        .clicked()
                    {
                        self.show_sidebar = !self.show_sidebar;
                    }
                    ui.separator();
                }

                if ui
                    .selectable_label(
                        self.active_tab == AppTab::MainMenu,
                        RichText::new("Main Menu").strong(),
                    )
                    .on_hover_text("Return to the Main Menu Hub and 8-Stage Portals (1 or Esc)")
                    .clicked()
                {
                    self.active_tab = AppTab::MainMenu;
                    self.menu_state.subview = MainMenuSubView::HubOverview;
                }
                ui.separator();

                ui.selectable_value(&mut self.active_tab, AppTab::Roadmap, "DAG Map")
                    .on_hover_text("Interactive 48-module prerequisite dependency graph (Hotkey: 2)");
                ui.selectable_value(&mut self.active_tab, AppTab::Notation, "Notation Studio")
                    .on_hover_text("Mathematical symbol lexicon, spoken phrasing, and flash drills (Hotkey: 3)");
                ui.selectable_value(&mut self.active_tab, AppTab::Lesson, "Lesson")
                    .on_hover_text("First-principles concept notes and worked 2D LaTeX examples (Hotkey: 4)");
                ui.selectable_value(&mut self.active_tab, AppTab::Practice, "Runs & Arena")
                    .on_hover_text("Guided walkthroughs and timed calculation sprints (Hotkey: 5)");
                ui.selectable_value(&mut self.active_tab, AppTab::Lab, "Visual Lab")
                    .on_hover_text("8 interactive mathematical visualization laboratories (Hotkey: 6)");
                ui.selectable_value(&mut self.active_tab, AppTab::Analytics, "Leaderboard")
                    .on_hover_text("SQLite run ledger, mastery analytics, and JSON export (Hotkey: 7, Ctrl+S)");

                ui.separator();
                ui.push_id("global_domain_selector", |ui| {
                    ComboBox::from_label("Domain")
                        .selected_text(self.selected_domain.short_tag())
                        .show_ui(ui, |ui| {
                            for dom in ScienceDomain::ALL {
                                ui.selectable_value(&mut self.selected_domain, dom, dom.label());
                            }
                        })
                        .response
                        .on_hover_text("Switch applied scientific domain framing across problems and lessons");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("About")
                        .on_hover_text("Show Calc-Lab version, license, and CRV Science links (F1)")
                        .clicked()
                    {
                        self.show_about = true;
                    }
                    ui.separator();
                    if ui
                        .button("Jump (Ctrl+K)")
                        .on_hover_text("Open Fuzzy Command Palette across tabs, ODE/PDE presets, integrators, and exports (Ctrl+K)")
                        .clicked()
                    {
                        self.cmd_palette_open = !self.cmd_palette_open;
                        self.cmd_query.clear();
                        self.cmd_selected_idx = 0;
                    }
                    ui.separator();
                    let theme_btn = match self.theme {
                        AcademicTheme::SlateDark => "Paper Light",
                        AcademicTheme::PaperLight => "Slate Dark",
                    };
                    if ui
                        .button(theme_btn)
                        .on_hover_text("Switch between Slate Dark and Paper Light color themes")
                        .clicked()
                    {
                        self.theme = match self.theme {
                            AcademicTheme::SlateDark => AcademicTheme::PaperLight,
                            AcademicTheme::PaperLight => AcademicTheme::SlateDark,
                        };
                        self.theme_dirty = true;
                    }
                    ui.separator();
                    ui.checkbox(&mut self.free_explore, "Unlock All")
                        .on_hover_text("Bypass prerequisite gating to jump freely into any of the 48 modules");
                });
            });
        });

        // Persistent GUI Copyright Footer & Quick Links
        TopBottomPanel::bottom("calclab_footer").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Calc-Lab v{}  |  Copyright CRV Science Network  |  GPL-3.0-or-later",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .small()
                    .weak(),
                );
                ui.separator();
                ui.hyperlink_to("crvscience.com", "https://crvscience.com");
                ui.separator();
                ui.hyperlink_to("crvscience.com/apps", "https://crvscience.com/apps");
                if let Some(err) = self.db_error_banner.clone() {
                    ui.separator();
                    ui.label(
                        RichText::new(err)
                            .small()
                            .color(self.theme.err_color()),
                    );
                    if ui.small_button("Dismiss").clicked() {
                        self.db_error_banner = None;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("About").clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        // Collapsible 48-Module Curriculum Index Sidebar (hidden automatically on Main Menu for clean focus!)
        let mut pending_nav: Option<NavCommand> = None;
        if self.show_sidebar && self.active_tab != AppTab::MainMenu {
            SidePanel::left("calclab_module_sidebar")
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| {
                    let matched_total = self
                        .catalog
                        .modules
                        .iter()
                        .filter(|spec| module_matches_query(spec, &self.sidebar_search))
                        .count();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Curriculum Index ({}/{})",
                                matched_total,
                                self.catalog.modules.len()
                            ))
                            .strong(),
                        );
                        if ui.small_button("Stage Portals").clicked() {
                            pending_nav = Some(NavCommand::OpenTab(AppTab::MainMenu));
                        }
                    });
                    ui.horizontal(|ui| {
                        let search_resp = ui.add(
                            TextEdit::singleline(&mut self.sidebar_search)
                                .desired_width(210.0)
                                .hint_text("Search 48 modules (Ctrl+F)..."),
                        );
                        if self.focus_sidebar_search {
                            search_resp.request_focus();
                            self.focus_sidebar_search = false;
                        }
                        if !self.sidebar_search.is_empty() && ui.small_button("x").clicked() {
                            self.sidebar_search.clear();
                        }
                    });
                    ui.separator();
                    ui.push_id("sidebar_scroll_region", |ui| {
                        ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if matched_total == 0 {
                                    ui.label(
                                        RichText::new("No modules match the current filter.")
                                            .small()
                                            .weak(),
                                    );
                                }
                                for stage in Stage::ALL {
                                    let matching: Vec<&ModuleSpec> = self
                                        .catalog
                                        .by_stage(stage)
                                        .into_iter()
                                        .filter(|spec| {
                                            module_matches_query(spec, &self.sidebar_search)
                                        })
                                        .collect();
                                    if matching.is_empty() {
                                        continue;
                                    }
                                    let accent = self.theme.stage_accent(stage);
                                    ui.add_space(4.0);
                                    if ui
                                        .small_button(
                                            RichText::new(stage.title()).strong().color(accent),
                                        )
                                        .clicked()
                                    {
                                        pending_nav = Some(NavCommand::OpenStagePortal(stage));
                                    }
                                    for spec in matching {
                                        let pct = self
                                            .mastery
                                            .get(spec.id)
                                            .map(|m| m.mastery_score * 100.0)
                                            .unwrap_or(0.0);
                                        let selected = self.selected_module_id == spec.id;
                                        let label = format!(
                                            "{} {} ({:.0}%)",
                                            spec.code,
                                            math_render::ui_safe_text(spec.title),
                                            pct
                                        );
                                        if ui.selectable_label(selected, label).clicked() {
                                            let target_tab = match self.active_tab {
                                                AppTab::MainMenu
                                                | AppTab::Roadmap
                                                | AppTab::Analytics
                                                | AppTab::Notation => AppTab::Lesson,
                                                other => other,
                                            };
                                            if target_tab == AppTab::Lab {
                                                pending_nav = Some(NavCommand::OpenLabPreset(
                                                    spec.id.to_string(),
                                                ));
                                            } else {
                                                pending_nav = Some(NavCommand::OpenModule {
                                                    module_id: spec.id.to_string(),
                                                    tab: target_tab,
                                                });
                                            }
                                        }
                                    }
                                }
                            });
                    });
                });
        }

        // Main Content Canvas
        CentralPanel::default().show(ctx, |ui| {
            ui.push_id("central_scroll_region", |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.active_tab {
                        AppTab::MainMenu => {
                            if let Some(cmd) = render_main_menu(
                                ui,
                                &mut self.menu_state,
                                &mut self.selected_domain,
                                &self.catalog,
                                &self.mastery,
                                &self.learning_runs,
                                self.free_explore,
                                self.theme,
                            ) {
                                pending_nav = Some(cmd);
                            }
                        }
                        AppTab::Roadmap => {
                            if let Some(cmd) = render_roadmap(
                                ui,
                                &self.catalog,
                                &self.mastery,
                                &self.selected_module_id,
                                self.free_explore,
                                self.theme,
                            ) {
                                pending_nav = Some(cmd);
                            }
                        }
                        AppTab::Notation => {
                            if let Some(cmd) = render_notation_studio(
                                ui,
                                &mut self.notation_studio,
                                self.selected_domain,
                                self.theme,
                            ) {
                                pending_nav = Some(cmd);
                            }
                        }
                        AppTab::Lesson => {
                            let spec = self
                                .catalog
                                .get(&self.selected_module_id)
                                .unwrap_or(&self.catalog.modules[0]);
                            if let Some(cmd) = render_lesson(
                                ui,
                                spec,
                                self.selected_domain,
                                &self.catalog,
                                &self.mastery,
                                self.theme,
                            ) {
                                pending_nav = Some(cmd);
                            }
                        }
                        AppTab::Practice => {
                            let actions = render_practice(
                                ctx,
                                ui,
                                &mut self.practice,
                                &self.selected_module_id,
                                self.selected_domain,
                                &self.catalog,
                                &self.mastery,
                                self.free_explore,
                                self.theme,
                            );
                            for act in actions {
                                match act {
                                    PracticeAction::RecordAttempt {
                                        module_id,
                                        title,
                                        user_input,
                                        canonical,
                                        correct,
                                        hints_used,
                                        solve_secs,
                                    } => {
                                        let now = Utc::now();
                                        let entry = self
                                            .mastery
                                            .entry(module_id.clone())
                                            .or_insert_with(|| ModuleMastery::new(&module_id));
                                        entry.record_outcome(correct, hints_used, solve_secs, now);
                                        if let Err(e) = self.store.save_mastery(entry) {
                                            self.db_error_banner =
                                                Some(format!("SQLite mastery save error: {}", e));
                                        }

                                        let rec = AttemptRecord {
                                            id: 0,
                                            timestamp_utc: now,
                                            module_id,
                                            prompt_title: title,
                                            user_input,
                                            expected_canonical: canonical,
                                            correct,
                                            hints_used,
                                            solve_secs,
                                        };
                                        if let Err(e) = self.store.append_attempt(&rec) {
                                            self.db_error_banner =
                                                Some(format!("SQLite attempt log error: {}", e));
                                        }
                                        match self.store.load_recent_attempts(100) {
                                            Ok(rows) => self.recent_attempts = rows,
                                            Err(e) => {
                                                self.db_error_banner = Some(format!(
                                                    "SQLite attempt reload error: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    PracticeAction::RecordLearningRun(run_rec) => {
                                        if let Err(e) = self.store.append_learning_run(&run_rec) {
                                            self.db_error_banner =
                                                Some(format!("SQLite run log error: {}", e));
                                        }
                                        match self.store.load_learning_runs(100) {
                                            Ok(rows) => self.learning_runs = rows,
                                            Err(e) => {
                                                self.db_error_banner =
                                                    Some(format!("SQLite run reload error: {}", e));
                                            }
                                        }
                                    }
                                    PracticeAction::Navigate(cmd) => {
                                        pending_nav = Some(cmd);
                                    }
                                }
                            }
                        }
                        AppTab::Lab => {
                            render_lab(ctx, ui, &mut self.lab, self.theme);
                        }
                        AppTab::Analytics => {
                            if let Some(act) = render_analytics(
                                ui,
                                &mut self.analytics_view,
                                &self.catalog,
                                &self.mastery,
                                &self.learning_runs,
                                &self.recent_attempts,
                                self.theme,
                            ) {
                                match act {
                                    AnalyticsAction::ExportJson => {
                                        self.export_progress_snapshot_dialog();
                                    }
                                    AnalyticsAction::ResetProfile => {
                                        match self.store.reset_all() {
                                            Ok(()) => {
                                                self.mastery = self
                                                    .store
                                                    .load_all_mastery(&self.catalog)
                                                    .unwrap_or_default();
                                                self.learning_runs.clear();
                                                self.recent_attempts.clear();
                                                self.analytics_view.status_is_error = false;
                                                self.analytics_view.status_banner = Some(
                                                    "Profile mastery, run history, and attempt ledger reset."
                                                        .to_string(),
                                                );
                                            }
                                            Err(e) => {
                                                self.analytics_view.status_is_error = true;
                                                self.analytics_view.status_banner = Some(format!(
                                                    "Profile reset error: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
            });
        });

        if self.show_about {
            let mut open = self.show_about;
            egui::Window::new("About Calc-Lab")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading("Calc-Lab — Mathematical Mastery & Scientific Calculus Workbench");
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                            .monospace()
                            .strong(),
                    );
                    ui.label("Copyright CRV Science Network");
                    ui.label("License: GNU General Public License v3.0 or later (GPL-3.0-or-later)");
                    ui.add_space(6.0);
                    ui.label("48-module mastery DAG across 8 stages, 2D LaTeX walkthroughs, ODE phase portraits, and 1D spectral PDEs.");
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Shortcuts: 1-7 Tabs | Ctrl+K Palette | Ctrl+F Search | Space PDE Play/Pause | Ctrl+S Export | F1 About | Esc Back")
                            .small()
                            .weak(),
                    );
                    ui.add_space(6.0);
                    ui.hyperlink_to("CRV Science Network (https://crvscience.com)", "https://crvscience.com");
                    ui.hyperlink_to("CRV Science Apps Hub (https://crvscience.com/apps)", "https://crvscience.com/apps");
                });
            self.show_about = open;
        }

        if self.cmd_palette_open {
            let mut open = self.cmd_palette_open;
            let mut chosen: Option<PaletteAction> = None;
            egui::Window::new("Quick-Jump Command Palette (Ctrl+K)")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(500.0)
                .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 46.0))
                .show(ctx, |ui| {
                    let resp = ui.text_edit_singleline(&mut self.cmd_query);
                    if resp.changed() {
                        self.cmd_selected_idx = 0;
                    }
                    resp.request_focus();
                    let hits = search_palette_entries(&self.cmd_entries, &self.cmd_query, 18);
                    if !hits.is_empty() {
                        self.cmd_selected_idx = self.cmd_selected_idx.min(hits.len() - 1);
                        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                            self.cmd_selected_idx = (self.cmd_selected_idx + 1).min(hits.len() - 1);
                        }
                        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                            self.cmd_selected_idx = self.cmd_selected_idx.saturating_sub(1);
                        }
                        if ctx.input(|i| i.key_pressed(Key::Enter)) {
                            chosen = Some(hits[self.cmd_selected_idx].action.clone());
                        }
                    } else {
                        self.cmd_selected_idx = 0;
                    }
                    ui.small("Use Up/Down to select, Enter to execute, Esc to close.");
                    ui.separator();
                    ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        for (idx, entry) in hits.iter().enumerate() {
                            let is_sel = idx == self.cmd_selected_idx;
                            let lbl = format!("[{}] {} — {}", entry.category, entry.title, entry.subtitle);
                            if ui.selectable_label(is_sel, lbl).clicked() {
                                chosen = Some(entry.action.clone());
                            }
                        }
                    });
                });
            self.cmd_palette_open = open;
            if let Some(act) = chosen {
                self.cmd_palette_open = false;
                self.execute_palette_action(act);
            }
        }

        if let Some(cmd) = pending_nav {
            self.apply_nav_command(cmd);
        }
    }
}

impl eframe::App for CalcLabApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.render_ui(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_module_search_filters_by_code_title_and_stage() {
        let catalog = build_curriculum();
        assert_eq!(catalog.modules.len(), 48);
        let all: Vec<_> = catalog
            .modules
            .iter()
            .filter(|m| module_matches_query(m, ""))
            .collect();
        assert_eq!(all.len(), 48);

        let deq_hits: Vec<_> = catalog
            .modules
            .iter()
            .filter(|m| module_matches_query(m, "DEQ-503"))
            .collect();
        assert_eq!(deq_hits.len(), 1);
        assert_eq!(deq_hits[0].id, "deq-linear-systems-phase");

        let fourier_hits: Vec<_> = catalog
            .modules
            .iter()
            .filter(|m| module_matches_query(m, "fourier"))
            .collect();
        assert!(!fourier_hits.is_empty());
    }

    fn step_frame(app: &mut CalcLabApp, ctx: &Context) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run(raw_input, |c| {
            app.render_ui(c);
        });
    }

    fn step_key_with_mods(
        app: &mut CalcLabApp,
        ctx: &Context,
        key: Key,
        modifiers: egui::Modifiers,
    ) {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            modifiers,
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        };
        let _ = ctx.run(raw_input, |c| {
            app.render_ui(c);
        });
    }

    fn step_key(app: &mut CalcLabApp, ctx: &Context, key: Key) {
        step_key_with_mods(app, ctx, key, egui::Modifiers::NONE);
    }

    #[test]
    fn headless_f1_then_escape_closes_about_without_leaving_lab_tab() {
        let ctx = Context::default();
        let mut app = CalcLabApp::new_with_mode(true);
        app.active_tab = AppTab::Lab;
        app.analytics_view.confirm_reset = true;

        assert!(!app.show_about);

        // F1 opens About modal while staying in AppTab::Lab
        step_key(&mut app, &ctx, Key::F1);
        assert!(app.show_about);
        assert_eq!(app.active_tab, AppTab::Lab);

        // 1st Escape closes About modal and preserves AppTab::Lab & confirm_reset
        step_key(&mut app, &ctx, Key::Escape);
        assert!(!app.show_about);
        assert_eq!(app.active_tab, AppTab::Lab);
        assert!(app.analytics_view.confirm_reset);

        // 2nd Escape dismisses confirm_reset and preserves AppTab::Lab
        step_key(&mut app, &ctx, Key::Escape);
        assert!(!app.analytics_view.confirm_reset);
        assert_eq!(app.active_tab, AppTab::Lab);

        // 3rd Escape returns to MainMenu
        step_key(&mut app, &ctx, Key::Escape);
        assert_eq!(app.active_tab, AppTab::MainMenu);
    }

    #[test]
    fn headless_tab_hotkeys_1_to_7_and_text_edit_guard() {
        let ctx = Context::default();
        let mut app = CalcLabApp::new_with_mode(true);

        // Run one blank frame first
        step_frame(&mut app, &ctx);
        assert_eq!(app.active_tab, AppTab::MainMenu);

        let expected = [
            (Key::Num2, AppTab::Roadmap),
            (Key::Num3, AppTab::Notation),
            (Key::Num4, AppTab::Lesson),
            (Key::Num6, AppTab::Lab),
            (Key::Num7, AppTab::Analytics),
            (Key::Num1, AppTab::MainMenu),
            (Key::Num5, AppTab::Practice),
        ];
        for (k, tab) in expected {
            step_key(&mut app, &ctx, k);
            assert_eq!(app.active_tab, tab);
        }

        // Practice focuses answer_input TextEdit on first render; once focused, Num1 must NOT leave Practice
        assert!(ctx.wants_keyboard_input());
        step_key(&mut app, &ctx, Key::Num1);
        assert_eq!(app.active_tab, AppTab::Practice);
    }

    #[test]
    fn headless_space_toggles_pde_animation_and_ctrl_f_focuses_search() {
        let ctx = Context::default();
        let mut app = CalcLabApp::new_with_mode(true);
        app.active_tab = AppTab::Lab;
        app.lab.mode = crate::ui_lab::LabMode::OdePdeLab;
        app.lab.ode_submode = crate::ui_lab::OdePdeSubmode::PdeEvolution1D;

        step_frame(&mut app, &ctx);
        assert!(!app.lab.pde_animate);

        step_key(&mut app, &ctx, Key::Space);
        assert!(app.lab.pde_animate);

        step_key(&mut app, &ctx, Key::Space);
        assert!(!app.lab.pde_animate);

        // Ctrl+F opens sidebar and focuses sidebar_search TextEdit
        assert!(!app.show_sidebar);
        step_key_with_mods(&mut app, &ctx, Key::F, egui::Modifiers::CTRL);
        assert!(app.show_sidebar);
        assert!(ctx.wants_keyboard_input());

        // With sidebar_search focused, Space must NOT toggle pde_animate and Num1 must NOT switch tabs
        step_key(&mut app, &ctx, Key::Space);
        assert!(!app.lab.pde_animate);
        step_key(&mut app, &ctx, Key::Num1);
        assert_eq!(app.active_tab, AppTab::Lab);
    }

    #[test]
    fn headless_ctrl_k_palette_searches_switches_labs_and_runs_ode_benchmark() {
        let ctx = Context::default();
        let mut app = CalcLabApp::new_with_mode(true);
        step_frame(&mut app, &ctx);

        // Ctrl+K opens palette and focuses search input
        step_key_with_mods(&mut app, &ctx, Key::K, egui::Modifiers::CTRL);
        assert!(app.cmd_palette_open);
        assert!(ctx.wants_keyboard_input());

        // Single-key tab hotkey must be blocked while palette is open
        step_key(&mut app, &ctx, Key::Num6);
        assert_eq!(app.active_tab, AppTab::MainMenu);

        // Escape closes palette first without leaving current tab
        step_key(&mut app, &ctx, Key::Escape);
        assert!(!app.cmd_palette_open);

        // Palette search & execution for ODE preset and convergence benchmark
        let hits = search_palette_entries(&app.cmd_entries, "lotka", 5);
        assert!(hits
            .iter()
            .any(|e| matches!(e.action, PaletteAction::ApplyOdePreset("lotka"))));
        app.execute_palette_action(PaletteAction::ApplyOdePreset("lotka"));
        assert_eq!(app.active_tab, AppTab::Lab);
        assert_eq!(app.lab.mode, LabMode::OdePdeLab);

        app.execute_palette_action(PaletteAction::RunOdeConvergenceBenchmark);
        let bench = app
            .lab
            .ode_convergence_bench
            .as_ref()
            .expect("ODE benchmark computed via palette");
        assert_eq!(bench.rows.len(), 12);
        assert!(bench.mean_order_rk4 > 3.5);
    }
}

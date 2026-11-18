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
use crate::ui_lab::{render_lab, LabState};
use crate::ui_lesson::render_lesson;
use crate::ui_menu::{render_main_menu, MainMenuState, MainMenuSubView};
use crate::ui_notation::{render_notation_studio, NotationStudioState};
use crate::ui_practice::{render_practice, PracticeAction, PracticeState, ProblemSelectionPool};
use crate::ui_roadmap::render_roadmap;
use calc_core::{
    build_curriculum, AttemptRecord, CurriculumCatalog, LearningRunRecord, ModuleMastery,
    ModuleSpec, ProfileStore, RunMode, ScienceDomain, Stage, StageScopeFilter,
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
    pub show_about: bool,
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
            show_about: false,
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
    pub(crate) fn render_ui(&mut self, ctx: &Context) {
        if self.theme_dirty {
            self.theme.apply(ctx);
            self.theme_dirty = false;
        }

        // Escape shortcut returns cleanly to Main Menu Hub when no text box is focused
        if !ctx.wants_keyboard_input() && ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.active_tab == AppTab::MainMenu {
                self.menu_state.subview = MainMenuSubView::HubOverview;
            } else {
                self.active_tab = AppTab::MainMenu;
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
                    .clicked()
                {
                    self.active_tab = AppTab::MainMenu;
                    self.menu_state.subview = MainMenuSubView::HubOverview;
                }
                ui.separator();

                ui.selectable_value(&mut self.active_tab, AppTab::Roadmap, "DAG Map");
                ui.selectable_value(&mut self.active_tab, AppTab::Notation, "Notation Studio");
                ui.selectable_value(&mut self.active_tab, AppTab::Lesson, "Lesson");
                ui.selectable_value(&mut self.active_tab, AppTab::Practice, "Runs & Arena");
                ui.selectable_value(&mut self.active_tab, AppTab::Lab, "Visual Lab");
                ui.selectable_value(&mut self.active_tab, AppTab::Analytics, "Leaderboard");

                ui.separator();
                ui.push_id("global_domain_selector", |ui| {
                    ComboBox::from_label("Domain")
                        .selected_text(self.selected_domain.short_tag())
                        .show_ui(ui, |ui| {
                            for dom in ScienceDomain::ALL {
                                ui.selectable_value(&mut self.selected_domain, dom, dom.label());
                            }
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                    }
                    ui.separator();
                    let theme_btn = match self.theme {
                        AcademicTheme::SlateDark => "Paper Light",
                        AcademicTheme::PaperLight => "Slate Dark",
                    };
                    if ui.button(theme_btn).clicked() {
                        self.theme = match self.theme {
                            AcademicTheme::SlateDark => AcademicTheme::PaperLight,
                            AcademicTheme::PaperLight => AcademicTheme::SlateDark,
                        };
                        self.theme_dirty = true;
                    }
                    ui.separator();
                    ui.checkbox(&mut self.free_explore, "Unlock All");
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
                        ui.add(
                            TextEdit::singleline(&mut self.sidebar_search)
                                .desired_width(210.0)
                                .hint_text("Search 48 modules (e.g. Fourier, DEQ)..."),
                        );
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
                                        match self.store.export_json(&self.catalog) {
                                            Ok(json) => {
                                                if let Some(out_path) = rfd::FileDialog::new()
                                                    .set_file_name("calclab_progress_export.json")
                                                    .add_filter("JSON Snapshot", &["json"])
                                                    .save_file()
                                                {
                                                    match fs::write(&out_path, json) {
                                                        Ok(_) => {
                                                            self.analytics_view.status_is_error =
                                                                false;
                                                            self.analytics_view.status_banner =
                                                                Some(format!(
                                                                "Saved progress & run history snapshot to {}",
                                                                out_path.display()
                                                            ));
                                                        }
                                                        Err(e) => {
                                                            self.analytics_view.status_is_error =
                                                                true;
                                                            self.analytics_view.status_banner =
                                                                Some(format!("Write error: {}", e));
                                                        }
                                                    }
                                                } else {
                                                    self.analytics_view.status_is_error = false;
                                                    self.analytics_view.status_banner = Some(
                                                        "JSON snapshot export cancelled."
                                                            .to_string(),
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                self.analytics_view.status_is_error = true;
                                                self.analytics_view.status_banner =
                                                    Some(format!("Export error: {}", e));
                                            }
                                        }
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
                    ui.add_space(6.0);
                    ui.hyperlink_to("CRV Science Network (https://crvscience.com)", "https://crvscience.com");
                    ui.hyperlink_to("CRV Science Apps Hub (https://crvscience.com/apps)", "https://crvscience.com/apps");
                });
            self.show_about = open;
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
}

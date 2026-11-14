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
    ProfileStore, RunMode, ScienceDomain, Stage, StageScopeFilter,
};
use chrono::Utc;
use egui::{CentralPanel, ComboBox, Context, Key, RichText, ScrollArea, SidePanel, TopBottomPanel};
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
}

impl CalcLabApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let catalog = build_curriculum();
        let db_path = PathBuf::from("calclab_profile.db");
        let store = ProfileStore::open(&db_path)
            .or_else(|_| ProfileStore::open_in_memory())
            .expect("Failed to initialize SQLite ProfileStore");

        let mastery = store
            .load_all_mastery(&catalog)
            .unwrap_or_else(|_| HashMap::new());
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
        }
    }

    fn apply_nav_command(&mut self, cmd: NavCommand) {
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

impl eframe::App for CalcLabApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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

        // Collapsible 48-Module Curriculum Index Sidebar (hidden automatically on Main Menu for clean focus!)
        let mut pending_nav: Option<NavCommand> = None;
        if self.show_sidebar && self.active_tab != AppTab::MainMenu {
            SidePanel::left("calclab_module_sidebar")
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Curriculum Index (48)").strong());
                        if ui.small_button("Stage Portals").clicked() {
                            pending_nav = Some(NavCommand::OpenTab(AppTab::MainMenu));
                        }
                    });
                    ui.separator();
                    ui.push_id("sidebar_scroll_region", |ui| {
                        ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for stage in Stage::ALL {
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
                                    for spec in self.catalog.by_stage(stage) {
                                        let pct = self
                                            .mastery
                                            .get(spec.id)
                                            .map(|m| m.mastery_score * 100.0)
                                            .unwrap_or(0.0);
                                        let selected = self.selected_module_id == spec.id;
                                        let label =
                                            format!("{} {} ({:.0}%)", spec.code, spec.title, pct);
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
                                        let _ = self.store.save_mastery(entry);

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
                                        let _ = self.store.append_attempt(&rec);
                                        self.recent_attempts =
                                            self.store.load_recent_attempts(100).unwrap_or_default();
                                    }
                                    PracticeAction::RecordLearningRun(run_rec) => {
                                        let _ = self.store.append_learning_run(&run_rec);
                                        self.learning_runs =
                                            self.store.load_learning_runs(100).unwrap_or_default();
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
                                                let out_file = "calclab_progress_export.json";
                                                match fs::write(out_file, json) {
                                                    Ok(_) => {
                                                        self.analytics_view.status_banner = Some(
                                                            format!(
                                                                "Saved progress & run history snapshot to {}",
                                                                out_file
                                                            ),
                                                        );
                                                    }
                                                    Err(e) => {
                                                        self.analytics_view.status_banner = Some(
                                                            format!("Write error: {}", e),
                                                        );
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                self.analytics_view.status_banner =
                                                    Some(format!("Export error: {}", e));
                                            }
                                        }
                                    }
                                    AnalyticsAction::ResetProfile => {
                                        if self.store.reset_all().is_ok() {
                                            self.mastery = self
                                                .store
                                                .load_all_mastery(&self.catalog)
                                                .unwrap_or_default();
                                            self.learning_runs.clear();
                                            self.recent_attempts.clear();
                                            self.analytics_view.status_banner = Some(
                                                "Profile mastery, run history, and attempt ledger reset."
                                                    .to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    });
            });
        });

        if let Some(cmd) = pending_nav {
            self.apply_nav_command(cmd);
        }
    }
}

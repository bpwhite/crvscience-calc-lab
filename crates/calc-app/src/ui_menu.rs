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
use crate::theme::AcademicTheme;
use calc_core::{
    is_module_unlocked, pick_next_adaptive_module, summarize_stages, CurriculumCatalog,
    LearningRunRecord, ModuleMastery, RunMode, ScienceDomain, Stage, StageScopeFilter,
};
use chrono::Utc;
use egui::{Frame, Margin, ProgressBar, RichText, Rounding, Stroke, Ui};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuSubView {
    HubOverview,
    StageDrillDown(Stage),
}

pub struct MainMenuState {
    pub subview: MainMenuSubView,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            subview: MainMenuSubView::HubOverview,
        }
    }
}

fn stage_code_tag(stage: Stage) -> &'static str {
    match stage {
        Stage::Notation => "STAGE 0  |  NOT-001..006",
        Stage::Algebra => "STAGE I  |  ALG-101..106",
        Stage::Trigonometry => "STAGE II |  TRG-201..206",
        Stage::Calculus => "STAGE III|  CAL-301..306",
        Stage::LinearAlgebra => "BRIDGE   |  LIN-351..356",
        Stage::AdvancedCalculus => "STAGE IV |  ADV-401..406",
        Stage::DifferentialEquations => "STAGE V  |  DEQ-501..506",
        Stage::ProbabilityStats => "STAGE VI |  STA-601..606",
    }
}

fn stage_tagline(stage: Stage) -> &'static str {
    match stage {
        Stage::Notation => "Decode sets, quantifiers, Sigma/Pi loops, Leibniz bars, Nabla & estimator hats.",
        Stage::Algebra => "Linear systems, completing the square, partial fractions, exponents, logs & series.",
        Stage::Trigonometry => "Radians, unit-circle waves, Pythagorean reductions, inverse triangles & vectors.",
        Stage::Calculus => "Indeterminate limits, chain/product derivatives, Riemann/FTC integrals & Taylor series.",
        Stage::LinearAlgebra => "Vector spaces, linear transformations, determinants, projections & eigensystems.",
        Stage::AdvancedCalculus => "Epsilon-delta bounds, gradients, Hessians, curvilinear Jacobians & Stokes/Fourier.",
        Stage::DifferentialEquations => "Integrating factors, damped oscillators, phase portraits, Laplace & Heat/Wave PDEs.",
        Stage::ProbabilityStats => "Bayes' theorem, Binomial/Poisson/Normal laws, CLT, Maximum Likelihood & regression.",
    }
}

pub fn render_main_menu(
    ui: &mut Ui,
    menu_state: &mut MainMenuState,
    selected_domain: &mut ScienceDomain,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    learning_runs: &[LearningRunRecord],
    free_explore: bool,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    match menu_state.subview {
        MainMenuSubView::HubOverview => render_hub_overview(
            ui,
            menu_state,
            selected_domain,
            catalog,
            mastery,
            learning_runs,
            free_explore,
            theme,
        ),
        MainMenuSubView::StageDrillDown(stage) => render_stage_drill_down(
            ui,
            menu_state,
            stage,
            selected_domain,
            catalog,
            mastery,
            free_explore,
            theme,
        ),
    }
}

fn render_hub_overview(
    ui: &mut Ui,
    menu_state: &mut MainMenuState,
    selected_domain: &mut ScienceDomain,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    learning_runs: &[LearningRunRecord],
    free_explore: bool,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    let mut cmd = None;
    let now = Utc::now();
    let summaries = summarize_stages(catalog, mastery, now);

    let total_modules = catalog.modules.len().max(1);
    let total_mastered: usize = summaries.iter().map(|s| s.mastered_modules).sum();
    let total_due: usize = summaries.iter().map(|s| s.due_reviews).sum();
    let overall_mastery =
        summaries.iter().map(|s| s.mean_mastery).sum::<f64>() / (summaries.len().max(1) as f64);

    let next_rec = pick_next_adaptive_module(catalog, mastery, free_explore, now);
    let next_spec = catalog
        .get(&next_rec.module_id)
        .unwrap_or(&catalog.modules[0]);

    // =========================================================================
    // 1. HERO HEADER & RECOMMENDED NEXT STEP CARD
    // =========================================================================
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(16.0))
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].label(
                    RichText::new("CALCLAB  |  MATHEMATICAL MASTERY WORKBENCH")
                        .small()
                        .monospace()
                        .strong()
                        .color(theme.stage_accent(Stage::Calculus)),
                );
                cols[0].heading("Notation & Algebra -> Calculus -> Linear Algebra -> PDEs -> Statistics");
                cols[0].add_space(4.0);
                cols[0].label(
                    "Choose a scientific domain below, launch a 1-click timed or visual learning run, or step into any of the 8 curriculum stages.",
                );
                cols[0].add_space(8.0);
                cols[0].horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Mastered: {} / {} modules ({:.0}%)",
                            total_mastered,
                            total_modules,
                            overall_mastery * 100.0
                        ))
                        .monospace()
                        .strong()
                        .color(theme.ok_color()),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new(format!("Logged Runs: {}", learning_runs.len()))
                            .monospace(),
                    );
                    ui.separator();
                    if total_due > 0 {
                        ui.label(
                            RichText::new(format!("Due Reviews: {}", total_due))
                                .monospace()
                                .color(theme.warn_color()),
                        );
                    } else {
                        ui.label(RichText::new("Spaced Reviews Up to Date").monospace().weak());
                    }
                });

                // Right Half of Hero: Recommended Next Action + Quick Studio Shortcuts
                let rec_accent = theme.stage_accent(next_spec.stage);
                Frame::none()
                    .fill(theme.card_fill())
                    .stroke(Stroke::new(1.5, rec_accent))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(12.0))
                    .show(&mut cols[1], |ui| {
                        ui.label(
                            RichText::new(format!("RECOMMENDED NEXT: {}", next_rec.reason))
                                .small()
                                .monospace()
                                .strong()
                                .color(rec_accent),
                        );
                        ui.label(
                            RichText::new(format!("{} — {}", next_spec.code, next_spec.title))
                                .strong()
                                .size(16.0),
                        );
                        ui.label(RichText::new(next_spec.subtitle).small().weak());
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Continue Module Lesson").clicked() {
                                cmd = Some(NavCommand::OpenModule {
                                    module_id: next_spec.id.to_string(),
                                    tab: AppTab::Lesson,
                                });
                            }
                            if ui.button("Step-by-Step Practice").clicked() {
                                cmd = Some(NavCommand::OpenModule {
                                    module_id: next_spec.id.to_string(),
                                    tab: AppTab::Practice,
                                });
                            }
                            if ui.button("Notation Studio").clicked() {
                                cmd = Some(NavCommand::OpenTab(AppTab::Notation));
                            }
                            if ui.button("8-Mode Visual Lab").clicked() {
                                cmd = Some(NavCommand::OpenTab(AppTab::Lab));
                            }
                        });
                    });
            });
        });

    ui.add_space(10.0);

    // =========================================================================
    // 2. SCIENTIFIC DOMAIN SELECTION PILLS
    // =========================================================================
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(7.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Equation & Context Domain:").strong());
                for dom in ScienceDomain::ALL {
                    let active = *selected_domain == dom;
                    let label = format!("[{}] {}", dom.short_tag(), dom.label());
                    if ui.selectable_label(active, label).clicked() {
                        *selected_domain = dom;
                    }
                }
            });
            if let Some(sample_ctx) =
                selected_domain.context_banner(Stage::DifferentialEquations, "deq-first-order")
            {
                ui.label(
                    RichText::new(format!("Active Lens Preview — {}", sample_ctx))
                        .small()
                        .italics()
                        .weak(),
                );
            } else {
                ui.label(
                    RichText::new(
                        "Active Lens Preview — Pure mathematical formulation across all 8 stages (select Biology, Physics, Chemistry, Ecology, or Microbiology above to contextualize problems).",
                    )
                    .small()
                    .italics()
                    .weak(),
                );
            }
        });

    ui.add_space(10.0);

    // =========================================================================
    // 3. 1-CLICK GAME & LEARNING RUN LAUNCHERS (4 MODE CARDS)
    // =========================================================================
    ui.label(
        RichText::new("1-Click Game & Learning Run Launchers (All 8 Stages)")
            .strong()
            .size(16.0),
    );
    ui.add_space(4.0);

    let mode_descriptions: [(RunMode, &str, &str); 4] = [
        (
            RunMode::LearningWalkthrough,
            "Untimed guided runs with step-by-step named methods & live companion plots.",
            "Start Learning Run",
        ),
        (
            RunMode::QuickBlitz90s,
            "90-second high-tempo sprint with speed bonuses and up to 3.0x combo multipliers.",
            "Launch 90s Blitz",
        ),
        (
            RunMode::MediumStandard5m,
            "5-minute endurance run mixing concepts across stages with full score tracking.",
            "Launch 5m Challenge",
        ),
        (
            RunMode::InfiniteEndless,
            "Untimed endless score & streak run—practice as long as you like and bank your score.",
            "Launch Infinite Run",
        ),
    ];

    ui.columns(4, |cols| {
        for (idx, (mode, desc, btn_label)) in mode_descriptions.iter().enumerate() {
            let code = mode.short_code();
            let best_score = learning_runs
                .iter()
                .filter(|r| r.mode == code)
                .map(|r| r.score)
                .max()
                .unwrap_or(0);
            let run_count = learning_runs.iter().filter(|r| r.mode == code).count();

            let card_accent = match mode {
                RunMode::LearningWalkthrough => theme.stage_accent(Stage::Notation),
                RunMode::QuickBlitz90s => theme.stage_accent(Stage::DifferentialEquations),
                RunMode::MediumStandard5m => theme.stage_accent(Stage::Algebra),
                RunMode::InfiniteEndless => theme.stage_accent(Stage::ProbabilityStats),
            };

            Frame::none()
                .fill(theme.card_fill())
                .stroke(Stroke::new(1.2, card_accent))
                .rounding(Rounding::same(7.0))
                .inner_margin(Margin::same(12.0))
                .show(&mut cols[idx], |ui| {
                    ui.label(
                        RichText::new(mode.short_code())
                            .small()
                            .monospace()
                            .strong()
                            .color(card_accent),
                    );
                    ui.label(RichText::new(mode.label()).strong().size(15.0));
                    ui.add_space(3.0);
                    ui.label(RichText::new(*desc).small().weak());
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "Best: {} pts  ({} runs)",
                            best_score, run_count
                        ))
                        .small()
                        .monospace(),
                    );
                    ui.add_space(6.0);
                    if ui.button(RichText::new(*btn_label).strong()).clicked() {
                        cmd = Some(NavCommand::LaunchRun {
                            mode: *mode,
                            scope: StageScopeFilter::AllStages,
                        });
                    }
                });
        }
    });

    ui.add_space(12.0);

    // =========================================================================
    // 4. 8-STAGE CURRICULUM PORTAL CARDS (2 ROWS x 4 CARDS)
    // =========================================================================
    ui.horizontal_wrapped(|ui| {
        ui.label(
            RichText::new("Curriculum Stage Portals (Click Any Stage for Focused 6-Module View)")
                .strong()
                .size(16.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("View Full 48-Module DAG Board").clicked() {
                cmd = Some(NavCommand::OpenTab(AppTab::Roadmap));
            }
        });
    });
    ui.add_space(4.0);

    for row in summaries.chunks(4) {
        ui.columns(4, |cols| {
            for (idx, summary) in row.iter().enumerate() {
                let st = summary.stage;
                let accent = theme.stage_accent(st);

                Frame::none()
                    .fill(theme.card_fill())
                    .stroke(Stroke::new(1.3, accent))
                    .rounding(Rounding::same(7.0))
                    .inner_margin(Margin::same(12.0))
                    .show(&mut cols[idx], |ui| {
                        ui.label(
                            RichText::new(stage_code_tag(st))
                                .small()
                                .monospace()
                                .strong()
                                .color(accent),
                        );
                        ui.label(RichText::new(st.title()).strong().size(15.5));
                        ui.add_space(3.0);
                        ui.label(RichText::new(stage_tagline(st)).small().weak());
                        ui.add_space(6.0);

                        ui.add(ProgressBar::new(summary.mean_mastery as f32).text(format!(
                            "{}/{} mastered ({:.0}%)",
                            summary.mastered_modules,
                            summary.total_modules,
                            summary.mean_mastery * 100.0
                        )));

                        if summary.due_reviews > 0 {
                            ui.label(
                                RichText::new(format!("{} module(s) due for review", summary.due_reviews))
                                    .small()
                                    .color(theme.warn_color()),
                            );
                        } else {
                            ui.label(RichText::new("6 focused modules").small().weak());
                        }

                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui
                                .button(RichText::new("Open Stage >>").strong())
                                .clicked()
                            {
                                menu_state.subview = MainMenuSubView::StageDrillDown(st);
                            }
                            if ui.small_button("90s Blitz").clicked() {
                                cmd = Some(NavCommand::LaunchRun {
                                    mode: RunMode::QuickBlitz90s,
                                    scope: StageScopeFilter::Single(st),
                                });
                            }
                            if ui.small_button("Learn").clicked() {
                                cmd = Some(NavCommand::LaunchRun {
                                    mode: RunMode::LearningWalkthrough,
                                    scope: StageScopeFilter::Single(st),
                                });
                            }
                        });
                    });
            }
        });
        ui.add_space(8.0);
    }

    cmd
}

fn render_stage_drill_down(
    ui: &mut Ui,
    menu_state: &mut MainMenuState,
    stage: Stage,
    selected_domain: &mut ScienceDomain,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    free_explore: bool,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    let mut cmd = None;
    let now = Utc::now();
    let accent = theme.stage_accent(stage);
    let stage_mods = catalog.by_stage(stage);

    // Top Breadcrumb & Instant Stage Switcher Strip
    ui.horizontal_wrapped(|ui| {
        if ui.button("<< Back to Main Menu Hub").clicked() {
            menu_state.subview = MainMenuSubView::HubOverview;
        }
        ui.separator();
        for st in Stage::ALL {
            if ui
                .selectable_label(st == stage, st.short_label())
                .clicked()
            {
                menu_state.subview = MainMenuSubView::StageDrillDown(st);
            }
        }
    });

    ui.add_space(8.0);

    // Stage Banner + Domain Pills + Stage-Scoped Run Launchers
    let mastered_cnt = stage_mods
        .iter()
        .filter(|m| {
            mastery
                .get(m.id)
                .map(|s| s.is_mastered())
                .unwrap_or(false)
        })
        .count();
    let mean_pct = stage_mods
        .iter()
        .map(|m| mastery.get(m.id).map(|s| s.mastery_score).unwrap_or(0.0))
        .sum::<f64>()
        / (stage_mods.len().max(1) as f64)
        * 100.0;

    Frame::none()
        .fill(theme.card_fill())
        .stroke(Stroke::new(1.6, accent))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(stage_code_tag(stage))
                        .monospace()
                        .strong()
                        .color(accent),
                );
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Stage Mastery: {:.0}% ({} / {} modules mastered)",
                        mean_pct,
                        mastered_cnt,
                        stage_mods.len()
                    ))
                    .monospace(),
                );
                if stage == Stage::Notation && ui.small_button("Open Interactive Notation Studio").clicked() {
                    cmd = Some(NavCommand::OpenTab(AppTab::Notation));
                }
            });

            ui.heading(stage.title());
            ui.label(stage_tagline(stage));

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Domain Lens:").strong());
                for dom in ScienceDomain::ALL {
                    if ui
                        .selectable_label(*selected_domain == dom, dom.short_tag())
                        .on_hover_text(dom.label())
                        .clicked()
                    {
                        *selected_domain = dom;
                    }
                }
                ui.separator();
                ui.label(RichText::new("Stage Runs:").strong());
                if ui.button("Start Stage Learning Run").clicked() {
                    cmd = Some(NavCommand::LaunchRun {
                        mode: RunMode::LearningWalkthrough,
                        scope: StageScopeFilter::Single(stage),
                    });
                }
                if ui.button("90s Stage Blitz").clicked() {
                    cmd = Some(NavCommand::LaunchRun {
                        mode: RunMode::QuickBlitz90s,
                        scope: StageScopeFilter::Single(stage),
                    });
                }
                if ui.button("5m Stage Challenge").clicked() {
                    cmd = Some(NavCommand::LaunchRun {
                        mode: RunMode::MediumStandard5m,
                        scope: StageScopeFilter::Single(stage),
                    });
                }
                if ui.button("Infinite Stage Run").clicked() {
                    cmd = Some(NavCommand::LaunchRun {
                        mode: RunMode::InfiniteEndless,
                        scope: StageScopeFilter::Single(stage),
                    });
                }
            });
        });

    ui.add_space(12.0);

    // Focused 3x2 Grid of the 6 Modules in This Stage
    for row_mods in stage_mods.chunks(3) {
        ui.columns(3, |cols| {
            for (idx, spec) in row_mods.iter().enumerate() {
                let st = mastery.get(spec.id);
                let score = st.map(|m| m.mastery_score).unwrap_or(0.0);
                let attempts = st.map(|m| m.attempts).unwrap_or(0);
                let streak = st.map(|m| m.streak).unwrap_or(0);
                let unlocked = is_module_unlocked(spec, mastery, free_explore);
                let due = st.map(|m| m.is_due(now)).unwrap_or(false);

                Frame::none()
                    .fill(theme.card_fill())
                    .stroke(theme.card_stroke())
                    .rounding(Rounding::same(7.0))
                    .inner_margin(Margin::same(12.0))
                    .show(&mut cols[idx], |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Module {}/6  |  {}",
                                    spec.order_in_stage, spec.code
                                ))
                                .monospace()
                                .strong()
                                .color(accent),
                            );
                            if due {
                                ui.label(
                                    RichText::new("[REVIEW DUE]")
                                        .small()
                                        .strong()
                                        .color(theme.warn_color()),
                                );
                            } else if score >= 0.75 {
                                ui.label(
                                    RichText::new("[MASTERED]")
                                        .small()
                                        .strong()
                                        .color(theme.ok_color()),
                                );
                            } else if !unlocked {
                                ui.label(RichText::new("[LOCKED]").small().weak());
                            }
                        });

                        ui.add_space(2.0);
                        ui.label(RichText::new(spec.title).strong().size(15.5));
                        ui.label(RichText::new(spec.subtitle).small().italics());
                        ui.add_space(4.0);
                        ui.label(RichText::new(spec.summary).small().weak());
                        ui.add_space(6.0);

                        // Prerequisite Pills (clickable to jump directly to upstream stage/lesson!)
                        if spec.prerequisites.is_empty() {
                            ui.label(
                                RichText::new("Prerequisites: None (Entry Module)")
                                    .small()
                                    .monospace(),
                            );
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("Prereqs:").small().monospace());
                                for &pid in spec.prerequisites {
                                    if let Some(p_spec) = catalog.get(pid) {
                                        let ready = mastery
                                            .get(pid)
                                            .map(|m| m.is_unlocked_ready())
                                            .unwrap_or(false);
                                        let p_col = if ready {
                                            theme.ok_color()
                                        } else {
                                            theme.warn_color()
                                        };
                                        if ui
                                            .small_button(
                                                RichText::new(p_spec.code)
                                                    .monospace()
                                                    .color(p_col),
                                            )
                                            .on_hover_text(p_spec.title)
                                            .clicked()
                                        {
                                            cmd = Some(NavCommand::OpenModule {
                                                module_id: p_spec.id.to_string(),
                                                tab: AppTab::Lesson,
                                            });
                                        }
                                    }
                                }
                            });
                        }

                        ui.add_space(4.0);
                        ui.add(ProgressBar::new(score as f32).text(format!(
                            "{:.0}% mastery  |  {} attempts  |  streak {}",
                            score * 100.0,
                            attempts,
                            streak
                        )));

                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Read Lesson").clicked() {
                                cmd = Some(NavCommand::OpenModule {
                                    module_id: spec.id.to_string(),
                                    tab: AppTab::Lesson,
                                });
                            }
                            ui.add_enabled_ui(unlocked, |ui| {
                                if ui.button("Practice / Learn").clicked() {
                                    cmd = Some(NavCommand::OpenModule {
                                        module_id: spec.id.to_string(),
                                        tab: AppTab::Practice,
                                    });
                                }
                            });
                            if ui.button("Visual Lab").clicked() {
                                cmd = Some(NavCommand::OpenLabPreset(spec.id.to_string()));
                            }
                        });
                    });
            }
        });
        ui.add_space(10.0);
    }

    cmd
}

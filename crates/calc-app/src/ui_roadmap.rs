use crate::app::{AppTab, NavCommand};
use crate::theme::AcademicTheme;
use calc_core::{
    is_module_unlocked, summarize_stages, CurriculumCatalog, ModuleMastery, ModuleSpec, Stage,
};
use chrono::{DateTime, Utc};
use egui::{Frame, Margin, ProgressBar, RichText, Rounding, Ui};
use std::collections::HashMap;

fn render_module_card(
    ui: &mut Ui,
    spec: &ModuleSpec,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    selected_module_id: &str,
    free_explore: bool,
    theme: AcademicTheme,
    now: DateTime<Utc>,
    cmd: &mut Option<NavCommand>,
) {
    let accent = theme.stage_accent(spec.stage);
    let st = mastery.get(spec.id);
    let score = st.map(|m| m.mastery_score).unwrap_or(0.0);
    let unlocked = is_module_unlocked(spec, mastery, free_explore);
    let due = st.map(|m| m.is_due(now)).unwrap_or(false);
    let is_selected = spec.id == selected_module_id;

    let mut border = theme.card_stroke();
    if is_selected {
        border = egui::Stroke::new(2.0, accent);
    }

    Frame::none()
        .fill(theme.card_fill())
        .stroke(border)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(spec.code)
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

            ui.label(RichText::new(spec.title).strong());
            ui.label(RichText::new(spec.subtitle).small().weak());

            if !spec.prerequisites.is_empty() {
                let prereq_codes: Vec<String> = spec
                    .prerequisites
                    .iter()
                    .filter_map(|pid| catalog.get(pid).map(|m| m.code.to_string()))
                    .collect();
                ui.label(
                    RichText::new(format!("Requires: {}", prereq_codes.join(", ")))
                        .small()
                        .monospace(),
                );
            } else {
                ui.label(RichText::new("Entry Foundation").small().monospace());
            }

            ui.add(
                ProgressBar::new(score as f32).text(format!("{:.0}% mastery", score * 100.0)),
            );

            ui.horizontal(|ui| {
                if ui.small_button("Lesson").clicked() {
                    *cmd = Some(NavCommand::OpenModule {
                        module_id: spec.id.to_string(),
                        tab: AppTab::Lesson,
                    });
                }
                ui.add_enabled_ui(unlocked, |ui| {
                    if ui.small_button("Train / Run").clicked() {
                        *cmd = Some(NavCommand::OpenModule {
                            module_id: spec.id.to_string(),
                            tab: AppTab::Practice,
                        });
                    }
                });
                if ui.small_button("Visual Lab").clicked() {
                    *cmd = Some(NavCommand::OpenLabPreset(spec.id.to_string()));
                }
            });
        });
}

fn render_stage_column(
    ui: &mut Ui,
    stage: Stage,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    selected_module_id: &str,
    free_explore: bool,
    theme: AcademicTheme,
    now: DateTime<Utc>,
    cmd: &mut Option<NavCommand>,
) {
    let accent = theme.stage_accent(stage);
    ui.horizontal(|ui| {
        ui.label(RichText::new(stage.title()).strong().size(15.5).color(accent));
        if stage == Stage::Notation && ui.small_button("Open Notation Studio").clicked() {
            *cmd = Some(NavCommand::OpenTab(AppTab::Notation));
        }
    });
    ui.add_space(4.0);

    for spec in catalog.by_stage(stage) {
        render_module_card(
            ui,
            spec,
            catalog,
            mastery,
            selected_module_id,
            free_explore,
            theme,
            now,
            cmd,
        );
        ui.add_space(8.0);
    }
}

pub fn render_roadmap(
    ui: &mut Ui,
    catalog: &CurriculumCatalog,
    mastery: &HashMap<String, ModuleMastery>,
    selected_module_id: &str,
    free_explore: bool,
    theme: AcademicTheme,
) -> Option<NavCommand> {
    let mut cmd = None;
    let now = Utc::now();
    let summaries = summarize_stages(catalog, mastery, now);

    ui.heading(
        "8-Stage Curriculum DAG (48 Modules): Notation -> Algebra -> Trig -> Calc -> LinAlg -> AdvCalc -> ODE/PDE -> Prob & Stats",
    );
    ui.label(
        "Begin with Stage 0 (Notation & Symbolic Literacy) or the interactive Notation Studio tab to decode mathematical symbols into plain-text operations before tackling advanced stages.",
    );
    ui.add_space(6.0);

    // 8-Stage Summary Cards in two clean 4-column rows
    for band in summaries.chunks(4) {
        ui.columns(band.len(), |cols| {
            for (idx, summary) in band.iter().enumerate() {
                let accent = theme.stage_accent(summary.stage);
                Frame::none()
                    .fill(theme.card_fill())
                    .stroke(theme.card_stroke())
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(9.0))
                    .show(&mut cols[idx], |ui| {
                        ui.label(RichText::new(summary.stage.title()).strong().color(accent));
                        ui.label(format!(
                            "Mastered: {} / {} modules",
                            summary.mastered_modules, summary.total_modules
                        ));
                        ui.add(ProgressBar::new(summary.mean_mastery as f32).text(format!(
                            "Mean {:.0}%",
                            summary.mean_mastery * 100.0
                        )));
                        if summary.due_reviews > 0 {
                            ui.label(
                                RichText::new(format!(
                                    "{} due for review",
                                    summary.due_reviews
                                ))
                                .small()
                                .color(theme.warn_color()),
                            );
                        } else {
                            ui.label(RichText::new("Review queue clear").small().weak());
                        }
                    });
            }
        });
        ui.add_space(6.0);
    }

    // 4 spacious 2-column stage bands (8 stages total)
    for pair in Stage::ALL.chunks(2) {
        ui.separator();
        ui.add_space(6.0);
        ui.columns(2, |cols| {
            for (idx, &st) in pair.iter().enumerate() {
                render_stage_column(
                    &mut cols[idx],
                    st,
                    catalog,
                    mastery,
                    selected_module_id,
                    free_explore,
                    theme,
                    now,
                    &mut cmd,
                );
            }
        });
        ui.add_space(4.0);
    }

    cmd
}

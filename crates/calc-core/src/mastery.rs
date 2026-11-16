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

use crate::curriculum::{CurriculumCatalog, ModuleSpec, Stage};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMastery {
    pub module_id: String,
    pub mastery_score: f64,
    pub ease_factor: f64,
    pub interval_hours: f64,
    pub streak: u32,
    pub attempts: u32,
    pub correct_count: u32,
    pub avg_solve_secs: f64,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub next_review: Option<DateTime<Utc>>,
}

impl ModuleMastery {
    pub fn new(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            mastery_score: 0.0,
            ease_factor: 2.5,
            interval_hours: 0.0,
            streak: 0,
            attempts: 0,
            correct_count: 0,
            avg_solve_secs: 0.0,
            last_reviewed: None,
            next_review: None,
        }
    }

    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.correct_count as f64) / (self.attempts as f64)
        }
    }

    pub fn is_mastered(&self) -> bool {
        self.mastery_score >= 0.75
    }

    pub fn is_unlocked_ready(&self) -> bool {
        self.mastery_score >= 0.60
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        match self.next_review {
            Some(due) => self.attempts > 0 && due <= now,
            None => false,
        }
    }

    /// Updates SM-2 spaced repetition & continuous mastery score based on outcome, hints used, and solve duration.
    pub fn record_outcome(
        &mut self,
        correct: bool,
        hints_used: usize,
        solve_secs: f64,
        now: DateTime<Utc>,
    ) {
        let prev_attempts = self.attempts as f64;
        self.attempts += 1;
        let clamped_secs = solve_secs.clamp(1.0, 600.0);
        self.avg_solve_secs =
            (self.avg_solve_secs * prev_attempts + clamped_secs) / (self.attempts as f64);
        self.last_reviewed = Some(now);

        if correct {
            self.correct_count += 1;
            self.streak += 1;

            // Quality 3..=5 depending on hints and speed
            let quality: f64 = match hints_used {
                0 if clamped_secs <= 45.0 => 5.0,
                0 => 4.5,
                1 => 3.8,
                _ => 3.1,
            };

            let gain = match hints_used {
                0 => 0.28,
                1 => 0.18,
                2 => 0.12,
                _ => 0.08,
            };
            self.mastery_score = (self.mastery_score + gain * (1.0 - 0.35 * self.mastery_score))
                .clamp(0.0, 1.0);

            self.ease_factor = (self.ease_factor
                + (0.1 - (5.0 - quality) * (0.08 + (5.0 - quality) * 0.02)))
                .clamp(1.3, 3.2);

            self.interval_hours = match self.streak {
                1 => 12.0,
                2 => 36.0,
                _ => (self.interval_hours.max(24.0) * self.ease_factor).min(24.0 * 60.0),
            };
        } else {
            self.streak = 0;
            self.mastery_score = (self.mastery_score * 0.72).clamp(0.0, 1.0);
            self.ease_factor = (self.ease_factor - 0.20).clamp(1.3, 3.2);
            self.interval_hours = 4.0;
        }

        let mins = (self.interval_hours * 60.0).round() as i64;
        self.next_review = Some(now + Duration::minutes(mins));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptRecord {
    pub id: i64,
    pub timestamp_utc: DateTime<Utc>,
    pub module_id: String,
    pub prompt_title: String,
    pub user_input: String,
    pub expected_canonical: String,
    pub correct: bool,
    pub hints_used: usize,
    pub solve_secs: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionMode {
    AdaptiveFrontierAndReview,
    FocusedModule,
}

#[derive(Debug, Clone)]
pub struct SessionItem {
    pub module_id: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone)]
pub struct StageProgressSummary {
    pub stage: Stage,
    pub total_modules: usize,
    pub mastered_modules: usize,
    pub mean_mastery: f64,
    pub due_reviews: usize,
}

pub fn is_module_unlocked(
    spec: &ModuleSpec,
    mastery_map: &HashMap<String, ModuleMastery>,
    free_explore: bool,
) -> bool {
    if free_explore || spec.prerequisites.is_empty() {
        return true;
    }
    spec.prerequisites.iter().all(|prereq_id| {
        mastery_map
            .get(*prereq_id)
            .map(|m| m.is_unlocked_ready())
            .unwrap_or(false)
    })
}

pub fn summarize_stages(
    catalog: &CurriculumCatalog,
    mastery_map: &HashMap<String, ModuleMastery>,
    now: DateTime<Utc>,
) -> Vec<StageProgressSummary> {
    Stage::ALL
        .iter()
        .map(|&stage| {
            let mods = catalog.by_stage(stage);
            let total = mods.len().max(1);
            let mut mastered = 0usize;
            let mut sum_mastery = 0.0;
            let mut due = 0usize;

            for m in &mods {
                if let Some(st) = mastery_map.get(m.id) {
                    sum_mastery += st.mastery_score;
                    if st.is_mastered() {
                        mastered += 1;
                    }
                    if st.is_due(now) {
                        due += 1;
                    }
                }
            }

            StageProgressSummary {
                stage,
                total_modules: mods.len(),
                mastered_modules: mastered,
                mean_mastery: sum_mastery / (total as f64),
                due_reviews: due,
            }
        })
        .collect()
}

pub fn pick_next_adaptive_module(
    catalog: &CurriculumCatalog,
    mastery_map: &HashMap<String, ModuleMastery>,
    free_explore: bool,
    now: DateTime<Utc>,
) -> SessionItem {
    // 1. Prioritize overdue spaced-repetition modules
    for m in &catalog.modules {
        if let Some(st) = mastery_map.get(m.id) {
            if st.is_due(now) {
                return SessionItem {
                    module_id: m.id.to_string(),
                    reason: "Due for Spaced-Repetition Review",
                };
            }
        }
    }

    // 2. Otherwise select the earliest unlocked module not yet mastered
    for m in &catalog.modules {
        let unlocked = is_module_unlocked(m, mastery_map, free_explore);
        let mastered = mastery_map
            .get(m.id)
            .map(|s| s.is_mastered())
            .unwrap_or(false);
        if unlocked && !mastered {
            return SessionItem {
                module_id: m.id.to_string(),
                reason: "Active Frontier Progression",
            };
        }
    }

    // 3. Fallback: lowest-mastery module for synthesis polish
    let lowest = catalog
        .modules
        .iter()
        .min_by(|a, b| {
            let ma = mastery_map.get(a.id).map(|s| s.mastery_score).unwrap_or(0.0);
            let mb = mastery_map.get(b.id).map(|s| s.mastery_score).unwrap_or(0.0);
            ma.partial_cmp(&mb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| m.id)
        .unwrap_or("alg-linear-systems");

    SessionItem {
        module_id: lowest.to_string(),
        reason: "Full-Curriculum Reinforcement",
    }
}

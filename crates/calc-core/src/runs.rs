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

use crate::curriculum::Stage;
use crate::domains::ScienceDomain;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    /// Untimed step-by-step guided walkthrough with named mathematical methods & visual step plots
    LearningWalkthrough,
    /// 90-second fast sprint with speed bonus & combo multiplier
    QuickBlitz90s,
    /// 5-minute (300s) standard timed run
    MediumStandard5m,
    /// Endless score & streak run (untimed continuous play; user finishes anytime or on 3 strikes if survival enabled)
    InfiniteEndless,
}

impl RunMode {
    pub const ALL: [RunMode; 4] = [
        RunMode::LearningWalkthrough,
        RunMode::QuickBlitz90s,
        RunMode::MediumStandard5m,
        RunMode::InfiniteEndless,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::LearningWalkthrough => "Learning Mode (Step-by-Step Visual)",
            Self::QuickBlitz90s => "Quick Blitz (90s Timed)",
            Self::MediumStandard5m => "Medium Run (5m Timed)",
            Self::InfiniteEndless => "Infinite Mode (Endless Score)",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            Self::LearningWalkthrough => "LEARN",
            Self::QuickBlitz90s => "BLITZ-90S",
            Self::MediumStandard5m => "MEDIUM-5M",
            Self::InfiniteEndless => "INFINITE",
        }
    }

    pub fn duration_limit_secs(&self) -> Option<f64> {
        match self {
            Self::LearningWalkthrough => None,
            Self::QuickBlitz90s => Some(90.0),
            Self::MediumStandard5m => Some(300.0),
            Self::InfiniteEndless => None,
        }
    }

    pub fn is_timed(&self) -> bool {
        self.duration_limit_secs().is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRunRecord {
    pub id: i64,
    pub finished_at_utc: DateTime<Utc>,
    pub mode: String,
    pub domain: String,
    pub stage_scope: String,
    pub score: i64,
    pub solved_count: u32,
    pub attempted_count: u32,
    pub max_streak: u32,
    pub elapsed_secs: f64,
}

impl LearningRunRecord {
    pub fn accuracy_pct(&self) -> f64 {
        if self.attempted_count == 0 {
            0.0
        } else {
            100.0 * (self.solved_count as f64) / (self.attempted_count as f64)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreDelta {
    pub points_awarded: i64,
    pub new_streak: u32,
    pub multiplier: f64,
    pub speed_bonus: i64,
}

/// Computes points awarded for a single problem attempt in a Learning or Timed Run.
pub fn score_attempt(
    mode: RunMode,
    difficulty: u8,
    correct: bool,
    current_streak: u32,
    solve_secs: f64,
    steps_revealed: usize,
) -> ScoreDelta {
    if !correct {
        return ScoreDelta {
            points_awarded: 0,
            new_streak: 0,
            multiplier: 1.0,
            speed_bonus: 0,
        };
    }

    let next_streak = current_streak + 1;
    let multiplier = (1.0 + 0.25 * ((next_streak.saturating_sub(1)).min(8) as f64)).min(3.0);
    let base = (difficulty.max(1) as i64) * 100;

    let speed_bonus = if mode.is_timed() {
        let fast_factor = (20.0 - solve_secs.clamp(1.0, 20.0)) / 20.0;
        (75.0 * fast_factor).round() as i64
    } else {
        0
    };

    // In Learning Mode, stepping through guided derivations still awards constructive study points!
    let step_factor = match mode {
        RunMode::LearningWalkthrough => {
            if steps_revealed == 0 {
                1.0
            } else {
                0.70
            }
        }
        _ => {
            if steps_revealed == 0 {
                1.0
            } else {
                0.45
            }
        }
    };

    let raw = ((base + speed_bonus) as f64) * multiplier * step_factor;
    ScoreDelta {
        points_awarded: raw.round() as i64,
        new_streak: next_streak,
        multiplier,
        speed_bonus,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageScopeFilter {
    AllStages,
    Single(Stage),
}

impl StageScopeFilter {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AllStages => "All 8 Stages (Full Curriculum)",
            Self::Single(st) => st.short_label(),
        }
    }
}

/// Summarizes an active run configuration for display & SQLite persistence.
pub fn format_run_scope(scope: StageScopeFilter, domain: ScienceDomain) -> (String, String) {
    (scope.label().to_string(), domain.label().to_string())
}

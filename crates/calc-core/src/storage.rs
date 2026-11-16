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

use crate::curriculum::CurriculumCatalog;
use crate::mastery::{AttemptRecord, ModuleMastery};
use crate::runs::LearningRunRecord;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExportBundle {
    pub exported_at_utc: DateTime<Utc>,
    pub mastery: Vec<ModuleMastery>,
    pub learning_runs: Vec<LearningRunRecord>,
    pub recent_attempts: Vec<AttemptRecord>,
}

pub struct ProfileStore {
    conn: Connection,
}

impl ProfileStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("SQLite open error: {}", e))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let conn =
            Connection::open_in_memory().map_err(|e| format!("SQLite memory error: {}", e))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS module_mastery (
                    module_id       TEXT PRIMARY KEY,
                    mastery_score   REAL NOT NULL,
                    ease_factor     REAL NOT NULL,
                    interval_hours  REAL NOT NULL,
                    streak          INTEGER NOT NULL,
                    attempts        INTEGER NOT NULL,
                    correct_count   INTEGER NOT NULL,
                    avg_solve_secs  REAL NOT NULL,
                    last_reviewed   TEXT,
                    next_review     TEXT
                );

                CREATE TABLE IF NOT EXISTS attempt_ledger (
                    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp_utc       TEXT NOT NULL,
                    module_id           TEXT NOT NULL,
                    prompt_title        TEXT NOT NULL,
                    user_input          TEXT NOT NULL,
                    expected_canonical  TEXT NOT NULL,
                    correct             INTEGER NOT NULL,
                    hints_used          INTEGER NOT NULL,
                    solve_secs          REAL NOT NULL
                );

                CREATE TABLE IF NOT EXISTS learning_runs (
                    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                    finished_at_utc     TEXT NOT NULL,
                    mode                TEXT NOT NULL,
                    domain              TEXT NOT NULL,
                    stage_scope         TEXT NOT NULL,
                    score               INTEGER NOT NULL,
                    solved_count        INTEGER NOT NULL,
                    attempted_count     INTEGER NOT NULL,
                    max_streak          INTEGER NOT NULL,
                    elapsed_secs        REAL NOT NULL
                );
                ",
            )
            .map_err(|e| format!("Schema initialization failed: {}", e))
    }

    pub fn load_all_mastery(
        &self,
        catalog: &CurriculumCatalog,
    ) -> Result<HashMap<String, ModuleMastery>, String> {
        let mut map = HashMap::new();
        for m in &catalog.modules {
            map.insert(m.id.to_string(), ModuleMastery::new(m.id));
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT module_id, mastery_score, ease_factor, interval_hours, streak,
                        attempts, correct_count, avg_solve_secs, last_reviewed, next_review
                 FROM module_mastery",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let module_id: String = row.get(0)?;
                let last_str: Option<String> = row.get(8)?;
                let next_str: Option<String> = row.get(9)?;
                let last_reviewed = last_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));
                let next_review = next_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(ModuleMastery {
                    module_id,
                    mastery_score: row.get(1)?,
                    ease_factor: row.get(2)?,
                    interval_hours: row.get(3)?,
                    streak: row.get::<_, i64>(4)? as u32,
                    attempts: row.get::<_, i64>(5)? as u32,
                    correct_count: row.get::<_, i64>(6)? as u32,
                    avg_solve_secs: row.get(7)?,
                    last_reviewed,
                    next_review,
                })
            })
            .map_err(|e| e.to_string())?;

        for r in rows {
            let item = r.map_err(|e| e.to_string())?;
            map.insert(item.module_id.clone(), item);
        }

        Ok(map)
    }

    pub fn save_mastery(&self, m: &ModuleMastery) -> Result<(), String> {
        let last_s = m.last_reviewed.map(|d| d.to_rfc3339());
        let next_s = m.next_review.map(|d| d.to_rfc3339());

        self.conn
            .execute(
                "INSERT INTO module_mastery (
                    module_id, mastery_score, ease_factor, interval_hours, streak,
                    attempts, correct_count, avg_solve_secs, last_reviewed, next_review
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(module_id) DO UPDATE SET
                    mastery_score  = excluded.mastery_score,
                    ease_factor    = excluded.ease_factor,
                    interval_hours = excluded.interval_hours,
                    streak         = excluded.streak,
                    attempts       = excluded.attempts,
                    correct_count  = excluded.correct_count,
                    avg_solve_secs = excluded.avg_solve_secs,
                    last_reviewed  = excluded.last_reviewed,
                    next_review    = excluded.next_review",
                params![
                    m.module_id,
                    m.mastery_score,
                    m.ease_factor,
                    m.interval_hours,
                    m.streak as i64,
                    m.attempts as i64,
                    m.correct_count as i64,
                    m.avg_solve_secs,
                    last_s,
                    next_s
                ],
            )
            .map_err(|e| format!("Failed saving mastery: {}", e))?;
        Ok(())
    }

    pub fn append_attempt(&self, rec: &AttemptRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO attempt_ledger (
                    timestamp_utc, module_id, prompt_title, user_input,
                    expected_canonical, correct, hints_used, solve_secs
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    rec.timestamp_utc.to_rfc3339(),
                    rec.module_id,
                    rec.prompt_title,
                    rec.user_input,
                    rec.expected_canonical,
                    if rec.correct { 1i64 } else { 0i64 },
                    rec.hints_used as i64,
                    rec.solve_secs,
                ],
            )
            .map_err(|e| format!("Failed recording attempt: {}", e))?;
        Ok(())
    }

    pub fn load_recent_attempts(&self, limit: usize) -> Result<Vec<AttemptRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, timestamp_utc, module_id, prompt_title, user_input,
                        expected_canonical, correct, hints_used, solve_secs
                 FROM attempt_ledger
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let ts_str: String = row.get(1)?;
                let timestamp_utc = DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(AttemptRecord {
                    id: row.get(0)?,
                    timestamp_utc,
                    module_id: row.get(2)?,
                    prompt_title: row.get(3)?,
                    user_input: row.get(4)?,
                    expected_canonical: row.get(5)?,
                    correct: row.get::<_, i64>(6)? != 0,
                    hints_used: row.get::<_, i64>(7)? as usize,
                    solve_secs: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    pub fn append_learning_run(&self, run: &LearningRunRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO learning_runs (
                    finished_at_utc, mode, domain, stage_scope, score,
                    solved_count, attempted_count, max_streak, elapsed_secs
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    run.finished_at_utc.to_rfc3339(),
                    run.mode,
                    run.domain,
                    run.stage_scope,
                    run.score,
                    run.solved_count as i64,
                    run.attempted_count as i64,
                    run.max_streak as i64,
                    run.elapsed_secs,
                ],
            )
            .map_err(|e| format!("Failed recording learning run: {}", e))?;
        Ok(())
    }

    pub fn load_learning_runs(&self, limit: usize) -> Result<Vec<LearningRunRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, finished_at_utc, mode, domain, stage_scope, score,
                        solved_count, attempted_count, max_streak, elapsed_secs
                 FROM learning_runs
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                let ts_str: String = row.get(1)?;
                let finished_at_utc = DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                Ok(LearningRunRecord {
                    id: row.get(0)?,
                    finished_at_utc,
                    mode: row.get(2)?,
                    domain: row.get(3)?,
                    stage_scope: row.get(4)?,
                    score: row.get(5)?,
                    solved_count: row.get::<_, i64>(6)? as u32,
                    attempted_count: row.get::<_, i64>(7)? as u32,
                    max_streak: row.get::<_, i64>(8)? as u32,
                    elapsed_secs: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    pub fn reset_all(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "DELETE FROM module_mastery; DELETE FROM attempt_ledger; DELETE FROM learning_runs;",
            )
            .map_err(|e| e.to_string())
    }

    pub fn export_json(&self, catalog: &CurriculumCatalog) -> Result<String, String> {
        let map = self.load_all_mastery(catalog)?;
        let mut mastery: Vec<ModuleMastery> = catalog
            .modules
            .iter()
            .filter_map(|m| map.get(m.id).cloned())
            .collect();
        mastery.sort_by(|a, b| a.module_id.cmp(&b.module_id));
        let learning_runs = self.load_learning_runs(100)?;
        let recent_attempts = self.load_recent_attempts(250)?;
        let bundle = ProfileExportBundle {
            exported_at_utc: Utc::now(),
            mastery,
            learning_runs,
            recent_attempts,
        };
        serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
    }
}

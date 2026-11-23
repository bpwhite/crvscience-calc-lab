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

pub mod curriculum;
pub mod domains;
pub mod expr;
pub mod generator;
pub mod linalg;
pub mod mastery;
pub mod notation;
pub mod ode_pde;
pub mod prob_stats;
pub mod runs;
pub mod storage;

pub use curriculum::{
    build_curriculum, CurriculumCatalog, LabPreset, ModuleSpec, Stage,
};
pub use domains::ScienceDomain;
pub use expr::{
    adaptive_simpson, check_numeric_equivalence, check_scalar_equivalence,
    check_vector2_equivalence, evaluate_field_2d, parse_vector2, plain_input_to_latex,
    riemann_sum, taylor_polynomial_at, EvalEnv, Expr, FieldDiagnostics2D, MathFunc,
    RiemannMethod, RiemannSummary,
};
pub use generator::{
    AnswerFormat, GeneratedProblem, GradeOutcome, OperationKind, ProblemGenerator, SolutionStep,
    WalkthroughVisual,
};
pub use linalg::{
    det3, orthogonal_projection_2d, EigenSpectrum2D, Mat2, Mat2Diagnostics,
};
pub use mastery::{
    is_module_unlocked, pick_next_adaptive_module, summarize_stages, AttemptRecord,
    ModuleMastery, SessionItem, SessionMode, StageProgressSummary,
};
pub use notation::{
    build_notation_lexicon, generate_notation_flash_question, NotationEntry,
    NotationFlashQuestion,
};
pub use ode_pde::{
    evaluate_1d_pde, evaluate_1d_pde_custom, extract_zero_level_segments, find_planar_equilibria,
    solve_planar_system_euler_timed, solve_planar_system_rk4, solve_planar_system_rk4_timed,
    solve_scalar_ode_rk4, EquilibriumPoint, PdeInitialProfile, PdeKind, PdeSnapshot,
};
pub use prob_stats::{
    binomial_pmf, compute_bayes_binary, erf_approx, exponential_cdf, exponential_pdf,
    fit_ols_linear, ln_factorial, ln_gamma, normal_cdf, normal_pdf, poisson_pmf,
    BayesDiagnosticSummary, DistributionKind, OlsRegressionSummary,
};
pub use runs::{
    format_run_scope, score_attempt, LearningRunRecord, RunMode, ScoreDelta,
    StageScopeFilter,
};
pub use storage::{ProfileExportBundle, ProfileStore};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curriculum_dag_has_48_valid_modules_across_8_stages() {
        let cat = build_curriculum();
        assert_eq!(cat.modules.len(), 48);
        for st in Stage::ALL {
            assert_eq!(cat.by_stage(st).len(), 6);
        }
        for m in &cat.modules {
            for &pre in m.prerequisites {
                assert!(
                    cat.get(pre).is_some(),
                    "Missing prerequisite {} on {}",
                    pre,
                    m.id
                );
            }
        }
    }

    #[test]
    fn notation_lexicon_and_flash_questions_work() {
        let lex = build_notation_lexicon();
        assert!(lex.len() >= 14);
        for seed in 0..8 {
            let q = generate_notation_flash_question(seed);
            assert!(q.correct_index < 4);
        }
    }

    #[test]
    fn all_48_generators_and_domains_self_grade() {
        let cat = build_curriculum();
        for m in &cat.modules {
            for (idx, &dom) in ScienceDomain::ALL.iter().enumerate() {
                let prob = ProblemGenerator::generate_with_domain(m.id, 17 + idx as u64, dom);
                assert!(!prob.steps.is_empty(), "Missing SolutionSteps for {}", m.id);
                let canonical = prob.answer.canonical();
                assert!(
                    prob.validate_input_syntax(&canonical).is_ok(),
                    "Live syntax check rejected canonical '{}' for module {} domain {:?}",
                    canonical,
                    m.id,
                    dom
                );
                let res = prob.grade(&canonical);
                assert!(
                    res.correct,
                    "Self-grade failed for module {} domain {:?}: canonical='{}', msg='{}'",
                    m.id, dom, canonical, res.feedback
                );
            }
        }
    }
}

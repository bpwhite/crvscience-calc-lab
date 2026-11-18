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
use crate::expr::{
    check_numeric_equivalence, check_scalar_equivalence, check_vector2_equivalence, Expr,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerFormat {
    /// Exact or decimal numeric constant, e.g. "7/2", "3*pi/4", "sqrt(3)/2"
    Numeric {
        expected: f64,
        tolerance: f64,
        canonical_display: String,
    },
    /// Symbolic scalar expression in given variables, e.g. "6*x^2 - 4*x"
    ScalarExpr {
        expected_expr: String,
        vars: Vec<String>,
    },
    /// 2D Vector expression `<P, Q>`, e.g. `<2*x*y, x^2 + 3>`
    Vector2Expr {
        expected_p: String,
        expected_q: String,
        vars: Vec<String>,
        canonical_display: String,
    },
}

impl AnswerFormat {
    pub fn format_hint(&self) -> String {
        match self {
            Self::Numeric { .. } => {
                "Enter a number or constant expression (e.g. 14/3, -2.5, sqrt(3)/2, pi/4)"
                    .to_string()
            }
            Self::ScalarExpr { vars, .. } => {
                format!(
                    "Enter a plain-text expression in {} (e.g. 3*x^2 + cos(x))",
                    vars.join(", ")
                )
            }
            Self::Vector2Expr { vars, .. } => {
                format!(
                    "Enter a 2D vector <P, Q> in {} (e.g. <2*x, -3*y^2>)",
                    vars.join(", ")
                )
            }
        }
    }

    pub fn canonical(&self) -> String {
        match self {
            Self::Numeric {
                canonical_display, ..
            } => canonical_display.clone(),
            Self::ScalarExpr { expected_expr, .. } => expected_expr.clone(),
            Self::Vector2Expr {
                canonical_display, ..
            } => canonical_display.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    SetTheoryNotation,
    IndexedSumProduct,
    DifferentialOperator,
    LinearFormNorm,
    PartialDifferentialOp,
    StatisticalNotation,
    LinearSystemSolve,
    QuadraticVertex,
    PartialFractions,
    DifferenceQuotient,
    ExponentialLogSolve,
    GeometricSeries,
    FiniteSeriesSum,
    ExactTrigonometric,
    HarmonicWavePeriod,
    PythagoreanIdentity,
    PowerReduction,
    InverseTrigAlgebraic,
    VectorDotProduct,
    LimitEvaluation,
    Differentiation,
    ImplicitDifferentiation,
    DefiniteIntegral,
    IndefiniteIntegral,
    TaylorMaclaurinSeries,
    EpsilonDeltaProof,
    GradientVectorField,
    HessianDeterminant,
    MultipleIntegralPolar,
    VectorDivergence,
    LineIntegralCirculation,
    BasisDecomposition,
    LinearMapMatrix,
    MatrixInverse,
    OrthogonalProjection,
    Eigenvalues,
    QuadraticFormExpansion,
    FirstOrderOde,
    SecondOrderOde,
    PlanarSystemDiscriminant,
    LaplaceTransform,
    HeatPdeEigenmode,
    WavePdeStanding,
    BayesianPosterior,
    PoissonDistribution,
    ExponentialCdfSurvival,
    CltStandardization,
    MaximumLikelihood,
    OlsRegression,
}

impl OperationKind {
    pub fn category_badge(&self) -> &'static str {
        match self {
            Self::SetTheoryNotation => "SET & MAPPING NOTATION",
            Self::IndexedSumProduct => "SIGMA / PI INDEXED OPERATOR",
            Self::DifferentialOperator => "LEIBNIZ / LAGRANGE OPERATOR",
            Self::LinearFormNorm => "VECTOR NORM & CONTRACTION",
            Self::PartialDifferentialOp => "LAPLACIAN PDE OPERATOR",
            Self::StatisticalNotation => "ESTIMATOR / EXPECTATION",
            Self::LinearSystemSolve => "LINEAR SYSTEM",
            Self::QuadraticVertex => "VERTEX / COMPLETE SQUARE",
            Self::PartialFractions => "PARTIAL FRACTIONS",
            Self::DifferenceQuotient => "ALGEBRAIC QUOTIENT",
            Self::ExponentialLogSolve => "EXPONENTIAL & LOGARITHM",
            Self::GeometricSeries => "INFINITE GEOMETRIC SERIES",
            Self::FiniteSeriesSum => "FINITE SIGMA SUM",
            Self::ExactTrigonometric => "UNIT CIRCLE TRIGONOMETRY",
            Self::HarmonicWavePeriod => "HARMONIC WAVE PERIOD",
            Self::PythagoreanIdentity => "TRIGONOMETRIC IDENTITY",
            Self::PowerReduction => "POWER-REDUCTION IDENTITY",
            Self::InverseTrigAlgebraic => "INVERSE TRIG TRIANGLE",
            Self::VectorDotProduct => "EUCLIDEAN DOT PRODUCT",
            Self::LimitEvaluation => "LIMIT",
            Self::Differentiation => "DERIVATIVE",
            Self::ImplicitDifferentiation => "IMPLICIT DERIVATIVE",
            Self::DefiniteIntegral => "DEFINITE INTEGRAL",
            Self::IndefiniteIntegral => "INDEFINITE INTEGRAL",
            Self::TaylorMaclaurinSeries => "MACLAURIN SERIES",
            Self::EpsilonDeltaProof => "EPSILON-DELTA PROOF",
            Self::GradientVectorField => "GRADIENT FIELD",
            Self::HessianDeterminant => "HESSIAN DETERMINANT",
            Self::MultipleIntegralPolar => "POLAR DOUBLE INTEGRAL",
            Self::VectorDivergence => "VECTOR DIVERGENCE",
            Self::LineIntegralCirculation => "GREEN'S THEOREM INTEGRAL",
            Self::BasisDecomposition => "BASIS COORDINATES",
            Self::LinearMapMatrix => "MATRIX LINEAR MAP",
            Self::MatrixInverse => "MATRIX INVERSE",
            Self::OrthogonalProjection => "ORTHOGONAL PROJECTION",
            Self::Eigenvalues => "EIGENVALUES",
            Self::QuadraticFormExpansion => "QUADRATIC FORM",
            Self::FirstOrderOde => "FIRST-ORDER LINEAR ODE",
            Self::SecondOrderOde => "SECOND-ORDER ODE IVP",
            Self::PlanarSystemDiscriminant => "PHASE DISCRIMINANT",
            Self::LaplaceTransform => "LAPLACE TRANSFORM",
            Self::HeatPdeEigenmode => "1D HEAT DIFFUSION PDE",
            Self::WavePdeStanding => "1D WAVE PDE",
            Self::BayesianPosterior => "BAYESIAN POSTERIOR",
            Self::PoissonDistribution => "POISSON DISTRIBUTION",
            Self::ExponentialCdfSurvival => "EXPONENTIAL CDF / SURVIVAL",
            Self::CltStandardization => "CLT STANDARDIZATION",
            Self::MaximumLikelihood => "MAXIMUM LIKELIHOOD (MLE)",
            Self::OlsRegression => "OLS LINEAR REGRESSION",
        }
    }

    pub fn operator_latex(&self) -> &'static str {
        match self {
            Self::SetTheoryNotation => r"x \in A,\; (f \circ g)(x)",
            Self::IndexedSumProduct => r"\prod_{k=1}^{n},\; \sum_{k=1}^{n}",
            Self::DifferentialOperator => r"\frac{d}{dx}\Big|_{x=a},\; f^{(2)}(x)",
            Self::LinearFormNorm => r"\left\|u\right\|_{2}^{2} = u^{T}u",
            Self::PartialDifferentialOp => r"\nabla^{2}u = u_{xx} + u_{yy}",
            Self::StatisticalNotation => r"\hat{\theta} = \operatorname{argmax}_{\theta}",
            Self::LinearSystemSolve => r"Ax = b,\; m = \frac{\Delta y}{\Delta x}",
            Self::QuadraticVertex => r"a(x - h)^{2} + k",
            Self::PartialFractions => r"\frac{A}{x - r_{1}} + \frac{B}{x - r_{2}}",
            Self::DifferenceQuotient => r"\frac{f(x+h) - f(x)}{h}",
            Self::ExponentialLogSolve => r"\ln(e^{u}) = u",
            Self::GeometricSeries => r"\sum_{k=0}^{\infty} a r^{k} = \frac{a}{1 - r}",
            Self::FiniteSeriesSum => r"\sum_{k=1}^{n} k^{2}",
            Self::ExactTrigonometric => r"(\cos\theta,\; \sin\theta)",
            Self::HarmonicWavePeriod => r"T = \frac{2\pi}{\omega}",
            Self::PythagoreanIdentity => r"1 + \tan^{2}(x) = \sec^{2}(x)",
            Self::PowerReduction => r"\sin^{2}(u) = \frac{1 - \cos(2u)}{2}",
            Self::InverseTrigAlgebraic => r"\sin(\arctan(u))",
            Self::VectorDotProduct => r"u \cdot v = u_{1}v_{1} + u_{2}v_{2}",
            Self::LimitEvaluation => r"\lim_{x \to c} f(x)",
            Self::Differentiation => r"\frac{d}{dx}\left[f(x)\right]",
            Self::ImplicitDifferentiation => r"\frac{dy}{dx} = -\frac{F_{x}}{F_{y}}",
            Self::DefiniteIntegral => r"\int_{a}^{b} f(x)\,dx",
            Self::IndefiniteIntegral => r"\int u\,dv = uv - \int v\,du",
            Self::TaylorMaclaurinSeries => r"T_{n}(x) = \sum_{k=0}^{n} \frac{f^{(k)}(0)}{k!} x^{k}",
            Self::EpsilonDeltaProof => r"\forall \varepsilon > 0,\; \exists \delta > 0",
            Self::GradientVectorField => r"\nabla f = \left\langle \frac{\partial f}{\partial x},\, \frac{\partial f}{\partial y} \right\rangle",
            Self::HessianDeterminant => r"D = f_{xx}f_{yy} - (f_{xy})^{2}",
            Self::MultipleIntegralPolar => r"\iint_{D} f(r,\theta)\, r\,dr\,d\theta",
            Self::VectorDivergence => r"\nabla \cdot F = \frac{\partial P}{\partial x} + \frac{\partial Q}{\partial y}",
            Self::LineIntegralCirculation => r"\oint_{C} (P\,dx + Q\,dy)",
            Self::BasisDecomposition => r"w = c_{1}u + c_{2}v",
            Self::LinearMapMatrix => r"v \mapsto Av",
            Self::MatrixInverse => r"A^{-1} = \frac{1}{\det(A)} \operatorname{adj}(A)",
            Self::OrthogonalProjection => r"\operatorname{proj}_{u}(y) = \frac{y \cdot u}{u \cdot u} u",
            Self::Eigenvalues => r"\det(A - \lambda I) = 0",
            Self::QuadraticFormExpansion => r"Q(x,y) = v^{T} A v",
            Self::FirstOrderOde => r"\frac{dy}{dt} + p(t)y = q(t)",
            Self::SecondOrderOde => r"y'' + ay' + by = 0",
            Self::PlanarSystemDiscriminant => r"\Delta = \operatorname{tr}(A)^{2} - 4\det(A)",
            Self::LaplaceTransform => r"\mathcal{L}\{f(t)\} = \int_{0}^{\infty} e^{-st}f(t)\,dt",
            Self::HeatPdeEigenmode => r"u_{t} = \alpha u_{xx}",
            Self::WavePdeStanding => r"u_{tt} = c^{2} u_{xx}",
            Self::BayesianPosterior => r"P(H \mid +) = \frac{P(+\mid H)P(H)}{P(+)}",
            Self::PoissonDistribution => r"P(X = k) = \frac{\lambda^{k} e^{-\lambda}}{k!}",
            Self::ExponentialCdfSurvival => r"F(t) = \int_{0}^{t} \lambda e^{-\lambda u}\,du",
            Self::CltStandardization => r"Z = \frac{\bar{x} - \mu}{\sigma / \sqrt{n}}",
            Self::MaximumLikelihood => r"\frac{d\ell}{d\lambda} = 0",
            Self::OlsRegression => r"\hat{y}(x) = b_{1}x + b_{0}",
        }
    }

    pub fn accent_rgb(&self) -> [u8; 3] {
        match self {
            Self::SetTheoryNotation
            | Self::IndexedSumProduct
            | Self::DifferentialOperator
            | Self::LinearFormNorm
            | Self::PartialDifferentialOp
            | Self::StatisticalNotation => [232, 196, 92],
            Self::LinearSystemSolve
            | Self::QuadraticVertex
            | Self::PartialFractions
            | Self::DifferenceQuotient
            | Self::ExponentialLogSolve
            | Self::GeometricSeries
            | Self::FiniteSeriesSum => [92, 164, 230],
            Self::ExactTrigonometric
            | Self::HarmonicWavePeriod
            | Self::PythagoreanIdentity
            | Self::PowerReduction
            | Self::InverseTrigAlgebraic
            | Self::VectorDotProduct => [82, 190, 152],
            Self::LimitEvaluation
            | Self::Differentiation
            | Self::ImplicitDifferentiation
            | Self::DefiniteIntegral
            | Self::IndefiniteIntegral
            | Self::TaylorMaclaurinSeries => [224, 166, 78],
            Self::BasisDecomposition
            | Self::LinearMapMatrix
            | Self::MatrixInverse
            | Self::OrthogonalProjection
            | Self::Eigenvalues
            | Self::QuadraticFormExpansion => [76, 196, 208],
            Self::EpsilonDeltaProof
            | Self::GradientVectorField
            | Self::HessianDeterminant
            | Self::MultipleIntegralPolar
            | Self::VectorDivergence
            | Self::LineIntegralCirculation => [198, 126, 214],
            Self::FirstOrderOde
            | Self::SecondOrderOde
            | Self::PlanarSystemDiscriminant
            | Self::LaplaceTransform
            | Self::HeatPdeEigenmode
            | Self::WavePdeStanding => [230, 122, 108],
            Self::BayesianPosterior
            | Self::PoissonDistribution
            | Self::ExponentialCdfSurvival
            | Self::CltStandardization
            | Self::MaximumLikelihood
            | Self::OlsRegression => [154, 208, 96],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionStep {
    pub step_number: usize,
    pub rule_name: String,
    pub rule_identity_latex: String,
    pub explanation: String,
    pub before_latex: String,
    pub after_latex: String,
    pub method_name: String,
    pub resulting_expr: String,
    pub visual_plot_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalkthroughVisual {
    pub var: String,
    pub x_range: Option<[f64; 2]>,
    pub primary_expr: Option<String>,
    pub primary_label: Option<String>,
    pub secondary_expr: Option<String>,
    pub secondary_label: Option<String>,
    pub shade_interval: Option<[f64; 2]>,
    pub tangent_point: Option<[f64; 2]>,
    pub tangent_slope: Option<f64>,
    pub highlight_points: Vec<[f64; 2]>,
    pub vline_x: Option<f64>,
    pub vline_label: Option<String>,
    pub hline_y: Option<f64>,
    pub hline_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedProblem {
    pub module_id: String,
    pub seed: u64,
    pub difficulty: u8,
    pub domain: ScienceDomain,
    pub domain_context: Option<String>,
    pub operation: OperationKind,
    pub title: String,
    pub prompt: String,
    pub target_equation_latex: String,
    pub answer_latex: String,
    pub answer: AnswerFormat,
    pub hints: Vec<String>,
    pub derivation_steps: Vec<String>,
    pub steps: Vec<SolutionStep>,
    pub visual_var: String,
    pub visual_primary_expr: Option<String>,
    #[serde(default)]
    pub visual: WalkthroughVisual,
    pub remediate_module_id: Option<String>,
}

#[derive(Debug, Clone)]
struct RawStep {
    rule_name: String,
    rule_identity_latex: String,
    explanation: String,
    before_latex: String,
    after_latex: String,
    state_expr: String,
}

struct RawProblem {
    module_id: String,
    seed: u64,
    difficulty: u8,
    operation: OperationKind,
    title: String,
    prompt: String,
    target_equation_latex: String,
    answer_latex: String,
    answer: AnswerFormat,
    hints: Vec<String>,
    derivation_steps: Vec<String>,
    raw_steps: Vec<RawStep>,
    visual: Option<WalkthroughVisual>,
    remediate_module_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GradeOutcome {
    pub correct: bool,
    pub parse_error: Option<String>,
    pub canonical_answer: String,
    pub feedback: String,
}

impl GeneratedProblem {
    pub fn grade(&self, raw_input: &str) -> GradeOutcome {
        let trimmed = raw_input.trim();
        if trimmed.is_empty() {
            return GradeOutcome {
                correct: false,
                parse_error: Some("Please type an answer before checking.".to_string()),
                canonical_answer: self.answer.canonical(),
                feedback: "Input was empty.".to_string(),
            };
        }

        match &self.answer {
            AnswerFormat::Numeric {
                expected,
                tolerance,
                canonical_display,
            } => match check_numeric_equivalence(trimmed, *expected, *tolerance) {
                Ok(true) => GradeOutcome {
                    correct: true,
                    parse_error: None,
                    canonical_answer: canonical_display.clone(),
                    feedback: format!("Verified! Equivalent to {}.", canonical_display),
                },
                Ok(false) => {
                    let user_val = Expr::parse(trimmed)
                        .ok()
                        .map(|e| e.eval_1d("x", 0.0))
                        .unwrap_or(f64::NAN);
                    GradeOutcome {
                        correct: false,
                        parse_error: None,
                        canonical_answer: canonical_display.clone(),
                        feedback: format!(
                            "Your value (~{:.5}) differs from expected {} (~{:.5}). Review the step-by-step derivation below.",
                            user_val, canonical_display, expected
                        ),
                    }
                }
                Err(err) => GradeOutcome {
                    correct: false,
                    parse_error: Some(err.clone()),
                    canonical_answer: canonical_display.clone(),
                    feedback: err,
                },
            },
            AnswerFormat::ScalarExpr {
                expected_expr,
                vars,
            } => {
                let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                match check_scalar_equivalence(trimmed, expected_expr, &var_refs) {
                    Ok(true) => GradeOutcome {
                        correct: true,
                        parse_error: None,
                        canonical_answer: expected_expr.clone(),
                        feedback: format!(
                            "Verified symbolically across domain test points! Reference form: {}",
                            expected_expr
                        ),
                    },
                    Ok(false) => GradeOutcome {
                        correct: false,
                        parse_error: None,
                        canonical_answer: expected_expr.clone(),
                        feedback: format!(
                            "Expression does not match across domain points. Reference form: {}",
                            expected_expr
                        ),
                    },
                    Err(err) => GradeOutcome {
                        correct: false,
                        parse_error: Some(err.clone()),
                        canonical_answer: expected_expr.clone(),
                        feedback: err,
                    },
                }
            }
            AnswerFormat::Vector2Expr {
                expected_p,
                expected_q,
                vars,
                canonical_display,
            } => {
                let var_refs: Vec<&str> = vars.iter().map(|s| s.as_str()).collect();
                match check_vector2_equivalence(trimmed, expected_p, expected_q, &var_refs) {
                    Ok(true) => GradeOutcome {
                        correct: true,
                        parse_error: None,
                        canonical_answer: canonical_display.clone(),
                        feedback: format!(
                            "Both vector components match! Reference form: {}",
                            canonical_display
                        ),
                    },
                    Ok(false) => GradeOutcome {
                        correct: false,
                        parse_error: None,
                        canonical_answer: canonical_display.clone(),
                        feedback: format!(
                            "One or both vector components differ. Expected: {}",
                            canonical_display
                        ),
                    },
                    Err(err) => GradeOutcome {
                        correct: false,
                        parse_error: Some(err.clone()),
                        canonical_answer: canonical_display.clone(),
                        feedback: err,
                    },
                }
            }
        }
    }
}

fn fmt_add_const(prefix: &str, c: i32) -> String {
    if c == 0 {
        prefix.to_string()
    } else if c > 0 {
        format!("{} + {}", prefix, c)
    } else {
        format!("{} - {}", prefix, -c)
    }
}

pub struct ProblemGenerator;


fn module_stage(module_id: &str) -> Stage {
    if module_id.starts_with("not-") {
        Stage::Notation
    } else if module_id.starts_with("alg-") {
        Stage::Algebra
    } else if module_id.starts_with("trig-") {
        Stage::Trigonometry
    } else if module_id.starts_with("calc-") {
        Stage::Calculus
    } else if module_id.starts_with("lin-") {
        Stage::LinearAlgebra
    } else if module_id.starts_with("adv-") {
        Stage::AdvancedCalculus
    } else if module_id.starts_with("deq-") {
        Stage::DifferentialEquations
    } else {
        Stage::ProbabilityStats
    }
}

fn infer_visual_curves(raw: &RawProblem) -> (String, Option<String>, Option<String>, WalkthroughVisual) {
    if let Some(vis) = &raw.visual {
        let var = if vis.var.is_empty() {
            "x".to_string()
        } else {
            vis.var.clone()
        };
        let mut normalized = vis.clone();
        normalized.var = var.clone();
        return (
            var,
            normalized.primary_expr.clone(),
            normalized.secondary_expr.clone(),
            normalized,
        );
    }

    match &raw.answer {
        AnswerFormat::ScalarExpr { expected_expr, vars } => {
            if vars.len() == 1 {
                let v = vars[0].clone();
                let primary = if let Ok(ast) = Expr::parse(expected_expr) {
                    Some(ast.differentiate(&v).to_plain_string())
                } else {
                    None
                };
                let vis = WalkthroughVisual {
                    var: v.clone(),
                    primary_expr: primary.clone(),
                    secondary_expr: Some(expected_expr.clone()),
                    ..Default::default()
                };
                return (v, primary, Some(expected_expr.clone()), vis);
            } else if vars.len() == 2 && vars.contains(&"t".to_string()) && vars.contains(&"x".to_string()) {
                let slice_0 = expected_expr.replace("*t)", "*0.0)");
                let slice_t = expected_expr.replace("*t)", "*0.25)");
                let vis = WalkthroughVisual {
                    var: "x".to_string(),
                    primary_expr: Some(slice_0.clone()),
                    primary_label: Some("t = 0.0 slice".to_string()),
                    secondary_expr: Some(slice_t.clone()),
                    secondary_label: Some("t = 0.25 slice".to_string()),
                    ..Default::default()
                };
                return ("x".to_string(), Some(slice_0), Some(slice_t), vis);
            }
            (
                "x".to_string(),
                None,
                None,
                WalkthroughVisual {
                    var: "x".to_string(),
                    ..Default::default()
                },
            )
        }
        AnswerFormat::Vector2Expr { expected_p, expected_q, .. } => {
            let p1 = expected_p.replace("y", "1.0");
            let q1 = expected_q.replace("y", "1.0");
            let vis = WalkthroughVisual {
                var: "x".to_string(),
                primary_expr: Some(p1.clone()),
                primary_label: Some("Component P(x, 1)".to_string()),
                secondary_expr: Some(q1.clone()),
                secondary_label: Some("Component Q(x, 1)".to_string()),
                ..Default::default()
            };
            ("x".to_string(), Some(p1), Some(q1), vis)
        }
        AnswerFormat::Numeric { expected, .. } => {
            if raw.module_id.starts_with("sta-") {
                let pdf = "exp(-0.5*x^2)/sqrt(2*pi)".to_string();
                let vis = WalkthroughVisual {
                    var: "x".to_string(),
                    primary_expr: Some(pdf.clone()),
                    primary_label: Some("Standard Normal PDF".to_string()),
                    vline_x: Some(*expected),
                    vline_label: Some(format!("Target = {:.3}", expected)),
                    ..Default::default()
                };
                return (
                    "x".to_string(),
                    Some(pdf),
                    Some(format!("{:.4}", expected)),
                    vis,
                );
            }
            let vis = WalkthroughVisual {
                var: "x".to_string(),
                hline_y: Some(*expected),
                hline_label: Some(format!("Target = {:.4}", expected)),
                ..Default::default()
            };
            (
                "x".to_string(),
                None,
                Some(format!("{:.4}", expected)),
                vis,
            )
        }
    }
}

fn raw_step(
    rule: &str,
    identity_latex: &str,
    explanation: impl Into<String>,
    before_latex: impl Into<String>,
    after_latex: impl Into<String>,
    state_expr: impl Into<String>,
) -> RawStep {
    RawStep {
        rule_name: rule.to_string(),
        rule_identity_latex: identity_latex.to_string(),
        explanation: explanation.into(),
        before_latex: before_latex.into(),
        after_latex: after_latex.into(),
        state_expr: state_expr.into(),
    }
}

fn signed_i(val: i32) -> String {
    if val >= 0 {
        format!("+ {}", val)
    } else {
        format!("- {}", -val)
    }
}

fn plain_to_latex_or(raw: &str) -> String {
    Expr::parse(raw)
        .map(|e| e.to_latex())
        .unwrap_or_else(|_| raw.to_string())
}

fn finalize_problem(raw: RawProblem, domain: ScienceDomain) -> GeneratedProblem {
    let stage = module_stage(&raw.module_id);
    let domain_context = domain.context_banner(stage, &raw.module_id);
    let (visual_var, visual_primary_expr, final_curve, visual) = infer_visual_curves(&raw);

    let total = raw.raw_steps.len();
    let mut steps = Vec::with_capacity(total);
    for (idx, rs) in raw.raw_steps.into_iter().enumerate() {
        let visual_plot_expr = if idx + 1 == total {
            final_curve.clone()
        } else {
            visual_primary_expr.clone()
        };
        let resulting_expr = if rs.state_expr.is_empty() && idx + 1 == total {
            raw.answer.canonical()
        } else {
            rs.state_expr
        };
        steps.push(SolutionStep {
            step_number: idx + 1,
            rule_name: rs.rule_name.clone(),
            rule_identity_latex: rs.rule_identity_latex,
            explanation: rs.explanation,
            before_latex: rs.before_latex,
            after_latex: rs.after_latex,
            method_name: rs.rule_name,
            resulting_expr,
            visual_plot_expr,
        });
    }

    GeneratedProblem {
        module_id: raw.module_id,
        seed: raw.seed,
        difficulty: raw.difficulty,
        domain,
        domain_context,
        operation: raw.operation,
        title: raw.title,
        prompt: raw.prompt,
        target_equation_latex: raw.target_equation_latex,
        answer_latex: raw.answer_latex,
        answer: raw.answer,
        hints: raw.hints,
        derivation_steps: raw.derivation_steps,
        steps,
        visual_var,
        visual_primary_expr,
        visual,
        remediate_module_id: raw.remediate_module_id,
    }
}

impl ProblemGenerator {
    pub fn generate(module_id: &str, seed: u64) -> GeneratedProblem {
        Self::generate_with_domain(module_id, seed, ScienceDomain::General)
    }

    pub fn generate_with_domain(
        module_id: &str,
        seed: u64,
        domain: ScienceDomain,
    ) -> GeneratedProblem {
        let raw = Self::generate_raw(module_id, seed, domain);
        finalize_problem(raw, domain)
    }

    fn generate_raw(module_id: &str, seed: u64, domain: ScienceDomain) -> RawProblem {
        let mut rng = StdRng::seed_from_u64(seed);
        let variant = rng.gen_range(0..2usize);

        match module_id {
            // =====================================================================
            // STAGE 0: NOTATION & SYMBOLIC LITERACY (6 Modules)
            // =====================================================================
            "not-sets-quantifiers" => {
                if variant == 0 {
                    let center = rng.gen_range(-4..=6i32);
                    let radius = rng.gen_range(2..=7i32);
                    let lower = center - radius;
                    let upper = center + radius;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        operation: OperationKind::SetTheoryNotation,
                        title: "Unpack Set-Builder Metric Neighborhood Notation".to_string(),
                        prompt: format!(
                            "The set S = {{ x ∈ ℝ : |x - ({})| <= {} }} equals a closed interval [L, U]. Find the exact upper endpoint U.",
                            center, radius
                        ),
                        target_equation_latex: format!(
                            r"S = \left\{{ x \in \mathbb{{R}} : \left|x - ({})\right| \le {} \right\}} = [L,\, U]",
                            center, radius
                        ),
                        answer_latex: format!(r"U = {}", upper),
                        answer: AnswerFormat::Numeric {
                            expected: upper as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", upper),
                        },
                        hints: vec![
                            format!("|x - ({})| <= {} means the distance between x and {} is at most {}.", center, radius, center, radius),
                            format!("Unpack the absolute inequality: -{} <= x - ({}) <= {}.", radius, center, radius),
                            format!("Add {} across: [{} - {}, {} + {}], so U = {}.", center, center, radius, center, radius, upper),
                        ],
                        derivation_steps: vec![
                            format!("1. Decode set-builder condition: -{} <= x - ({}) <= {}.", radius, center, radius),
                            format!("2. Shift by center {}: L = {}, U = {} + {}.", center, lower, center, radius),
                            format!("3. Closed interval is [{}, {}], giving upper endpoint U = {}.", lower, upper, upper),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Absolute Metric Inequality Expansion",
                                r"\left|u\right| \le r \iff -r \le u \le r",
                                format!("Unpack distance bound around {} with radius {}.", center, radius),
                                format!(r"\left|x - ({})\right| \le {}", center, radius),
                                format!(r"-{} \le x - ({}) \le {}", radius, center, radius),
                                format!("-{} <= x - ({}) <= {}", radius, center, radius),
                            ),
                            raw_step(
                                "Translation by Interval Center",
                                r"a \le x - c \le b \implies c + a \le x \le c + b",
                                format!("Add {} across all three parts of the inequality.", center),
                                format!(r"-{} \le x - ({}) \le {}", radius, center, radius),
                                format!(r"{} \le x \le {}", lower, upper),
                                format!("[{}, {}]", lower, upper),
                            ),
                            raw_step(
                                "Extract Upper Endpoint",
                                r"x \in [L, U] \implies \max(S) = U",
                                format!("Read the right-hand closed interval endpoint U = {}.", upper),
                                format!(r"S = [{},\, {}]", lower, upper),
                                format!(r"U = {}", upper),
                                format!("{}", upper),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: None,
                    }
                } else {
                    let a = rng.gen_range(2..=5i32);
                    let b = rng.gen_range(1..=6i32);
                    let expected = format!("{}*x^2 + {}", a, b);
                    let ans_tex = plain_to_latex_or(&expected);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        operation: OperationKind::SetTheoryNotation,
                        title: "Decode Element-Mapping (↦) & Composition (∘) Notation".to_string(),
                        prompt: format!(
                            "Given f : ℝ → ℝ, u ↦ {}*u + {} and g : ℝ → ℝ, x ↦ x^2, write the plain-text expression in x for the composition (f ∘ g)(x).",
                            a, b
                        ),
                        target_equation_latex: format!(
                            r"(f \circ g)(x),\quad f(u) = {}u + {},\quad g(x) = x^{{2}}",
                            a, b
                        ),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            "Composition (f ∘ g)(x) means f(g(x)): apply right-hand map g to x FIRST.".to_string(),
                            format!("Here g sends x ↦ x^2, so feed u = x^2 into f: u ↦ {}*u + {}.", a, b),
                            format!("Resulting expression: {}.", expected),
                        ],
                        derivation_steps: vec![
                            "1. Unpack composition symbol: (f ∘ g)(x) := f(g(x)).".to_string(),
                            format!("2. Evaluate inner map g: x ↦ x^2, then apply outer map f: u ↦ {}*u + {}.", a, b),
                            format!("3. Plain-text formula: {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Composition Operator Definition",
                                r"(f \circ g)(x) := f\left(g(x)\right)",
                                "Apply the right-hand inner map g to x first, then feed into outer map f.",
                                r"(f \circ g)(x)".to_string(),
                                r"f\left(g(x)\right)".to_string(),
                                "f(g(x))",
                            ),
                            raw_step(
                                "Substitute Inner Mapping Rule",
                                r"u \leftarrow g(x) = x^{2}",
                                format!("Replace u with x^2 inside f(u) = {}*u + {}.", a, b),
                                format!(r"f\left(x^{{2}}\right),\; f(u) = {}u + {}", a, b),
                                ans_tex.clone(),
                                expected.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: None,
                    }
                }
            }

            "not-indices-sigma-pi" => {
                if variant == 0 {
                    let c = rng.gen_range(1..=3i32);
                    let f1 = 1 + c;
                    let f2 = 2 + c;
                    let f3 = 3 + c;
                    let val = f1 * f2 * f3;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        operation: OperationKind::IndexedSumProduct,
                        title: "Unroll Capital-Pi (∏) Product Notation".to_string(),
                        prompt: format!(
                            "Evaluate the Capital-Pi product expression:  ∏_{{k=1}}^{{3}} (k + {}).",
                            c
                        ),
                        target_equation_latex: format!(r"\prod_{{k=1}}^{{3}} (k + {})", c),
                        answer_latex: format!("{}", val),
                        answer: AnswerFormat::Numeric {
                            expected: val as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", val),
                        },
                        hints: vec![
                            "Capital Pi ∏ multiplies each indexed factor from k = 1 up to k = 3.".to_string(),
                            format!("Unroll factors: (1 + {}) * (2 + {}) * (3 + {}).", c, c, c),
                            format!("Multiply {} * {} * {} = {}.", f1, f2, f3, val),
                        ],
                        derivation_steps: vec![
                            format!("1. Substitute dummy index k = 1, 2, 3: factors are {}, {}, and {}.", f1, f2, f3),
                            format!("2. Multiply sequentially: {} * {} = {}, times {} = {}.", f1, f2, f1 * f2, f3, val),
                            format!("3. Exact product value: {}.", val),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Unroll Capital-Pi Index Bounds",
                                r"\prod_{k=1}^{3} a_{k} = a_{1} \cdot a_{2} \cdot a_{3}",
                                "Substitute k = 1, 2, 3 to expand the product into three factors.",
                                format!(r"\prod_{{k=1}}^{{3}} (k + {})", c),
                                format!(r"(1 + {})(2 + {})(3 + {})", c, c, c),
                                format!("{} * {} * {}", f1, f2, f3),
                            ),
                            raw_step(
                                "Evaluate Numerical Factors",
                                r"a \cdot b \cdot c",
                                format!("Evaluate {} * {} * {} = {}.", f1, f2, f3, val),
                                format!(r"{} \cdot {} \cdot {}", f1, f2, f3),
                                format!(r"{}", val),
                                format!("{}", val),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("not-sets-quantifiers".to_string()),
                    }
                } else {
                    let a = rng.gen_range(2..=4i32);
                    let n = rng.gen_range(3..=5i32);
                    let tri = n * (n + 1) / 2;
                    let coeff = a * tri;
                    let expected = format!("{}*x", coeff);
                    let ans_tex = format!("{}x", coeff);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::IndexedSumProduct,
                        title: "Convert Log-of-Product ln(∏) into Sigma Sum".to_string(),
                        prompt: format!(
                            "Simplify the indexed notation  ln( ∏_{{k=1}}^{{{}}} exp({}*k*x) )  into a single plain-text expression in x (with no dummy index k remaining).",
                            n, a
                        ),
                        target_equation_latex: format!(
                            r"\ln\left(\prod_{{k=1}}^{{{}}} e^{{{}kx}}\right)",
                            n, a
                        ),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            "Use the log-product rule: ln(∏ a_k) = ∑ ln(a_k).".to_string(),
                            format!("Since ln(exp({}*k*x)) = {}*k*x, this becomes ∑_{{k=1..{}}} {}*k*x.", a, a, n, a),
                            format!("Sum k from 1 to {} (= {}), giving {}.", n, tri, expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Convert ln(∏) to ∑ ln: sum_{{k=1..{}}} ln(exp({}*k*x)) = sum_{{k=1..{}}} {}*k*x.", n, a, n, a),
                            format!("2. Factor free variable x and constant {}: {}*x * (1 + ... + {}).", a, a, n),
                            format!("3. Evaluate index sum {}*{} = {} => {}.", a, tri, coeff, expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Log-Product to Sigma Bridge",
                                r"\ln\left(\prod_{k=1}^{n} a_{k}\right) = \sum_{k=1}^{n} \ln(a_{k})",
                                "Turn the logarithm of a product into a sum of logarithms and cancel ln(exp(.)).",
                                format!(r"\ln\left(\prod_{{k=1}}^{{{}}} e^{{{}kx}}\right)", n, a),
                                format!(r"\sum_{{k=1}}^{{{}}} {}kx", n, a),
                                format!("sum_{{k=1..{}}} {}*k*x", n, a),
                            ),
                            raw_step(
                                "Factor Free Variables Outside Sigma",
                                r"\sum_{k=1}^{n} c f(k) = c \sum_{k=1}^{n} f(k)",
                                format!("Factor out {}*x because neither {} nor x depends on index k.", a, a),
                                format!(r"\sum_{{k=1}}^{{{}}} {}kx", n, a),
                                format!(r"{}x \sum_{{k=1}}^{{{}}} k", a, n),
                                format!("{}*x * sum_{{k=1..{}}} k", a, n),
                            ),
                            raw_step(
                                "Gauss Triangular Sum Formula",
                                r"\sum_{k=1}^{n} k = \frac{n(n+1)}{2}",
                                format!("Evaluate sum k = {}/2 = {}, so {} * {}x = {}x.", n * (n + 1), tri, a, tri, coeff),
                                format!(r"{}x \cdot \frac{{{}\cdot {}}}{{2}}", a, n, n + 1),
                                ans_tex,
                                expected,
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("not-sets-quantifiers".to_string()),
                    }
                }
            }

            "not-calculus-operators" => {
                if variant == 0 {
                    let a = rng.gen_range(1..=4i32);
                    let b = rng.gen_range(1..=5i32);
                    let x0 = rng.gen_range(1..=3i32);
                    let val = 3 * a * x0 * x0 + b;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::DifferentialOperator,
                        title: "Decode Leibniz Derivative Evaluation Bar |_{x=a}".to_string(),
                        prompt: format!(
                            "Evaluate the Leibniz bar notation:   d/dx [ {}*x^3 + {}*x ] |_{{x = {}}}.",
                            a, b, x0
                        ),
                        target_equation_latex: format!(
                            r"\frac{{d}}{{dx}}\left[{}x^{{3}} + {}x\right]\Big|_{{x={}}}",
                            a, b, x0
                        ),
                        answer_latex: format!("{}", val),
                        answer: AnswerFormat::Numeric {
                            expected: val as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", val),
                        },
                        hints: vec![
                            format!("The vertical bar |_{{x={}}} means differentiate wrt x FIRST, then plug in x = {}.", x0, x0),
                            format!("Symbolic derivative is {}*x^2 + {}.", 3 * a, b),
                            format!("Substitute x = {}: {}*({})^2 + {} = {}.", x0, 3 * a, x0, b, val),
                        ],
                        derivation_steps: vec![
                            format!("1. Differentiate inside operator wrt x: {}*x^2 + {}.", 3 * a, b),
                            format!("2. Apply evaluation bar at x = {}: {}*{} + {}.", x0, 3 * a, x0 * x0, b),
                            format!("3. Exact value: {}.", val),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Differentiate Inside Operator First",
                                r"\frac{d}{dx}\left[x^{n}\right] = n x^{n-1}",
                                "Differentiate before evaluating the vertical bar x = a.",
                                format!(r"\frac{{d}}{{dx}}\left[{}x^{{3}} + {}x\right]\Big|_{{x={}}}", a, b, x0),
                                format!(r"\left({}x^{{2}} + {}\right)\Big|_{{x={}}}", 3 * a, b, x0),
                                format!("{}*x^2 + {}", 3 * a, b),
                            ),
                            raw_step(
                                "Evaluate Leibniz Bar at x0",
                                r"\left[g(x)\right]\big|_{x=a} := g(a)",
                                format!("Substitute x = {} into {}*x^2 + {}.", x0, 3 * a, b),
                                format!(r"{}({})^2 + {}", 3 * a, x0, b),
                                format!(r"{}", val),
                                format!("{}", val),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-1.0, x0 as f64 + 1.5]),
                            primary_expr: Some(format!("{}*x^3 + {}*x", a, b)),
                            primary_label: Some(format!("f(x) = {}x^3 + {}x", a, b)),
                            secondary_expr: Some(format!("{}*x^2 + {}", 3 * a, b)),
                            secondary_label: Some(format!("f'(x) = {}x^2 + {}", 3 * a, b)),
                            tangent_point: Some([x0 as f64, (a * x0.pow(3) + b * x0) as f64]),
                            tangent_slope: Some(val as f64),
                            vline_x: Some(x0 as f64),
                            vline_label: Some(format!("x = {}", x0)),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                    }
                } else {
                    let a = rng.gen_range(1..=3i32);
                    let b = rng.gen_range(1..=5i32);
                    let expected = format!("{}*x + {}", 6 * a, 2 * b);
                    let ans_tex = format!("{}x + {}", 6 * a, 2 * b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::DifferentialOperator,
                        title: "Distinguish Lagrange Superscript f^(2)(x) from Squaring".to_string(),
                        prompt: format!(
                            "For f(x) = {}*x^3 + {}*x^2, decode the parenthesized superscript notation f^(2)(x) and write its plain-text expression in x.",
                            a, b
                        ),
                        target_equation_latex: format!(
                            r"f^{{(2)}}(x),\quad f(x) = {}x^{{3}} + {}x^{{2}}",
                            a, b
                        ),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            "Parenthesized superscript f^(n)(x) denotes the n-th derivative d^n f / dx^n (NOT (f(x))^n!).".to_string(),
                            format!("First derivative f'(x) = {}*x^2 + {}*x.", 3 * a, 2 * b),
                            format!("Differentiate a second time: f^(2)(x) = {}.", expected),
                        ],
                        derivation_steps: vec![
                            "1. Decode notation: f^(2)(x) := d^2f / dx^2 = f''(x).".to_string(),
                            format!("2. First derivative: f^(1)(x) = {}*x^2 + {}*x.", 3 * a, 2 * b),
                            format!("3. Second derivative: f^(2)(x) = {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "First-Order Power Rule Derivative",
                                r"f^{(1)}(x) = \frac{df}{dx}",
                                "Parenthesized superscript indicates derivative order, not power.",
                                format!(r"f(x) = {}x^{{3}} + {}x^{{2}}", a, b),
                                format!(r"f'(x) = {}x^{{2}} + {}x", 3 * a, 2 * b),
                                format!("{}*x^2 + {}*x", 3 * a, 2 * b),
                            ),
                            raw_step(
                                "Second-Order Differentiation",
                                r"f^{(2)}(x) = \frac{d}{dx}\left[f'(x)\right]",
                                "Differentiate f'(x) with respect to x.",
                                format!(r"\frac{{d}}{{dx}}\left[{}x^{{2}} + {}x\right]", 3 * a, 2 * b),
                                ans_tex,
                                expected.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-2.5, 2.5]),
                            primary_expr: Some(format!("{}*x^3 + {}*x^2", a, b)),
                            primary_label: Some("f(x)".to_string()),
                            secondary_expr: Some(expected),
                            secondary_label: Some("f^(2)(x) = f''(x)".to_string()),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                    }
                }
            }

            "not-matrix-inner-norms" => {
                let u1 = rng.gen_range(-3..=4i32);
                let u2 = rng.gen_range(1..=4i32);
                let u3 = rng.gen_range(-3..=3i32);
                let norm_sq = u1 * u1 + u2 * u2 + u3 * u3;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::LinearFormNorm,
                    title: "Evaluate Squared Double-Bar Norm ‖u‖₂² = uᵀ u".to_string(),
                    prompt: format!(
                        "For column vector u = [{}, {}, {}]ᵀ ∈ ℝ³, evaluate the squared Euclidean norm notation ‖u‖₂² = ⟨u, u⟩ = uᵀ u.",
                        u1, u2, u3
                    ),
                    target_equation_latex: format!(
                        r"\left\|u\right\|_{{2}}^{{2}} = u^{{T}}u,\quad u = \left\langle {},\, {},\, {} \right\rangle",
                        u1, u2, u3
                    ),
                    answer_latex: format!("{}", norm_sq),
                    answer: AnswerFormat::Numeric {
                        expected: norm_sq as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", norm_sq),
                    },
                    hints: vec![
                        "‖u‖₂² = ⟨u, u⟩ sums the squares of each vector component (without taking a final square root).".to_string(),
                        format!("Compute ({})^2 + ({})^2 + ({})^2.", u1, u2, u3),
                        format!("Sum = {} + {} + {} = {}.", u1 * u1, u2 * u2, u3 * u3, norm_sq),
                    ],
                    derivation_steps: vec![
                        "1. Unpack notation: ‖u‖₂² = uᵀ u = sum_{i=1..3} u_i^2.".to_string(),
                        format!("2. Square entries: {} + {} + {}.", u1 * u1, u2 * u2, u3 * u3),
                        format!("3. Exact scalar result: {}.", norm_sq),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Expand Squared Euclidean Norm",
                            r"\left\|u\right\|_{2}^{2} = \left\langle u, u \right\rangle = \sum_{i=1}^{3} u_{i}^{2}",
                            "Squared double-bar norm omits the final square root.",
                            r"\left\|u\right\|_{2}^{2}".to_string(),
                            format!(r"({})^{{2}} + ({})^{{2}} + ({})^{{2}}", u1, u2, u3),
                            format!("{}^2 + {}^2 + {}^2", u1, u2, u3),
                        ),
                        raw_step(
                            "Sum Squared Coordinates",
                            r"u_{1}^{2} + u_{2}^{2} + u_{3}^{2}",
                            format!("Sum {} + {} + {} = {}.", u1 * u1, u2 * u2, u3 * u3, norm_sq),
                            format!(r"{} + {} + {}", u1 * u1, u2 * u2, u3 * u3),
                            format!(r"{}", norm_sq),
                            format!("{}", norm_sq),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                }
            }

            "not-nabla-pde-fields" => {
                let a = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=4i32);
                let expected = format!("{}*x + {}*y", 6 * a, 6 * b);
                let ans_tex = format!("{}x + {}y", 6 * a, 6 * b);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::PartialDifferentialOp,
                    title: "Unpack Laplacian Symbol ∇²u = u_{xx} + u_{yy}".to_string(),
                    prompt: format!(
                        "For the 2D scalar field u(x, y) = {}*x^3 + {}*y^3, unpack and evaluate the Laplacian notation ∇²u (equivalently written u_{{xx}} + u_{{yy}}) as a plain-text expression in x and y.",
                        a, b
                    ),
                    target_equation_latex: format!(
                        r"\nabla^{{2}}u,\quad u(x,y) = {}x^{{3}} + {}y^{{3}}",
                        a, b
                    ),
                    answer_latex: ans_tex.clone(),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string(), "y".to_string()],
                    },
                    hints: vec![
                        "∇²u = ∇ · (∇u) = ∂²u/∂x² + ∂²u/∂y² is a SCALAR expression (not a vector!).".to_string(),
                        format!("Compute u_{{xx}} = {}*x and u_{{yy}} = {}*y.", 6 * a, 6 * b),
                        format!("Sum second partials: {}.", expected),
                    ],
                    derivation_steps: vec![
                        "1. Decode operator: ∇²u := ∂²u/∂x² + ∂²u/∂y².".to_string(),
                        format!("2. Pure second partials: u_{{xx}} = {}*x,  u_{{yy}} = {}*y.", 6 * a, 6 * b),
                        format!("3. Scalar Laplacian: {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Unpack Pure Second Partials",
                            r"u_{xx} = \frac{\partial^{2}u}{\partial x^{2}},\; u_{yy} = \frac{\partial^{2}u}{\partial y^{2}}",
                            "Differentiate twice wrt x (holding y constant) and twice wrt y.",
                            format!(r"u(x,y) = {}x^{{3}} + {}y^{{3}}", a, b),
                            format!(r"u_{{xx}} = {}x,\quad u_{{yy}} = {}y", 6 * a, 6 * b),
                            format!("u_xx = {}*x, u_yy = {}*y", 6 * a, 6 * b),
                        ),
                        raw_step(
                            "Sum Scalar Laplacian Trace",
                            r"\nabla^{2}u = u_{xx} + u_{yy}",
                            "Sum the two pure second derivatives into a scalar field.",
                            r"u_{xx} + u_{yy}".to_string(),
                            ans_tex,
                            expected,
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("not-calculus-operators".to_string()),
                }
            }

            "not-prob-stats-inference" => {
                if variant == 0 {
                    let theta_star = rng.gen_range(-3..=7i32);
                    let theta_star = if theta_star == 0 { 4 } else { theta_star };
                    let peak_height = rng.gen_range(12..=40i32);
                    let scale = rng.gen_range(2..=5i32);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::StatisticalNotation,
                        title: "Decode argmax_{θ} vs max_{θ} Estimator Notation".to_string(),
                        prompt: format!(
                            "Evaluate the parameter-estimator notation:   θ̂ = argmax_{{θ ∈ ℝ}} ( {} - {}*(θ - ({}))^2 ).",
                            peak_height, scale, theta_star
                        ),
                        target_equation_latex: format!(
                            r"\hat{{\theta}} = \operatorname{{argmax}}_{{\theta \in \mathbb{{R}}}} \left({} - {}(\theta - ({}))^{{2}}\right)",
                            peak_height, scale, theta_star
                        ),
                        answer_latex: format!(r"\hat{{\theta}} = {}", theta_star),
                        answer: AnswerFormat::Numeric {
                            expected: theta_star as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", theta_star),
                        },
                        hints: vec![
                            format!("Careful: max returns the peak height ({}), whereas argmax returns the INPUT value θ where that peak occurs!", peak_height),
                            format!("Since -{}*(θ - ({}))^2 <= 0, the objective is maximized when (θ - ({}))^2 = 0.", scale, theta_star, theta_star),
                            format!("Therefore θ̂ = {}.", theta_star),
                        ],
                        derivation_steps: vec![
                            "1. Decode argmax_θ g(θ): find the coordinate θ maximizing g(θ), NOT the maximum output value.".to_string(),
                            format!("2. Non-positive squared penalty -{}*(θ - ({}))^2 vanishes at θ = {}.", scale, theta_star, theta_star),
                            format!("3. Optimal argument: θ̂ = {} (while max height is {}).", theta_star, peak_height),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Distinguish argmax from max",
                                r"g(\theta) = M - c(\theta - \theta^{*})^{2} \le M",
                                format!("Peak height is {}, attained when the squared penalty vanishes.", peak_height),
                                format!(r"{} - {}(\theta - ({}))^{{2}}", peak_height, scale, theta_star),
                                format!(r"(\theta - ({}))^{{2}} = 0", theta_star),
                                format!("(theta - ({}))^2 = 0", theta_star),
                            ),
                            raw_step(
                                "Extract Maximizing Argument",
                                r"\hat{\theta} = \operatorname{argmax}_{\theta} g(\theta)",
                                format!("Solve for theta = {}.", theta_star),
                                format!(r"\theta - ({}) = 0", theta_star),
                                format!(r"\hat{{\theta}} = {}", theta_star),
                                format!("{}", theta_star),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("not-sets-quantifiers".to_string()),
                    }
                } else {
                    let mu = rng.gen_range(3..=12i32);
                    let sigma_sq = rng.gen_range(4..=16i32);
                    let a = rng.gen_range(2..=5i32);
                    let b = rng.gen_range(-6..=8i32);
                    let val = a * mu + b;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::StatisticalNotation,
                        title: "Decode Distribution (~) & Expectation E[·] Notation".to_string(),
                        prompt: format!(
                            "Given the distributional declaration X ~ N({}, {}), evaluate the expectation operator E[ {}*X + ({}) ].",
                            mu, sigma_sq, a, b
                        ),
                        target_equation_latex: format!(
                            r"E\left[{}X {} \right],\quad X \sim N({},\, {})",
                            a, signed_i(b), mu, sigma_sq
                        ),
                        answer_latex: format!("{}", val),
                        answer: AnswerFormat::Numeric {
                            expected: val as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", val),
                        },
                        hints: vec![
                            format!("In X ~ N(μ, σ²), the first parameter is mean E[X] = {} and the second is variance σ² = {}.", mu, sigma_sq),
                            format!("By linearity of expectation, E[{}*X + ({})] = {}*E[X] + ({}).", a, b, a, b),
                            format!("Compute {}*{} + ({}) = {}.", a, mu, b, val),
                        ],
                        derivation_steps: vec![
                            format!("1. Read X ~ N({}, {}): mean E[X] = {} (variance {} is not needed for mean).", mu, sigma_sq, mu, sigma_sq),
                            format!("2. Apply linearity: E[{}*X + ({})] = {}*{} + ({}).", a, b, a, mu, b),
                            format!("3. Exact expectation: {}.", val),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Linearity of Expectation",
                                r"E[aX + b] = a E[X] + b",
                                format!("Read E[X] = {} from N({}, {}).", mu, mu, sigma_sq),
                                format!(r"E\left[{}X {} \right]", a, signed_i(b)),
                                format!(r"{} E[X] {}", a, signed_i(b)),
                                format!("{}*E[X] {}", a, signed_i(b)),
                            ),
                            raw_step(
                                "Substitute Distribution Mean",
                                r"X \sim N(\mu, \sigma^{2}) \implies E[X] = \mu",
                                format!("Evaluate {}*({}) {} = {}.", a, mu, signed_i(b), val),
                                format!(r"{}({}) {}", a, mu, signed_i(b)),
                                format!(r"{}", val),
                                format!("{}", val),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                    }
                }
            }

            // =====================================================================
            // STAGE 1: ALGEBRA
            // =====================================================================
            "alg-linear-systems" => {
                if variant == 0 {
                    let x1 = rng.gen_range(-3..=2i32);
                    let x2 = x1 + rng.gen_range(2..=4i32);
                    let m = rng.gen_range(-4..=5i32);
                    let m = if m == 0 { 3 } else { m };
                    let b = rng.gen_range(-6..=6i32);
                    let y1 = m * x1 + b;
                    let y2 = m * x2 + b;
                    let expr = fmt_add_const(&format!("{}*x", m), b);
                    let expr_tex = plain_to_latex_or(&expr);
                    let ps_tex = format!(r"{} + {} \left(x - ({})\right)", y1, m, x1);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        operation: OperationKind::LinearSystemSolve,
                        title: "Linear Function from Two Points".to_string(),
                        prompt: format!(
                            "Find the linear function f(x) passing through P1 = ({}, {}) and P2 = ({}, {}).",
                            x1, y1, x2, y2
                        ),
                        target_equation_latex: format!(
                            r"P_1 = ({},\, {}),\quad P_2 = ({},\, {}),\quad f(x) = mx + b",
                            x1, y1, x2, y2
                        ),
                        answer_latex: format!(r"f(x) = {}", expr_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expr.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Compute slope m = (y2 - y1) / (x2 - x1) = ({} - ({})) / ({} - ({})).", y2, y1, x2, x1),
                            format!("Slope m = {}. Now apply point-slope form: y = {} + {} * (x - ({})).", m, y1, m, x1),
                            format!("Expand and collect constant terms to get {}.", expr),
                        ],
                        derivation_steps: vec![
                            format!("1. Rate of change: m = ({} - ({})) / ({} - ({})) = {}.", y2, y1, x2, x1, m),
                            format!("2. Point-slope at P1: f(x) = {} + {}*(x - ({})).", y1, m, x1),
                            format!("3. Simplify: f(x) = {}.", expr),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Difference-of-Coordinates Slope",
                                r"m = \frac{y_2 - y_1}{x_2 - x_1}",
                                format!("Compute the rise-over-run slope between P1({}, {}) and P2({}, {}).", x1, y1, x2, y2),
                                format!(r"m = \frac{{{} - ({})}}{{{} - ({})}}", y2, y1, x2, x1),
                                format!(r"m = {}", m),
                                format!("{}", m),
                            ),
                            raw_step(
                                "Point-Slope Linearization",
                                r"f(x) = y_1 + m(x - x_1)",
                                format!("Anchor the line at P1 = ({}, {}) with slope m = {}.", x1, y1, m),
                                format!(r"m = {},\; P_1 = ({},\, {})", m, x1, y1),
                                format!(r"f(x) = {}", ps_tex),
                                format!("{} + {}*(x - ({}))", y1, m, x1),
                            ),
                            raw_step(
                                "Simplify to Slope-Intercept Form",
                                r"f(x) = mx + b",
                                format!("Distribute {} and combine constants.", m),
                                format!(r"f(x) = {}", ps_tex),
                                format!(r"f(x) = {}", expr_tex),
                                expr.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([(x1 - 2) as f64, (x2 + 2) as f64]),
                            primary_expr: Some(expr),
                            primary_label: Some("f(x) = mx + b".to_string()),
                            highlight_points: vec![[x1 as f64, y1 as f64], [x2 as f64, y2 as f64]],
                            ..Default::default()
                        }),
                        remediate_module_id: None,
                    }
                } else {
                    let sol_x = rng.gen_range(-4..=5i32);
                    let sol_y = rng.gen_range(-4..=5i32);
                    let a1 = rng.gen_range(1..=3i32);
                    let b1 = rng.gen_range(1..=3i32);
                    let a2 = rng.gen_range(2..=4i32);
                    let b2 = -rng.gen_range(1..=3i32);
                    let c1 = a1 * sol_x + b1 * sol_y;
                    let c2 = a2 * sol_x + b2 * sol_y;
                    let det_x = a1 * (-b2) + a2 * b1;
                    let rhs_x = c1 * (-b2) + c2 * b1;
                    let sys_tex = format!(
                        r"\begin{{cases}} {}x + {}y = {} \\ {}x - {}y = {} \end{{cases}}",
                        a1, b1, c1, a2, -b2, c2
                    );
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::LinearSystemSolve,
                        title: "2x2 Linear System Intersection".to_string(),
                        prompt: format!(
                            "Solve the linear system for the x-coordinate of intersection:\n  {}*x + {}*y = {}\n  {}*x - {}*y = {}",
                            a1, b1, c1, a2, -b2, c2
                        ),
                        target_equation_latex: sys_tex.clone(),
                        answer_latex: format!(r"x = {}", sol_x),
                        answer: AnswerFormat::Numeric {
                            expected: sol_x as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", sol_x),
                        },
                        hints: vec![
                            format!("Eliminate y by multiplying the first equation by {} and the second by {}, then adding.", -b2, b1),
                            format!("Combined x equation: ({}*{} + {}*{})*x = {}*{} + {}*{}.", a1, -b2, a2, b1, c1, -b2, c2, b1),
                            format!("Divide both sides by {} to isolate x.", det_x),
                        ],
                        derivation_steps: vec![
                            format!("1. Scale Eq1 by {} and Eq2 by {} so y-coefficients cancel.", -b2, b1),
                            format!("2. Sum equations: {}*x = {}.", det_x, rhs_x),
                            format!("3. Intersection coordinate: x = {} (with y = {}).", sol_x, sol_y),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Scale Rows to Eliminate y",
                                r"|b_2| R_1 + b_1 R_2 \implies \text{cancel } y",
                                format!("Scale Eq1 by {} and Eq2 by {} so the y-terms sum to zero.", -b2, b1),
                                sys_tex,
                                format!(r"{}({}x + {}y) + {}({}x - {}y) = {}({}) + {}({})", -b2, a1, b1, b1, a2, -b2, -b2, c1, b1, c2),
                                format!("{}*({}) + {}*({})", -b2, c1, b1, c2),
                            ),
                            raw_step(
                                "Sum Scaled Equations",
                                r"(a_1|b_2| + a_2 b_1)x = c_1|b_2| + c_2 b_1",
                                "Collect like x-coefficients and constant sums.",
                                format!(r"({}\cdot{} + {}\cdot{})x = {}\cdot{} + {}\cdot{}", a1, -b2, a2, b1, c1, -b2, c2, b1),
                                format!(r"{}x = {}", det_x, rhs_x),
                                format!("{}*x = {}", det_x, rhs_x),
                            ),
                            raw_step(
                                "Isolate Intersection Coordinate",
                                r"x = \frac{\text{RHS}}{\Delta_x}",
                                format!("Divide both sides by {} (giving y = {} as companion coordinate).", det_x, sol_y),
                                format!(r"{}x = {}", det_x, rhs_x),
                                format!(r"x = \frac{{{}}}{{{}}} = {}", rhs_x, det_x, sol_x),
                                format!("{}", sol_x),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([(sol_x - 4) as f64, (sol_x + 4) as f64]),
                            primary_expr: Some(format!("({} - {}*x)/{}", c1, a1, b1)),
                            primary_label: Some(format!("Line 1: {}x + {}y = {}", a1, b1, c1)),
                            secondary_expr: Some(format!("({}*x - {})/{}", a2, c2, -b2)),
                            secondary_label: Some(format!("Line 2: {}x - {}y = {}", a2, -b2, c2)),
                            highlight_points: vec![[sol_x as f64, sol_y as f64]],
                            vline_x: Some(sol_x as f64),
                            vline_label: Some(format!("x = {}", sol_x)),
                            ..Default::default()
                        }),
                        remediate_module_id: None,
                    }
                }
            }

            "alg-quadratics" => {
                let h = rng.gen_range(-5..=5i32);
                let h = if h == 0 { 3 } else { h };
                let k = rng.gen_range(-7..=7i32);
                let a = rng.gen_range(1..=3i32);
                let b = -2 * a * h;
                let c = a * h * h + k;
                let shift_part = if h > 0 {
                    format!("x - {}", h)
                } else {
                    format!("x + {}", -h)
                };
                let vertex_expr = fmt_add_const(&format!("{}*({})^2", a, shift_part), k);
                let a_tex = if a == 1 { String::new() } else { a.to_string() };
                let q_tex = format!(r"{}x^{{2}} {}x {}", a_tex, signed_i(b), signed_i(c));
                let inner_tex = format!(r"x^{{2}} {}x", signed_i(-2 * h));
                let vertex_tex = plain_to_latex_or(&vertex_expr);
                if variant == 0 {
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::QuadraticVertex,
                        title: "Vertex Minimum via Completing the Square".to_string(),
                        prompt: format!(
                            "Complete the square on q(x) = {}*x^2 + ({})*x + ({}) and find its minimum output value k (the y-coordinate of the vertex).",
                            a, b, c
                        ),
                        target_equation_latex: format!(r"q(x) = {}", q_tex),
                        answer_latex: format!(r"k = {}", k),
                        answer: AnswerFormat::Numeric {
                            expected: k as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", k),
                        },
                        hints: vec![
                            format!("Factor {} from the x-terms: {} * (x^2 + ({})*x) + ({}).", a, a, -2 * h, c),
                            format!("Half of {} is {}, whose square is {}.", -2 * h, -h, h * h),
                            format!("Vertex form is {}, so minimum value is {}.", vertex_expr, k),
                        ],
                        derivation_steps: vec![
                            format!("1. Factor leading coefficient: q(x) = {}*(x^2 + ({})*x) + ({}).", a, -2 * h, c),
                            format!("2. Complete the square inside parentheses: {}.", vertex_expr),
                            format!("3. Vertex sits at ({}, {}), giving minimum value k = {}.", h, k, k),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Factor Leading Quadratic Coefficient",
                                r"ax^{2} + bx + c = a\!\left(x^{2} + \frac{b}{a}x\right) + c",
                                format!("Group the quadratic and linear x-terms with factor a = {}.", a),
                                format!(r"q(x) = {}", q_tex),
                                format!(r"q(x) = {}({}) {}", a, inner_tex, signed_i(c)),
                                format!("{}*(x^2 + ({})*x) + ({})", a, -2 * h, c),
                            ),
                            raw_step(
                                "Complete the Square Inside Parentheses",
                                r"x^{2} + px = \left(x + \frac{p}{2}\right)^{2} - \left(\frac{p}{2}\right)^{2}",
                                format!("Half of {} is {}, so add and subtract {} inside.", -2 * h, -h, h * h),
                                format!(r"{}({}) {}", a, inner_tex, signed_i(c)),
                                format!(r"q(x) = {}", vertex_tex),
                                vertex_expr.clone(),
                            ),
                            raw_step(
                                "Read Global Minimum at Vertex",
                                r"\min_{x \in \mathbb{R}}\left[a(x-h)^{2}+k\right] = k \quad (a > 0)",
                                format!("Since {}({})^2 >= 0, the minimum occurs at x = {}.", a, shift_part, h),
                                format!(r"q(x) = {}", vertex_tex),
                                format!(r"k = q({}) = {}", h, k),
                                format!("{}", k),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([(h - 4) as f64, (h + 4) as f64]),
                            primary_expr: Some(format!("{}*x^2 + ({})*x + ({})", a, b, c)),
                            primary_label: Some("q(x)".to_string()),
                            highlight_points: vec![[h as f64, k as f64]],
                            hline_y: Some(k as f64),
                            hline_label: Some(format!("min k = {}", k)),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("alg-linear-systems".to_string()),
                    }
                } else {
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::QuadraticVertex,
                        title: "Rewrite Quadratic in Vertex Form".to_string(),
                        prompt: format!(
                            "Write an expression equivalent to {}*x^2 + ({})*x + ({}) in completed-square vertex form.",
                            a, b, c
                        ),
                        target_equation_latex: format!(r"q(x) = {}", q_tex),
                        answer_latex: format!(r"q(x) = {}", vertex_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: vertex_expr.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Axis of symmetry is x = -b/(2*a) = -({}) / (2*{}) = {}.", b, a, h),
                            format!("Evaluate q({}) to obtain the vertical offset k = {}.", h, k),
                            format!("Assemble a*(x - h)^2 + k -> {}.", vertex_expr),
                        ],
                        derivation_steps: vec![
                            format!("1. Compute h = -b/(2*a) = {}.", h),
                            format!("2. Compute k = c - a*h^2 = {} - {}*{} = {}.", c, a, h * h, k),
                            format!("3. Vertex form: {}.", vertex_expr),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Locate Axis of Symmetry",
                                r"h = -\frac{b}{2a}",
                                format!("Identify vertex abscissa h using a = {} and b = {}.", a, b),
                                format!(r"q(x) = {}", q_tex),
                                format!(r"h = -\frac{{{}}}{{2 \cdot {}}} = {}", b, a, h),
                                format!("{}", h),
                            ),
                            raw_step(
                                "Compute Vertical Vertex Offset",
                                r"k = c - a h^{2}",
                                format!("Evaluate constant shift k at x = {}.", h),
                                format!(r"k = {} - {} \cdot ({})^{{2}}", c, a, h),
                                format!(r"k = {}", k),
                                format!("{}", k),
                            ),
                            raw_step(
                                "Assemble Vertex Form",
                                r"q(x) = a(x - h)^{2} + k",
                                "Substitute a, h, and k into canonical vertex form.",
                                format!(r"a = {},\; h = {},\; k = {}", a, h, k),
                                format!(r"q(x) = {}", vertex_tex),
                                vertex_expr.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([(h - 4) as f64, (h + 4) as f64]),
                            primary_expr: Some(format!("{}*x^2 + ({})*x + ({})", a, b, c)),
                            primary_label: Some("q(x) standard".to_string()),
                            secondary_expr: Some(vertex_expr),
                            secondary_label: Some("q(x) vertex form".to_string()),
                            highlight_points: vec![[h as f64, k as f64]],
                            ..Default::default()
                        }),
                        remediate_module_id: Some("alg-linear-systems".to_string()),
                    }
                }
            }

            "alg-polynomial-rational" => {
                let r1 = rng.gen_range(1..=4i32);
                let r2 = -rng.gen_range(1..=4i32);
                let cap_a = rng.gen_range(1..=4i32);
                let cap_b = rng.gen_range(1..=4i32);
                let num_m = cap_a + cap_b;
                let num_c = -(cap_a * r2 + cap_b * r1);
                let target_tex = format!(
                    r"\frac{{{}x {}}}{{(x - {})(x + {})}} = \frac{{A}}{{x - {}}} + \frac{{B}}{{x + {}}}",
                    num_m, signed_i(num_c), r1, -r2, r1, -r2
                );
                let cleared_tex = format!(
                    r"{}x {} = A(x + {}) + B(x - {})",
                    num_m, signed_i(num_c), -r2, r1
                );
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::PartialFractions,
                    title: "Partial Fraction Coefficient Cover-Up".to_string(),
                    prompt: format!(
                        "Decompose ({}*x + ({})) / ((x - {})*(x + {})) = A / (x - {}) + B / (x + {}). Find the exact value of A.",
                        num_m, num_c, r1, -r2, r1, -r2
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"A = {}", cap_a),
                    answer: AnswerFormat::Numeric {
                        expected: cap_a as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", cap_a),
                    },
                    hints: vec![
                        format!("Multiply both sides by (x - {})*(x + {}): {}*x + ({}) = A*(x + {}) + B*(x - {}).", r1, -r2, num_m, num_c, -r2, r1),
                        format!("Heaviside cover-up: evaluate both sides at x = {} so the B-term vanishes.", r1),
                        format!("Solve ({}*{} + ({})) / ({} + {}) = A.", num_m, r1, num_c, r1, -r2),
                    ],
                    derivation_steps: vec![
                        format!("1. Clear denominators: {}*x + ({}) = A*(x + {}) + B*(x - {}).", num_m, num_c, -r2, r1),
                        format!("2. Substitute x = {}: {} = A * ({}).", r1, num_m * r1 + num_c, r1 - r2),
                        format!("3. Divide by {} to obtain A = {} (and B = {}).", r1 - r2, cap_a, cap_b),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Clear Rational Denominator",
                            r"\frac{N(x)}{(x-r_1)(x-r_2)} = \frac{A}{x-r_1} + \frac{B}{x-r_2}",
                            format!("Multiply both sides by (x - {})(x + {}).", r1, -r2),
                            target_tex,
                            cleared_tex.clone(),
                            format!("{}*x + ({})", num_m, num_c),
                        ),
                        raw_step(
                            "Heaviside Cover-Up at Pole",
                            r"\text{Substitute } x = r_1 \implies B\text{-term vanishes}",
                            format!("Evaluate the polynomial identity at x = {}.", r1),
                            cleared_tex,
                            format!(r"{} = A({} + {}) = {}A", num_m * r1 + num_c, r1, -r2, r1 - r2),
                            format!("{}*A = {}", r1 - r2, num_m * r1 + num_c),
                        ),
                        raw_step(
                            "Isolate Residue Coefficient A",
                            r"A = \frac{N(r_1)}{r_1 - r_2}",
                            format!("Divide by {} (companion coefficient B = {}).", r1 - r2, cap_b),
                            format!(r"{}A = {}", r1 - r2, num_m * r1 + num_c),
                            format!(r"A = \frac{{{}}}{{{}}} = {}", num_m * r1 + num_c, r1 - r2, cap_a),
                            format!("{}", cap_a),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-quadratics".to_string()),
                }
            }

            "alg-radicals-exponents" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(-4..=5i32);
                if variant == 0 {
                    let clean_ans = fmt_add_const(&format!("{}*x + {}*h", 2 * a, a), b);
                    let f_tex = format!(r"{}x^{{2}} {}x", a, signed_i(b));
                    let ans_tex = plain_to_latex_or(&clean_ans);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::DifferenceQuotient,
                        title: "Simplify Algebraic Difference Quotient".to_string(),
                        prompt: format!(
                            "For f(x) = {}*x^2 + ({})*x, simplify the difference quotient (f(x + h) - f(x)) / h for h != 0 (in variables x and h).",
                            a, b
                        ),
                        target_equation_latex: format!(
                            r"f(x) = {},\qquad Q(x,h) = \frac{{f(x+h) - f(x)}}{{h}}",
                            f_tex
                        ),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: clean_ans.clone(),
                            vars: vec!["x".to_string(), "h".to_string()],
                        },
                        hints: vec![
                            format!("Expand f(x + h) = {}*(x^2 + 2*x*h + h^2) + ({})*(x + h).", a, b),
                            format!("Subtract f(x) = {}*x^2 + ({})*x; every remaining term contains h.", a, b),
                            format!("Factor out h and cancel with denominator to get {}.", clean_ans),
                        ],
                        derivation_steps: vec![
                            format!("1. Numerator: f(x+h) - f(x) = {}*(2*x*h + h^2) + ({})*h.", a, b),
                            format!("2. Factor h: h * ({}).", clean_ans),
                            format!("3. Cancel h (h != 0): {}.", clean_ans),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Expand Shifted Binomial f(x + h)",
                                r"(x+h)^{2} = x^{2} + 2xh + h^{2}",
                                "Subtract f(x) so all h-independent terms cancel.",
                                format!(r"\frac{{\left[{} (x+h)^{{2}} {} (x+h)\right] - \left[{}x^{{2}} {}x\right]}}{{h}}", a, signed_i(b), a, signed_i(b)),
                                format!(r"\frac{{{}(2xh + h^{{2}}) {} h}}{{h}}", a, signed_i(b)),
                                format!("({}*(2*x*h + h^2) + ({})*h)/h", a, b),
                            ),
                            raw_step(
                                "Factor Common Step h",
                                r"\frac{h \cdot g(x,h)}{h} = g(x,h) \quad (h \ne 0)",
                                "Factor h from every numerator term.",
                                format!(r"\frac{{{}(2xh + h^{{2}}) {} h}}{{h}}", a, signed_i(b)),
                                format!(r"\frac{{h \cdot \left({}\right)}}{{h}}", ans_tex),
                                format!("h*({})/h", clean_ans),
                            ),
                            raw_step(
                                "Cancel Removable Factor h",
                                r"\lim_{h\to 0}\text{ secant slope}",
                                "Divide numerator and denominator by h.",
                                format!(r"\frac{{h \cdot \left({}\right)}}{{h}}", ans_tex),
                                ans_tex,
                                clean_ans.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("alg-quadratics".to_string()),
                    }
                } else {
                    let p = rng.gen_range(2..=5i32);
                    let q = rng.gen_range(2..=4i32);
                    let num_exp = p * q - 1;
                    let expected = format!("x^({}/{})", num_exp, q);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        operation: OperationKind::DifferenceQuotient,
                        title: "Convert Radical Quotient to Power Form".to_string(),
                        prompt: format!(
                            "Rewrite x^{} / x^(1/{}) as a single power of x (valid for x > 0).",
                            p, q
                        ),
                        target_equation_latex: format!(r"\frac{{x^{{{}}}}}{{\sqrt[{}]{{x}}}} = x^{{\alpha}}", p, q),
                        answer_latex: format!(r"x^{{\frac{{{}}}{{{}}}}}", num_exp, q),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Apply quotient exponent law x^A / x^B = x^(A - B) with A = {} and B = 1/{}.", p, q),
                            format!("Common denominator: {} - 1/{} = {}/{}.", p, q, num_exp, q),
                            format!("Result: {}.", expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Subtract exponents: {} - 1/{}.", p, q),
                            format!("2. Combine fraction: {}/{}.", num_exp, q),
                            format!("3. Power expression: {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Quotient Exponent Law",
                                r"\frac{x^{A}}{x^{B}} = x^{A - B}",
                                format!("Rewrite root x^(1/{}) as fractional exponent and subtract.", q),
                                format!(r"\frac{{x^{{{}}}}}{{x^{{1/{}}}}}", p, q),
                                format!(r"x^{{{} - \frac{{1}}{{{}}}}}", p, q),
                                format!("x^({} - 1/{})", p, q),
                            ),
                            raw_step(
                                "Combine Fractional Exponent",
                                r"p - \frac{1}{q} = \frac{pq - 1}{q}",
                                format!("Place {} over common denominator {}.", p, q),
                                format!(r"x^{{\frac{{{}}}{{{}}}-\frac{{1}}{{{}}}}}", p * q, q, q),
                                format!(r"x^{{\frac{{{}}}{{{}}}}}", num_exp, q),
                                expected.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: None,
                    }
                }
            }

            "alg-exp-log" => {
                let k = rng.gen_range(2..=5i32);
                let ratio = rng.gen_range(2..=6i32);
                let expected = (ratio as f64).ln() / (k as f64);
                let disp = format!("ln({})/{}", ratio, k);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::ExponentialLogSolve,
                    title: "Solve Exponential Growth Equation".to_string(),
                    prompt: format!(
                        "Solve exactly for t:  3 * exp({}*t) = {}. (You may enter exact plain-text form like ln(a)/b.)",
                        k, 3 * ratio
                    ),
                    target_equation_latex: format!(r"3 e^{{{}t}} = {}", k, 3 * ratio),
                    answer_latex: format!(r"t = \frac{{\ln({})}}{{{}}}", ratio, k),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Divide both sides by 3 first to isolate exp({}*t) = {}.", k, ratio),
                        format!("Apply the natural logarithm ln(.) to both sides: {}*t = ln({}).", k, ratio),
                        format!("Divide by {}: t = {}.", k, disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Isolate exponential: exp({}*t) = {} / 3 = {}.", k, 3 * ratio, ratio),
                        format!("2. Invert via ln: {}*t = ln({}).", k, ratio),
                        format!("3. Exact solution: t = {} (~{:.5}).", disp, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Isolate Exponential Term",
                            r"A e^{kt} = B \implies e^{kt} = \frac{B}{A}",
                            "Divide both sides by leading scalar 3.",
                            format!(r"3 e^{{{}t}} = {}", k, 3 * ratio),
                            format!(r"e^{{{}t}} = \frac{{{}}}{{3}} = {}", k, 3 * ratio, ratio),
                            format!("exp({}*t) = {}", k, ratio),
                        ),
                        raw_step(
                            "Invert via Natural Logarithm",
                            r"\ln(e^{u}) = u",
                            "Take the natural logarithm of both sides.",
                            format!(r"e^{{{}t}} = {}", k, ratio),
                            format!(r"{}t = \ln({})", k, ratio),
                            format!("{}*t = ln({})", k, ratio),
                        ),
                        raw_step(
                            "Divide by Growth Rate",
                            r"t = \frac{\ln(r)}{k}",
                            format!("Divide by k = {} (~{:.5}).", k, expected),
                            format!(r"{}t = \ln({})", k, ratio),
                            format!(r"t = \frac{{\ln({})}}{{{}}}", ratio, k),
                            disp.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "t".to_string(),
                        x_range: Some([-0.2, expected + 1.0]),
                        primary_expr: Some(format!("3*exp({}*t)", k)),
                        primary_label: Some(format!("3 e^({}t)", k)),
                        hline_y: Some((3 * ratio) as f64),
                        hline_label: Some(format!("y = {}", 3 * ratio)),
                        vline_x: Some(expected),
                        vline_label: Some(format!("t = {:.3}", expected)),
                        highlight_points: vec![[expected, (3 * ratio) as f64]],
                        ..Default::default()
                    }),
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "alg-sequences-series" => {
                if variant == 0 {
                    let a = rng.gen_range(2..=8i32);
                    let den = rng.gen_range(3..=6i32);
                    let num = rng.gen_range(1..den);
                    let ans_num = a * den;
                    let ans_den = den - num;
                    let expected = (ans_num as f64) / (ans_den as f64);
                    let disp = format!("{}/{}", ans_num, ans_den);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::GeometricSeries,
                        title: "Infinite Geometric Series Sum".to_string(),
                        prompt: format!(
                            "Evaluate the convergent infinite geometric series: sum_{{k=0..inf}} {} * ({}/{})^k.",
                            a, num, den
                        ),
                        target_equation_latex: format!(
                            r"S = \sum_{{k=0}}^{{\infty}} {} \left(\frac{{{}}}{{{}}}\right)^{{k}}",
                            a, num, den
                        ),
                        answer_latex: format!(r"S = \frac{{{}}}{{{}}}", ans_num, ans_den),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-5,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Identify first term a = {} (at k = 0) and common ratio r = {}/{} (|r| < 1).", a, num, den),
                            format!("Apply S = a / (1 - r) = {} / (1 - {}/{}).", a, num, den),
                            format!("Simplify {} / ({}/{}) = {}.", a, ans_den, den, disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Since |{}/{}| < 1, geometric sum converges to a / (1 - r).", num, den),
                            format!("2. Denominator: 1 - {}/{} = {}/{}.", num, den, ans_den, den),
                            format!("3. Exact sum: {} * {} / {} = {}.", a, den, ans_den, disp),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Verify Geometric Convergence",
                                r"\sum_{k=0}^{\infty} a r^{k} = \frac{a}{1 - r} \quad (|r| < 1)",
                                format!("Identify first term a = {} and ratio r = {}/{} with |r| < 1.", a, num, den),
                                format!(r"\sum_{{k=0}}^{{\infty}} {} \left(\frac{{{}}}{{{}}}\right)^{{k}}", a, num, den),
                                format!(r"S = \frac{{{}}}{{1 - \frac{{{}}}{{{}}}}}", a, num, den),
                                format!("{} / (1 - {}/{})", a, num, den),
                            ),
                            raw_step(
                                "Evaluate Rational Denominator",
                                r"1 - \frac{p}{q} = \frac{q - p}{q}",
                                "Combine denominator into a single fraction.",
                                format!(r"\frac{{{}}}{{1 - \frac{{{}}}{{{}}}}}", a, num, den),
                                format!(r"\frac{{{}}}{{\frac{{{}}}{{{}}}}}", a, ans_den, den),
                                format!("{} / ({}/{})", a, ans_den, den),
                            ),
                            raw_step(
                                "Invert and Multiply",
                                r"\frac{a}{u/v} = \frac{a v}{u}",
                                "Multiply numerator by reciprocal of denominator fraction.",
                                format!(r"\frac{{{}}}{{\frac{{{}}}{{{}}}}}", a, ans_den, den),
                                format!(r"\frac{{{}}}{{{}}}", ans_num, ans_den),
                                disp.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("alg-polynomial-rational".to_string()),
                    }
                } else {
                    let n = rng.gen_range(6..=14i32);
                    let c = rng.gen_range(2..=4i32);
                    let sq_sum = n * (n + 1) * (2 * n + 1) / 6;
                    let val = c * sq_sum;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::FiniteSeriesSum,
                        title: "Finite Quadratic Sigma Sum".to_string(),
                        prompt: format!("Evaluate the finite sum: sum_{{k=1..{}}} {} * k^2.", n, c),
                        target_equation_latex: format!(r"\sum_{{k=1}}^{{{}}} {} k^{{2}}", n, c),
                        answer_latex: format!("{}", val),
                        answer: AnswerFormat::Numeric {
                            expected: val as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", val),
                        },
                        hints: vec![
                            format!("Factor constant {} outside the sum: {} * sum_{{k=1..{}}} k^2.", c, c, n),
                            "Recall sum_{k=1..n} k^2 = n * (n + 1) * (2*n + 1) / 6.".to_string(),
                            format!("Compute {} * {} * {} * {} / 6.", c, n, n + 1, 2 * n + 1),
                        ],
                        derivation_steps: vec![
                            format!("1. Factor linearity: {} * [n*(n+1)*(2*n+1)/6] at n = {}.", c, n),
                            format!("2. Sum of squares 1..{}: {}.", n, sq_sum),
                            format!("3. Multiply by {}: {}.", c, val),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Factor Constant Outside Sigma",
                                r"\sum_{k=1}^{n} c\, k^{2} = c \cdot \frac{n(n+1)(2n+1)}{6}",
                                format!("Pull out c = {} and apply Faulhaber's quadratic sum identity at n = {}.", c, n),
                                format!(r"\sum_{{k=1}}^{{{}}} {} k^{{2}}", n, c),
                                format!(r"{} \cdot \frac{{{}\cdot{}\cdot{}}}{{6}}", c, n, n + 1, 2 * n + 1),
                                format!("{}*{}*{}/6", c, n, (n + 1) * (2 * n + 1)),
                            ),
                            raw_step(
                                "Evaluate Sum of Squares",
                                r"S_2(n) = \frac{n(n+1)(2n+1)}{6}",
                                format!("Evaluate the integer sum of squares for 1..{}.", n),
                                format!(r"{} \cdot \frac{{{}\cdot{}\cdot{}}}{{6}}", c, n, n + 1, 2 * n + 1),
                                format!(r"{} \cdot {}", c, sq_sum),
                                format!("{}*{}", c, sq_sum),
                            ),
                            raw_step(
                                "Multiply Scaled Sum",
                                r"\text{Exact integer total}",
                                format!("Compute final product {} * {} = {}.", c, sq_sum, val),
                                format!(r"{} \cdot {}", c, sq_sum),
                                format!(r"{}", val),
                                format!("{}", val),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: None,
                    }
                }
            }

            // =====================================================================
            // STAGE 2: TRIGONOMETRY
            // =====================================================================
            "trig-unit-circle" => {
                let choices: &[(&str, f64, &str, &str, &str, &str, &str)] = &[
                    (
                        "cos(2*pi/3)",
                        -0.5,
                        "-1/2",
                        "Quadrant II reference angle pi/3 has cosine -1/2",
                        r"\cos\!\left(\frac{2\pi}{3}\right)",
                        r"-\frac{1}{2}",
                        r"\cos\!\left(\pi - \frac{\pi}{3}\right) = -\cos\!\left(\frac{\pi}{3}\right)",
                    ),
                    (
                        "sin(5*pi/6)",
                        0.5,
                        "1/2",
                        "Quadrant II reference angle pi/6 has sine +1/2",
                        r"\sin\!\left(\frac{5\pi}{6}\right)",
                        r"\frac{1}{2}",
                        r"\sin\!\left(\pi - \frac{\pi}{6}\right) = +\sin\!\left(\frac{\pi}{6}\right)",
                    ),
                    (
                        "cos(5*pi/4)",
                        -std::f64::consts::FRAC_1_SQRT_2,
                        "-sqrt(2)/2",
                        "Quadrant III reference angle pi/4 has cosine -sqrt(2)/2",
                        r"\cos\!\left(\frac{5\pi}{4}\right)",
                        r"-\frac{\sqrt{2}}{2}",
                        r"\cos\!\left(\pi + \frac{\pi}{4}\right) = -\cos\!\left(\frac{\pi}{4}\right)",
                    ),
                    (
                        "sin(4*pi/3)",
                        -(3.0_f64).sqrt() / 2.0,
                        "-sqrt(3)/2",
                        "Quadrant III reference angle pi/3 has sine -sqrt(3)/2",
                        r"\sin\!\left(\frac{4\pi}{3}\right)",
                        r"-\frac{\sqrt{3}}{2}",
                        r"\sin\!\left(\pi + \frac{\pi}{3}\right) = -\sin\!\left(\frac{\pi}{3}\right)",
                    ),
                    (
                        "tan(3*pi/4)",
                        -1.0,
                        "-1",
                        "Quadrant II angle 3*pi/4 has sin = sqrt(2)/2, cos = -sqrt(2)/2",
                        r"\tan\!\left(\frac{3\pi}{4}\right)",
                        r"-1",
                        r"\frac{\sin(3\pi/4)}{\cos(3\pi/4)} = \frac{\sqrt{2}/2}{-\sqrt{2}/2}",
                    ),
                ];
                let pick = choices[rng.gen_range(0..choices.len())];
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    operation: OperationKind::ExactTrigonometric,
                    title: "Exact Unit Circle Coordinate".to_string(),
                    prompt: format!("Evaluate {} exactly (enter a fraction or radical such as -sqrt(3)/2).", pick.0),
                    target_equation_latex: pick.4.to_string(),
                    answer_latex: pick.5.to_string(),
                    answer: AnswerFormat::Numeric {
                        expected: pick.1,
                        tolerance: 1e-4,
                        canonical_display: pick.2.to_string(),
                    },
                    hints: vec![
                        "Locate the quadrant of the radian angle and determine whether x = cos(theta) or y = sin(theta) is positive or negative.".to_string(),
                        pick.3.to_string(),
                        format!("Exact value: {}.", pick.2),
                    ],
                    derivation_steps: vec![
                        format!("1. Symmetry & reference angle check: {}.", pick.3),
                        format!("2. Exact coordinate value: {}.", pick.2),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Quadrant Symmetry & Reference Angle",
                            r"\theta_{\text{ref}} \in \left(0,\, \frac{\pi}{2}\right),\; (\cos\theta,\sin\theta) \in S^{1}",
                            pick.3,
                            pick.4,
                            pick.6,
                            pick.0,
                        ),
                        raw_step(
                            "Exact Unit Circle Ratio",
                            r"\cos(\theta) = x,\quad \sin(\theta) = y",
                            format!("Evaluate reference angle exact value with quadrant sign -> {}.", pick.2),
                            pick.6,
                            pick.5,
                            pick.2,
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "trig-graphs-waves" => {
                let amp = rng.gen_range(2..=7i32);
                let omega = rng.gen_range(2..=6i32);
                let expected = 2.0 * std::f64::consts::PI / (omega as f64);
                let disp = format!("2*pi/{}", omega);
                let target_tex = format!(r"y(t) = {} \cos\!\left({}t - \frac{{\pi}}{{3}}\right)", amp, omega);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    operation: OperationKind::HarmonicWavePeriod,
                    title: "Fundamental Period of a Harmonic Wave".to_string(),
                    prompt: format!(
                        "Find the fundamental period T of y(t) = {} * cos({}*t - pi/3). (Enter exact form in terms of pi.)",
                        amp, omega
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"T = \frac{{2\pi}}{{{}}}", omega),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Amplitude {} and phase -pi/3 do not alter the period; only angular frequency omega = {} does.", amp, omega),
                        "One full cycle completes when omega * T = 2*pi.".to_string(),
                        format!("Solve T = 2*pi / {} = {}.", omega, disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Set argument increment {} * T = 2*pi.", omega),
                        format!("2. Fundamental period: T = {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Identify Angular Frequency",
                            r"y(t) = A\cos(\omega t - \phi) \implies \omega T = 2\pi",
                            format!("Extract angular frequency omega = {}; amplitude {} and phase -pi/3 shift do not alter T.", omega, amp),
                            target_tex,
                            format!(r"\omega = {},\quad {}T = 2\pi", omega, omega),
                            format!("{}*T = 2*pi", omega),
                        ),
                        raw_step(
                            "Solve for Fundamental Period T",
                            r"T = \frac{2\pi}{\omega}",
                            format!("Divide 2*pi by omega = {} to obtain exact period.", omega),
                            format!(r"{}T = 2\pi", omega),
                            format!(r"T = \frac{{2\pi}}{{{}}}", omega),
                            disp.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "t".to_string(),
                        x_range: Some([0.0, expected * 1.5]),
                        primary_expr: Some(format!("{}*cos({}*t - pi/3)", amp, omega)),
                        primary_label: Some("y(t)".to_string()),
                        vline_x: Some(expected),
                        vline_label: Some(format!("Period T = {:.3}", expected)),
                        ..Default::default()
                    }),
                    remediate_module_id: Some("trig-unit-circle".to_string()),
                }
            }

            "trig-identities" => {
                let k = rng.gen_range(2..=6i32);
                let expected = format!("{} * sec(x)^2", k);
                let target_tex = format!(r"{} + {} \tan^{{2}}(x)", k, k);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::PythagoreanIdentity,
                    title: "Pythagorean Secant-Tangent Reduction".to_string(),
                    prompt: format!(
                        "Simplify {} + {} * tan(x)^2 into a single trigonometric term.",
                        k, k
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"{} \sec^{{2}}(x)", k),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        format!("Factor out {} to get {} * (1 + tan(x)^2).", k, k),
                        "Apply the Pythagorean identity 1 + tan(x)^2 = sec(x)^2.".to_string(),
                        format!("Simplified form: {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Factor common scalar: {} * (1 + tan(x)^2).", k),
                        "2. Substitute 1 + tan(x)^2 = 1/cos(x)^2 = sec(x)^2.".to_string(),
                        format!("3. Result: {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Factor Common Scalar",
                            r"ka + kb = k(a + b)",
                            format!("Factor {} out of both trigonometric terms.", k),
                            target_tex,
                            format!(r"{}\left(1 + \tan^{{2}}(x)\right)", k),
                            format!("{}*(1 + tan(x)^2)", k),
                        ),
                        raw_step(
                            "Pythagorean Secant Identity",
                            r"1 + \tan^{2}(x) = \sec^{2}(x)",
                            "Replace 1 + tan(x)^2 with sec(x)^2.",
                            format!(r"{}\left(1 + \tan^{{2}}(x)\right)", k),
                            format!(r"{} \sec^{{2}}(x)", k),
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("trig-unit-circle".to_string()),
                }
            }

            "trig-compound-double" => {
                let m = rng.gen_range(1..=4i32);
                let coeff = 2 * rng.gen_range(1..=4i32);
                let half = coeff / 2;
                let expected = format!("{} * (1 - cos({}*x))", half, 2 * m);
                let target_tex = format!(r"{} \sin^{{2}}({}x)", coeff, m);
                let ans_tex = format!(r"{}\left(1 - \cos({}x)\right)", half, 2 * m);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::PowerReduction,
                    title: "Power-Reduction Linearization".to_string(),
                    prompt: format!(
                        "Rewrite {} * sin({}*x)^2 using the power-reduction identity (no exponents > 1) to prepare it for integration.",
                        coeff, m
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: ans_tex.clone(),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        "Recall sin(u)^2 = (1 - cos(2*u)) / 2.".to_string(),
                        format!("Set u = {}*x so 2*u = {}*x.", m, 2 * m),
                        format!("Multiply by {}: {}.", coeff, expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Substitute u = {}*x into sin(u)^2 = (1 - cos(2*u))/2.", m),
                        format!("2. Scale by {}: ({}/2) * (1 - cos({}*x)) = {}.", coeff, coeff, 2 * m, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Apply Half-Angle Power Reduction",
                            r"\sin^{2}(u) = \frac{1 - \cos(2u)}{2}",
                            format!("Substitute u = {}x so the doubled angle is 2u = {}x.", m, 2 * m),
                            target_tex,
                            format!(r"{} \cdot \frac{{1 - \cos({}x)}}{{2}}", coeff, 2 * m),
                            format!("{}*(1 - cos({}*x))/2", coeff, 2 * m),
                        ),
                        raw_step(
                            "Simplify Leading Fraction",
                            r"\frac{c}{2}\left(1 - \cos(2mx)\right)",
                            format!("Reduce {}/2 = {} into a linearized cosine expression.", coeff, half),
                            format!(r"{} \cdot \frac{{1 - \cos({}x)}}{{2}}", coeff, 2 * m),
                            ans_tex,
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("trig-identities".to_string()),
                }
            }

            "trig-inverse-eqs" => {
                let a = rng.gen_range(2..=5i32);
                let expected = format!("({}*x) / sqrt(1 + {}*x^2)", a, a * a);
                let target_tex = format!(r"\sin\!\left(\arctan({}x)\right)", a);
                let ans_tex = format!(r"\frac{{{}x}}{{\sqrt{{1 + {}x^{{2}}}}}}", a, a * a);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::InverseTrigAlgebraic,
                    title: "Algebraic Reduction of Trig-Inverse Composition".to_string(),
                    prompt: format!(
                        "Express sin(atan({}*x)) as a purely algebraic expression in x.",
                        a
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: ans_tex.clone(),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        format!("Let theta = atan({}*x), so tan(theta) = ({}*x) / 1 with theta in (-pi/2, pi/2).", a, a),
                        format!("In a right triangle with opposite = {}*x and adjacent = 1, hypotenuse = sqrt(1 + {}*x^2).", a, a * a),
                        format!("Read off sin(theta) = opposite / hypotenuse = {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Set theta = atan({}*x) => tan(theta) = {}*x.", a, a),
                        format!("2. Pythagorean hypotenuse: r = sqrt(1 + ({}*x)^2) = sqrt(1 + {}*x^2).", a, a * a),
                        format!("3. sin(theta) = ({}*x) / r = {}.", a, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Construct Reference Right Triangle",
                            r"\theta = \arctan(u) \implies \tan\theta = \frac{u}{1}",
                            format!("Let theta = atan({}*x) so opposite = {}x and adjacent = 1.", a, a),
                            target_tex,
                            format!(r"\tan\theta = \frac{{{}x}}{{1}},\quad \text{{opp}} = {}x,\; \text{{adj}} = 1", a, a),
                            format!("tan(theta) = {}*x", a),
                        ),
                        raw_step(
                            "Pythagorean Hypotenuse",
                            r"r = \sqrt{\text{adj}^{2} + \text{opp}^{2}}",
                            format!("Compute hypotenuse r = sqrt(1 + {}*x^2).", a * a),
                            format!(r"r = \sqrt{{1^{{2}} + ({}x)^{{2}}}}", a),
                            format!(r"r = \sqrt{{1 + {}x^{{2}}}}", a * a),
                            format!("sqrt(1 + {}*x^2)", a * a),
                        ),
                        raw_step(
                            "Evaluate Sine Ratio",
                            r"\sin\theta = \frac{\text{opp}}{r}",
                            "Take opposite side over Pythagorean hypotenuse.",
                            format!(r"\sin\theta = \frac{{\text{{opp}}}}{{r}}"),
                            ans_tex,
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("trig-identities".to_string()),
                }
            }

            "trig-polar-parametric" => {
                let u1 = rng.gen_range(-4..=5i32);
                let u2 = rng.gen_range(1..=5i32);
                let v1 = rng.gen_range(-4..=5i32);
                let v2 = rng.gen_range(-4..=5i32);
                let dot = u1 * v1 + u2 * v2;
                let p1 = u1 * v1;
                let p2 = u2 * v2;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    operation: OperationKind::VectorDotProduct,
                    title: "2D Vector Dot Product & Orthogonality Check".to_string(),
                    prompt: format!(
                        "Compute the scalar dot product u . v for u = <{}, {}> and v = <{}, {}>.",
                        u1, u2, v1, v2
                    ),
                    target_equation_latex: format!(
                        r"\mathbf{{u}} = \langle {},\, {} \rangle,\quad \mathbf{{v}} = \langle {},\, {} \rangle,\quad \mathbf{{u}} \cdot \mathbf{{v}}",
                        u1, u2, v1, v2
                    ),
                    answer_latex: format!(r"\mathbf{{u}} \cdot \mathbf{{v}} = {}", dot),
                    answer: AnswerFormat::Numeric {
                        expected: dot as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", dot),
                    },
                    hints: vec![
                        "Use component-wise multiplication: <u1, u2> . <v1, v2> = u1*v1 + u2*v2.".to_string(),
                        format!("Compute ({})*({}) + ({})*({}).", u1, v1, u2, v2),
                        format!("Sum = {}.", dot),
                    ],
                    derivation_steps: vec![
                        format!("1. u1*v1 = ({})*({}) = {}.", u1, v1, p1),
                        format!("2. u2*v2 = ({})*({}) = {}.", u2, v2, p2),
                        format!("3. Dot product u . v = {} + ({}) = {}.", p1, p2, dot),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Componentwise Product Formula",
                            r"\langle u_1, u_2 \rangle \cdot \langle v_1, v_2 \rangle = u_1 v_1 + u_2 v_2",
                            "Multiply matched horizontal and vertical vector components.",
                            format!(r"\langle {},\, {} \rangle \cdot \langle {},\, {} \rangle", u1, u2, v1, v2),
                            format!(r"({}\cdot {}) + ({}\cdot {})", u1, v1, u2, v2),
                            format!("{}*{} + {}*{}", u1, v1, u2, v2),
                        ),
                        raw_step(
                            "Sum Scalar Contraction",
                            r"u_1 v_1 + u_2 v_2 \in \mathbb{R}",
                            format!("Add {} and {} to get the inner product.", p1, p2),
                            format!(r"{} {}", p1, signed_i(p2)),
                            format!(r"\mathbf{{u}} \cdot \mathbf{{v}} = {}", dot),
                            format!("{}", dot),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-linear-systems".to_string()),
                }
            }

            // =====================================================================
            // STAGE 3: SINGLE-VARIABLE CALCULUS
            // =====================================================================
            "calc-limits-continuity" => {
                if variant == 0 {
                    let a = rng.gen_range(2..=7i32);
                    let a2 = a * a;
                    let expected = 1.0 / (2.0 * a as f64);
                    let disp = format!("1/{}", 2 * a);
                    let target_tex = format!(
                        r"\lim_{{x \to {}}} \frac{{\sqrt{{x}} - {}}}{{x - {}}}",
                        a2, a, a2
                    );
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::LimitEvaluation,
                        title: "Indeterminate 0/0 Radical Limit".to_string(),
                        prompt: format!(
                            "Evaluate exactly: lim_{{x -> {}}} (sqrt(x) - {}) / (x - {}).",
                            a2, a, a2
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: format!(r"\frac{{1}}{{{}}}", 2 * a),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-5,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Direct substitution gives 0/0. Factor denominator x - {} = (sqrt(x) - {})*(sqrt(x) + {}).", a2, a, a),
                            format!("Cancel (sqrt(x) - {}) for x != {} to obtain 1 / (sqrt(x) + {}).", a, a2, a),
                            format!("Evaluate at x = {}: 1 / ({} + {}) = {}.", a2, a, a, disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Factor x - {} = (sqrt(x) - {})*(sqrt(x) + {}).", a2, a, a),
                            format!("2. Cancel removable factor: lim_{{x -> {}}} 1 / (sqrt(x) + {}).", a2, a),
                            format!("3. Continuous substitution: 1 / ({} + {}) = {}.", a, a, disp),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Difference of Squares Factorization",
                                r"x - a^{2} = (\sqrt{x} - a)(\sqrt{x} + a)",
                                format!("Direct substitution yields 0/0; factor x - {} as a difference of squares in sqrt(x).", a2),
                                target_tex,
                                format!(r"\lim_{{x \to {}}} \frac{{\sqrt{{x}} - {}}}{{(\sqrt{{x}} - {})(\sqrt{{x}} + {})}}", a2, a, a, a),
                                format!("(sqrt(x) - {}) / ((sqrt(x) - {})*(sqrt(x) + {}))", a, a, a),
                            ),
                            raw_step(
                                "Cancel Removable 0/0 Factor",
                                r"\lim_{x \to c} \frac{(x-c)g(x)}{(x-c)h(x)} = \lim_{x \to c} \frac{g(x)}{h(x)}",
                                format!("Cancel (sqrt(x) - {}) for x != {}.", a, a2),
                                format!(r"\lim_{{x \to {}}} \frac{{\sqrt{{x}} - {}}}{{(\sqrt{{x}} - {})(\sqrt{{x}} + {})}}", a2, a, a, a),
                                format!(r"\lim_{{x \to {}}} \frac{{1}}{{\sqrt{{x}} + {}}}", a2, a),
                                format!("1 / (sqrt(x) + {})", a),
                            ),
                            raw_step(
                                "Direct Continuous Substitution",
                                r"\text{Substitute } x = a^{2} \text{ into reduced form}",
                                format!("Evaluate at x = {} to find 1 / ({} + {}).", a2, a, a),
                                format!(r"\frac{{1}}{{\sqrt{{{}}} + {}}}", a2, a),
                                format!(r"\frac{{1}}{{{}}}", 2 * a),
                                disp.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([0.05, (a2 + 4) as f64]),
                            primary_expr: Some(format!("(sqrt(x) - {})/(x - {})", a, a2)),
                            primary_label: Some("f(x)".to_string()),
                            highlight_points: vec![[a2 as f64, expected]],
                            vline_x: Some(a2 as f64),
                            vline_label: Some(format!("x -> {}", a2)),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("alg-radicals-exponents".to_string()),
                    }
                } else {
                    let p = rng.gen_range(3..=8i32);
                    let q = rng.gen_range(2..=6i32);
                    let expected = (p as f64) / (q as f64);
                    let disp = format!("{}/{}", p, q);
                    let target_tex = format!(r"\lim_{{x \to 0}} \frac{{\sin({}x)}}{{{}x}}", p, q);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::LimitEvaluation,
                        title: "Small-Angle Trigonometric Limit".to_string(),
                        prompt: format!("Evaluate exactly: lim_{{x -> 0}} sin({}*x) / ({}*x).", p, q),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: format!(r"\frac{{{}}}{{{}}}", p, q),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-5,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Rewrite sin({}*x) / ({}*x) = ({}/{}) * [sin({}*x) / ({}*x)].", p, q, p, q, p, p),
                            format!("Let u = {}*x -> 0 and apply lim_{{u->0}} sin(u)/u = 1.", p),
                            format!("Limit equals {}.", disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Match numerator angle: ({}/{}) * [sin({}*x)/({}*x)].", p, q, p, p),
                            "2. Apply fundamental radian limit lim_{u->0} sin(u)/u = 1.".to_string(),
                            format!("3. Exact value: {}.", disp),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Rescale Small-Angle Argument",
                                r"\frac{\sin(px)}{qx} = \frac{p}{q} \cdot \frac{\sin(px)}{px}",
                                format!("Match denominator angle to {}x by scaling by {}/{}.", p, p, q),
                                target_tex,
                                format!(r"\frac{{{}}}{{{}}} \lim_{{x \to 0}} \frac{{\sin({}x)}}{{{}x}}", p, q, p, p),
                                format!("({}/{}) * (sin({}*x)/({}*x))", p, q, p, p),
                            ),
                            raw_step(
                                "Fundamental Radian Limit",
                                r"\lim_{u \to 0} \frac{\sin u}{u} = 1",
                                format!("With u = {}x -> 0, sin(u)/u -> 1.", p),
                                format!(r"\frac{{{}}}{{{}}} \lim_{{u \to 0}} \frac{{\sin u}}{{u}}", p, q),
                                format!(r"\frac{{{}}}{{{}}}", p, q),
                                disp.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-3.0, 3.0]),
                            primary_expr: Some(format!("sin({}*x)/({}*x)", p, q)),
                            primary_label: Some(format!("sin({}x)/({}x)", p, q)),
                            highlight_points: vec![[0.0, expected]],
                            vline_x: Some(0.0),
                            vline_label: Some("x -> 0".to_string()),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("trig-unit-circle".to_string()),
                    }
                }
            }

            "calc-derivatives-rules" => {
                if variant == 0 {
                    let a = rng.gen_range(2..=5i32);
                    let b = rng.gen_range(1..=4i32);
                    let expected = format!("exp({}*x) * ({}*sin({}*x) + {}*cos({}*x))", a, a, b, b, b);
                    let target_tex = format!(r"\frac{{d}}{{dx}}\!\left[e^{{{}x}} \sin({}x)\right]", a, b);
                    let ans_tex = format!(r"e^{{{}x}}\left({}\sin({}x) + {}\cos({}x)\right)", a, a, b, b, b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::Differentiation,
                        title: "Product & Chain Rule Differentiation".to_string(),
                        prompt: format!(
                            "Compute the derivative f'(x) of f(x) = exp({}*x) * sin({}*x).",
                            a, b
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Apply product rule (u*v)' = u'*v + u*v' with u = exp({}*x) and v = sin({}*x).", a, b),
                            format!("By chain rule, u' = {}*exp({}*x) and v' = {}*cos({}*x).", a, a, b, b),
                            format!("Combine and optionally factor exp({}*x): {}.", a, expected),
                        ],
                        derivation_steps: vec![
                            format!("1. u(x) = exp({}*x) => u'(x) = {}*exp({}*x).", a, a, a),
                            format!("2. v(x) = sin({}*x) => v'(x) = {}*cos({}*x).", b, b, b),
                            format!("3. Product rule: {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Chain Rule on Exponential Factor",
                                r"\frac{d}{dx}\left[e^{ax}\right] = a e^{ax}",
                                format!("Differentiate first factor u(x) = exp({}*x).", a),
                                format!(r"u(x) = e^{{{}x}}", a),
                                format!(r"u'(x) = {} e^{{{}x}}", a, a),
                                format!("{}*exp({}*x)", a, a),
                            ),
                            raw_step(
                                "Chain Rule on Sine Factor",
                                r"\frac{d}{dx}\left[\sin(bx)\right] = b \cos(bx)",
                                format!("Differentiate second factor v(x) = sin({}*x).", b),
                                format!(r"v(x) = \sin({}x)", b),
                                format!(r"v'(x) = {} \cos({}x)", b, b),
                                format!("{}*cos({}*x)", b, b),
                            ),
                            raw_step(
                                "Combine via Product Rule",
                                r"(uv)' = u'v + uv'",
                                format!("Assemble u'v + uv' and factor exp({}*x).", a),
                                target_tex,
                                ans_tex,
                                expected.clone(),
                            ),
                        ],
                        visual: Some({
                            let x0 = 0.5_f64;
                            let f0 = ((a as f64) * x0).exp() * ((b as f64) * x0).sin();
                            let df0 = ((a as f64) * x0).exp()
                                * ((a as f64) * ((b as f64) * x0).sin()
                                    + (b as f64) * ((b as f64) * x0).cos());
                            WalkthroughVisual {
                                var: "x".to_string(),
                                x_range: Some([-1.0, 2.5]),
                                primary_expr: Some(format!("exp({}*x)*sin({}*x)", a, b)),
                                primary_label: Some("f(x)".to_string()),
                                secondary_expr: Some(expected),
                                secondary_label: Some("f'(x)".to_string()),
                                tangent_point: Some([x0, f0]),
                                tangent_slope: Some(df0),
                                ..Default::default()
                            }
                        }),
                        remediate_module_id: Some("alg-exp-log".to_string()),
                    }
                } else {
                    let k = rng.gen_range(2..=6i32);
                    let p = rng.gen_range(2..=4i32);
                    let expected = format!("({}*x^{}) / (1 + {}*x^{})", k * p, p - 1, k, p);
                    let x_pow_minus = if p == 2 {
                        "x".to_string()
                    } else {
                        format!(r"x^{{{}}}", p - 1)
                    };
                    let target_tex = format!(r"\frac{{d}}{{dx}}\!\left[\ln\!\left(1 + {}x^{{{}}}\right)\right]", k, p);
                    let ans_tex = format!(r"\frac{{{}{}}}{{1 + {}x^{{{}}}}}", k * p, x_pow_minus, k, p);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::Differentiation,
                        title: "Logarithmic Chain Rule".to_string(),
                        prompt: format!("Differentiate with respect to x: f(x) = ln(1 + {}*x^{}).", k, p),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            "Recall d/dx [ln(g(x))] = g'(x) / g(x).".to_string(),
                            format!("Here g(x) = 1 + {}*x^{}, so g'(x) = {}*x^{}.", k, p, k * p, p - 1),
                            format!("Quotient: {}.", expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Inner derivative g'(x) = d/dx [1 + {}*x^{}] = {}*x^{}.", k, p, k * p, p - 1),
                            format!("2. Divide by inner function: {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Power Rule on Inner Polynomial",
                                r"g(x) = 1 + k x^{p} \implies g'(x) = k p\, x^{p-1}",
                                format!("Differentiate g(x) = 1 + {}x^{}.", k, p),
                                format!(r"\frac{{d}}{{dx}}\!\left[1 + {}x^{{{}}}\right]", k, p),
                                format!(r"g'(x) = {}{}", k * p, x_pow_minus),
                                format!("{}*x^{}", k * p, p - 1),
                            ),
                            raw_step(
                                "Logarithmic Chain Derivative",
                                r"\frac{d}{dx}\left[\ln g(x)\right] = \frac{g'(x)}{g(x)}",
                                "Divide inner derivative by inner polynomial g(x).",
                                target_tex,
                                ans_tex,
                                expected.clone(),
                            ),
                        ],
                        visual: Some({
                            let x0 = 1.0_f64;
                            let f0 = (1.0 + (k as f64)).ln();
                            let df0 = ((k * p) as f64) / (1.0 + (k as f64));
                            WalkthroughVisual {
                                var: "x".to_string(),
                                x_range: Some([-0.8, 2.5]),
                                primary_expr: Some(format!("ln(1 + {}*x^{})", k, p)),
                                primary_label: Some("f(x)".to_string()),
                                secondary_expr: Some(expected),
                                secondary_label: Some("f'(x)".to_string()),
                                tangent_point: Some([x0, f0]),
                                tangent_slope: Some(df0),
                                ..Default::default()
                            }
                        }),
                        remediate_module_id: Some("alg-exp-log".to_string()),
                    }
                }
            }

            "calc-implicit-opt" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(2..=5i32);
                let x0 = rng.gen_range(1..=3i32);
                let y0 = rng.gen_range(1..=3i32);
                let rhs = a * x0 * x0 + b * y0 * y0;
                let num = -a * x0;
                let den = b * y0;
                let expected = (num as f64) / (den as f64);
                let disp = format!("{}/{}", num, den);
                let curve_tex = format!(r"{}x^{{2}} + {}y^{{2}} = {}", a, b, rhs);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::ImplicitDifferentiation,
                    title: "Implicit Tangent Slope on an Ellipse".to_string(),
                    prompt: format!(
                        "Find the exact slope dy/dx of the curve {}*x^2 + {}*y^2 = {} at the point ({}, {}).",
                        a, b, rhs, x0, y0
                    ),
                    target_equation_latex: format!(
                        r"{},\qquad \left.\frac{{dy}}{{dx}}\right|_{{({},\,{})}}",
                        curve_tex, x0, y0
                    ),
                    answer_latex: format!(r"-\frac{{{}}}{{{}}}", -num, den),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-5,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Differentiate both sides wrt x: {}*x + {}*y*(dy/dx) = 0.", 2 * a, 2 * b),
                        format!("Solve for dy/dx = -({}*x) / ({}*y).", a, b),
                        format!("Substitute (x, y) = ({}, {}): {}.", x0, y0, disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Implicit differentiation: {}*x + {}*y*(dy/dx) = 0.", 2 * a, 2 * b),
                        format!("2. Isolate dy/dx = -({}*x)/({}*y).", a, b),
                        format!("3. At ({}, {}): dy/dx = {}.", x0, y0, disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Implicit Chain Differentiation wrt x",
                            r"\frac{d}{dx}\left[y(x)^{2}\right] = 2y\frac{dy}{dx}",
                            "Differentiate each term with respect to x, treating y as y(x).",
                            curve_tex,
                            format!(r"{}x + {}y\frac{{dy}}{{dx}} = 0", 2 * a, 2 * b),
                            format!("{}*x + {}*y*dy/dx = 0", 2 * a, 2 * b),
                        ),
                        raw_step(
                            "Isolate Slope Derivative dy/dx",
                            r"\frac{dy}{dx} = -\frac{ax}{by}",
                            "Move x-term to RHS and divide by 2by.",
                            format!(r"{}x + {}y\frac{{dy}}{{dx}} = 0", 2 * a, 2 * b),
                            format!(r"\frac{{dy}}{{dx}} = -\frac{{{}x}}{{{}y}}", a, b),
                            format!("-({}*x)/({}*y)", a, b),
                        ),
                        raw_step(
                            "Evaluate at Curve Point",
                            r"\left.\frac{dy}{dx}\right|_{(x_0, y_0)}",
                            format!("Substitute x = {}, y = {}.", x0, y0),
                            format!(r"\left.-\frac{{{}x}}{{{}y}}\right|_{{({},\,{})}}", a, b, x0, y0),
                            format!(r"-\frac{{{}}}{{{}}}", -num, den),
                            disp.clone(),
                        ),
                    ],
                    visual: Some({
                        let x_max = ((rhs as f64) / (a as f64)).sqrt();
                        WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-x_max - 0.5, x_max + 0.5]),
                            primary_expr: Some(format!("sqrt(({} - {}*x^2)/{})", rhs, a, b)),
                            primary_label: Some("Upper ellipse branch y+(x)".to_string()),
                            secondary_expr: Some(format!("-sqrt(({} - {}*x^2)/{})", rhs, a, b)),
                            secondary_label: Some("Lower ellipse branch y-(x)".to_string()),
                            tangent_point: Some([x0 as f64, y0 as f64]),
                            tangent_slope: Some(expected),
                            ..Default::default()
                        }
                    }),
                    remediate_module_id: Some("calc-derivatives-rules".to_string()),
                }
            }

            "calc-integrals-ftc" => {
                let a = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=5i32);
                let upper = rng.gen_range(2..=4i32);
                let val = a * upper.pow(3) + b * upper.pow(2);
                let integrand_tex = format!(r"{}x^{{2}} + {}x", 3 * a, 2 * b);
                let antideriv_tex = format!(r"{}x^{{3}} + {}x^{{2}}", a, b);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::DefiniteIntegral,
                    title: "Definite Integral via Fundamental Theorem".to_string(),
                    prompt: format!(
                        "Evaluate the definite integral: integral_{{0..{}}} ({}*x^2 + {}*x) dx.",
                        upper, 3 * a, 2 * b
                    ),
                    target_equation_latex: format!(r"\int_{{0}}^{{{}}} \left({}\right) dx", upper, integrand_tex),
                    answer_latex: format!("{}", val),
                    answer: AnswerFormat::Numeric {
                        expected: val as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", val),
                    },
                    hints: vec![
                        format!("Find an antiderivative F(x) of {}*x^2 + {}*x using power rule.", 3 * a, 2 * b),
                        format!("F(x) = {}*x^3 + {}*x^2.", a, b),
                        format!("Apply FTC Part 2: F({}) - F(0) = {}*{}^3 + {}*{}^2 = {}.", upper, a, upper, b, upper, val),
                    ],
                    derivation_steps: vec![
                        format!("1. Antiderivative: F(x) = {}*x^3 + {}*x^2.", a, b),
                        format!("2. Boundary evaluation: F({}) - F(0) = {} + {} - 0 = {}.", upper, a * upper.pow(3), b * upper.pow(2), val),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Power-Rule Antiderivative",
                            r"\int x^{n}\,dx = \frac{x^{n+1}}{n+1} + C",
                            "Construct F(x) via inverse power rule.",
                            format!(r"\int \left({}\right) dx", integrand_tex),
                            format!(r"F(x) = {}", antideriv_tex),
                            format!("{}*x^3 + {}*x^2", a, b),
                        ),
                        raw_step(
                            "Fundamental Theorem Part 2",
                            r"\int_{a}^{b} f(x)\,dx = F(b) - F(a)",
                            format!("Evaluate F({}) - F(0) = {} + {} - 0.", upper, a * upper.pow(3), b * upper.pow(2)),
                            format!(r"\left[{}\right]_{{0}}^{{{}}}", antideriv_tex, upper),
                            format!(r"{}\cdot{}^{{3}} + {}\cdot{}^{{2}} - 0 = {}", a, upper, b, upper, val),
                            format!("{}", val),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "x".to_string(),
                        x_range: Some([-0.5, upper as f64 + 0.75]),
                        primary_expr: Some(format!("{}*x^2 + {}*x", 3 * a, 2 * b)),
                        primary_label: Some(format!("f(x) = {}x^2 + {}x", 3 * a, 2 * b)),
                        secondary_expr: Some(format!("{}*x^3 + {}*x^2", a, b)),
                        secondary_label: Some(format!("F(x) = {}x^3 + {}x^2", a, b)),
                        shade_interval: Some([0.0, upper as f64]),
                        ..Default::default()
                    }),
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "calc-integration-tech" => {
                if variant == 0 {
                    let k = rng.gen_range(2..=5i32);
                    let expected = ((k as f64).exp() - 1.0) / (k as f64);
                    let disp = format!("(exp({}) - 1)/{}", k, k);
                    let target_tex = format!(r"\int_{{0}}^{{1}} 2x\, e^{{{}x^{{2}}}}\, dx", k);
                    let ans_tex = format!(r"\frac{{e^{{{}}} - 1}}{{{}}}", k, k);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::DefiniteIntegral,
                        title: "Definite Integral by u-Substitution".to_string(),
                        prompt: format!(
                            "Evaluate exactly: integral_{{0..1}} 2*x * exp({}*x^2) dx. (You may enter exact form in terms of exp(...) or e.)",
                            k
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: ans_tex.clone(),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-4,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Let u = {}*x^2 so du = {}*x dx, meaning 2*x dx = (1/{}) du.", k, 2 * k, k),
                            format!("Transform bounds: x = 0 -> u = 0, and x = 1 -> u = {}.", k),
                            format!("Integrate (1/{}) * integral_{{0..{}}} exp(u) du = {}.", k, k, disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Substitution u = {}*x^2 gives 2*x dx = du/{}.", k, k),
                            format!("2. Transformed integral: (1/{}) * [exp(u)]_{{0..{}}}.", k, k),
                            format!("3. Exact value: {}.", disp),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Quadratic u-Substitution",
                                r"u = k x^{2} \implies du = 2k x\,dx \implies 2x\,dx = \frac{du}{k}",
                                format!("Set u = {}x^2 and map bounds x in [0, 1] -> u in [0, {}].", k, k),
                                target_tex,
                                format!(r"\frac{{1}}{{{}}} \int_{{0}}^{{{}}} e^{{u}}\, du", k, k),
                                format!("(1/{}) * exp(u)", k),
                            ),
                            raw_step(
                                "Integrate Exponential Mode",
                                r"\int e^{u}\,du = e^{u}",
                                "Apply exponential antiderivative across u-bounds.",
                                format!(r"\frac{{1}}{{{}}} \int_{{0}}^{{{}}} e^{{u}}\, du", k, k),
                                format!(r"\frac{{1}}{{{}}} \left[e^{{u}}\right]_{{0}}^{{{}}}", k, k),
                                format!("(1/{})*[exp(u)]_{{0..{}}}", k, k),
                            ),
                            raw_step(
                                "Evaluate Transformed Limits",
                                r"\frac{e^{u_{\max}} - e^{0}}{k}",
                                format!("Note e^0 = 1 -> {}.", disp),
                                format!(r"\frac{{e^{{{}}} - e^{{0}}}}{{{}}}", k, k),
                                ans_tex,
                                disp.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-0.25, 1.35]),
                            primary_expr: Some(format!("2*x*exp({}*x^2)", k)),
                            primary_label: Some(format!("f(x) = 2x e^({}x^2)", k)),
                            shade_interval: Some([0.0, 1.0]),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                } else {
                    let a = rng.gen_range(1..=4i32);
                    let expected = format!("exp({}*x) * ({}*x - 1) / {}", a, a, a * a);
                    let target_tex = format!(r"\int x e^{{{}x}}\, dx \quad (C = 0)", a);
                    let ans_tex = plain_to_latex_or(&expected);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::IndefiniteIntegral,
                        title: "Integration by Parts Antiderivative".to_string(),
                        prompt: format!(
                            "Find the antiderivative F(x) of f(x) = x * exp({}*x) satisfying F(0) = -1/{} (i.e. constant C = 0).",
                            a, a * a
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: format!(r"F(x) = {}", ans_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Choose u = x (so du = dx) and dv = exp({}*x) dx (so v = exp({}*x)/{}).", a, a, a),
                            format!("Apply u*v - integral v du = x*exp({}*x)/{} - (1/{}) * integral exp({}*x) dx.", a, a, a, a),
                            format!("Integrate remaining exponential to get {}.", expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Set u = x, dv = exp({}*x) dx => du = dx, v = exp({}*x)/{}.", a, a, a),
                            format!("2. IBP: (x/{})*exp({}*x) - (1/{})*exp({}*x) + C.", a, a, a * a, a),
                            format!("3. With C = 0: {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Assign Integration by Parts Pairs",
                                r"\int u\,dv = uv - \int v\,du",
                                format!("Choose u = x (du = dx) and dv = exp({}*x) dx (v = exp({}*x)/{}).", a, a, a),
                                target_tex,
                                format!(r"\frac{{x e^{{{}x}}}}{{{}}} - \frac{{1}}{{{}}} \int e^{{{}x}}\, dx", a, a, a, a),
                                format!("(x/{})*exp({}*x) - (1/{})*integral exp({}*x) dx", a, a, a, a),
                            ),
                            raw_step(
                                "Integrate Exponential Remainder",
                                r"\int e^{ax}\,dx = \frac{1}{a}e^{ax}",
                                format!("Integrate remaining exp({}*x) factor to add denominator {}.", a, a * a),
                                format!(r"\frac{{x e^{{{}x}}}}{{{}}} - \frac{{1}}{{{}}} \int e^{{{}x}}\, dx", a, a, a, a),
                                format!(r"\frac{{x e^{{{}x}}}}{{{}}} - \frac{{e^{{{}x}}}}{{{}}}", a, a, a, a * a),
                                format!("(x/{})*exp({}*x) - (1/{})*exp({}*x)", a, a, a * a, a),
                            ),
                            raw_step(
                                "Factor Common Exponential Term",
                                r"F(x) = \frac{e^{ax}(ax - 1)}{a^{2}}",
                                format!("Combine over common denominator {} (matching F(0) = -1/{}).", a * a, a * a),
                                format!(r"\frac{{x e^{{{}x}}}}{{{}}} - \frac{{e^{{{}x}}}}{{{}}}", a, a, a, a * a),
                                ans_tex,
                                expected.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([-1.5, 1.5]),
                            primary_expr: Some(format!("x*exp({}*x)", a)),
                            primary_label: Some(format!("f(x) = x e^({}x)", a)),
                            secondary_expr: Some(expected),
                            secondary_label: Some("F(x) antiderivative".to_string()),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                }
            }

            "calc-series-odes" => {
                let k = rng.gen_range(2..=4i32);
                let k2 = k * k;
                let k3 = k * k * k;
                let expected = format!("1 + {}*x + ({}/2)*x^2 + ({}/6)*x^3", k, k2, k3);
                let ans_tex = format!(
                    r"1 + {}x + \frac{{{}}}{{2}}x^{{2}} + \frac{{{}}}{{6}}x^{{3}}",
                    k, k2, k3
                );
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::TaylorMaclaurinSeries,
                    title: "Cubic Maclaurin Polynomial Approximation".to_string(),
                    prompt: format!(
                        "Write the degree-3 Maclaurin polynomial T_3(x) centered at x = 0 for f(x) = exp({}*x).",
                        k
                    ),
                    target_equation_latex: format!(r"T_{{3}}(x) \text{{ for }} f(x) = e^{{{}x}}\quad (x_0 = 0)", k),
                    answer_latex: format!(r"T_{{3}}(x) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        "Recall exp(u) = 1 + u + u^2/2 + u^3/6 + O(u^4).".to_string(),
                        format!("Substitute u = {}*x into each power up to degree 3.", k),
                        format!("Expand powers ({}*x)^2 = {}*x^2 and ({}*x)^3 = {}*x^3: {}.", k, k2, k, k3, expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Derivatives at 0: f(0)=1, f'(0)={}, f''(0)={}, f'''(0)={}.", k, k2, k3),
                        format!("2. Divide by 0!, 1!, 2!, 3!: T_3(x) = {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Evaluate Derivatives at Origin",
                            r"f^{(n)}(0) = k^{n} \quad \text{for } f(x) = e^{kx}",
                            format!("Compute f(0)=1, f'(0)={}, f''(0)={}, f'''(0)={}.", k, k2, k3),
                            format!(r"f(x) = e^{{{}x}}", k),
                            format!(r"f(0)=1,\; f'(0)={},\; f''(0)={},\; f'''(0)={}", k, k2, k3),
                            format!("[1, {}, {}, {}]", k, k2, k3),
                        ),
                        raw_step(
                            "Assemble Taylor Polynomial Weights",
                            r"T_{3}(x) = \sum_{n=0}^{3} \frac{f^{(n)}(0)}{n!}\, x^{n}",
                            "Divide derivatives by 0!, 1!, 2!, 3! and simplify powers.",
                            format!(r"1 + {}x + \frac{{({}x)^{{2}}}}{{2!}} + \frac{{({}x)^{{3}}}}{{3!}}", k, k, k),
                            ans_tex,
                            expected.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "x".to_string(),
                        x_range: Some([-1.2, 1.2]),
                        primary_expr: Some(format!("exp({}*x)", k)),
                        primary_label: Some(format!("f(x) = e^({}x)", k)),
                        secondary_expr: Some(expected),
                        secondary_label: Some("T_3(x) Maclaurin".to_string()),
                        ..Default::default()
                    }),
                    remediate_module_id: Some("alg-sequences-series".to_string()),
                }
            }

            // =====================================================================
            // STAGE 4: ADVANCED & MULTIVARIABLE CALCULUS
            // =====================================================================
            "adv-real-analysis" => {
                let m = rng.gen_range(2..=6i32);
                let b = rng.gen_range(-5..=5i32);
                let eps_num = rng.gen_range(1..=5i32);
                let expected = (eps_num as f64) / (10.0 * m as f64);
                let disp = format!("{}/{}", eps_num, 10 * m);
                let eps_dec = (eps_num as f64) / 10.0;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::EpsilonDeltaProof,
                    title: "Exact Epsilon-Delta Continuity Threshold".to_string(),
                    prompt: format!(
                        "For f(x) = {}*x + ({}) near x0 = 2 with target tolerance epsilon = {:.1}, find the largest delta > 0 such that |x - 2| < delta guarantees |f(x) - f(2)| < {:.1}.",
                        m, b, eps_dec, eps_dec
                    ),
                    target_equation_latex: format!(
                        r"f(x) = {}x {},\quad |x - 2| < \delta \implies |f(x) - f(2)| < {:.1}",
                        m, signed_i(b), eps_dec
                    ),
                    answer_latex: format!(r"\delta = \frac{{{}}}{{{}}}", eps_num, 10 * m),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-5,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Compute |f(x) - f(2)| = |({}*x + ({})) - ({}*2 + ({}))| = {} * |x - 2|.", m, b, m, b, m),
                        format!("Set {} * |x - 2| < {}/10 and isolate |x - 2|.", m, eps_num),
                        format!("Largest valid threshold is delta = {}.", disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Error relation: |f(x) - L| = {} * |x - 2|.", m),
                        format!("2. Require {} * |x - 2| < {}/10 <=> |x - 2| < {}.", m, eps_num, disp),
                        format!("3. Optimal uniform delta = {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Factor Linear Lipschitz Bound",
                            r"|f(x) - f(x_0)| = |(mx + b) - (mx_0 + b)| = |m|\,|x - x_0|",
                            format!("Express output deviation in terms of input distance |x - 2| with |m| = {}.", m),
                            format!(r"\left|({}x {}) - ({}\cdot 2 {})\right|", m, signed_i(b), m, signed_i(b)),
                            format!(r"{}\,|x - 2|", m),
                            format!("{}*|x - 2|", m),
                        ),
                        raw_step(
                            "Impose Epsilon Target Inequality",
                            r"|m|\,|x - x_0| < \varepsilon \iff |x - x_0| < \frac{\varepsilon}{|m|}",
                            format!("Require {}*|x - 2| < {}/10.", m, eps_num),
                            format!(r"{}\,|x - 2| < \frac{{{}}}{{10}}", m, eps_num),
                            format!(r"|x - 2| < \frac{{{}}}{{{}}}", eps_num, 10 * m),
                            format!("|x - 2| < {}", disp),
                        ),
                        raw_step(
                            "Pin Maximal Uniform Delta",
                            r"\delta_{\max} = \frac{\varepsilon}{|m|}",
                            format!("Largest threshold guaranteeing |f(x) - f(2)| < {:.1} is {}.", eps_dec, disp),
                            format!(r"|x - 2| < \frac{{{}}}{{{}}}", eps_num, 10 * m),
                            format!(r"\delta = \frac{{{}}}{{{}}}", eps_num, 10 * m),
                            disp.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "x".to_string(),
                        x_range: Some([0.5, 3.5]),
                        primary_expr: Some(format!("{}*x + ({})", m, b)),
                        primary_label: Some(format!("f(x) = {}x + ({})", m, b)),
                        highlight_points: vec![[2.0, (2 * m + b) as f64]],
                        vline_x: Some(2.0),
                        vline_label: Some("x0 = 2".to_string()),
                        ..Default::default()
                    }),
                    remediate_module_id: Some("calc-limits-continuity".to_string()),
                }
            }

            "adv-partial-grad" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(1..=4i32);
                let p = format!("{}*x*y", 2 * a);
                let q = format!("{}*x^2 + {}*y^2", a, 3 * b);
                let canon = format!("<{}, {}>", p, q);
                let f_tex = format!(r"{}x^{{2}}y + {}y^{{3}}", a, b);
                let px_tex = format!(r"{}xy", 2 * a);
                let py_tex = format!(r"{}x^{{2}} + {}y^{{2}}", a, 3 * b);
                let grad_tex = format!(r"\left\langle {},\; {} \right\rangle", px_tex, py_tex);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::GradientVectorField,
                    title: "Symbolic Gradient Vector Field".to_string(),
                    prompt: format!(
                        "Compute the gradient vector grad f(x, y) = <df/dx, df/dy> for f(x, y) = {}*x^2*y + {}*y^3.",
                        a, b
                    ),
                    target_equation_latex: format!(r"\nabla f(x,y),\qquad f(x,y) = {}", f_tex),
                    answer_latex: format!(r"\nabla f(x,y) = {}", grad_tex),
                    answer: AnswerFormat::Vector2Expr {
                        expected_p: p.clone(),
                        expected_q: q.clone(),
                        vars: vec!["x".to_string(), "y".to_string()],
                        canonical_display: canon.clone(),
                    },
                    hints: vec![
                        format!("Hold y constant to differentiate wrt x: df/dx = {}.", p),
                        format!("Hold x constant to differentiate wrt y: df/dy = {}.", q),
                        format!("Format as a 2-component vector: {}.", canon),
                    ],
                    derivation_steps: vec![
                        format!("1. Partial wrt x (y fixed): d/dx [{}*x^2*y + {}*y^3] = {}.", a, b, p),
                        format!("2. Partial wrt y (x fixed): d/dy [{}*x^2*y + {}*y^3] = {}.", a, b, q),
                        format!("3. Gradient vector: {}.", canon),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Partial Derivative wrt x",
                            r"\frac{\partial f}{\partial x} \quad (y \text{ held constant})",
                            "Differentiate treating y and y^3 as constants.",
                            format!(r"\frac{{\partial}}{{\partial x}}\!\left({}\right)", f_tex),
                            format!(r"f_x = {}", px_tex),
                            p.clone(),
                        ),
                        raw_step(
                            "Partial Derivative wrt y",
                            r"\frac{\partial f}{\partial y} \quad (x \text{ held constant})",
                            "Differentiate treating x^2 as constant.",
                            format!(r"\frac{{\partial}}{{\partial y}}\!\left({}\right)", f_tex),
                            format!(r"f_y = {}", py_tex),
                            q.clone(),
                        ),
                        raw_step(
                            "Assemble Gradient Vector",
                            r"\nabla f = \left\langle \frac{\partial f}{\partial x},\, \frac{\partial f}{\partial y} \right\rangle",
                            "Package partial derivatives into the steepest-ascent vector.",
                            format!(r"f_x = {},\; f_y = {}", px_tex, py_tex),
                            grad_tex,
                            canon.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("calc-derivatives-rules".to_string()),
                }
            }

            "adv-lagrange-hessian" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(2..=5i32);
                let c = rng.gen_range(1..=3i32);
                let det = 4 * a * b - 4 * c * c;
                let f_tex = format!(r"{}x^{{2}} + {}y^{{2}} + {}xy", a, b, 2 * c);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::HessianDeterminant,
                    title: "2D Hessian Discriminant Calculation".to_string(),
                    prompt: format!(
                        "Compute the Hessian determinant D = f_xx * f_yy - (f_xy)^2 at (0, 0) for f(x, y) = {}*x^2 + {}*y^2 + {}*x*y.",
                        a, b, 2 * c
                    ),
                    target_equation_latex: format!(r"f(x,y) = {},\quad D = f_{{xx}}f_{{yy}} - f_{{xy}}^{{2}}", f_tex),
                    answer_latex: format!(r"D = {}", det),
                    answer: AnswerFormat::Numeric {
                        expected: det as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", det),
                    },
                    hints: vec![
                        format!("Compute second partials: f_xx = {}, f_yy = {}, and mixed partial f_xy = {}.", 2 * a, 2 * b, 2 * c),
                        format!("Evaluate D = ({})*({}) - ({})^2.", 2 * a, 2 * b, 2 * c),
                        format!("D = {} - {} = {}.", 4 * a * b, 4 * c * c, det),
                    ],
                    derivation_steps: vec![
                        format!("1. First partials: f_x = {}*x + {}*y, f_y = {}*y + {}*x.", 2 * a, 2 * c, 2 * b, 2 * c),
                        format!("2. Second partials: f_xx = {}, f_yy = {}, f_xy = {}.", 2 * a, 2 * b, 2 * c),
                        format!("3. Determinant D = ({})*({}) - ({})^2 = {}.", 2 * a, 2 * b, 2 * c, det),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "First Partial Derivatives",
                            r"\nabla f = \langle f_x,\, f_y \rangle",
                            "Compute linear first-order partial derivatives.",
                            format!(r"f(x,y) = {}", f_tex),
                            format!(r"f_x = {}x + {}y,\quad f_y = {}y + {}x", 2 * a, 2 * c, 2 * b, 2 * c),
                            format!("<{}*x + {}*y, {}*y + {}*x>", 2 * a, 2 * c, 2 * b, 2 * c),
                        ),
                        raw_step(
                            "Second & Mixed Hessian Entries",
                            r"H = \begin{bmatrix} f_{xx} & f_{xy} \\ f_{xy} & f_{yy} \end{bmatrix}",
                            format!("Differentiate again: f_xx = {}, f_yy = {}, f_xy = {}.", 2 * a, 2 * b, 2 * c),
                            format!(r"f_x = {}x + {}y,\; f_y = {}y + {}x", 2 * a, 2 * c, 2 * b, 2 * c),
                            format!(r"H = \begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}}", 2 * a, 2 * c, 2 * c, 2 * b),
                            format!("[[{}, {}], [{}, {}]]", 2 * a, 2 * c, 2 * c, 2 * b),
                        ),
                        raw_step(
                            "Evaluate Hessian Determinant D",
                            r"D = f_{xx}f_{yy} - (f_{xy})^{2}",
                            format!("Compute ({})*({}) - ({})^2 = {} - {}.", 2 * a, 2 * b, 2 * c, 4 * a * b, 4 * c * c),
                            format!(r"{}\cdot {} - ({})^{{2}}", 2 * a, 2 * b, 2 * c),
                            format!(r"D = {}", det),
                            format!("{}", det),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("adv-partial-grad".to_string()),
                }
            }

            "adv-multiple-integrals" => {
                let rad = rng.gen_range(2..=4i32);
                let k = rng.gen_range(1..=3i32);
                let r4 = rad.pow(4);
                let expected = 0.5 * (k as f64) * (r4 as f64) * std::f64::consts::PI;
                let disp = format!("({}*pi)/2", k * r4);
                let target_tex = format!(
                    r"\iint_{{x^{{2}}+y^{{2}} \le {}}} {}\left(x^{{2}} + y^{{2}}\right) dA",
                    rad * rad, k
                );
                let ans_tex = format!(r"\frac{{{}\pi}}{{2}}", k * r4);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::MultipleIntegralPolar,
                    title: "Polar Double Integral Over a Disk".to_string(),
                    prompt: format!(
                        "Evaluate double_integral_D {}*(x^2 + y^2) dA over the disk D = {{ x^2 + y^2 <= {} }}. (Include the polar Jacobian r and express in terms of pi.)",
                        k, rad * rad
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: ans_tex.clone(),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Convert to polar: x^2 + y^2 = r^2 and dA = r dr dtheta over 0 <= r <= {}, 0 <= theta <= 2*pi.", rad),
                        format!("Integrand becomes {}*r^2 * r = {}*r^3.", k, k),
                        format!("Evaluate (2*pi) * [{} * r^4 / 4]_{{0..{}}} = {}.", k, rad, disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Polar setup: integral_{{0..2*pi}} integral_{{0..{}}} {}*r^3 dr dtheta.", rad, k),
                        format!("2. Radial integral: {} * {}^4 / 4 = {}/4.", k, rad, k * r4),
                        format!("3. Multiply by angular factor 2*pi: {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Polar Transformation & Jacobian",
                            r"x^{2} + y^{2} = r^{2},\quad dA = r\,dr\,d\theta",
                            format!("Transform integrand and bounds to 0 <= r <= {}, 0 <= theta <= 2*pi.", rad),
                            target_tex,
                            format!(r"\int_{{0}}^{{2\pi}} \int_{{0}}^{{{}}} {}r^{{3}}\, dr\, d\theta", rad, k),
                            format!("integral_{{0..2*pi}} integral_{{0..{}}} {}*r^3 dr dtheta", rad, k),
                        ),
                        raw_step(
                            "Integrate Radial Power r^3",
                            r"\int_{0}^{R} k r^{3}\,dr = \frac{k R^{4}}{4}",
                            format!("Integrate r^3 from 0 to {} giving {}/4.", rad, k * r4),
                            format!(r"\int_{{0}}^{{{}}} {}r^{{3}}\, dr", rad, k),
                            format!(r"\left[\frac{{{}r^{{4}}}}{{4}}\right]_{{0}}^{{{}}} = \frac{{{}}}{{4}}", k, rad, k * r4),
                            format!("{}/4", k * r4),
                        ),
                        raw_step(
                            "Integrate Angular Sector [0, 2pi]",
                            r"\int_{0}^{2\pi} C\,d\theta = 2\pi C",
                            format!("Multiply {}/4 by 2*pi.", k * r4),
                            format!(r"2\pi \cdot \frac{{{}}}{{4}}", k * r4),
                            ans_tex,
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("trig-polar-parametric".to_string()),
                }
            }

            "adv-vector-fields" => {
                let a = rng.gen_range(2..=6i32);
                let b = rng.gen_range(1..=5i32);
                let coeff = 2 * (a + b);
                let expected = format!("{}*x*y", coeff);
                let f_tex = format!(r"\left\langle {}x^{{2}}y,\; {}xy^{{2}} \right\rangle", a, b);
                let ans_tex = format!(r"{}xy", coeff);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::VectorDivergence,
                    title: "Divergence of a 2D Vector Field".to_string(),
                    prompt: format!(
                        "Compute the scalar divergence div F(x, y) = dP/dx + dQ/dy for F(x, y) = <{}*x^2*y, {}*x*y^2>.",
                        a, b
                    ),
                    target_equation_latex: format!(r"\nabla \cdot \mathbf{{F}},\qquad \mathbf{{F}}(x,y) = {}", f_tex),
                    answer_latex: format!(r"\nabla \cdot \mathbf{{F}} = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string(), "y".to_string()],
                    },
                    hints: vec![
                        format!("Compute dP/dx = d/dx [{}*x^2*y] = {}*x*y.", a, 2 * a),
                        format!("Compute dQ/dy = d/dy [{}*x*y^2] = {}*x*y.", b, 2 * b),
                        format!("Sum dP/dx + dQ/dy = {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. dP/dx = {}*x*y.", 2 * a),
                        format!("2. dQ/dy = {}*x*y.", 2 * b),
                        format!("3. div F = ({} + {})*x*y = {}.", 2 * a, 2 * b, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Horizontal Flux Partial dP/dx",
                            r"\frac{\partial P}{\partial x}",
                            format!("Differentiate P = {}x^2*y with respect to x.", a),
                            format!(r"\frac{{\partial}}{{\partial x}}\!\left({}x^{{2}}y\right)", a),
                            format!(r"{}xy", 2 * a),
                            format!("{}*x*y", 2 * a),
                        ),
                        raw_step(
                            "Vertical Flux Partial dQ/dy",
                            r"\frac{\partial Q}{\partial y}",
                            format!("Differentiate Q = {}x*y^2 with respect to y.", b),
                            format!(r"\frac{{\partial}}{{\partial y}}\!\left({}xy^{{2}}\right)", b),
                            format!(r"{}xy", 2 * b),
                            format!("{}*x*y", 2 * b),
                        ),
                        raw_step(
                            "Sum Scalar Divergence",
                            r"\nabla \cdot \mathbf{F} = \frac{\partial P}{\partial x} + \frac{\partial Q}{\partial y}",
                            format!("Add {}xy + {}xy.", 2 * a, 2 * b),
                            format!(r"{}xy + {}xy", 2 * a, 2 * b),
                            ans_tex,
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("adv-partial-grad".to_string()),
                }
            }

            "adv-stokes-fourier" => {
                let p_c = rng.gen_range(1..=4i32);
                let q_c = rng.gen_range(2..=6i32);
                let rad = rng.gen_range(1..=3i32);
                let factor = (p_c + q_c) * rad * rad;
                let expected = (factor as f64) * std::f64::consts::PI;
                let disp = format!("{}*pi", factor);
                let target_tex = format!(
                    r"\oint_{{x^{{2}}+y^{{2}}={}}} \left(-{}y\,dx + {}x\,dy\right)",
                    rad * rad, p_c, q_c
                );
                RawProblem {
                    module_id: "adv-stokes-fourier".to_string(),
                    seed,
                    difficulty: 4,
                    operation: OperationKind::LineIntegralCirculation,
                    title: "Counterclockwise Circulation via Green's Theorem".to_string(),
                    prompt: format!(
                        "Use Green's Theorem to evaluate the counterclockwise line integral oint_C (-{}*y dx + {}*x dy) around the circle x^2 + y^2 = {}. (Express in terms of pi.)",
                        p_c, q_c, rad * rad
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"{}\pi", factor),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Identify P(x, y) = -{}*y and Q(x, y) = {}*x.", p_c, q_c),
                        format!("Compute constant 2D curl: dQ/dx - dP/dy = {} - (-{}) = {}.", q_c, p_c, p_c + q_c),
                        format!("Multiply constant curl {} by disk area pi*{}^2 = {}.", p_c + q_c, rad, disp),
                    ],
                    derivation_steps: vec![
                        "1. By Green's Theorem, oint_C (P dx + Q dy) = double_integral_D (dQ/dx - dP/dy) dA.".to_string(),
                        format!("2. Local vorticity: dQ/dx - dP/dy = {} + {} = {}.", q_c, p_c, p_c + q_c),
                        format!("3. Area of disk (radius {}) is {}*pi, giving circulation = {}.", rad, rad * rad, disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Apply Green's Circulation Theorem",
                            r"\oint_{\partial D} (P\,dx + Q\,dy) = \iint_{D} \left(\frac{\partial Q}{\partial x} - \frac{\partial P}{\partial y}\right) dA",
                            format!("Identify P = -{}y and Q = {}x on disk of radius {}.", p_c, q_c, rad),
                            target_tex,
                            format!(r"\iint_{{D}} \left(\frac{{\partial}}{{\partial x}}({}x) - \frac{{\partial}}{{\partial y}}(-{}y)\right) dA", q_c, p_c),
                            format!("double_integral_D (dQ/dx - dP/dy) dA"),
                        ),
                        raw_step(
                            "Compute Constant 2D Curl",
                            r"\omega_z = \frac{\partial Q}{\partial x} - \frac{\partial P}{\partial y}",
                            format!("Vorticity simplifies to {} - (-{}) = {}.", q_c, p_c, p_c + q_c),
                            format!(r"\iint_{{D}} ({} - (-{}))\, dA", q_c, p_c),
                            format!(r"{} \iint_{{D}} dA", p_c + q_c),
                            format!("{} * Area(D)", p_c + q_c),
                        ),
                        raw_step(
                            "Multiply Curl by Disk Area",
                            r"\text{Area}(D) = \pi R^{2}",
                            format!("Multiply {} by pi*{}^2 = {}*pi.", p_c + q_c, rad, rad * rad),
                            format!(r"{} \cdot \left(\pi \cdot {}^{{2}}\right)", p_c + q_c, rad),
                            format!(r"{}\pi", factor),
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("adv-vector-fields".to_string()),
                }
            }

            // =====================================================================
            // BRIDGE STAGE: LINEAR ALGEBRA & EIGEN-SYSTEMS (6 Modules)
            // =====================================================================
            "lin-vectors-spans" => {
                let a = rng.gen_range(1..=3i32);
                let b = rng.gen_range(1..=3i32);
                let c1 = rng.gen_range(-4..=5i32);
                let c2 = rng.gen_range(-3..=4i32);
                let c2 = if c2 == 0 { 2 } else { c2 };
                let wx = c1 + c2 * b;
                let wy = c1 * a + c2 * (a * b + 1);
                let mat_sys = format!(
                    r"\begin{{bmatrix}} 1 & {} \\ {} & {} \end{{bmatrix}} \begin{{bmatrix}} c_1 \\ c_2 \end{{bmatrix}} = \begin{{bmatrix}} {} \\ {} \end{{bmatrix}}",
                    b, a, a * b + 1, wx, wy
                );
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::BasisDecomposition,
                    title: "Coordinate Weights in a 2D Basis".to_string(),
                    prompt: format!(
                        "Express w = <{}, {}> as a linear combination c1*u + c2*v of basis vectors u = <1, {}> and v = <{}, {}>. Find the exact scalar weight c1.",
                        wx, wy, a, b, a * b + 1
                    ),
                    target_equation_latex: format!(
                        r"\langle {},\, {} \rangle = c_1 \langle 1,\, {} \rangle + c_2 \langle {},\, {} \rangle",
                        wx, wy, a, b, a * b + 1
                    ),
                    answer_latex: format!(r"c_1 = {}", c1),
                    answer: AnswerFormat::Numeric {
                        expected: c1 as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", c1),
                    },
                    hints: vec![
                        format!("Write component equations: c1 + {}*c2 = {} and {}*c1 + {}*c2 = {}.", b, wx, a, a * b + 1, wy),
                        format!("Subtract {} times the first equation from the second to find c2 = {} - {}*{} = {}.", a, wy, a, wx, c2),
                        format!("Back-substitute c2 = {} into c1 = {} - {}*c2 = {}.", c2, wx, b, c1),
                    ],
                    derivation_steps: vec![
                        format!("1. System: [1, {}; {}, {}] * [c1; c2] = [{}; {}] (det = 1).", b, a, a * b + 1, wx, wy),
                        format!("2. Eliminate c1: c2 = {} - {}*({}) = {}.", wy, a, wx, c2),
                        format!("3. Solve for first basis weight: c1 = {} - {}*({}) = {}.", wx, b, c2, c1),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Assemble Basis Matrix System",
                            r"\begin{bmatrix}\mathbf{u} & \mathbf{v}\end{bmatrix}\begin{bmatrix}c_1\\c_2\end{bmatrix}=\mathbf{w}",
                            "Write coordinate decomposition in column-matrix form (det = 1).",
                            format!(r"\langle {},\, {} \rangle = c_1 \langle 1,\, {} \rangle + c_2 \langle {},\, {} \rangle", wx, wy, a, b, a * b + 1),
                            mat_sys.clone(),
                            format!("[[1, {}], [{}, {}]]*[c1;c2]=[{};{}]", b, a, a * b + 1, wx, wy),
                        ),
                        raw_step(
                            "Row Elimination for Weight c2",
                            r"R_2 \leftarrow R_2 - a R_1",
                            format!("Eliminate c1 by subtracting {} times Row 1 from Row 2.", a),
                            mat_sys,
                            format!(r"c_2 = {} - {}({}) = {}", wy, a, wx, c2),
                            format!("c2 = {}", c2),
                        ),
                        raw_step(
                            "Back-Substitute for Weight c1",
                            r"c_1 = w_x - b c_2",
                            format!("Substitute c2 = {} into first equation.", c2),
                            format!(r"c_1 + {}({}) = {}", b, c2, wx),
                            format!(r"c_1 = {} - {}({}) = {}", wx, b, c2, c1),
                            format!("{}", c1),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-linear-systems".to_string()),
                }
            }

            "lin-matrices-maps" => {
                let a11 = rng.gen_range(-3..=4i32);
                let a11 = if a11 == 0 { 2 } else { a11 };
                let a12 = rng.gen_range(1..=4i32);
                let a21 = rng.gen_range(-3..=3i32);
                let a21 = if a21 == 0 { -1 } else { a21 };
                let a22 = rng.gen_range(1..=4i32);
                let comp1 = format!("{}*x + {}*y", a11, a12);
                let comp2 = format!("{}*x + {}*y", a21, a22);
                let canon = format!("<{}, {}>", comp1, comp2);
                let c1_tex = format!(r"{}x + {}y", a11, a12);
                let c2_tex = format!(r"{}x + {}y", a21, a22);
                let target_tex = format!(
                    r"A\mathbf{{v}} = \begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}} \begin{{bmatrix}} x \\ y \end{{bmatrix}}",
                    a11, a12, a21, a22
                );
                let ans_tex = format!(r"\left\langle {},\; {} \right\rangle", c1_tex, c2_tex);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::LinearMapMatrix,
                    title: "Symbolic Linear Map Action A * v".to_string(),
                    prompt: format!(
                        "For matrix A = [[{}, {}], [{}, {}]] and input vector v = <x, y>, write the transformed output vector A * v = <P(x,y), Q(x,y)>.",
                        a11, a12, a21, a22
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: ans_tex.clone(),
                    answer: AnswerFormat::Vector2Expr {
                        expected_p: comp1.clone(),
                        expected_q: comp2.clone(),
                        vars: vec!["x".to_string(), "y".to_string()],
                        canonical_display: canon.clone(),
                    },
                    hints: vec![
                        format!("Column picture: A * <x, y> = x * <{}, {}> + y * <{}, {}>.", a11, a21, a12, a22),
                        format!("First component: {}. Second component: {}.", comp1, comp2),
                        format!("Assemble vector: {}.", canon),
                    ],
                    derivation_steps: vec![
                        format!("1. Row 1 dot <x, y>: {}.", comp1),
                        format!("2. Row 2 dot <x, y>: {}.", comp2),
                        format!("3. Output vector: {}.", canon),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Row 1 Dot Input Vector",
                            r"(A\mathbf{v})_1 = a_{11}x + a_{12}y",
                            "Take inner product of first row with <x, y>.",
                            format!(r"\begin{{bmatrix}} {} & {} \end{{bmatrix}} \begin{{bmatrix}} x \\ y \end{{bmatrix}}", a11, a12),
                            c1_tex.clone(),
                            comp1.clone(),
                        ),
                        raw_step(
                            "Row 2 Dot Input Vector",
                            r"(A\mathbf{v})_2 = a_{21}x + a_{22}y",
                            "Take inner product of second row with <x, y>.",
                            format!(r"\begin{{bmatrix}} {} & {} \end{{bmatrix}} \begin{{bmatrix}} x \\ y \end{{bmatrix}}", a21, a22),
                            c2_tex.clone(),
                            comp2.clone(),
                        ),
                        raw_step(
                            "Assemble Output Vector",
                            r"A\mathbf{v} = \langle P(x,y),\, Q(x,y) \rangle",
                            "Combine transformed scalar coordinates.",
                            target_tex,
                            ans_tex,
                            canon.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("lin-vectors-spans".to_string()),
                }
            }

            "lin-det-inverses" => {
                let a = rng.gen_range(2..=5i32);
                let d = rng.gen_range(2..=5i32);
                let b = rng.gen_range(1..=3i32);
                let c = rng.gen_range(1..=2i32);
                let det = a * d - b * c;
                let (d_adj, det_adj) = if det == 0 { (d + 1, a * (d + 1) - b * c) } else { (d, det) };
                let (num, den) = if det_adj > 0 {
                    (d_adj, det_adj)
                } else {
                    (-d_adj, -det_adj)
                };
                let expected = (num as f64) / (den as f64);
                let disp = format!("{}/{}", num, den);
                let mat_tex = format!(r"\begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}}", a, b, c, d_adj);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::MatrixInverse,
                    title: "2x2 Matrix Inverse Entry & Area Scaling".to_string(),
                    prompt: format!(
                        "Let A = [[{}, {}], [{}, {}]]. Compute the top-left (1, 1) entry of the inverse matrix A^-1.",
                        a, b, c, d_adj
                    ),
                    target_equation_latex: format!(r"A = {},\qquad (A^{{-1}})_{{1,1}}", mat_tex),
                    answer_latex: format!(r"\frac{{{}}}{{{}}}", num, den),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-5,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Compute det(A) = {}*{} - {}*{} = {}.", a, d_adj, b, c, det_adj),
                        format!("Recall [[a, b], [c, d]]^-1 = (1/det(A)) * [[d, -b], [-c, a]]."),
                        format!("Top-left entry is d / det(A) = {}.", disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Determinant: det(A) = {}*{} - {}*{} = {}.", a, d_adj, b, c, det_adj),
                        format!("2. Adjugate top-left cofactor is {}.", d_adj),
                        format!("3. (A^-1)_{{1,1}} = {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Compute 2x2 Matrix Determinant",
                            r"\det(A) = ad - bc",
                            format!("Evaluate signed area scaling factor: {}*{} - {}*{}.", a, d_adj, b, c),
                            format!(r"\det {}", mat_tex),
                            format!(r"\det(A) = {}", det_adj),
                            format!("{}", det_adj),
                        ),
                        raw_step(
                            "Form Adjugate Inverse Formula",
                            r"A^{-1} = \frac{1}{\det(A)}\begin{bmatrix}d & -b \\ -c & a\end{bmatrix}",
                            "Swap main diagonal entries and negate off-diagonals.",
                            format!(r"{}^{{-1}}", mat_tex),
                            format!(r"\frac{{1}}{{{}}}\begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}}", det_adj, d_adj, -b, -c, a),
                            format!("(1/{})*[[{},{}],[{},{}]]", det_adj, d_adj, -b, -c, a),
                        ),
                        raw_step(
                            "Extract (1, 1) Inverse Entry",
                            r"(A^{-1})_{1,1} = \frac{d}{ad - bc}",
                            "Divide top-left adjugate entry by det(A).",
                            format!(r"\frac{{{}}}{{{}}}", d_adj, det_adj),
                            format!(r"(A^{{-1}})_{{1,1}} = \frac{{{}}}{{{}}}", num, den),
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("lin-matrices-maps".to_string()),
                }
            }

            "lin-orthogonality-lsq" => {
                let u1 = rng.gen_range(1..=4i32);
                let u2 = rng.gen_range(1..=4i32);
                let y1 = rng.gen_range(-3..=6i32);
                let y2 = rng.gen_range(1..=6i32);
                let dot = y1 * u1 + y2 * u2;
                let norm_sq = u1 * u1 + u2 * u2;
                let expected = (dot as f64) / (norm_sq as f64);
                let disp = format!("{}/{}", dot, norm_sq);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::OrthogonalProjection,
                    title: "Orthogonal Projection Coefficient".to_string(),
                    prompt: format!(
                        "Inproj_u(y) = c * u, find the exact projection scalar c = (y . u) / (u . u) for y = <{}, {}> onto u = <{}, {}>.",
                        y1, y2, u1, u2
                    )
                    .replace("Inproj_u", "In proj_u"),
                    target_equation_latex: format!(
                        r"\mathbf{{y}} = \langle {},\, {} \rangle,\quad \mathbf{{u}} = \langle {},\, {} \rangle,\quad c = \frac{{\mathbf{{y}}\cdot\mathbf{{u}}}}{{\mathbf{{u}}\cdot\mathbf{{u}}}}",
                        y1, y2, u1, u2
                    ),
                    answer_latex: format!(r"c = \frac{{{}}}{{{}}}", dot, norm_sq),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-5,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Compute inner product y . u = ({})*{} + ({})*{} = {}.", y1, u1, y2, u2, dot),
                        format!("Compute squared length u . u = {}^2 + {}^2 = {}.", u1, u2, norm_sq),
                        format!("Ratio c = {}.", disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Numerator y . u = {}.", dot),
                        format!("2. Denominator u . u = {}.", norm_sq),
                        format!("3. Optimal least-squares scalar: c = {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Numerator Inner Product y . u",
                            r"\mathbf{y} \cdot \mathbf{u} = y_1 u_1 + y_2 u_2",
                            "Evaluate dot product between target y and direction u.",
                            format!(r"\langle {},\, {} \rangle \cdot \langle {},\, {} \rangle", y1, y2, u1, u2),
                            format!(r"\mathbf{{y}} \cdot \mathbf{{u}} = {}", dot),
                            format!("{}", dot),
                        ),
                        raw_step(
                            "Denominator Squared Norm ||u||^2",
                            r"\mathbf{u} \cdot \mathbf{u} = u_1^{2} + u_2^{2}",
                            "Evaluate squared Euclidean length of u.",
                            format!(r"{}^{{2}} + {}^{{2}}", u1, u2),
                            format!(r"\mathbf{{u}} \cdot \mathbf{{u}} = {}", norm_sq),
                            format!("{}", norm_sq),
                        ),
                        raw_step(
                            "Form Orthogonal Projection Scalar",
                            r"c = \frac{\mathbf{y} \cdot \mathbf{u}}{\left\|\mathbf{u}\right\|_2^{2}}",
                            "Divide inner product by squared length.",
                            format!(r"\frac{{\mathbf{{y}}\cdot\mathbf{{u}}}}{{\mathbf{{u}}\cdot\mathbf{{u}}}}"),
                            format!(r"c = \frac{{{}}}{{{}}}", dot, norm_sq),
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("trig-polar-parametric".to_string()),
                }
            }

            "lin-eigenvalues-diag" => {
                let l2 = rng.gen_range(-3..=2i32);
                let l1 = l2 + rng.gen_range(2..=5i32);
                let k = rng.gen_range(1..=3i32);
                let a11 = l1 - k;
                let a12 = k;
                let a21 = l1 - k - l2;
                let a22 = k + l2;
                let tr = a11 + a22;
                let det = a11 * a22 - a12 * a21;
                let mat_tex = format!(r"\begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}}", a11, a12, a21, a22);
                let char_tex = format!(r"\lambda^{{2}} {} \lambda {} = 0", signed_i(-tr), signed_i(det));
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::Eigenvalues,
                    title: "Dominant Eigenvalue of a 2x2 Matrix".to_string(),
                    prompt: format!(
                        "Find the larger real eigenvalue lambda_1 of A = [[{}, {}], [{}, {}]].",
                        a11, a12, a21, a22
                    ),
                    target_equation_latex: format!(r"A = {},\qquad \det(A - \lambda I) = 0", mat_tex),
                    answer_latex: format!(r"\lambda_1 = {}", l1),
                    answer: AnswerFormat::Numeric {
                        expected: l1 as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", l1),
                    },
                    hints: vec![
                        format!("Compute trace tr(A) = {} + ({}) = {} and det(A) = ({})*({}) - ({})*({}) = {}.", a11, a22, tr, a11, a22, a12, a21, det),
                        format!("Form characteristic quadratic: lambda^2 - ({})*lambda + ({}) = 0.", tr, det),
                        format!("Factor (lambda - {})*(lambda - ({})) = 0; larger root is {}.", l1, l2, l1),
                    ],
                    derivation_steps: vec![
                        format!("1. Invariants: tr(A) = {}, det(A) = {}.", tr, det),
                        format!("2. Characteristic equation: lambda^2 - ({})*lambda + ({}) = 0.", tr, det),
                        format!("3. Eigenvalues are {} and {}; larger root lambda_1 = {}.", l1, l2, l1),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Compute Matrix Trace & Determinant",
                            r"\operatorname{tr}(A) = a_{11} + a_{22},\quad \det(A) = a_{11}a_{22} - a_{12}a_{21}",
                            format!("Invariants equal tr(A) = {} and det(A) = {}.", tr, det),
                            format!(r"A = {}", mat_tex),
                            format!(r"\operatorname{{tr}}(A) = {},\quad \det(A) = {}", tr, det),
                            format!("tr={}, det={}", tr, det),
                        ),
                        raw_step(
                            "Characteristic Quadratic Equation",
                            r"\lambda^{2} - \operatorname{tr}(A)\lambda + \det(A) = 0",
                            "Substitute trace and determinant into characteristic polynomial.",
                            format!(r"\det(A - \lambda I) = 0"),
                            char_tex.clone(),
                            format!("lambda^2 - ({})*lambda + ({}) = 0", tr, det),
                        ),
                        raw_step(
                            "Extract Dominant Eigenvalue",
                            r"(\lambda - \lambda_1)(\lambda - \lambda_2) = 0",
                            format!("Roots are {} and {}; larger eigenvalue is {}.", l1, l2, l1),
                            char_tex,
                            format!(r"(\lambda - {})(\lambda - ({})) = 0 \implies \lambda_1 = {}", l1, l2, l1),
                            format!("{}", l1),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-quadratics".to_string()),
                }
            }

            "lin-symmetric-quadratic" => {
                let a = rng.gen_range(1..=5i32);
                let b = rng.gen_range(-4..=4i32);
                let b = if b == 0 { 2 } else { b };
                let c = rng.gen_range(1..=5i32);
                let mid = if b > 0 {
                    format!("+ {}*x*y", 2 * b)
                } else {
                    format!("- {}*x*y", -2 * b)
                };
                let expected = format!("{}*x^2 {} + {}*y^2", a, mid, c);
                let q_tex = plain_to_latex_or(&expected);
                let ax_tex = format!(
                    r"\begin{{bmatrix}} {}x {} y \\ {}x + {}y \end{{bmatrix}}",
                    a, signed_i(b), b, c
                );
                let target_tex = format!(
                    r"Q(x,y) = \begin{{bmatrix}} x & y \end{{bmatrix}} \begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}} \begin{{bmatrix}} x \\ y \end{{bmatrix}}",
                    a, b, b, c
                );
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::QuadraticFormExpansion,
                    title: "Symmetric Matrix Quadratic Form Expansion".to_string(),
                    prompt: format!(
                        "Expand the quadratic form Q(x, y) = [x, y] * A * [x, y]^T for the symmetric matrix A = [[{}, {}], [{}, {}]].",
                        a, b, b, c
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"Q(x,y) = {}", q_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string(), "y".to_string()],
                    },
                    hints: vec![
                        format!("First multiply A * [x, y]^T = <{}*x + ({})*y, ({})*x + {}*y>.", a, b, b, c),
                        "Take the dot product with <x, y> and combine the two identical off-diagonal x*y terms.".to_string(),
                        format!("Result: {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. A*[x,y]^T = <{}*x + ({})*y, ({})*x + {}*y>.", a, b, b, c),
                        format!("2. Dot with <x,y>: {}*x^2 + ({})*x*y + ({})*y*x + {}*y^2.", a, b, b, c),
                        format!("3. Collect cross terms: {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Right-Multiply Matrix Against Column",
                            r"A\mathbf{x} = \begin{bmatrix}ax + by \\ bx + cy\end{bmatrix}",
                            "Multiply symmetric matrix A by [x, y]^T.",
                            target_tex,
                            format!(r"\begin{{bmatrix}} x & y \end{{bmatrix}} {}", ax_tex),
                            format!("<{}*x + ({})*y, ({})*x + {}*y>", a, b, b, c),
                        ),
                        raw_step(
                            "Take Row Inner Product with [x, y]",
                            r"\mathbf{x}^{T}(A\mathbf{x}) = x(ax+by) + y(bx+cy)",
                            "Contract left row vector [x, y] against A*x.",
                            format!(r"\begin{{bmatrix}} x & y \end{{bmatrix}} {}", ax_tex),
                            format!(r"{}x^{{2}} {}xy {}yx + {}y^{{2}}", a, signed_i(b), signed_i(b), c),
                            format!("{}*x^2 + ({})*x*y + ({})*y*x + {}*y^2", a, b, b, c),
                        ),
                        raw_step(
                            "Combine Symmetric Cross Terms",
                            r"bxy + byx = 2bxy",
                            format!("Merge equal off-diagonal cross terms into {}xy.", 2 * b),
                            format!(r"{}x^{{2}} {}xy {}yx + {}y^{{2}}", a, signed_i(b), signed_i(b), c),
                            q_tex,
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("lin-matrices-maps".to_string()),
                }
            }

            // =====================================================================
            // STAGE V: ORDINARY & PARTIAL DIFFERENTIAL EQUATIONS (6 Modules)
            // =====================================================================
            "deq-first-order" => {
                let k = rng.gen_range(1..=4i32);
                let eq = rng.gen_range(2..=6i32);
                let delta = rng.gen_range(2..=5i32);
                let y0 = eq + delta;
                let expected = format!("{} + {}*exp(-{}*t)", eq, delta, k);
                let ode_tex = format!(r"\frac{{dy}}{{dt}} + {}y = {}", k, k * eq);
                let ans_tex = format!(r"{} + {}e^{{-{}t}}", eq, delta, k);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::FirstOrderOde,
                    title: "First-Order Linear IVP via Integrating Factor".to_string(),
                    prompt: format!(
                        "Solve the initial-value problem for y(t):  dy/dt + {}*y = {},  with y(0) = {}.",
                        k, k * eq, y0
                    ),
                    target_equation_latex: format!(r"{},\qquad y(0) = {}", ode_tex, y0),
                    answer_latex: format!(r"y(t) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["t".to_string()],
                    },
                    hints: vec![
                        format!("Multiply through by integrating factor mu(t) = exp({}*t): d/dt [exp({}*t)*y] = {}*exp({}*t).", k, k, k * eq, k),
                        format!("Integrate both sides wrt t: exp({}*t)*y(t) = {}*exp({}*t) + C.", k, eq, k),
                        format!("Apply y(0) = {} to get C = {}, giving y(t) = {}.", y0, delta, expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Integrating factor: mu(t) = exp({}*t).", k),
                        format!("2. General solution: y(t) = {} + C*exp(-{}*t).", eq, k),
                        format!("3. Initial value y(0) = {} => C = {} - {} = {}, so y(t) = {}.", y0, y0, eq, delta, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Integrating Factor Multiplication",
                            r"\mu(t) = e^{\int p(t)\,dt} \implies \frac{d}{dt}[\mu y] = \mu q(t)",
                            format!("Multiply ODE by mu(t) = exp({}*t) to collapse LHS into a product derivative.", k),
                            ode_tex,
                            format!(r"\frac{{d}}{{dt}}\!\left[e^{{{}t}} y\right] = {} e^{{{}t}}", k, k * eq, k),
                            format!("mu(t) = exp({}*t)", k),
                        ),
                        raw_step(
                            "Integrate Both Sides wrt t",
                            r"y(t) = \frac{1}{\mu(t)}\left(\int \mu(t) q(t)\,dt + C\right)",
                            format!("Integrate RHS and multiply by exp(-{}*t).", k),
                            format!(r"\frac{{d}}{{dt}}\!\left[e^{{{}t}} y\right] = {} e^{{{}t}}", k, k * eq, k),
                            format!(r"y(t) = {} + C e^{{-{}t}}", eq, k),
                            format!("{} + C*exp(-{}*t)", eq, k),
                        ),
                        raw_step(
                            "Apply Initial Condition y(0)",
                            r"y(0) = y_{\text{eq}} + C \implies C = y(0) - y_{\text{eq}}",
                            format!("Substitute t = 0: {} + C = {} gives C = {}.", eq, y0, delta),
                            format!(r"{} + C e^{{0}} = {}", eq, y0),
                            format!(r"y(t) = {}", ans_tex),
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("calc-integration-tech".to_string()),
                }
            }

            "deq-second-order-osc" => {
                if variant == 0 {
                    let w = rng.gen_range(2..=5i32);
                    let c1 = rng.gen_range(1..=4i32);
                    let c2 = rng.gen_range(1..=4i32);
                    let expected = format!("{}*cos({}*t) + {}*sin({}*t)", c1, w, c2, w);
                    let ans_tex = format!(r"{}\cos({}t) + {}\sin({}t)", c1, w, c2, w);
                    let ode_tex = format!(r"y'' + {}y = 0", w * w);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::SecondOrderOde,
                        title: "Undamped Harmonic Oscillator IVP".to_string(),
                        prompt: format!(
                            "Solve for y(t):  y'' + {}*y = 0,  given initial state y(0) = {} and y'(0) = {}.",
                            w * w, c1, c2 * w
                        ),
                        target_equation_latex: format!(r"{},\quad y(0) = {},\; y'(0) = {}", ode_tex, c1, c2 * w),
                        answer_latex: format!(r"y(t) = {}", ans_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["t".to_string()],
                        },
                        hints: vec![
                            format!("Characteristic equation r^2 + {} = 0 has roots r = +/- {}*i.", w * w, w),
                            format!("General solution is y(t) = A*cos({}*t) + B*sin({}*t).", w, w),
                            format!("Match y(0) = A = {} and y'(0) = {}*B = {} => B = {}: {}.", c1, w, c2 * w, c2, expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Natural angular frequency: omega = sqrt({}) = {}.", w * w, w),
                            format!("2. Basis superposition: y(t) = A*cos({}*t) + B*sin({}*t).", w, w),
                            format!("3. Apply initial conditions: A = {}, B = {} => y(t) = {}.", c1, c2, expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Solve Characteristic Quadratic",
                                r"r^{2} + \omega^{2} = 0 \implies r = \pm i\omega",
                                format!("Natural angular frequency omega = sqrt({}) = {}.", w * w, w),
                                ode_tex,
                                format!(r"r^{{2}} + {} = 0 \implies r = \pm {}i", w * w, w),
                                format!("r = +/- {}*i", w),
                            ),
                            raw_step(
                                "Form Harmonic Basis Superposition",
                                r"y(t) = A\cos(\omega t) + B\sin(\omega t)",
                                "Build real oscillatory general solution.",
                                format!(r"r = \pm {}i", w),
                                format!(r"y(t) = A\cos({}t) + B\sin({}t)", w, w),
                                format!("A*cos({}*t) + B*sin({}*t)", w, w),
                            ),
                            raw_step(
                                "Match Initial State y(0), y'(0)",
                                r"A = y(0),\quad B = \frac{y'(0)}{\omega}",
                                format!("Find A = {} and B = {}/{} = {}.", c1, c2 * w, w, c2),
                                format!(r"y(0)={},\; {}B={}", c1, w, c2 * w),
                                format!(r"y(t) = {}", ans_tex),
                                expected.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("trig-graphs-waves".to_string()),
                    }
                } else {
                    let r1 = rng.gen_range(1..=2i32);
                    let r2 = rng.gen_range(3..=5i32);
                    let expected = format!("exp(-{}*t) + exp(-{}*t)", r1, r2);
                    let ode_tex = format!(r"y'' + {}y' + {}y = 0", r1 + r2, r1 * r2);
                    let ans_tex = format!(r"e^{{-{}t}} + e^{{-{}t}}", r1, r2);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::SecondOrderOde,
                        title: "Overdamped Second-Order Homogeneous IVP".to_string(),
                        prompt: format!(
                            "Solve for y(t):  y'' + {}*y' + {}*y = 0,  with y(0) = 2 and y'(0) = -{}.",
                            r1 + r2, r1 * r2, r1 + r2
                        ),
                        target_equation_latex: format!(r"{},\quad y(0) = 2,\; y'(0) = -{}", ode_tex, r1 + r2),
                        answer_latex: format!(r"y(t) = {}", ans_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["t".to_string()],
                        },
                        hints: vec![
                            format!("Factor characteristic quadratic r^2 + {}*r + {} = (r + {})*(r + {}) = 0.", r1 + r2, r1 * r2, r1, r2),
                            format!("General solution: y(t) = c1*exp(-{}*t) + c2*exp(-{}*t).", r1, r2),
                            format!("Solve c1 + c2 = 2 and -{}*c1 - {}*c2 = -{} => c1 = 1, c2 = 1.", r1, r2, r1 + r2),
                        ],
                        derivation_steps: vec![
                            format!("1. Characteristic roots: r1 = -{}, r2 = -{}.", r1, r2),
                            format!("2. Linear system from ICs: c1 + c2 = 2, -{}*c1 - {}*c2 = -{}.", r1, r2, r1 + r2),
                            format!("3. Unique solution c1 = c2 = 1 gives y(t) = {}.", expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Factor Characteristic Polynomial",
                                r"r^{2} + (r_1+r_2)r + r_1 r_2 = 0",
                                format!("Factor into (r + {})(r + {}) = 0 for distinct negative roots.", r1, r2),
                                ode_tex,
                                format!(r"(r + {})(r + {}) = 0 \implies r_1 = -{}, r_2 = -{}", r1, r2, r1, r2),
                                format!("r1 = -{}, r2 = -{}", r1, r2),
                            ),
                            raw_step(
                                "Initial-Condition 2x2 Linear System",
                                r"\begin{cases} c_1 + c_2 = y(0) \\ r_1 c_1 + r_2 c_2 = y'(0) \end{cases}",
                                format!("Set c1 + c2 = 2 and -{}*c1 - {}*c2 = -{}.", r1, r2, r1 + r2),
                                format!(r"y(t) = c_1 e^{{-{}t}} + c_2 e^{{-{}t}}", r1, r2),
                                format!(r"c_1 = 1,\quad c_2 = 1"),
                                format!("c1 = 1, c2 = 1"),
                            ),
                            raw_step(
                                "Assemble Overdamped Solution",
                                r"y(t) = c_1 e^{r_1 t} + c_2 e^{r_2 t}",
                                "Substitute c1 = 1 and c2 = 1.",
                                format!(r"c_1 = 1,\; c_2 = 1"),
                                format!(r"y(t) = {}", ans_tex),
                                expected.clone(),
                            ),
                        ],
                        visual: None,
                        remediate_module_id: Some("alg-quadratics".to_string()),
                    }
                }
            }

            "deq-linear-systems-phase" => {
                let a11 = rng.gen_range(-4..=3i32);
                let a12 = rng.gen_range(-3..=4i32);
                let a12 = if a12 == 0 { 2 } else { a12 };
                let a21 = rng.gen_range(-4..=3i32);
                let a21 = if a21 == 0 { -2 } else { a21 };
                let a22 = rng.gen_range(-4..=3i32);
                let tr = a11 + a22;
                let det = a11 * a22 - a12 * a21;
                let disc = tr * tr - 4 * det;
                let mat_tex = format!(r"\begin{{bmatrix}} {} & {} \\ {} & {} \end{{bmatrix}}", a11, a12, a21, a22);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::PlanarSystemDiscriminant,
                    title: "Trace-Determinant Phase Discriminant".to_string(),
                    prompt: format!(
                        "For the planar linear system x' = A*x with A = [[{}, {}], [{}, {}]], compute the spiral-node discriminant Delta = tr(A)^2 - 4*det(A).",
                        a11, a12, a21, a22
                    ),
                    target_equation_latex: format!(
                        r"\mathbf{{x}}' = {}\mathbf{{x}},\qquad \Delta = \operatorname{{tr}}(A)^{{2}} - 4\det(A)",
                        mat_tex
                    ),
                    answer_latex: format!(r"\Delta = {}", disc),
                    answer: AnswerFormat::Numeric {
                        expected: disc as f64,
                        tolerance: 1e-5,
                        canonical_display: format!("{}", disc),
                    },
                    hints: vec![
                        format!("Compute trace tr(A) = {} + ({}) = {}.", a11, a22, tr),
                        format!("Compute determinant det(A) = ({})*({}) - ({})*({}) = {}.", a11, a22, a12, a21, det),
                        format!("Evaluate Delta = ({})^2 - 4*({}) = {}.", tr, det, disc),
                    ],
                    derivation_steps: vec![
                        format!("1. tr(A) = {} and det(A) = {}.", tr, det),
                        format!("2. Delta = ({})^2 - 4*({}) = {} - ({}) = {}.", tr, det, tr * tr, 4 * det, disc),
                        format!(
                            "3. Interpretation: Delta {} 0 indicates {} eigenvalues.",
                            if disc < 0 { "<" } else { ">=" },
                            if disc < 0 { "complex spiral/center" } else { "real node/saddle" }
                        ),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Trace & Determinant Invariants",
                            r"\operatorname{tr}(A) = a_{11} + a_{22},\quad \det(A) = a_{11}a_{22} - a_{12}a_{21}",
                            format!("Evaluate tr(A) = {} and det(A) = {}.", tr, det),
                            format!(r"A = {}", mat_tex),
                            format!(r"\operatorname{{tr}}(A) = {},\quad \det(A) = {}", tr, det),
                            format!("tr={}, det={}", tr, det),
                        ),
                        raw_step(
                            "Compute Phase Discriminant Delta",
                            r"\Delta = \operatorname{tr}(A)^{2} - 4\det(A)",
                            format!("Evaluate ({})^2 - 4*({}) = {} - ({}).", tr, det, tr * tr, 4 * det),
                            format!(r"({})^{{2}} - 4({})", tr, det),
                            format!(r"\Delta = {}", disc),
                            format!("{}", disc),
                        ),
                        raw_step(
                            "Classify Phase Regime",
                            r"\Delta < 0 \implies \text{spiral/center},\; \Delta \ge 0 \implies \text{node/saddle}",
                            if disc < 0 {
                                "Negative discriminant produces complex eigenvalues (spiral or center)."
                            } else {
                                "Nonnegative discriminant produces real eigenvalues (node or saddle)."
                            },
                            format!(r"\Delta = {}", disc),
                            format!(r"\Delta = {} \quad ({})", disc, if disc < 0 { r"\text{spiral/center}" } else { r"\text{real node/saddle}" }),
                            format!("{}", disc),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("lin-eigenvalues-diag".to_string()),
                }
            }

            "deq-laplace-nonlinear" => {
                let a = rng.gen_range(2..=5i32);
                let k = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=4i32);
                let w = rng.gen_range(2..=4i32);
                let expected = format!("{}/(s - {}) + ({}*s)/(s^2 + {})", a, k, b, w * w);
                let target_tex = format!(r"\mathcal{{L}}\!\left\{{ {} e^{{{}t}} + {} \cos({}t) \right\}}", a, k, b, w);
                let ans_tex = format!(r"\frac{{{}}}{{s - {}}} + \frac{{{}s}}{{s^{{2}} + {}}}", a, k, b, w * w);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::LaplaceTransform,
                    title: "Symbolic Laplace Transform in s-Domain".to_string(),
                    prompt: format!(
                        "Compute the Laplace transform F(s) = L{{ {}*exp({}*t) + {}*cos({}*t) }} as a rational expression in s (for s > {}).",
                        a, k, b, w, k
                    ),
                    target_equation_latex: target_tex.clone(),
                    answer_latex: format!(r"F(s) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["s".to_string()],
                    },
                    hints: vec![
                        "Use linearity of L{.} on each term independently.".to_string(),
                        format!("Recall L{{exp({}*t)}} = 1/(s - {}) and L{{cos({}*t)}} = s/(s^2 + {}).", k, k, w, w * w),
                        format!("Combine weighted terms: {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Exponential term: {} * L{{exp({}*t)}} = {}/(s - {}).", a, k, a, k),
                        format!("2. Cosine term: {} * L{{cos({}*t)}} = ({}*s)/(s^2 + {}).", b, w, b, w * w),
                        format!("3. Total s-domain transform: F(s) = {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Laplace Transform of Exponential Mode",
                            r"\mathcal{L}\{e^{kt}\} = \frac{1}{s - k} \quad (s > k)",
                            format!("Transform {}*exp({}*t) to rational pole at s = {}.", a, k, k),
                            format!(r"{}\,\mathcal{{L}}\!\left\{{e^{{{}t}}\right\}}", a, k),
                            format!(r"\frac{{{}}}{{s - {}}}", a, k),
                            format!("{}/(s - {})", a, k),
                        ),
                        raw_step(
                            "Laplace Transform of Cosine Mode",
                            r"\mathcal{L}\{\cos(\omega t)\} = \frac{s}{s^{2} + \omega^{2}}",
                            format!("Transform {}*cos({}*t) with omega^2 = {}.", b, w, w * w),
                            format!(r"{}\,\mathcal{{L}}\!\left\{{\cos({}t)\right\}}", b, w),
                            format!(r"\frac{{{}s}}{{s^{{2}} + {}}}", b, w * w),
                            format!("({}*s)/(s^2 + {})", b, w * w),
                        ),
                        raw_step(
                            "Linear Superposition in s-Domain",
                            r"\mathcal{L}\{a f + b g\} = a\mathcal{L}\{f\} + b\mathcal{L}\{g\}",
                            "Sum transformed rational modes.",
                            target_tex,
                            format!(r"F(s) = {}", ans_tex),
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-polynomial-rational".to_string()),
                }
            }

            "deq-heat-diffusion-pde" => {
                let alpha = rng.gen_range(1..=2i32);
                let mode_n = rng.gen_range(1..=2i32);
                let amp = rng.gen_range(3..=6i32);
                let decay_rate = alpha * mode_n * mode_n;
                let expected = format!("{} * exp(-{}*t) * sin({}*x)", amp, decay_rate, mode_n);
                let ans_tex = format!(r"{} e^{{-{}t}} \sin({}x)", amp, decay_rate, mode_n);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 4,
                    operation: OperationKind::HeatPdeEigenmode,
                    title: "1D Heat Equation Dirichlet Eigenmode Evolution".to_string(),
                    prompt: format!(
                        "Solve the 1D Heat PDE u_t = {} * u_xx on 0 <= x <= pi with zero ends u(0,t) = u(pi,t) = 0 and initial profile u(x, 0) = {} * sin({}*x). Write u(x, t) in variables x and t.",
                        alpha, amp, mode_n
                    ),
                    target_equation_latex: format!(
                        r"u_t = {} u_{{xx}},\quad u(x,0) = {}\sin({}x),\quad u(0,t)=u(\pi,t)=0",
                        alpha, amp, mode_n
                    ),
                    answer_latex: format!(r"u(x,t) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string(), "t".to_string()],
                    },
                    hints: vec![
                        format!("On [0, pi], sin({}*x) is already a single Dirichlet eigenfunction with eigenvalue lambda = {}^2 = {}.", mode_n, mode_n, mode_n * mode_n),
                        format!("Temporal amplitude T(t) satisfies T' = -{} * {} * T = -{} * T.", alpha, mode_n * mode_n, decay_rate),
                        format!("Combine spatial mode and exponential decay: u(x, t) = {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Spatial operator: d^2/dx^2 [sin({}*x)] = -{} * sin({}*x).", mode_n, mode_n * mode_n, mode_n),
                        format!("2. Temporal ODE: T'(t) = -{}*T(t) => T(t) = {}*exp(-{}*t).", decay_rate, amp, decay_rate),
                        format!("3. Exact solution: u(x, t) = {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Dirichlet Spatial Eigenvalue",
                            r"\frac{d^{2}}{dx^{2}}\sin(nx) = -n^{2}\sin(nx)",
                            format!("Differentiate sin({}*x) twice wrt x giving eigenvalue -{}.", mode_n, mode_n * mode_n),
                            format!(r"\frac{{\partial^{{2}}}}{{\partial x^{{2}}}}\sin({}x)", mode_n),
                            format!(r"-{} \sin({}x)", mode_n * mode_n, mode_n),
                            format!("-{}*sin({}*x)", mode_n * mode_n, mode_n),
                        ),
                        raw_step(
                            "Temporal Decay ODE",
                            r"T'(t) = -\alpha n^{2} T(t) \implies T(t) = T(0) e^{-\alpha n^{2} t}",
                            format!("Solve T' = -{}*{}*T = -{}*T with T(0) = {}.", alpha, mode_n * mode_n, decay_rate, amp),
                            format!(r"T'(t) = -{} T(t),\; T(0)={}", decay_rate, amp),
                            format!(r"T(t) = {} e^{{-{}t}}", amp, decay_rate),
                            format!("{}*exp(-{}*t)", amp, decay_rate),
                        ),
                        raw_step(
                            "Assemble Heat Eigenmode Solution",
                            r"u(x,t) = T(t)\sin(nx)",
                            "Multiply temporal decay factor by spatial sine mode.",
                            format!(r"T(t)\sin({}x)", mode_n),
                            format!(r"u(x,t) = {}", ans_tex),
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("deq-first-order".to_string()),
                }
            }

            "deq-wave-laplace-pde" => {
                let speed_c = rng.gen_range(2..=4i32);
                let mode_n = rng.gen_range(1..=4i32);
                let amp = rng.gen_range(2..=5i32);
                let omega = speed_c * mode_n;
                let expected = format!("{} * cos({}*t) * sin({}*x)", amp, omega, mode_n);
                let ans_tex = format!(r"{}\cos({}t)\sin({}x)", amp, omega, mode_n);
                RawProblem {
                    module_id: "deq-wave-laplace-pde".to_string(),
                    seed,
                    difficulty: 4,
                    operation: OperationKind::WavePdeStanding,
                    title: "Standing Harmonic Solution of the 1D Wave PDE".to_string(),
                    prompt: format!(
                        "Solve the 1D Wave PDE u_tt = {} * u_xx on 0 <= x <= pi (wave speed c = {}) with fixed ends u(0,t) = u(pi,t) = 0, released from rest u_t(x, 0) = 0 with initial shape u(x, 0) = {} * sin({}*x).",
                        speed_c * speed_c, speed_c, amp, mode_n
                    ),
                    target_equation_latex: format!(
                        r"u_{{tt}} = {} u_{{xx}},\quad u(x,0) = {}\sin({}x),\; u_t(x,0) = 0",
                        speed_c * speed_c, amp, mode_n
                    ),
                    answer_latex: format!(r"u(x,t) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string(), "t".to_string()],
                    },
                    hints: vec![
                        format!("Separation of variables on [0, pi] gives temporal oscillator T'' + ({}*{})^2 * T = 0.", speed_c, mode_n),
                        format!("Since initial velocity u_t(x, 0) = 0, the sine time term vanishes leaving T(t) = {}*cos({}*t).", amp, omega),
                        format!("Full standing wave: u(x, t) = {}.", expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Modal angular frequency: omega_n = c * n = {} * {} = {}.", speed_c, mode_n, omega),
                        format!("2. Zero initial velocity selects pure cosine time factor cos({}*t).", omega),
                        format!("3. Standing-wave solution: u(x, t) = {}.", expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Quantized Standing-Wave Frequency",
                            r"\omega_n = c n",
                            format!("With c = {} and n = {}, natural temporal frequency omega_n = {}.", speed_c, mode_n, omega),
                            format!(r"T''(t) + ({} \cdot {})^{{2}} T(t) = 0", speed_c, mode_n),
                            format!(r"\omega_{{{}}} = {},\quad T(t) = A\cos({}t) + B\sin({}t)", mode_n, omega, omega, omega),
                            format!("omega = {}", omega),
                        ),
                        raw_step(
                            "Apply Released-from-Rest Condition",
                            r"u_t(x,0) = 0 \implies B = 0",
                            format!("Zero initial velocity eliminates sin({}*t) and pins A = {}.", omega, amp),
                            format!(r"T'(0) = {}B = 0,\; T(0) = {}", omega, amp),
                            format!(r"T(t) = {}\cos({}t)", amp, omega),
                            format!("{}*cos({}*t)", amp, omega),
                        ),
                        raw_step(
                            "Assemble Standing-Wave Mode",
                            r"u(x,t) = T(t)\sin(nx)",
                            "Combine pure cosine time factor with Dirichlet spatial sine mode.",
                            format!(r"T(t)\sin({}x)", mode_n),
                            format!(r"u(x,t) = {}", ans_tex),
                            expected.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("deq-second-order-osc".to_string()),
                }
            }

            // =====================================================================
            // STAGE VI: PROBABILITY & STATISTICAL INFERENCE (6 Modules)
            // =====================================================================
            "sta-combinatorics-bayes" => {
                let prev_choices = [5_i32, 10, 20];
                let sens_choices = [90_i32, 95, 96];
                let spec_choices = [90_i32, 95, 96];
                let prev = prev_choices[rng.gen_range(0..prev_choices.len())];
                let sens = sens_choices[rng.gen_range(0..sens_choices.len())];
                let spec = spec_choices[rng.gen_range(0..spec_choices.len())];
                let tp = prev * sens;
                let fp = (100 - prev) * (100 - spec);
                let total_pos = tp + fp;
                let expected = (tp as f64) / (total_pos as f64);
                let disp = format!("{}/{}", tp, total_pos);
                let scenario = match domain {
                    ScienceDomain::Biology => format!(
                        "An ELISA biomarker screen targets a cellular phenotype with base prevalence {}%, assay sensitivity {}%, and specificity {}%. Given a positive test (+), find the exact posterior probability P(Trait | +).",
                        prev, sens, spec
                    ),
                    ScienceDomain::Microbiology => format!(
                        "A rapid selective-plate screen checks isolates where {}% carry a resistance plasmid, with true-positive detection {}% and specificity {}%. Given a positive screen (+), compute P(Plasmid | +).",
                        prev, sens, spec
                    ),
                    ScienceDomain::Ecology => format!(
                        "An acoustic wildlife classifier surveys wetlands where a cryptic amphibian occupies {}% of ponds (sensitivity {}%, specificity {}%). Given a positive detection (+), find P(Occupied | +).",
                        prev, sens, spec
                    ),
                    ScienceDomain::Chemistry => format!(
                        "A spectroscopic trace-impurity test screens batches with {}% defect base rate, {}% sensitivity, and {}% specificity. Given an alarm (+), find P(Impure | +).",
                        prev, sens, spec
                    ),
                    ScienceDomain::Physics => format!(
                        "A particle-trigger filter flags candidate events where {}% are true signal collisions (efficiency/sensitivity {}%, background rejection/specificity {}%). Given a trigger (+), find P(Signal | +).",
                        prev, sens, spec
                    ),
                    ScienceDomain::General => format!(
                        "Hypothesis H has prior probability P(H) = {}%. An indicator E has sensitivity P(+ | H) = {}% and specificity P(- | not H) = {}%. Compute the exact posterior probability P(H | +).",
                        prev, sens, spec
                    ),
                };
                let ans_tex = format!(r"\frac{{{}}}{{{}}}", tp, total_pos);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::BayesianPosterior,
                    title: "Bayesian Posterior Probability Update".to_string(),
                    prompt: scenario,
                    target_equation_latex: format!(
                        r"P(H) = 0.{:02},\; P(+|H) = 0.{:02},\; P(-|\neg H) = 0.{:02} \implies P(H|+)",
                        prev, sens, spec
                    ),
                    answer_latex: format!(r"P(H|+) = {}", ans_tex),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Out of 10,000 instances, true positives = {} * {} = {}, and false positives = {} * {} = {}.", prev, sens, tp, 100 - prev, 100 - spec, fp),
                        format!("By Law of Total Probability, total positive signals = {} + {} = {}.", tp, fp, total_pos),
                        format!("Apply Bayes' Rule: P(H | +) = TP / (TP + FP) = {}.", disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Prior & Likelihood Numerator: P(+ | H)*P(H) = ({}/100)*({}/100) = {}/10000.", sens, prev, tp),
                        format!("2. Law of Total Probability: P(+) = ({} + {})/10000 = {}/10000.", tp, fp, total_pos),
                        format!("3. Bayes' Posterior Ratio: P(H | +) = {}.", disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "True-Positive & False-Positive Joint Rates",
                            r"P(+, H) = P(+|H)P(H),\quad P(+,\neg H) = P(+|\neg H)P(\neg H)",
                            format!("Per 10,000 cases: TP = {}, FP = {}.", tp, fp),
                            format!(r"0.{:02}\cdot 0.{:02},\; 0.{:02}\cdot 0.{:02}", sens, prev, 100 - spec, 100 - prev),
                            format!(r"P(+,H) = \frac{{{}}}{{10000}},\quad P(+,\neg H) = \frac{{{}}}{{10000}}", tp, fp),
                            format!("TP={}, FP={}", tp, fp),
                        ),
                        raw_step(
                            "Law of Total Positive Probability",
                            r"P(+) = P(+,H) + P(+,\neg H)",
                            format!("Sum positive rates: {} + {} = {}.", tp, fp, total_pos),
                            format!(r"\frac{{{}}}{{10000}} + \frac{{{}}}{{10000}}", tp, fp),
                            format!(r"P(+) = \frac{{{}}}{{10000}}", total_pos),
                            format!("{}/10000", total_pos),
                        ),
                        raw_step(
                            "Bayes' Posterior Conditioning",
                            r"P(H|+) = \frac{P(+|H)P(H)}{P(+)}",
                            "Divide true-positive probability by total positive probability.",
                            format!(r"\frac{{{}/10000}}{{{}/10000}}", tp, total_pos),
                            format!(r"P(H|+) = {}", ans_tex),
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-polynomial-rational".to_string()),
                }
            }

            "sta-discrete-rv" => {
                let lam = rng.gen_range(2..=4i32);
                let k = rng.gen_range(0..=2i32);
                let (expected, disp, pow_tex, ans_tex) = match k {
                    0 => (
                        (-(lam as f64)).exp(),
                        format!("exp(-{})", lam),
                        r"1".to_string(),
                        format!(r"e^{{-{}}}", lam),
                    ),
                    1 => (
                        (lam as f64) * (-(lam as f64)).exp(),
                        format!("{}*exp(-{})", lam, lam),
                        format!("{}", lam),
                        format!(r"{}e^{{-{}}}", lam, lam),
                    ),
                    _ => (
                        0.5 * ((lam * lam) as f64) * (-(lam as f64)).exp(),
                        format!("({}/2)*exp(-{})", lam * lam, lam),
                        format!(r"\frac{{{}}}{{2}}", lam * lam),
                        format!(r"\frac{{{}}}{{2}}e^{{-{}}}", lam * lam, lam),
                    ),
                };
                let prompt_text = match domain {
                    ScienceDomain::Microbiology => format!(
                        "Bacteriophages adsorb to host cells at Multiplicity of Infection (Poisson mean) lambda = {}. Compute the exact probability P(X = {}) that a host cell adsorbs exactly {} phage particles.",
                        lam, k, k
                    ),
                    ScienceDomain::Ecology => format!(
                        "Alpine cushion plants occur randomly in 1 m^2 quadrats with Poisson density lambda = {} plants/quadrat. Find the exact probability P(X = {}) in a randomly sampled quadrat.",
                        lam, k
                    ),
                    ScienceDomain::Physics => format!(
                        "A low-flux photon counter records a Poisson mean of lambda = {} photons per window. Compute the exact probability P(X = {}).",
                        lam, k
                    ),
                    _ => format!(
                        "For a Poisson random variable X ~ Pois(lambda = {}), evaluate the exact probability mass P(X = {}). (Express using exp(...).)",
                        lam, k
                    ),
                };
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    operation: OperationKind::PoissonDistribution,
                    title: "Exact Poisson Rare-Event Probability".to_string(),
                    prompt: prompt_text,
                    target_equation_latex: format!(r"X \sim \operatorname{{Pois}}(\lambda = {}),\qquad P(X = {})", lam, k),
                    answer_latex: format!(r"P(X = {}) = {}", k, ans_tex),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-4,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Recall the Poisson PMF: P(X = k) = (lambda^k * exp(-lambda)) / k!."),
                        format!("Substitute lambda = {} and k = {} (with {}! = {}).", lam, k, k, if k == 2 { 2 } else { 1 }),
                        format!("Simplified exact probability: {}.", disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Identify Poisson rate parameter lambda = {} and count k = {}.", lam, k),
                        format!("2. Evaluate lambda^k / k! = {}^{} / {}!.", lam, k, k),
                        format!("3. Multiply by exp(-{}): P(X = {}) = {}.", lam, k, disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Poisson Probability Mass Formula",
                            r"P(X = k) = \frac{\lambda^{k} e^{-\lambda}}{k!}",
                            format!("Substitute rate lambda = {} and event count k = {}.", lam, k),
                            format!(r"\lambda = {},\; k = {}", lam, k),
                            format!(r"P(X = {}) = \frac{{{}^{{{}}} e^{{-{}}}}}{{{}!}}", k, lam, k, lam, k),
                            format!("({}^{} * exp(-{})) / {}!", lam, k, lam, k),
                        ),
                        raw_step(
                            "Evaluate Factorial Power Coefficient",
                            r"\frac{\lambda^{k}}{k!}",
                            format!("Reduce {}^{} / {}!.", lam, k, k),
                            format!(r"\frac{{{}^{{{}}}}}{{{}!}}", lam, k, k),
                            pow_tex.clone(),
                            pow_tex,
                        ),
                        raw_step(
                            "Multiply by Exponential Decay Factor",
                            r"P(X = k) = \left(\frac{\lambda^{k}}{k!}\right)e^{-\lambda}",
                            format!("Combine with exp(-{}).", lam),
                            format!(r"P(X = {}) = \left(\frac{{{}^{{{}}}}}{{{}!}}\right) e^{{-{}}}", k, lam, k, k, lam),
                            ans_tex,
                            disp.clone(),
                        ),
                    ],
                    visual: None,
                    remediate_module_id: Some("alg-exp-log".to_string()),
                }
            }

            "sta-continuous-pdf-cdf" => {
                let rate = rng.gen_range(1..=3i32);
                if variant == 0 {
                    let expected = format!("1 - exp(-{}*t)", rate);
                    let target_tex = format!(r"F(t) = \int_{{0}}^{{t}} {} e^{{-{}u}}\, du", rate, rate);
                    let ans_tex = format!(r"1 - e^{{-{}t}}", rate);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::ExponentialCdfSurvival,
                        title: "Cumulative Distribution Function via Density Integration".to_string(),
                        prompt: format!(
                            "A continuous waiting time T >= 0 has Exponential PDF f(u) = {} * exp(-{}*u). Integrate f(u) from 0 to t to obtain the cumulative distribution function F(t) = P(0 <= T <= t) (in variable t).",
                            rate, rate
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: format!(r"F(t) = {}", ans_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["t".to_string()],
                        },
                        hints: vec![
                            format!("Set up definite integral F(t) = integral_{{0..t}} {} * exp(-{}*u) du.", rate, rate),
                            format!("An antiderivative wrt u is -exp(-{}*u).", rate),
                            format!("Evaluate boundary difference (-exp(-{}*t)) - (-exp(0)) = {}.", rate, expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Integrate density: F(t) = integral_{{0..t}} {}*exp(-{}*u) du.", rate, rate),
                            format!("2. Antiderivative evaluation: [-exp(-{}*u)]_{{u=0..t}}.", rate),
                            format!("3. Apply FTC Part 2: -exp(-{}*t) - (-1) = {}.", rate, expected),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Exponential Density Integral",
                                r"F(t) = \int_{0}^{t} \lambda e^{-\lambda u}\,du",
                                "Find exponential antiderivative with chain factor -1/lambda.",
                                target_tex,
                                format!(r"\left[-e^{{-{}u}}\right]_{{0}}^{{t}}", rate),
                                format!("[-exp(-{}*u)]_{{0..t}}", rate),
                            ),
                            raw_step(
                                "Evaluate CDF Boundary Difference",
                                r"\left[-e^{-\lambda u}\right]_{0}^{t} = -e^{-\lambda t} - (-e^{0})",
                                "Substitute u = t and u = 0.",
                                format!(r"\left[-e^{{-{}u}}\right]_{{0}}^{{t}}", rate),
                                format!(r"F(t) = {}", ans_tex),
                                expected.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "t".to_string(),
                            x_range: Some([0.0, 4.0 / (rate as f64)]),
                            primary_expr: Some(format!("{}*exp(-{}*t)", rate, rate)),
                            primary_label: Some(format!("PDF f(t) = {}e^(-{}t)", rate, rate)),
                            secondary_expr: Some(expected),
                            secondary_label: Some("CDF F(t)".to_string()),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-integration-tech".to_string()),
                    }
                } else {
                    let t0 = rng.gen_range(1..=2i32);
                    let prod = rate * t0;
                    let expected = (-(prod as f64)).exp();
                    let disp = format!("exp(-{})", prod);
                    let target_tex = format!(r"P(T > {}) = \int_{{{}}}^{{\infty}} {} e^{{-{}t}}\, dt", t0, t0, rate, rate);
                    let ans_tex = format!(r"e^{{-{}}}", prod);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        operation: OperationKind::ExponentialCdfSurvival,
                        title: "Exponential Survival Tail Integral".to_string(),
                        prompt: format!(
                            "For T ~ Exp(lambda = {}) with PDF f(t) = {} * exp(-{}*t) (t >= 0), compute the exact survival probability P(T > {}).",
                            rate, rate, rate, t0
                        ),
                        target_equation_latex: target_tex.clone(),
                        answer_latex: format!(r"P(T > {}) = {}", t0, ans_tex),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-4,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Set up improper tail integral: P(T > {}) = integral_{{{}..inf}} {}*exp(-{}*t) dt.", t0, t0, rate, rate),
                            format!("Antiderivative [-exp(-{}*t)] vanishes as t -> inf.", rate),
                            format!("Lower-bound value gives {}.", disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Improper tail integral: lim_{{B->inf}} [-exp(-{}*t)]_{{{}..B}}.", rate, t0),
                            format!("2. Upper limit exp(-inf) -> 0, leaving exp(-{}*{}).", rate, t0),
                            format!("3. Survival probability: {}.", disp),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Improper Upper-Tail Limit",
                                r"\int_{t_0}^{\infty} \lambda e^{-\lambda t}\,dt = \lim_{B\to\infty}\left[-e^{-\lambda t}\right]_{t_0}^{B}",
                                format!("Compute antiderivative -exp(-{}*t) over [{}..inf).", rate, t0),
                                target_tex,
                                format!(r"\lim_{{B \to \infty}}\left(-e^{{-{}B}} + e^{{-{}\cdot{}}}\right)", rate, rate, t0),
                                format!("exp(-{}*{})", rate, t0),
                            ),
                            raw_step(
                                "Evaluate Survival Probability",
                                r"S(t_0) = P(T > t_0) = e^{-\lambda t_0}",
                                format!("Since exp(-{}B) -> 0 as B -> inf, only lower bound remains.", rate),
                                format!(r"0 + e^{{-{}\cdot{}}}", rate, t0),
                                ans_tex,
                                disp.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "t".to_string(),
                            x_range: Some([0.0, t0 as f64 + 4.0 / (rate as f64)]),
                            primary_expr: Some(format!("{}*exp(-{}*t)", rate, rate)),
                            primary_label: Some(format!("Exp PDF f(t) = {}e^(-{}t)", rate, rate)),
                            shade_interval: Some([t0 as f64, t0 as f64 + 4.0 / (rate as f64)]),
                            vline_x: Some(t0 as f64),
                            vline_label: Some(format!("t0 = {}", t0)),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-integration-tech".to_string()),
                    }
                }
            }

            "sta-joint-cov-clt" => {
                let roots = [4_i32, 5, 6, 8, 10];
                let root_n = roots[rng.gen_range(0..roots.len())];
                let n = root_n * root_n;
                let mu = rng.gen_range(10..=30i32);
                let sigma = rng.gen_range(2..=6i32);
                let delta = rng.gen_range(-3..=4i32);
                let delta = if delta == 0 { 2 } else { delta };
                let x_bar = mu + delta;
                let num = delta * root_n;
                let expected = (num as f64) / (sigma as f64);
                let disp = format!("{}/{}", num, sigma);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::CltStandardization,
                    title: "Central Limit Theorem Standardized Z-Statistic".to_string(),
                    prompt: format!(
                        "Independent measurements have population mean mu = {} and SD sigma = {}. For a sample of size n = {} with observed mean x_bar = {}, compute the standardized CLT score Z = (x_bar - mu) / (sigma / sqrt(n)).",
                        mu, sigma, n, x_bar
                    ),
                    target_equation_latex: format!(
                        r"\mu = {},\; \sigma = {},\; n = {},\; \bar{{x}} = {} \implies Z = \frac{{\bar{{x}} - \mu}}{{\sigma/\sqrt{{n}}}}",
                        mu, sigma, n, x_bar
                    ),
                    answer_latex: format!(r"Z = \frac{{{}}}{{{}}}", num, sigma),
                    answer: AnswerFormat::Numeric {
                        expected,
                        tolerance: 1e-5,
                        canonical_display: disp.clone(),
                    },
                    hints: vec![
                        format!("Compute standard error of the mean: SE = sigma / sqrt({}) = {} / {}.", n, sigma, root_n),
                        format!("Mean deviation numerator: x_bar - mu = {} - {} = {}.", x_bar, mu, delta),
                        format!("Divide {} by ({}/{}) to get Z = {}.", delta, sigma, root_n, disp),
                    ],
                    derivation_steps: vec![
                        format!("1. Sample-mean standard error: SE = {} / sqrt({}) = {}/{}.", sigma, n, sigma, root_n),
                        format!("2. Deviation from mu: {} - {} = {}.", x_bar, mu, delta),
                        format!("3. Standardized CLT statistic: Z = {} * ({}/{}) = {}.", delta, root_n, sigma, disp),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Root-n Sample Mean Standard Error",
                            r"\operatorname{SE}(\bar{x}) = \frac{\sigma}{\sqrt{n}}",
                            format!("Compute standard error {} / sqrt({}) = {}/{}.", sigma, n, sigma, root_n),
                            format!(r"\frac{{{}}}{{\sqrt{{{}}}}}", sigma, n),
                            format!(r"\operatorname{{SE}} = \frac{{{}}}{{{}}}", sigma, root_n),
                            format!("{}/{}", sigma, root_n),
                        ),
                        raw_step(
                            "Compute Deviation from Population Mean",
                            r"\Delta \mu = \bar{x} - \mu",
                            format!("Signed sample deviation is {} - {} = {}.", x_bar, mu, delta),
                            format!(r"{} - {}", x_bar, mu),
                            format!(r"\bar{{x}} - \mu = {}", delta),
                            format!("{}", delta),
                        ),
                        raw_step(
                            "Normalize by Standard Error",
                            r"Z = \frac{\bar{x} - \mu}{\sigma/\sqrt{n}}",
                            format!("Multiply {} by reciprocal {}/{}.", delta, root_n, sigma),
                            format!(r"\frac{{{}}}{{{}/{}}}", delta, sigma, root_n),
                            format!(r"Z = \frac{{{}}}{{{}}}", num, sigma),
                            disp.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "x".to_string(),
                        x_range: Some([(-3.5_f64).min(expected - 0.8), (3.5_f64).max(expected + 0.8)]),
                        primary_expr: Some("exp(-0.5*x^2)/sqrt(2*pi)".to_string()),
                        primary_label: Some("Standard Normal N(0,1) PDF".to_string()),
                        vline_x: Some(expected),
                        vline_label: Some(format!("Z = {:.3}", expected)),
                        ..Default::default()
                    }),
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "sta-mle-confidence" => {
                let n = rng.gen_range(4..=8i32);
                let sum_t = rng.gen_range(10..=25i32);
                if variant == 0 {
                    let expected = format!("{}/x - {}", n, sum_t);
                    let loglik_tex = format!(r"\ell(x) = {}\ln(x) - {}x", n, sum_t);
                    let ans_tex = format!(r"\frac{{{}}}{{x}} - {}", n, sum_t);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::MaximumLikelihood,
                        title: "Log-Likelihood Score Derivative for Rate Parameter".to_string(),
                        prompt: format!(
                            "Given n = {} independent Exponential observations with total waiting time sum(t_i) = {}, the log-likelihood for rate x > 0 is l(x) = {}*ln(x) - {}*x. Compute the score derivative dl/dx (in variable x).",
                            n, sum_t, n, sum_t
                        ),
                        target_equation_latex: format!(r"{},\qquad \frac{{d\ell}}{{dx}}", loglik_tex),
                        answer_latex: format!(r"\frac{{d\ell}}{{dx}} = {}", ans_tex),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            format!("Differentiate {}*ln(x) wrt x to obtain {}/x.", n, n),
                            format!("Differentiate -{}*x wrt x to obtain -{}.", sum_t, sum_t),
                            format!("Combine terms: dl/dx = {}.", expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Log-likelihood sum: l(x) = sum ln(x*exp(-x*t_i)) = {}*ln(x) - {}*x.", n, sum_t),
                            format!("2. Differentiate wrt parameter x: dl/dx = {}.", expected),
                            format!("3. Note d^2l/dx^2 = -{}/x^2 < 0 confirms concavity around MLE x_hat = {}/{}.", n, n, sum_t),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Differentiate Logarithmic Rate Term",
                                r"\frac{d}{dx}\left[n \ln(x)\right] = \frac{n}{x}",
                                format!("Differentiate {}*ln(x) wrt rate x.", n),
                                format!(r"\frac{{d}}{{dx}}\!\left({}\ln x\right)", n),
                                format!(r"\frac{{{}}}{{x}}", n),
                                format!("{}/x", n),
                            ),
                            raw_step(
                                "Differentiate Linear Waiting-Time Term",
                                r"\frac{d}{dx}\left[-\left(\sum t_i\right)x\right] = -\sum t_i",
                                format!("Combine with d/dx [-{}x] = -{}.", sum_t, sum_t),
                                format!(r"\frac{{d}}{{dx}}\!\left({}\ln x - {}x\right)", n, sum_t),
                                ans_tex,
                                expected.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([0.05, ((n as f64 / sum_t as f64) * 3.0).max(1.5)]),
                            primary_expr: Some(format!("{}*ln(x) - {}*x", n, sum_t)),
                            primary_label: Some(format!("l(x) = {}ln(x) - {}x", n, sum_t)),
                            secondary_expr: Some(expected),
                            secondary_label: Some("Score l'(x)".to_string()),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                } else {
                    let expected = (n as f64) / (sum_t as f64);
                    let disp = format!("{}/{}", n, sum_t);
                    let loglik_tex = format!(r"\ell(\lambda) = {}\ln(\lambda) - {}\lambda", n, sum_t);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        operation: OperationKind::MaximumLikelihood,
                        title: "Maximum Likelihood Estimator (MLE) via Calculus".to_string(),
                        prompt: format!(
                            "An experiment records n = {} independent Exponential(lambda) lifetimes whose sum is sum(t_i) = {}. Maximize l(lambda) = {}*ln(lambda) - {}*lambda to find the exact MLE lambda_hat.",
                            n, sum_t, n, sum_t
                        ),
                        target_equation_latex: format!(
                            r"\hat{{\lambda}} = \operatorname{{argmax}}_{{\lambda > 0}} \left({}\ln\lambda - {}\lambda\right)",
                            n, sum_t
                        ),
                        answer_latex: format!(r"\hat{{\lambda}} = \frac{{{}}}{{{}}}", n, sum_t),
                        answer: AnswerFormat::Numeric {
                            expected,
                            tolerance: 1e-5,
                            canonical_display: disp.clone(),
                        },
                        hints: vec![
                            format!("Differentiate log-likelihood wrt lambda: dl/dlambda = {}/lambda - {}.", n, sum_t),
                            format!("Set the score derivative equal to zero: {}/lambda = {}.", n, sum_t),
                            format!("Solve for lambda_hat = {} (reciprocal of sample mean).", disp),
                        ],
                        derivation_steps: vec![
                            format!("1. Score equation: d/dlambda [{}*ln(lambda) - {}*lambda] = {}/lambda - {} = 0.", n, sum_t, n, sum_t),
                            format!("2. Isolate parameter: lambda_hat = {}.", disp),
                            format!("3. Second-derivative test: -{}/lambda^2 < 0 verifies global maximum.", n),
                        ],
                        raw_steps: vec![
                            raw_step(
                                "Compute Log-Likelihood Score Derivative",
                                r"\frac{d\ell}{d\lambda} = \frac{n}{\lambda} - \sum_{i=1}^{n} t_i",
                                "Differentiate additive log-likelihood with respect to rate lambda.",
                                loglik_tex,
                                format!(r"\frac{{d\ell}}{{d\lambda}} = \frac{{{}}}{{\lambda}} - {}", n, sum_t),
                                format!("{}/lambda - {}", n, sum_t),
                            ),
                            raw_step(
                                "Solve Stationary Score Equation",
                                r"\left.\frac{d\ell}{d\lambda}\right|_{\hat{\lambda}} = 0 \implies \hat{\lambda} = \frac{n}{\sum t_i}",
                                format!("Verify concavity d^2l/dlambda^2 = -{}/lambda^2 < 0 and solve for lambda_hat.", n),
                                format!(r"\frac{{{}}}{{\lambda}} - {} = 0", n, sum_t),
                                format!(r"\hat{{\lambda}} = \frac{{{}}}{{{}}}", n, sum_t),
                                disp.clone(),
                            ),
                        ],
                        visual: Some(WalkthroughVisual {
                            var: "x".to_string(),
                            x_range: Some([0.05, ((n as f64 / sum_t as f64) * 3.0).max(1.5)]),
                            primary_expr: Some(format!("{}*ln(x) - {}*x", n, sum_t)),
                            primary_label: Some(format!("l(lambda) = {}ln(lambda) - {}lambda", n, sum_t)),
                            highlight_points: vec![[expected, (n as f64) * expected.ln() - (sum_t as f64) * expected]],
                            vline_x: Some(expected),
                            vline_label: Some(format!("lambda_hat = {:.3}", expected)),
                            ..Default::default()
                        }),
                        remediate_module_id: Some("calc-implicit-opt".to_string()),
                    }
                }
            }

            _ => {
                // sta-hypothesis-regression: 3 noisy points (0, y0), (1, y1), (2, y2) with exact integer OLS line!
                let m = rng.gen_range(1..=4i32);
                let c = rng.gen_range(1..=5i32);
                let e = rng.gen_range(1..=2i32);
                let y0 = c - e;
                let y1 = c + m + 2 * e;
                let y2 = c + 2 * m - e;
                let y_bar = c + m;
                let s_xy = 2 * m;
                let expected = format!("{}*x + {}", m, c);
                let ans_tex = format!(r"{}x + {}", m, c);
                RawProblem {
                    module_id: "sta-hypothesis-regression".to_string(),
                    seed,
                    difficulty: 3,
                    operation: OperationKind::OlsRegression,
                    title: "Ordinary Least-Squares Regression Line".to_string(),
                    prompt: format!(
                        "Fit the ordinary least-squares regression line y_hat(x) = b1*x + b0 to the three noisy data points (0, {}), (1, {}), and (2, {}). Write the linear expression in x.",
                        y0, y1, y2
                    ),
                    target_equation_latex: format!(
                        r"\left\{{(0,{}),\,(1,{}),\,(2,{})\right\}},\qquad \hat{{y}}(x) = b_1 x + b_0",
                        y0, y1, y2
                    ),
                    answer_latex: format!(r"\hat{{y}}(x) = {}", ans_tex),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        format!("Compute means: x_bar = (0 + 1 + 2)/3 = 1, and y_bar = ({} + {} + {})/3 = {}.", y0, y1, y2, y_bar),
                        format!("Compute S_xx = (-1)^2 + 0^2 + 1^2 = 2, and S_xy = (-1)*({} - {}) + 0 + (1)*({} - {}) = {}.", y0, y_bar, y2, y_bar, s_xy),
                        format!("Slope b1 = S_xy / S_xx = {}, intercept b0 = y_bar - b1*x_bar = {}, giving {}.", m, c, expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Sample means: x_bar = 1, y_bar = {}.", y_bar),
                        format!("2. Normal-equation slope: b1 = S_xy / S_xx = {} / 2 = {}.", s_xy, m),
                        format!("3. Intercept: b0 = {} - {}*(1) = {} => y_hat(x) = {}.", y_bar, m, c, expected),
                    ],
                    raw_steps: vec![
                        raw_step(
                            "Sample Means x_bar and y_bar",
                            r"\bar{x} = \frac{1}{n}\sum x_i,\quad \bar{y} = \frac{1}{n}\sum y_i",
                            format!("Means equal x_bar = 1 and y_bar = ({} + {} + {})/3 = {}.", y0, y1, y2, y_bar),
                            format!(r"\left\{{(0,{}),\,(1,{}),\,(2,{})\right\}}", y0, y1, y2),
                            format!(r"\bar{{x}} = 1,\qquad \bar{{y}} = {}", y_bar),
                            format!("x_bar=1, y_bar={}", y_bar),
                        ),
                        raw_step(
                            "Normal-Equation OLS Slope",
                            r"b_1 = \frac{S_{xy}}{S_{xx}} = \frac{\sum (x_i - \bar{x})(y_i - \bar{y})}{\sum (x_i - \bar{x})^{2}}",
                            format!("Evaluate S_xx = 2 and S_xy = {}.", s_xy),
                            format!(r"S_{{xx}} = 2,\quad S_{{xy}} = {}", s_xy),
                            format!(r"b_1 = \frac{{{}}}{{2}} = {}", s_xy, m),
                            format!("b1 = {}", m),
                        ),
                        raw_step(
                            "Back-Solve Regression Intercept",
                            r"b_0 = \bar{y} - b_1 \bar{x}",
                            format!("Intercept b0 = {} - {}*1 = {}.", y_bar, m, c),
                            format!(r"b_0 = {} - {}\cdot 1 = {}", y_bar, m, c),
                            format!(r"\hat{{y}}(x) = {}", ans_tex),
                            expected.clone(),
                        ),
                    ],
                    visual: Some(WalkthroughVisual {
                        var: "x".to_string(),
                        x_range: Some([-0.5, 2.5]),
                        primary_expr: Some(expected),
                        primary_label: Some("OLS y_hat(x)".to_string()),
                        highlight_points: vec![[0.0, y0 as f64], [1.0, y1 as f64], [2.0, y2 as f64]],
                        ..Default::default()
                    }),
                    remediate_module_id: Some("lin-orthogonality-lsq".to_string()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curriculum::build_curriculum;
    use crate::expr::plain_input_to_latex;

    #[test]
    fn all_modules_generate_structured_latex_steps_without_text_duplication() {
        let catalog = build_curriculum();
        let domains = [
            ScienceDomain::General,
            ScienceDomain::Biology,
            ScienceDomain::Physics,
            ScienceDomain::Chemistry,
            ScienceDomain::Ecology,
            ScienceDomain::Microbiology,
        ];
        for spec in &catalog.modules {
            for (d_idx, &domain) in domains.iter().enumerate() {
                for seed_off in 0..4u64 {
                    let seed = (d_idx as u64) * 101 + seed_off + 1;
                    let prob = ProblemGenerator::generate_with_domain(spec.id, seed, domain);
                    assert!(
                        !prob.target_equation_latex.trim().is_empty(),
                        "empty target_equation_latex for {}",
                        spec.id
                    );
                    assert!(
                        !prob.answer_latex.trim().is_empty(),
                        "empty answer_latex for {}",
                        spec.id
                    );
                    assert!(
                        !prob.steps.is_empty(),
                        "empty steps for {}",
                        spec.id
                    );
                    for st in &prob.steps {
                        assert!(!st.rule_name.trim().is_empty(), "empty rule_name in {}", spec.id);
                        assert!(
                            !st.rule_identity_latex.trim().is_empty(),
                            "empty rule_identity_latex in {}",
                            spec.id
                        );
                        assert!(
                            !st.before_latex.trim().is_empty(),
                            "empty before_latex in {}",
                            spec.id
                        );
                        assert!(
                            !st.after_latex.trim().is_empty(),
                            "empty after_latex in {}",
                            spec.id
                        );
                        assert_ne!(
                            st.resulting_expr.trim(),
                            st.explanation.trim(),
                            "step {} in {} must not copy explanation into resulting_expr",
                            st.step_number,
                            spec.id
                        );
                    }
                    assert!(
                        !prob.operation.category_badge().is_empty()
                            && !prob.operation.operator_latex().is_empty()
                    );
                }
            }
        }
    }

    #[test]
    fn expr_and_plain_input_to_latex_render_clean_fractions_radicals_and_trig() {
        let tex = plain_input_to_latex("(3*x^2 + 1) / sqrt(x)").expect("parseable fraction");
        assert!(tex.contains(r"\frac{"));
        assert!(tex.contains(r"\sqrt{x}"));
        let trig = plain_input_to_latex("2*pi/3").expect("parseable pi fraction");
        assert_eq!(trig, r"\frac{2\pi}{3}");
        let vec_tex = plain_input_to_latex("<2*x, -cos(pi*y)>").expect("parseable 2D vector");
        assert!(vec_tex.contains(r"\langle"));
        assert!(vec_tex.contains(r"\pi"));
    }

    #[test]
    fn walkthrough_visuals_cover_integrands_tangents_and_mle_parameters() {
        // Definite-integral modules must provide integrand primary_expr and valid shade_interval [a, b]
        for module_id in ["calc-integrals-ftc", "calc-integration-tech", "sta-continuous-pdf-cdf"] {
            let mut found_shaded = false;
            for seed in 0..8u64 {
                let p = ProblemGenerator::generate(module_id, seed);
                if let Some([a, b]) = p.visual.shade_interval {
                    assert!(a < b, "{} shade_interval must satisfy a < b", module_id);
                    let expr_str = p
                        .visual
                        .primary_expr
                        .as_deref()
                        .unwrap_or_else(|| panic!("{} missing visual.primary_expr", module_id));
                    assert!(
                        Expr::parse(expr_str).is_ok(),
                        "{} visual.primary_expr '{}' failed to parse",
                        module_id,
                        expr_str
                    );
                    found_shaded = true;
                }
            }
            assert!(found_shaded, "{} never produced a shaded integral walkthrough", module_id);
        }

        // Derivative / tangent modules must supply parseable f(x) and consistent tangent slope
        for module_id in ["calc-derivatives-rules", "calc-implicit-opt", "not-calculus-operators"] {
            let mut found_tangent = false;
            for seed in 0..8u64 {
                let p = ProblemGenerator::generate(module_id, seed);
                if let (Some([x0, y0]), Some(slope)) = (p.visual.tangent_point, p.visual.tangent_slope) {
                    let prim = p.visual.primary_expr.as_deref().expect("primary_expr");
                    let ast = Expr::parse(prim).expect("parse primary_expr");
                    let eval_y = ast.eval_1d(&p.visual.var, x0);
                    assert!(
                        (eval_y - y0).abs() < 1e-4,
                        "{} tangent_point y0={} mismatches f({})={}",
                        module_id,
                        y0,
                        x0,
                        eval_y
                    );
                    let h = 1e-6;
                    let fd = (ast.eval_1d(&p.visual.var, x0 + h) - ast.eval_1d(&p.visual.var, x0 - h))
                        / (2.0 * h);
                    assert!(
                        (fd - slope).abs() < 1e-3,
                        "{} tangent_slope={} mismatches finite diff {}",
                        module_id,
                        slope,
                        fd
                    );
                    found_tangent = true;
                }
            }
            assert!(found_tangent, "{} never produced tangent geometry", module_id);
        }

        // sta-mle-confidence must match generated parameters (never hardcoded "5*ln(x) - 12*x")
        let mut saw_dynamic = false;
        for seed in 0..8u64 {
            let p = ProblemGenerator::generate("sta-mle-confidence", seed);
            let prim = p.visual.primary_expr.as_deref().expect("MLE primary_expr");
            if prim != "5*ln(x) - 12*x" {
                saw_dynamic = true;
            }
            assert!(Expr::parse(prim).is_ok());
        }
        assert!(saw_dynamic, "sta-mle-confidence visual.primary_expr appeared static");
    }
}

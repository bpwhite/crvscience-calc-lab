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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionStep {
    pub step_number: usize,
    pub method_name: String,
    pub explanation: String,
    pub resulting_expr: String,
    pub visual_plot_expr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedProblem {
    pub module_id: String,
    pub seed: u64,
    pub difficulty: u8,
    pub domain: ScienceDomain,
    pub domain_context: Option<String>,
    pub title: String,
    pub prompt: String,
    pub answer: AnswerFormat,
    pub hints: Vec<String>,
    pub derivation_steps: Vec<String>,
    pub steps: Vec<SolutionStep>,
    pub visual_var: String,
    pub visual_primary_expr: Option<String>,
    pub remediate_module_id: Option<String>,
}

struct RawProblem {
    module_id: String,
    seed: u64,
    difficulty: u8,
    title: String,
    prompt: String,
    answer: AnswerFormat,
    hints: Vec<String>,
    derivation_steps: Vec<String>,
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

fn method_labels_for_module(module_id: &str) -> &'static [&'static str] {
    match module_id {
        "not-sets-quantifiers" => &[
            "Decode Set-Builder / Function Mapping Arrow Notation",
            "Substitute Inner Rule / Symmetric Radius Bound",
            "Simplify Resulting Interval Endpoint or Composite Expression",
        ],
        "not-indices-sigma-pi" => &[
            "Identify Bound Loop Index and Inclusive Upper Bound",
            "Unroll Product (Pi) or Apply Log-Product Summation Rule",
            "Combine Terms into Closed Form",
        ],
        "not-calculus-operators" => &[
            "Identify Derivative Order or Leibniz Evaluation Bar",
            "Differentiate Symbolic Expression wrt Target Variable FIRST",
            "Apply Post-Derivative Point Evaluation or Simplify",
        ],
        "not-matrix-inner-norms" => &[
            "Unpack Transpose Inner-Product / Squared Euclidean Norm",
            "Multiply Matched Coordinate Entries Component-by-Component",
            "Sum Contracted Terms",
        ],
        "not-nabla-pde-fields" => &[
            "Decode Nabla-Squared (Laplacian) or PDE Coordinate Subscripts",
            "Compute Pure Second Partial Derivatives u_xx and u_yy",
            "Combine Operator Terms",
        ],
        "not-prob-stats-inference" => &[
            "Distinguish argmax Parameter Location vs Peak Value / Apply E[.] Linearity",
            "Identify Stationary Peak Coordinate or Mean Scaling",
            "Extract Exact Target Quantity",
        ],
        "alg-linear-systems" => &[
            "Compute Difference-of-Coordinates Slope",
            "Apply Point-Slope Linearization / Row Scaling",
            "Simplify & Collect Linear Terms",
        ],
        "alg-quadratics" => &[
            "Factor Leading Quadratic Coefficient",
            "Complete the Square ((b/2)^2 Addition-Subtraction)",
            "Read Off Vertex Coordinates & Extremum",
        ],
        "alg-polynomial-rational" => &[
            "Clear Common Rational Denominator",
            "Apply Heaviside Cover-Up Root Substitution",
            "Isolate Partial-Fraction Coefficient",
        ],
        "alg-radicals-exponents" => &[
            "Expand Difference-Quotient / Apply Index Law",
            "Factor Common Step h / Unify Fractional Base",
            "Cancel Removable Factor to Obtain Closed Form",
        ],
        "alg-exp-log" => &[
            "Isolate Exponential Base Factor",
            "Apply Natural Logarithm Inverse ln(exp(u)) = u",
            "Solve Linearized Exponent Equation",
        ],
        "alg-sequences-series" => &[
            "Identify Common Ratio / Factor Summation Linearity",
            "Apply Closed-Form Geometric or Faulhaber Sum Formula",
            "Reduce Rational Sum to Exact Value",
        ],
        "trig-unit-circle" => &[
            "Locate Radian Quadrant & Reference Angle",
            "Evaluate Special Right-Triangle Ratio & Sign",
        ],
        "trig-graphs-waves" => &[
            "Extract Angular Frequency Coefficient omega",
            "Apply Fundamental Period Relation T = 2*pi / omega",
        ],
        "trig-identities" => &[
            "Factor Common Trigonometric Scalar",
            "Apply Pythagorean Secant-Tangent Identity 1 + tan^2 = sec^2",
            "Write Single-Term Trig Reduction",
        ],
        "trig-compound-double" => &[
            "Apply Half-Angle / Power-Reduction Identity",
            "Distribute Scalar Coefficient to Linear Harmonics",
        ],
        "trig-inverse-eqs" => &[
            "Define Reference Angle theta = atan(u) on (-pi/2, pi/2)",
            "Apply Pythagorean Theorem for Right-Triangle Hypotenuse",
            "Read Off Trigonometric Ratio Purely Algebraically",
        ],
        "trig-polar-parametric" => &[
            "Multiply First-Coordinate Components u1 * v1",
            "Multiply Second-Coordinate Components u2 * v2",
            "Sum Components for Euclidean Inner Product u . v",
        ],
        "calc-limits-continuity" => &[
            "Factor Conjugate / Rescale Small-Angle Argument",
            "Cancel 0/0 Indeterminate Factor / Apply Fundamental Limit",
            "Evaluate Continuous Limit Expression",
        ],
        "calc-derivatives-rules" => &[
            "Differentiate Outer / First Factor via Power or Exp/Log Rule",
            "Apply Chain Rule to Inner Composite Function",
            "Combine via Product / Quotient Rule",
        ],
        "calc-implicit-opt" => &[
            "Differentiate Implicitly wrt x Using Chain Rule on y(x)",
            "Algebraically Isolate Derivative Slope dy/dx",
            "Evaluate Tangent Slope at Given Curve Coordinates",
        ],
        "calc-integrals-ftc" => &[
            "Construct Power-Rule Antiderivative F(x)",
            "Apply Fundamental Theorem of Calculus Part 2: F(b) - F(a)",
        ],
        "calc-integration-tech" => &[
            "Select u-Substitution or Integration by Parts (u, dv)",
            "Transform Differential & Definite Bounds / Apply u*v - integral v du",
            "Evaluate Elementary Antiderivative",
        ],
        "calc-series-odes" => &[
            "Evaluate Higher-Order Derivatives f^(k)(0) at Anchor",
            "Divide by Factorials k! to Assemble Maclaurin Polynomial",
        ],
        "lin-vectors-spans" => &[
            "Assemble 2x2 Basis Coordinate Linear System",
            "Apply Gaussian Row Elimination",
            "Back-Substitute for Target Basis Coordinate Weight",
        ],
        "lin-matrices-maps" => &[
            "Compute Row-1 Inner Product with Input Vector",
            "Compute Row-2 Inner Product with Input Vector",
            "Assemble Transformed Image Vector A * v",
        ],
        "lin-det-inverses" => &[
            "Evaluate 2x2 Area Determinant det(A) = a*d - b*c",
            "Form Adjugate Cofactor Matrix",
            "Divide by det(A) for Exact Inverse Entry",
        ],
        "lin-orthogonality-lsq" => &[
            "Compute Numerator Inner Product y . u",
            "Compute Squared Basis Norm u . u",
            "Form Orthogonal Projection Ratio (y . u) / (u . u)",
        ],
        "lin-eigenvalues-diag" => &[
            "Compute Matrix Invariants tr(A) and det(A)",
            "Construct Characteristic Quadratic lambda^2 - tr(A)*lambda + det(A) = 0",
            "Factor Roots to Extract Dominant Eigenvalue",
        ],
        "lin-symmetric-quadratic" => &[
            "Multiply Symmetric Matrix A Against Column Vector [x, y]^T",
            "Take Inner Product with [x, y]",
            "Combine Symmetric Cross-Terms 2*b*x*y",
        ],
        "adv-real-analysis" => &[
            "Express Output Deviation |f(x) - L| in Terms of |x - x0|",
            "Impose Target Bound |f(x) - L| < epsilon",
            "Isolate |x - x0| to Pin Optimal Uniform Delta",
        ],
        "adv-partial-grad" => &[
            "Differentiate Partially wrt x (Holding y Constant)",
            "Differentiate Partially wrt y (Holding x Constant)",
            "Package Partial Derivatives into Gradient Vector <f_x, f_y>",
        ],
        "adv-lagrange-hessian" => &[
            "Compute First Partial Derivatives f_x and f_y",
            "Compute Second & Mixed Partials f_xx, f_yy, f_xy",
            "Evaluate Symmetric Hessian Discriminant D = f_xx*f_yy - f_xy^2",
        ],
        "adv-multiple-integrals" => &[
            "Convert Region & Integrand to Polar Coordinates (dA = r dr dtheta)",
            "Integrate Inner Radial Power r^3 dr",
            "Integrate Outer Angular Factor Over [0, 2*pi]",
        ],
        "adv-vector-fields" => &[
            "Compute Horizontal Flux Derivative dP/dx",
            "Compute Vertical Flux Derivative dQ/dy",
            "Sum Partial Rates for Scalar Divergence div F",
        ],
        "adv-stokes-fourier" => &[
            "Convert Closed Line Integral to Double Integral via Green's Theorem",
            "Compute Constant 2D Vorticity Curl dQ/dx - dP/dy",
            "Multiply Constant Curl by Enclosed Disk Area pi*R^2",
        ],
        "deq-first-order" => &[
            "Compute Integrating Factor mu(t) = exp(integral p(t) dt)",
            "Integrate Product-Rule Derivative for General Solution",
            "Apply Initial Condition y(0) to Pin Constant C",
        ],
        "deq-second-order-osc" => &[
            "Solve Characteristic Quadratic m*r^2 + c*r + k = 0",
            "Form Homogeneous Basis Superposition",
            "Match Initial Position y(0) and Velocity y'(0)",
        ],
        "deq-linear-systems-phase" => &[
            "Compute State-Matrix Trace tr(A) and Determinant det(A)",
            "Evaluate Parabola Discriminant Delta = tr(A)^2 - 4*det(A)",
            "Classify Real Node/Saddle vs Complex Spiral/Center Regime",
        ],
        "deq-laplace-nonlinear" => &[
            "Apply Laplace Transform Pair for Exponential Mode",
            "Apply Laplace Transform Pair for Cosine Mode",
            "Combine Rational s-Domain Transfer Terms by Linearity",
        ],
        "deq-heat-diffusion-pde" => &[
            "Evaluate Spatial Laplacian Eigenvalue on sin(n*x)",
            "Solve Temporal Decay ODE T'(t) = -alpha*n^2 * T(t)",
            "Assemble Decaying Dirichlet Eigenmode u(x, t)",
        ],
        "deq-wave-laplace-pde" => &[
            "Compute Quantized Standing-Wave Frequency omega_n = c * n",
            "Eliminate Sine Time Mode Using Zero Initial Velocity u_t(x,0) = 0",
            "Assemble Harmonic Standing-Wave Solution u(x, t)",
        ],
        "sta-combinatorics-bayes" => &[
            "Compute True-Positive Joint Rate P(+ | H) * P(H)",
            "Apply Law of Total Probability for Marginal P(+)",
            "Apply Bayes' Theorem Ratio P(H | +)",
        ],
        "sta-discrete-rv" => &[
            "Identify Poisson Rate Parameter lambda and Count k",
            "Compute Combinatorial Power Factor lambda^k / k!",
            "Multiply by Zero-Event Decay Factor exp(-lambda)",
        ],
        "sta-continuous-pdf-cdf" => &[
            "Set Up Definite Integral of Continuous Density f(t)",
            "Find Exponential Antiderivative",
            "Apply Fundamental Theorem Across Integration Bounds",
        ],
        "sta-joint-cov-clt" => &[
            "Apply Root-n Law for Sample-Mean Standard Error sigma / sqrt(n)",
            "Compute Sample Deviation x_bar - mu",
            "Normalize by Standard Error for CLT Z-Score",
        ],
        "sta-mle-confidence" => &[
            "Construct Additive Log-Likelihood Function l(lambda)",
            "Differentiate wrt Parameter & Set Score Equation dl/dlambda = 0",
            "Verify Concavity d^2l/dlambda^2 < 0 at Maximum Likelihood Estimate",
        ],
        _ => &[
            "Compute Sample Means x_bar and y_bar",
            "Apply Normal-Equation Covariance Ratio b1 = S_xy / S_xx",
            "Back-Solve Intercept b0 = y_bar - b1 * x_bar",
        ],
    }
}

fn infer_visual_curves(raw: &RawProblem) -> (String, Option<String>, Option<String>) {
    match &raw.answer {
        AnswerFormat::ScalarExpr { expected_expr, vars } => {
            if vars.len() == 1 {
                let v = vars[0].clone();
                // Also try to plot an antiderivative/derivative companion when single-variable
                let primary = if let Ok(ast) = Expr::parse(expected_expr) {
                    let d = ast.differentiate(&v).to_plain_string();
                    Some(d)
                } else {
                    None
                };
                return (v, primary, Some(expected_expr.clone()));
            } else if vars.len() == 2 && vars.contains(&"t".to_string()) && vars.contains(&"x".to_string()) {
                // Evaluate spatial slice at t = 0.25 vs t = 0.0 for PDEs!
                let slice_0 = expected_expr.replace("*t)", "*0.0)");
                let slice_t = expected_expr.replace("*t)", "*0.25)");
                return ("x".to_string(), Some(slice_0), Some(slice_t));
            }
            ("x".to_string(), None, None)
        }
        AnswerFormat::Vector2Expr { expected_p, expected_q, .. } => {
            let p1 = expected_p.replace("y", "1.0");
            let q1 = expected_q.replace("y", "1.0");
            ("x".to_string(), Some(p1), Some(q1))
        }
        AnswerFormat::Numeric { expected, .. } => {
            // Provide stage-appropriate illustrative curve anchored at the numeric result
            if raw.module_id == "sta-mle-confidence" {
                return ("x".to_string(), Some("5*ln(x) - 12*x".to_string()), Some(format!("{:.4}", expected)));
            } else if raw.module_id.starts_with("sta-") {
                return ("x".to_string(), Some("exp(-0.5*x^2)/sqrt(2*pi)".to_string()), Some(format!("{:.4}", expected)));
            }
            ("x".to_string(), None, Some(format!("{:.4}", expected)))
        }
    }
}

fn finalize_problem(raw: RawProblem, domain: ScienceDomain) -> GeneratedProblem {
    let stage = module_stage(&raw.module_id);
    let domain_context = domain.context_banner(stage, &raw.module_id);
    let labels = method_labels_for_module(&raw.module_id);
    let (visual_var, visual_primary_expr, final_curve) = infer_visual_curves(&raw);

    let total = raw.derivation_steps.len();
    let mut steps = Vec::with_capacity(total);
    for (idx, line) in raw.derivation_steps.iter().enumerate() {
        let clean_line = line
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' ')
            .to_string();
        let method_name = labels
            .get(idx)
            .copied()
            .unwrap_or("Algebraic Simplification & Verification")
            .to_string();
        let resulting_expr = if idx + 1 == total {
            raw.answer.canonical()
        } else {
            clean_line.clone()
        };
        let visual_plot_expr = if idx + 1 == total {
            final_curve.clone()
        } else {
            visual_primary_expr.clone()
        };
        steps.push(SolutionStep {
            step_number: idx + 1,
            method_name,
            explanation: clean_line,
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
        title: raw.title,
        prompt: raw.prompt,
        answer: raw.answer,
        hints: raw.hints,
        derivation_steps: raw.derivation_steps,
        steps,
        visual_var,
        visual_primary_expr,
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
                    let upper = center + radius;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        title: "Unpack Set-Builder Metric Neighborhood Notation".to_string(),
                        prompt: format!(
                            "The set S = {{ x ∈ ℝ : |x - ({})| <= {} }} equals a closed interval [L, U]. Find the exact upper endpoint U.",
                            center, radius
                        ),
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
                            format!("2. Shift by center {}: L = {}, U = {} + {}.", center, center - radius, center, radius),
                            format!("3. Closed interval is [{}, {}], giving upper endpoint U = {}.", center - radius, upper, upper),
                        ],
                        remediate_module_id: None,
                    }
                } else {
                    let a = rng.gen_range(2..=5i32);
                    let b = rng.gen_range(1..=6i32);
                    let expected = format!("{}*x^2 + {}", a, b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        title: "Decode Element-Mapping (↦) & Composition (∘) Notation".to_string(),
                        prompt: format!(
                            "Given f : ℝ → ℝ, u ↦ {}*u + {} and g : ℝ → ℝ, x ↦ x^2, write the plain-text expression in x for the composition (f ∘ g)(x).",
                            a, b
                        ),
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
                        remediate_module_id: None,
                    }
                }
            }

            "not-indices-sigma-pi" => {
                if variant == 0 {
                    let c = rng.gen_range(1..=3i32);
                    // prod_{k=1..3} (k + c) = (1+c)*(2+c)*(3+c)
                    let val = (1 + c) * (2 + c) * (3 + c);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        title: "Unroll Capital-Pi (∏) Product Notation".to_string(),
                        prompt: format!(
                            "Evaluate the Capital-Pi product expression:  ∏_{{k=1}}^{{3}} (k + {}).",
                            c
                        ),
                        answer: AnswerFormat::Numeric {
                            expected: val as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", val),
                        },
                        hints: vec![
                            "Capital Pi ∏ multiplies each indexed factor from k = 1 up to k = 3.".to_string(),
                            format!("Unroll factors: (1 + {}) * (2 + {}) * (3 + {}).", c, c, c),
                            format!("Multiply {} * {} * {} = {}.", 1 + c, 2 + c, 3 + c, val),
                        ],
                        derivation_steps: vec![
                            format!("1. Substitute dummy index k = 1, 2, 3: factors are {}, {}, and {}.", 1 + c, 2 + c, 3 + c),
                            format!("2. Multiply sequentially: {} * {} = {}, times {} = {}.", 1 + c, 2 + c, (1 + c) * (2 + c), 3 + c, val),
                            format!("3. Exact product value: {}.", val),
                        ],
                        remediate_module_id: Some("not-sets-quantifiers".to_string()),
                    }
                } else {
                    let a = rng.gen_range(2..=4i32);
                    let n = rng.gen_range(3..=5i32);
                    // ln( prod_{k=1..n} exp(a*k*x) ) = sum_{k=1..n} a*k*x = a*(n*(n+1)/2)*x
                    let coeff = a * n * (n + 1) / 2;
                    let expected = format!("{}*x", coeff);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Convert Log-of-Product ln(∏) into Sigma Sum".to_string(),
                        prompt: format!(
                            "Simplify the indexed notation  ln( ∏_{{k=1}}^{{{}}} exp({}*k*x) )  into a single plain-text expression in x (with no dummy index k remaining).",
                            n, a
                        ),
                        answer: AnswerFormat::ScalarExpr {
                            expected_expr: expected.clone(),
                            vars: vec!["x".to_string()],
                        },
                        hints: vec![
                            "Use the log-product rule: ln(∏ a_k) = ∑ ln(a_k).".to_string(),
                            format!("Since ln(exp({}*k*x)) = {}*k*x, this becomes ∑_{{k=1..{}}} {}*k*x.", a, a, n, a),
                            format!("Sum k from 1 to {} (= {}), giving {}.", n, n * (n + 1) / 2, expected),
                        ],
                        derivation_steps: vec![
                            format!("1. Convert ln(∏) to ∑ ln: sum_{{k=1..{}}} ln(exp({}*k*x)) = sum_{{k=1..{}}} {}*k*x.", n, a, n, a),
                            format!("2. Factor free variable x and constant {}: {}*x * (1 + ... + {}).", a, a, n),
                            format!("3. Evaluate index sum {}*{} = {} => {}.", a, n * (n + 1) / 2, coeff, expected),
                        ],
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
                        title: "Decode Leibniz Derivative Evaluation Bar |_{x=a}".to_string(),
                        prompt: format!(
                            "Evaluate the Leibniz bar notation:   d/dx [ {}*x^3 + {}*x ] |_{{x = {}}}.",
                            a, b, x0
                        ),
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
                        remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                    }
                } else {
                    let a = rng.gen_range(1..=3i32);
                    let b = rng.gen_range(1..=5i32);
                    // f(x) = a*x^3 + b*x^2 => f^(2)(x) = 6*a*x + 2*b
                    let expected = format!("{}*x + {}", 6 * a, 2 * b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Distinguish Lagrange Superscript f^(2)(x) from Squaring".to_string(),
                        prompt: format!(
                            "For f(x) = {}*x^3 + {}*x^2, decode the parenthesized superscript notation f^(2)(x) and write its plain-text expression in x.",
                            a, b
                        ),
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
                    title: "Evaluate Squared Double-Bar Norm ‖u‖₂² = uᵀ u".to_string(),
                    prompt: format!(
                        "For column vector u = [{}, {}, {}]ᵀ ∈ ℝ³, evaluate the squared Euclidean norm notation ‖u‖₂² = ⟨u, u⟩ = uᵀ u.",
                        u1, u2, u3
                    ),
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
                    remediate_module_id: Some("not-indices-sigma-pi".to_string()),
                }
            }

            "not-nabla-pde-fields" => {
                let a = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=4i32);
                // u(x, y) = a*x^3 + b*y^3 => nabla^2 u = u_xx + u_yy = 6*a*x + 6*b*y
                let expected = format!("{}*x + {}*y", 6 * a, 6 * b);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Unpack Laplacian Symbol ∇²u = u_{xx} + u_{yy}".to_string(),
                    prompt: format!(
                        "For the 2D scalar field u(x, y) = {}*x^3 + {}*y^3, unpack and evaluate the Laplacian notation ∇²u (equivalently written u_{{xx}} + u_{{yy}}) as a plain-text expression in x and y.",
                        a, b
                    ),
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
                        title: "Decode argmax_{θ} vs max_{θ} Estimator Notation".to_string(),
                        prompt: format!(
                            "Evaluate the parameter-estimator notation:   θ̂ = argmax_{{θ ∈ ℝ}} ( {} - {}*(θ - ({}))^2 ).",
                            peak_height, scale, theta_star
                        ),
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
                        title: "Decode Distribution (~) & Expectation E[·] Notation".to_string(),
                        prompt: format!(
                            "Given the distributional declaration X ~ N({}, {}), evaluate the expectation operator E[ {}*X + ({}) ].",
                            mu, sigma_sq, a, b
                        ),
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
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 1,
                        title: "Linear Function from Two Points".to_string(),
                        prompt: format!(
                            "Find the linear function f(x) passing through P1 = ({}, {}) and P2 = ({}, {}).",
                            x1, y1, x2, y2
                        ),
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
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "2x2 Linear System Intersection".to_string(),
                        prompt: format!(
                            "Solve the linear system for the x-coordinate of intersection:\n  {}*x + {}*y = {}\n  {}*x - {}*y = {}",
                            a1, b1, c1, a2, -b2, c2
                        ),
                        answer: AnswerFormat::Numeric {
                            expected: sol_x as f64,
                            tolerance: 1e-5,
                            canonical_display: format!("{}", sol_x),
                        },
                        hints: vec![
                            format!("Eliminate y by multiplying the first equation by {} and the second by {}, then adding.", -b2, b1),
                            format!("Combined x equation: ({}*{} + {}*{})*x = {}*{} + {}*{}.", a1, -b2, a2, b1, c1, -b2, c2, b1),
                            format!("Divide both sides by {} to isolate x.", a1 * (-b2) + a2 * b1),
                        ],
                        derivation_steps: vec![
                            format!("1. Scale Eq1 by {} and Eq2 by {} so y-coefficients cancel.", -b2, b1),
                            format!("2. Sum equations: {}*x = {}.", a1 * (-b2) + a2 * b1, c1 * (-b2) + c2 * b1),
                            format!("3. Intersection coordinate: x = {} (with y = {}).", sol_x, sol_y),
                        ],
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
                if variant == 0 {
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Vertex Minimum via Completing the Square".to_string(),
                        prompt: format!(
                            "Complete the square on q(x) = {}*x^2 + ({})*x + ({}) and find its minimum output value k (the y-coordinate of the vertex).",
                            a, b, c
                        ),
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
                        remediate_module_id: Some("alg-linear-systems".to_string()),
                    }
                } else {
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Rewrite Quadratic in Vertex Form".to_string(),
                        prompt: format!(
                            "Write an expression equivalent to {}*x^2 + ({})*x + ({}) in completed-square vertex form.",
                            a, b, c
                        ),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Partial Fraction Coefficient Cover-Up".to_string(),
                    prompt: format!(
                        "Decompose ({}*x + ({})) / ((x - {})*(x + {})) = A / (x - {}) + B / (x + {}). Find the exact value of A.",
                        num_m, num_c, r1, -r2, r1, -r2
                    ),
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
                    remediate_module_id: Some("alg-quadratics".to_string()),
                }
            }

            "alg-radicals-exponents" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(-4..=5i32);
                if variant == 0 {
                    let clean_ans = fmt_add_const(&format!("{}*x + {}*h", 2 * a, a), b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Simplify Algebraic Difference Quotient".to_string(),
                        prompt: format!(
                            "For f(x) = {}*x^2 + ({})*x, simplify the difference quotient (f(x + h) - f(x)) / h for h != 0 (in variables x and h).",
                            a, b
                        ),
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
                        title: "Convert Radical Quotient to Power Form".to_string(),
                        prompt: format!(
                            "Rewrite x^{} / x^(1/{}) as a single power of x (valid for x > 0).",
                            p, q
                        ),
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
                    title: "Solve Exponential Growth Equation".to_string(),
                    prompt: format!(
                        "Solve exactly for t:  3 * exp({}*t) = {}. (You may enter exact plain-text form like ln(a)/b.)",
                        k, 3 * ratio
                    ),
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
                        title: "Infinite Geometric Series Sum".to_string(),
                        prompt: format!(
                            "Evaluate the convergent infinite geometric series: sum_{{k=0..inf}} {} * ({}/{})^k.",
                            a, num, den
                        ),
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
                        remediate_module_id: Some("alg-polynomial-rational".to_string()),
                    }
                } else {
                    let n = rng.gen_range(6..=14i32);
                    let c = rng.gen_range(2..=4i32);
                    let val = c * n * (n + 1) * (2 * n + 1) / 6;
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Finite Quadratic Sigma Sum".to_string(),
                        prompt: format!("Evaluate the finite sum: sum_{{k=1..{}}} {} * k^2.", n, c),
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
                            format!("2. Sum of squares 1..{}: {}.", n, n * (n + 1) * (2 * n + 1) / 6),
                            format!("3. Multiply by {}: {}.", c, val),
                        ],
                        remediate_module_id: None,
                    }
                }
            }

            // =====================================================================
            // STAGE 2: TRIGONOMETRY
            // =====================================================================
            "trig-unit-circle" => {
                let choices: &[(&str, f64, &str, &str)] = &[
                    ("cos(2*pi/3)", -0.5, "-1/2", "Quadrant II reference angle pi/3 has cosine -1/2"),
                    ("sin(5*pi/6)", 0.5, "1/2", "Quadrant II reference angle pi/6 has sine +1/2"),
                    ("cos(5*pi/4)", -std::f64::consts::FRAC_1_SQRT_2, "-sqrt(2)/2", "Quadrant III reference angle pi/4 has cosine -sqrt(2)/2"),
                    ("sin(4*pi/3)", -(3.0_f64).sqrt() / 2.0, "-sqrt(3)/2", "Quadrant III reference angle pi/3 has sine -sqrt(3)/2"),
                    ("tan(3*pi/4)", -1.0, "-1", "Quadrant II angle 3*pi/4 has sin = sqrt(2)/2, cos = -sqrt(2)/2"),
                ];
                let pick = choices[rng.gen_range(0..choices.len())];
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    title: "Exact Unit Circle Coordinate".to_string(),
                    prompt: format!("Evaluate {} exactly (enter a fraction or radical such as -sqrt(3)/2).", pick.0),
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
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "trig-graphs-waves" => {
                let amp = rng.gen_range(2..=7i32);
                let omega = rng.gen_range(2..=6i32);
                let expected = 2.0 * std::f64::consts::PI / (omega as f64);
                let disp = format!("2*pi/{}", omega);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    title: "Fundamental Period of a Harmonic Wave".to_string(),
                    prompt: format!(
                        "Find the fundamental period T of y(t) = {} * cos({}*t - pi/3). (Enter exact form in terms of pi.)",
                        amp, omega
                    ),
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
                    remediate_module_id: Some("trig-unit-circle".to_string()),
                }
            }

            "trig-identities" => {
                let k = rng.gen_range(2..=6i32);
                let expected = format!("{} * sec(x)^2", k);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Pythagorean Secant-Tangent Reduction".to_string(),
                    prompt: format!(
                        "Simplify {} + {} * tan(x)^2 into a single trigonometric term.",
                        k, k
                    ),
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
                    remediate_module_id: Some("trig-unit-circle".to_string()),
                }
            }

            "trig-compound-double" => {
                let m = rng.gen_range(1..=4i32);
                let coeff = 2 * rng.gen_range(1..=4i32);
                let half = coeff / 2;
                let expected = format!("{} * (1 - cos({}*x))", half, 2 * m);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Power-Reduction Linearization".to_string(),
                    prompt: format!(
                        "Rewrite {} * sin({}*x)^2 using the power-reduction identity (no exponents > 1) to prepare it for integration.",
                        coeff, m
                    ),
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
                    remediate_module_id: Some("trig-identities".to_string()),
                }
            }

            "trig-inverse-eqs" => {
                let a = rng.gen_range(2..=5i32);
                let expected = format!("({}*x) / sqrt(1 + {}*x^2)", a, a * a);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Algebraic Reduction of Trig-Inverse Composition".to_string(),
                    prompt: format!(
                        "Express sin(atan({}*x)) as a purely algebraic expression in x.",
                        a
                    ),
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
                    remediate_module_id: Some("trig-identities".to_string()),
                }
            }

            "trig-polar-parametric" => {
                let u1 = rng.gen_range(-4..=5i32);
                let u2 = rng.gen_range(1..=5i32);
                let v1 = rng.gen_range(-4..=5i32);
                let v2 = rng.gen_range(-4..=5i32);
                let dot = u1 * v1 + u2 * v2;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 1,
                    title: "2D Vector Dot Product & Orthogonality Check".to_string(),
                    prompt: format!(
                        "Compute the scalar dot product u . v for u = <{}, {}> and v = <{}, {}>.",
                        u1, u2, v1, v2
                    ),
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
                        format!("1. u1*v1 = ({})*({}) = {}.", u1, v1, u1 * v1),
                        format!("2. u2*v2 = ({})*({}) = {}.", u2, v2, u2 * v2),
                        format!("3. Dot product u . v = {} + ({}) = {}.", u1 * v1, u2 * v2, dot),
                    ],
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
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Indeterminate 0/0 Radical Limit".to_string(),
                        prompt: format!(
                            "Evaluate exactly: lim_{{x -> {}}} (sqrt(x) - {}) / (x - {}).",
                            a2, a, a2
                        ),
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
                        remediate_module_id: Some("alg-radicals-exponents".to_string()),
                    }
                } else {
                    let p = rng.gen_range(3..=8i32);
                    let q = rng.gen_range(2..=6i32);
                    let expected = (p as f64) / (q as f64);
                    let disp = format!("{}/{}", p, q);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Small-Angle Trigonometric Limit".to_string(),
                        prompt: format!("Evaluate exactly: lim_{{x -> 0}} sin({}*x) / ({}*x).", p, q),
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
                        remediate_module_id: Some("trig-unit-circle".to_string()),
                    }
                }
            }

            "calc-derivatives-rules" => {
                if variant == 0 {
                    let a = rng.gen_range(2..=5i32);
                    let b = rng.gen_range(1..=4i32);
                    let expected = format!("exp({}*x) * ({}*sin({}*x) + {}*cos({}*x))", a, a, b, b, b);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Product & Chain Rule Differentiation".to_string(),
                        prompt: format!(
                            "Compute the derivative f'(x) of f(x) = exp({}*x) * sin({}*x).",
                            a, b
                        ),
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
                        remediate_module_id: Some("alg-exp-log".to_string()),
                    }
                } else {
                    let k = rng.gen_range(2..=6i32);
                    let p = rng.gen_range(2..=4i32);
                    let expected = format!("({}*x^{}) / (1 + {}*x^{})", k * p, p - 1, k, p);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Logarithmic Chain Rule".to_string(),
                        prompt: format!("Differentiate with respect to x: f(x) = ln(1 + {}*x^{}).", k, p),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Implicit Tangent Slope on an Ellipse".to_string(),
                    prompt: format!(
                        "Find the exact slope dy/dx of the curve {}*x^2 + {}*y^2 = {} at the point ({}, {}).",
                        a, b, rhs, x0, y0
                    ),
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
                    remediate_module_id: Some("calc-derivatives-rules".to_string()),
                }
            }

            "calc-integrals-ftc" => {
                let a = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=5i32);
                let upper = rng.gen_range(2..=4i32);
                let val = a * upper.pow(3) + b * upper.pow(2);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Definite Integral via Fundamental Theorem".to_string(),
                    prompt: format!(
                        "Evaluate the definite integral: integral_{{0..{}}} ({}*x^2 + {}*x) dx.",
                        upper, 3 * a, 2 * b
                    ),
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
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "calc-integration-tech" => {
                if variant == 0 {
                    let k = rng.gen_range(2..=5i32);
                    let expected = ((k as f64).exp() - 1.0) / (k as f64);
                    let disp = format!("(exp({}) - 1)/{}", k, k);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Definite Integral by u-Substitution".to_string(),
                        prompt: format!(
                            "Evaluate exactly: integral_{{0..1}} 2*x * exp({}*x^2) dx. (You may enter exact form in terms of exp(...) or e.)",
                            k
                        ),
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
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                } else {
                    let a = rng.gen_range(1..=4i32);
                    let expected = format!("exp({}*x) * ({}*x - 1) / {}", a, a, a * a);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Integration by Parts Antiderivative".to_string(),
                        prompt: format!(
                            "Find the antiderivative F(x) of f(x) = x * exp({}*x) satisfying F(0) = -1/{} (i.e. constant C = 0).",
                            a, a * a
                        ),
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
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                }
            }

            "calc-series-odes" => {
                let k = rng.gen_range(2..=4i32);
                let k2 = k * k;
                let k3 = k * k * k;
                let expected = format!("1 + {}*x + ({}/2)*x^2 + ({}/6)*x^3", k, k2, k3);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Cubic Maclaurin Polynomial Approximation".to_string(),
                    prompt: format!(
                        "Write the degree-3 Maclaurin polynomial T_3(x) centered at x = 0 for f(x) = exp({}*x).",
                        k
                    ),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Exact Epsilon-Delta Continuity Threshold".to_string(),
                    prompt: format!(
                        "For f(x) = {}*x + ({}) near x0 = 2 with target tolerance epsilon = {:.1}, find the largest delta > 0 such that |x - 2| < delta guarantees |f(x) - f(2)| < {:.1}.",
                        m, b, (eps_num as f64) / 10.0, (eps_num as f64) / 10.0
                    ),
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
                    remediate_module_id: Some("calc-limits-continuity".to_string()),
                }
            }

            "adv-partial-grad" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(1..=4i32);
                let p = format!("{}*x*y", 2 * a);
                let q = format!("{}*x^2 + {}*y^2", a, 3 * b);
                let canon = format!("<{}, {}>", p, q);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Symbolic Gradient Vector Field".to_string(),
                    prompt: format!(
                        "Compute the gradient vector grad f(x, y) = <df/dx, df/dy> for f(x, y) = {}*x^2*y + {}*y^3.",
                        a, b
                    ),
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
                    remediate_module_id: Some("calc-derivatives-rules".to_string()),
                }
            }

            "adv-lagrange-hessian" => {
                let a = rng.gen_range(2..=5i32);
                let b = rng.gen_range(2..=5i32);
                let c = rng.gen_range(1..=3i32);
                let det = 4 * a * b - 4 * c * c;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "2D Hessian Discriminant Calculation".to_string(),
                    prompt: format!(
                        "Compute the Hessian determinant D = f_xx * f_yy - (f_xy)^2 at (0, 0) for f(x, y) = {}*x^2 + {}*y^2 + {}*x*y.",
                        a, b, 2 * c
                    ),
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
                    remediate_module_id: Some("adv-partial-grad".to_string()),
                }
            }

            "adv-multiple-integrals" => {
                let rad = rng.gen_range(2..=4i32);
                let k = rng.gen_range(1..=3i32);
                let r4 = rad.pow(4);
                let expected = 0.5 * (k as f64) * (r4 as f64) * std::f64::consts::PI;
                let disp = format!("({}*pi)/2", k * r4);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Polar Double Integral Over a Disk".to_string(),
                    prompt: format!(
                        "Evaluate double_integral_D {}*(x^2 + y^2) dA over the disk D = {{ x^2 + y^2 <= {} }}. (Include the polar Jacobian r and express in terms of pi.)",
                        k, rad * rad
                    ),
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
                    remediate_module_id: Some("trig-polar-parametric".to_string()),
                }
            }

            "adv-vector-fields" => {
                let a = rng.gen_range(2..=6i32);
                let b = rng.gen_range(1..=5i32);
                let coeff = 2 * (a + b);
                let expected = format!("{}*x*y", coeff);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Divergence of a 2D Vector Field".to_string(),
                    prompt: format!(
                        "Compute the scalar divergence div F(x, y) = dP/dx + dQ/dy for F(x, y) = <{}*x^2*y, {}*x*y^2>.",
                        a, b
                    ),
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
                RawProblem {
                    module_id: "adv-stokes-fourier".to_string(),
                    seed,
                    difficulty: 4,
                    title: "Counterclockwise Circulation via Green's Theorem".to_string(),
                    prompt: format!(
                        "Use Green's Theorem to evaluate the counterclockwise line integral oint_C (-{}*y dx + {}*x dy) around the circle x^2 + y^2 = {}. (Express in terms of pi.)",
                        p_c, q_c, rad * rad
                    ),
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
                    remediate_module_id: Some("adv-vector-fields".to_string()),
                }
            }

            // =====================================================================
            // BRIDGE STAGE: LINEAR ALGEBRA & EIGEN-SYSTEMS (6 Modules)
            // =====================================================================
            "lin-vectors-spans" => {
                let a = rng.gen_range(1..=3i32);
                let b = rng.gen_range(1..=3i32);
                // Basis u = <1, a>, v = <b, a*b + 1> (det = 1 so unique integer coords!)
                let c1 = rng.gen_range(-4..=5i32);
                let c2 = rng.gen_range(-3..=4i32);
                let c2 = if c2 == 0 { 2 } else { c2 };
                let wx = c1 + c2 * b;
                let wy = c1 * a + c2 * (a * b + 1);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Coordinate Weights in a 2D Basis".to_string(),
                    prompt: format!(
                        "Express w = <{}, {}> as a linear combination c1*u + c2*v of basis vectors u = <1, {}> and v = <{}, {}>. Find the exact scalar weight c1.",
                        wx, wy, a, b, a * b + 1
                    ),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Symbolic Linear Map Action A * v".to_string(),
                    prompt: format!(
                        "For matrix A = [[{}, {}], [{}, {}]] and input vector v = <x, y>, write the transformed output vector A * v = <P(x,y), Q(x,y)>.",
                        a11, a12, a21, a22
                    ),
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
                    remediate_module_id: Some("lin-vectors-spans".to_string()),
                }
            }

            "lin-det-inverses" => {
                let a = rng.gen_range(2..=5i32);
                let d = rng.gen_range(2..=5i32);
                let b = rng.gen_range(1..=3i32);
                let c = rng.gen_range(1..=2i32);
                // Ensure nonzero determinant a*d - b*c
                let det = a * d - b * c;
                let (d_adj, det_adj) = if det == 0 { (d + 1, a * (d + 1) - b * c) } else { (d, det) };
                let (num, den) = if det_adj > 0 {
                    (d_adj, det_adj)
                } else {
                    (-d_adj, -det_adj)
                };
                let expected = (num as f64) / (den as f64);
                let disp = format!("{}/{}", num, den);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "2x2 Matrix Inverse Entry & Area Scaling".to_string(),
                    prompt: format!(
                        "Let A = [[{}, {}], [{}, {}]]. Compute the top-left (1, 1) entry of the inverse matrix A^-1.",
                        a, b, c, d_adj
                    ),
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
                    title: "Orthogonal Projection Coefficient".to_string(),
                    prompt: format!(
                        "Inproj_u(y) = c * u, find the exact projection scalar c = (y . u) / (u . u) for y = <{}, {}> onto u = <{}, {}>.",
                        y1, y2, u1, u2
                    )
                    .replace("Inproj_u", "In proj_u"),
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
                    remediate_module_id: Some("trig-polar-parametric".to_string()),
                }
            }

            "lin-eigenvalues-diag" => {
                let l2 = rng.gen_range(-3..=2i32);
                let l1 = l2 + rng.gen_range(2..=5i32);
                let k = rng.gen_range(1..=3i32);
                // Unimodular similarity P = [[1,0],[1,1]] applied to [[l1, k], [0, l2]]:
                let a11 = l1 - k;
                let a12 = k;
                let a21 = l1 - k - l2;
                let a22 = k + l2;
                let tr = a11 + a22;
                let det = a11 * a22 - a12 * a21;
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Dominant Eigenvalue of a 2x2 Matrix".to_string(),
                    prompt: format!(
                        "Find the larger real eigenvalue lambda_1 of A = [[{}, {}], [{}, {}]].",
                        a11, a12, a21, a22
                    ),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Symmetric Matrix Quadratic Form Expansion".to_string(),
                    prompt: format!(
                        "Expand the quadratic form Q(x, y) = [x, y] * A * [x, y]^T for the symmetric matrix A = [[{}, {}], [{}, {}]].",
                        a, b, b, c
                    ),
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
                // y' + k*y = k*eq, y(0) = y0 => y(t) = eq + delta * exp(-k*t)
                let expected = format!("{} + {}*exp(-{}*t)", eq, delta, k);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "First-Order Linear IVP via Integrating Factor".to_string(),
                    prompt: format!(
                        "Solve the initial-value problem for y(t):  dy/dt + {}*y = {},  with y(0) = {}.",
                        k, k * eq, y0
                    ),
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
                    remediate_module_id: Some("calc-integration-tech".to_string()),
                }
            }

            "deq-second-order-osc" => {
                if variant == 0 {
                    let w = rng.gen_range(2..=5i32);
                    let c1 = rng.gen_range(1..=4i32);
                    let c2 = rng.gen_range(1..=4i32);
                    // y'' + w^2 * y = 0, y(0) = c1, y'(0) = c2*w
                    let expected = format!("{}*cos({}*t) + {}*sin({}*t)", c1, w, c2, w);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Undamped Harmonic Oscillator IVP".to_string(),
                        prompt: format!(
                            "Solve for y(t):  y'' + {}*y = 0,  given initial state y(0) = {} and y'(0) = {}.",
                            w * w, c1, c2 * w
                        ),
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
                        remediate_module_id: Some("trig-graphs-waves".to_string()),
                    }
                } else {
                    let r1 = rng.gen_range(1..=2i32);
                    let r2 = rng.gen_range(3..=5i32);
                    let expected = format!("exp(-{}*t) + exp(-{}*t)", r1, r2);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Overdamped Second-Order Homogeneous IVP".to_string(),
                        prompt: format!(
                            "Solve for y(t):  y'' + {}*y' + {}*y = 0,  with y(0) = 2 and y'(0) = -{}.",
                            r1 + r2, r1 * r2, r1 + r2
                        ),
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Trace-Determinant Phase Discriminant".to_string(),
                    prompt: format!(
                        "For the planar linear system x' = A*x with A = [[{}, {}], [{}, {}]], compute the spiral-node discriminant Delta = tr(A)^2 - 4*det(A).",
                        a11, a12, a21, a22
                    ),
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
                    remediate_module_id: Some("lin-eigenvalues-diag".to_string()),
                }
            }

            "deq-laplace-nonlinear" => {
                let a = rng.gen_range(2..=5i32);
                let k = rng.gen_range(1..=4i32);
                let b = rng.gen_range(1..=4i32);
                let w = rng.gen_range(2..=4i32);
                // L{ a*exp(k*t) + b*cos(w*t) } = a/(s - k) + (b*s)/(s^2 + w^2)
                let expected = format!("{}/(s - {}) + ({}*s)/(s^2 + {})", a, k, b, w * w);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Symbolic Laplace Transform in s-Domain".to_string(),
                    prompt: format!(
                        "Compute the Laplace transform F(s) = L{{ {}*exp({}*t) + {}*cos({}*t) }} as a rational expression in s (for s > {}).",
                        a, k, b, w, k
                    ),
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
                    remediate_module_id: Some("alg-polynomial-rational".to_string()),
                }
            }

            "deq-heat-diffusion-pde" => {
                let alpha = rng.gen_range(1..=2i32);
                let mode_n = rng.gen_range(1..=2i32);
                let amp = rng.gen_range(3..=6i32);
                let decay_rate = alpha * mode_n * mode_n;
                // u_t = alpha * u_xx on [0, pi], u(0,t)=u(pi,t)=0, u(x,0) = amp*sin(mode_n*x)
                let expected = format!("{} * exp(-{}*t) * sin({}*x)", amp, decay_rate, mode_n);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 4,
                    title: "1D Heat Equation Dirichlet Eigenmode Evolution".to_string(),
                    prompt: format!(
                        "Solve the 1D Heat PDE u_t = {} * u_xx on 0 <= x <= pi with zero ends u(0,t) = u(pi,t) = 0 and initial profile u(x, 0) = {} * sin({}*x). Write u(x, t) in variables x and t.",
                        alpha, amp, mode_n
                    ),
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
                    remediate_module_id: Some("deq-first-order".to_string()),
                }
            }

            "deq-wave-laplace-pde" => {
                let speed_c = rng.gen_range(2..=4i32);
                let mode_n = rng.gen_range(1..=4i32);
                let amp = rng.gen_range(2..=5i32);
                let omega = speed_c * mode_n;
                let expected = format!("{} * cos({}*t) * sin({}*x)", amp, omega, mode_n);
                RawProblem {
                    module_id: "deq-wave-laplace-pde".to_string(),
                    seed,
                    difficulty: 4,
                    title: "Standing Harmonic Solution of the 1D Wave PDE".to_string(),
                    prompt: format!(
                        "Solve the 1D Wave PDE u_tt = {} * u_xx on 0 <= x <= pi (wave speed c = {}) with fixed ends u(0,t) = u(pi,t) = 0, released from rest u_t(x, 0) = 0 with initial shape u(x, 0) = {} * sin({}*x).",
                        speed_c * speed_c, speed_c, amp, mode_n
                    ),
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
                // Out of 10,000 trials:
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
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 2,
                    title: "Bayesian Posterior Probability Update".to_string(),
                    prompt: scenario,
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
                    remediate_module_id: Some("alg-polynomial-rational".to_string()),
                }
            }

            "sta-discrete-rv" => {
                let lam = rng.gen_range(2..=4i32);
                let k = rng.gen_range(0..=2i32);
                let (expected, disp) = match k {
                    0 => ((-(lam as f64)).exp(), format!("exp(-{})", lam)),
                    1 => ((lam as f64) * (-(lam as f64)).exp(), format!("{}*exp(-{})", lam, lam)),
                    _ => (
                        0.5 * ((lam * lam) as f64) * (-(lam as f64)).exp(),
                        format!("({}/2)*exp(-{})", lam * lam, lam),
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
                    title: "Exact Poisson Rare-Event Probability".to_string(),
                    prompt: prompt_text,
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
                    remediate_module_id: Some("alg-exp-log".to_string()),
                }
            }

            "sta-continuous-pdf-cdf" => {
                let rate = rng.gen_range(1..=3i32);
                if variant == 0 {
                    let expected = format!("1 - exp(-{}*t)", rate);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Cumulative Distribution Function via Density Integration".to_string(),
                        prompt: format!(
                            "A continuous waiting time T >= 0 has Exponential PDF f(u) = {} * exp(-{}*u). Integrate f(u) from 0 to t to obtain the cumulative distribution function F(t) = P(0 <= T <= t) (in variable t).",
                            rate, rate
                        ),
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
                        remediate_module_id: Some("calc-integration-tech".to_string()),
                    }
                } else {
                    let t0 = rng.gen_range(1..=2i32);
                    let prod = rate * t0;
                    let expected = (-(prod as f64)).exp();
                    let disp = format!("exp(-{})", prod);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 2,
                        title: "Exponential Survival Tail Integral".to_string(),
                        prompt: format!(
                            "For T ~ Exp(lambda = {}) with PDF f(t) = {} * exp(-{}*t) (t >= 0), compute the exact survival probability P(T > {}).",
                            rate, rate, rate, t0
                        ),
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
                // Z = (x_bar - mu) / (sigma / sqrt(n)) = delta * root_n / sigma
                let num = delta * root_n;
                let expected = (num as f64) / (sigma as f64);
                let disp = format!("{}/{}", num, sigma);
                RawProblem {
                    module_id: module_id.to_string(),
                    seed,
                    difficulty: 3,
                    title: "Central Limit Theorem Standardized Z-Statistic".to_string(),
                    prompt: format!(
                        "Independent measurements have population mean mu = {} and SD sigma = {}. For a sample of size n = {} with observed mean x_bar = {}, compute the standardized CLT score Z = (x_bar - mu) / (sigma / sqrt(n)).",
                        mu, sigma, n, x_bar
                    ),
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
                    remediate_module_id: Some("alg-radicals-exponents".to_string()),
                }
            }

            "sta-mle-confidence" => {
                let n = rng.gen_range(4..=8i32);
                let sum_t = rng.gen_range(10..=25i32);
                if variant == 0 {
                    // Score derivative of log-likelihood l(x) = n*ln(x) - sum_t*x
                    let expected = format!("{}/x - {}", n, sum_t);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Log-Likelihood Score Derivative for Rate Parameter".to_string(),
                        prompt: format!(
                            "Given n = {} independent Exponential observations with total waiting time sum(t_i) = {}, the log-likelihood for rate x > 0 is l(x) = {}*ln(x) - {}*x. Compute the score derivative dl/dx (in variable x).",
                            n, sum_t, n, sum_t
                        ),
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
                        remediate_module_id: Some("calc-derivatives-rules".to_string()),
                    }
                } else {
                    let expected = (n as f64) / (sum_t as f64);
                    let disp = format!("{}/{}", n, sum_t);
                    RawProblem {
                        module_id: module_id.to_string(),
                        seed,
                        difficulty: 3,
                        title: "Maximum Likelihood Estimator (MLE) via Calculus".to_string(),
                        prompt: format!(
                            "An experiment records n = {} independent Exponential(lambda) lifetimes whose sum is sum(t_i) = {}. Maximize l(lambda) = {}*ln(lambda) - {}*lambda to find the exact MLE lambda_hat.",
                            n, sum_t, n, sum_t
                        ),
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
                let expected = format!("{}*x + {}", m, c);
                RawProblem {
                    module_id: "sta-hypothesis-regression".to_string(),
                    seed,
                    difficulty: 3,
                    title: "Ordinary Least-Squares Regression Line".to_string(),
                    prompt: format!(
                        "Fit the ordinary least-squares regression line y_hat(x) = b1*x + b0 to the three noisy data points (0, {}), (1, {}), and (2, {}). Write the linear expression in x.",
                        y0, y1, y2
                    ),
                    answer: AnswerFormat::ScalarExpr {
                        expected_expr: expected.clone(),
                        vars: vec!["x".to_string()],
                    },
                    hints: vec![
                        format!("Compute means: x_bar = (0 + 1 + 2)/3 = 1, and y_bar = ({} + {} + {})/3 = {}.", y0, y1, y2, c + m),
                        format!("Compute S_xx = (-1)^2 + 0^2 + 1^2 = 2, and S_xy = (-1)*({} - {}) + 0 + (1)*({} - {}) = {}.", y0, c + m, y2, c + m, 2 * m),
                        format!("Slope b1 = S_xy / S_xx = {}, intercept b0 = y_bar - b1*x_bar = {}, giving {}.", m, c, expected),
                    ],
                    derivation_steps: vec![
                        format!("1. Sample means: x_bar = 1, y_bar = {}.", c + m),
                        format!("2. Normal-equation slope: b1 = S_xy / S_xx = {} / 2 = {}.", 2 * m, m),
                        format!("3. Intercept: b0 = {} - {}*(1) = {} => y_hat(x) = {}.", c + m, m, c, expected),
                    ],
                    remediate_module_id: Some("lin-orthogonality-lsq".to_string()),
                }
            }
        }
    }
}

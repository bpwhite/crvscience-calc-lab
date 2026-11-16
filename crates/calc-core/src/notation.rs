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
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct NotationEntry {
    pub id: &'static str,
    pub stage: Stage,
    pub symbol_display: &'static str,
    pub name: &'static str,
    pub plain_text_form: &'static str,
    pub spoken_phrasing: &'static str,
    pub anatomy_points: &'static [&'static str],
    pub dialect_equivalents: &'static [&'static str],
    pub common_pitfall: &'static str,
}

impl NotationEntry {
    pub fn domain_application(&self, domain: ScienceDomain) -> &'static str {
        match (self.stage, domain) {
            (_, ScienceDomain::General) => {
                "Pure Mathematics: Expresses structural relationships compactly across arbitrary variables and dimensions."
            }
            (Stage::Notation | Stage::Algebra, ScienceDomain::Biology) => {
                "Biology: Indexes multi-gene expression vectors x_i, summation of allele frequencies sum p_i = 1, and physiological bounds."
            }
            (Stage::Trigonometry, ScienceDomain::Biology) => {
                "Biology: Parameterizes circadian phase angles phi in [0, 2*pi) and joint biomechanics."
            }
            (Stage::Calculus, ScienceDomain::Biology) => {
                "Biology: Leibniz d[P]/dt tracks instantaneous enzyme product velocity (mM/s) while integral C(t) dt measures drug exposure (AUC)."
            }
            (Stage::LinearAlgebra, ScienceDomain::Biology) => {
                "Biology: Matrix projection n(t+1) = L * n(t) advances age-structured cohorts across discrete breeding seasons."
            }
            (Stage::AdvancedCalculus, ScienceDomain::Biology) => {
                "Biology: Nabla grad c(x,y) points up morphogen signaling gradients during embryonic tissue patterning."
            }
            (Stage::DifferentialEquations, ScienceDomain::Biology) => {
                "Biology: Laplacian nabla^2 u drives reaction-diffusion Turing spots and membrane cable equations."
            }
            (Stage::ProbabilityStats, ScienceDomain::Biology) => {
                "Biology: Conditional P(Disease | +) and log-likelihood l(theta) govern diagnostic assays and phylogenetic inference."
            }

            (_, ScienceDomain::Physics) => match self.stage {
                Stage::Calculus => {
                    "Physics: Newton dot x_dot = dx/dt denotes velocity, x_ddot = d^2x/dt^2 acceleration, and integral F . dx mechanical work."
                }
                Stage::AdvancedCalculus | Stage::DifferentialEquations => {
                    "Physics: div E = rho/eps_0, curl B, and wave d'Alembertian u_tt - c^2 * nabla^2 u encode conservation and field propagation."
                }
                _ => {
                    "Physics: Compactly tracks dimensional units, tensor contractions, and ensemble averages <E>."
                }
            },

            (_, ScienceDomain::Chemistry) => match self.stage {
                Stage::Algebra | Stage::Notation => {
                    "Chemistry: Product pi_i [A_i]^(nu_i) forms the reaction quotient Q and equilibrium constant K_eq!"
                }
                Stage::Calculus | Stage::DifferentialEquations => {
                    "Chemistry: Partial (dG/dT)_P = -S holds pressure fixed (subscript P), while Fick's law J = -D * grad c drives diffusion."
                }
                _ => {
                    "Chemistry: Connects molecular symmetry operators, Huckel secular determinants, and Boltzmann distributions."
                }
            },

            (_, ScienceDomain::Ecology) => match self.stage {
                Stage::Notation | Stage::Algebra => {
                    "Ecology: Sigma sum_{i=1..S} p_i * ln(p_i) defines Shannon species diversity H' across S richness classes."
                }
                Stage::LinearAlgebra | Stage::DifferentialEquations => {
                    "Ecology: Dominant eigenvalue lambda_1 of projection matrix A sets long-term population growth; Lotka-Volterra couples dN/dt, dP/dt."
                }
                _ => {
                    "Ecology: Quantifies spatial dispersal flux, quadrat Poisson counts X ~ Pois(lambda), and mark-recapture estimators."
                }
            },

            (_, ScienceDomain::Microbiology) => match self.stage {
                Stage::Notation | Stage::ProbabilityStats => {
                    "Microbiology: Product likelihood prod_{i=1..n} Pois(k_i; m) estimates phage Multiplicity of Infection (MOI) across culture wells."
                }
                Stage::Calculus | Stage::DifferentialEquations => {
                    "Microbiology: Specific growth rate mu = (1/X)*(dX/dt) normalizes Leibniz rate by biomass X in chemostat bioreactors."
                }
                _ => {
                    "Microbiology: Tracks stoichiometric matrix nullspaces S*v = 0 (FBA) and chemotactic attractant gradients grad c."
                }
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotationFlashQuestion {
    pub entry_id: String,
    pub symbol_banner: String,
    pub question: String,
    pub choices: [String; 4],
    pub correct_index: usize,
    pub explanation: String,
}

pub fn build_notation_lexicon() -> Vec<NotationEntry> {
    vec![
        // ==================== STAGE 0 / SETS, LOGIC & INDICES ====================
        NotationEntry {
            id: "sym-element-subset",
            stage: Stage::Notation,
            symbol_display: "x ∈ [a, b],   A ⊆ ℝ",
            name: "Set Membership & Interval Inclusion",
            plain_text_form: "x in [a, b]  (a <= x <= b);  A subset R",
            spoken_phrasing: "'x is an element of the closed interval a to b; A is a subset of the real numbers'",
            anatomy_points: &[
                "∈ ('is in'): binds a single element on the left to a set/domain on the right.",
                "Square brackets [a, b] INCLUDE endpoints a and b; parentheses (a, b) EXCLUDE endpoints.",
                "ℝ denotes all real numbers; ℝ^n denotes n-component real vectors.",
            ],
            dialect_equivalents: &[
                "Inequality form: a <= x <= b",
                "Set-builder form: { x in R : a <= x <= b }",
            ],
            common_pitfall: "Confusing x ∈ A (single member) with A ⊆ B (every member of A is also in B), or confusing open interval (0, 1) with 2D point (0, 1).",
        },
        NotationEntry {
            id: "sym-quantifiers",
            stage: Stage::Notation,
            symbol_display: "∀ ε > 0,  ∃ δ > 0  s.t.  |x - a| < δ ⇒ |f(x) - L| < ε",
            name: "Universal (∀), Existential (∃) & Implication (⇒) Quantifiers",
            plain_text_form: "for all eps > 0, there exists delta > 0 such that |x - a| < delta => |f(x) - L| < eps",
            spoken_phrasing: "'For every epsilon greater than zero, there exists a delta greater than zero such that...'",
            anatomy_points: &[
                "∀ ('for all / for every'): universal guarantee across every admissible challenge tolerance.",
                "∃ ('there exists'): asserts you can construct at least one witness value (which may depend on earlier variables!).",
                "Order matters! '∀ ε ∃ δ' lets δ depend on ε; reversing to '∃ δ ∀ ε' claims one fixed δ works for every ε.",
            ],
            dialect_equivalents: &[
                "Implication P ⇒ Q ('if P then Q') vs Equivalence P ⇔ Q ('P if and only if Q / iff')",
            ],
            common_pitfall: "Swapping quantifier order: reading ∃ before ∀ changes a local continuity statement into a trivial/impossible constant bound.",
        },
        NotationEntry {
            id: "sym-function-map",
            stage: Stage::Notation,
            symbol_display: "f : ℝ² → ℝ,   (x, y) ↦ x² + y²",
            name: "Domain-Codomain Arrow (→) vs Element Mapping Arrow (↦)",
            plain_text_form: "f: R^2 -> R,  f(x, y) = x^2 + y^2",
            spoken_phrasing: "'f maps R-two into R, sending the pair (x, y) to x squared plus y squared'",
            anatomy_points: &[
                "Plain arrow → connects SPACES (input domain ℝ² to output codomain ℝ).",
                "Barred arrow ↦ ('maps to') connects an INDIVIDUAL INPUT (x, y) to its formula output.",
                "Immediately tells you whether output is scalar (→ ℝ) or a vector field (→ ℝ²).",
            ],
            dialect_equivalents: &[
                "Formula definition: f(x, y) := x^2 + y^2 (':=' means 'is defined as')",
            ],
            common_pitfall: "Misreading f: ℝⁿ → ℝᵐ dimensions when assembling Jacobian matrices (which always have m rows and n columns).",
        },
        NotationEntry {
            id: "sym-sigma-sum",
            stage: Stage::Notation,
            symbol_display: "∑_{k=1}^{n} a_k = a_1 + a_2 + ... + a_n",
            name: "Capital Sigma Summation Operator (∑)",
            plain_text_form: "sum_{k=1..n} a_k",
            spoken_phrasing: "'the sum as index k goes from 1 to n of a-sub-k'",
            anatomy_points: &[
                "k is a 'bound / dummy index': renaming k to j everywhere inside ∑_{j=1}^n a_j leaves the value 100% identical.",
                "Lower label k=1 sets starting integer; top label n sets inclusive stopping integer.",
                "Factors not containing k pull straight outside: ∑ c * a_k = c * ∑ a_k.",
            ],
            dialect_equivalents: &[
                "Einstein summation (physics/tensors): repeated index a_i * b^i implies sum over i without writing ∑",
                "Code loop: (1..=n).map(|k| a(k)).sum()",
            ],
            common_pitfall: "Leaving the dummy index k in your final simplified answer—after summing k = 1..n, only n can remain!",
        },
        NotationEntry {
            id: "sym-pi-product",
            stage: Stage::Notation,
            symbol_display: "∏_{i=1}^{n} x_i = x_1 · x_2 · ... · x_n",
            name: "Capital Pi Product Operator (∏)",
            plain_text_form: "prod_{i=1..n} x_i",
            spoken_phrasing: "'the product as i runs from 1 to n of x-sub-i'",
            anatomy_points: &[
                "Multiplicative twin of ∑: multiplies terms instead of adding them; empty product convention equals 1.",
                "Factorial n! is simply ∏_{k=1}^n k.",
                "Crucial Logarithm Bridge: ln( ∏_{i=1}^n L_i ) = ∑_{i=1}^n ln(L_i)—turning product likelihoods into sums!",
            ],
            dialect_equivalents: &[
                "Code fold: (1..=n).map(|i| x(i)).product()",
            ],
            common_pitfall: "Distributing a constant wrongly: ∏_{i=1}^n (c * x_i) = c^n * ∏ x_i (factors out as c^n, NOT c!).",
        },

        // ==================== CALCULUS & ASYMPTOTIC NOTATION ====================
        NotationEntry {
            id: "sym-deltas-differentials",
            stage: Stage::Calculus,
            symbol_display: "Δx  vs  dx  vs  ∂x",
            name: "Finite Increment (Δ), Total Differential (d) & Partial Symbol (∂)",
            plain_text_form: "Delta_x (finite step h)  vs  dx (linear differential)  vs  d/dx partial",
            spoken_phrasing: "'finite change Delta-x versus infinitesimal/tangent differential d-x versus curly partial del-x'",
            anatomy_points: &[
                "Δy = f(x + Δx) - f(x) is an EXACT finite secant change over a non-zero step Δx.",
                "dy = f'(x) * dx is the TANGENT-LINE linear change.",
                "Curly ∂ ('partial' / 'del') signals other independent variables exist and are being held fixed.",
            ],
            dialect_equivalents: &[
                "Forward finite difference: Delta f_i = f(x_{i+1}) - f(x_i)",
            ],
            common_pitfall: "Treating ∂z/∂x + ∂z/∂y like fraction cancellation: unlike 1D chain rule (dy/du)*(du/dx), multivariable chain rule sums across ALL paths!",
        },
        NotationEntry {
            id: "sym-derivative-dialects",
            stage: Stage::Calculus,
            symbol_display: "df/dx   ≡   f'(x)   ≡   ẋ(t)   ≡   D_x f",
            name: "The Four Derivative Dialects (Leibniz, Lagrange, Newton, Euler)",
            plain_text_form: "df/dx  =  f'(x)  =  x_dot(t)  =  D_x[f]",
            spoken_phrasing: "'d-f d-x (Leibniz), f-prime of x (Lagrange), x-dot (Newton), D-sub-x of f (Euler operator)'",
            anatomy_points: &[
                "Leibniz df/dx: explicitly names both output and input variable + carries physical units (e.g. m/s); best for chain rule & separation of variables.",
                "Lagrange f'(x), f''(x), f^(n)(x): cleanest for evaluating at a specific point like f'(0) in Taylor series (note parenthesized superscript ^(n) means n-th derivative, NOT power!).",
                "Newton dot ẋ, ẍ: reserved almost exclusively for time derivatives d/dt, d²/dt² in mechanics and ODE phase portraits.",
                "Operator D_x or d/dx: treats differentiation itself as a linear map acting on a function vector space.",
            ],
            dialect_equivalents: &[
                "Evaluation bar: df/dx |_{x=a}  means  f'(a) (differentiate FIRST, substitute x = a SECOND!)",
            ],
            common_pitfall: "Confusing f^(3)(x) (3rd derivative) with (f(x))^3 (cubed value), or plugging in x = a BEFORE differentiating in df/dx |_{x=a}.",
        },
        NotationEntry {
            id: "sym-big-o-asymptotic",
            stage: Stage::Calculus,
            symbol_display: "f(x + h) = f(x) + h·f'(x) + O(h²)   (as h → 0)",
            name: "Big-O Landau Error / Asymptotic Bound Notation",
            plain_text_form: "error = O(h^2)  (|error| <= C * h^2 near h = 0)",
            spoken_phrasing: "'plus big-O of h squared as h approaches zero'",
            anatomy_points: &[
                "O(h^p) stands for an unspecified remainder term bounded by C * |h|^p for small h (or large n).",
                "If h shrinks by 10x, an O(h) Euler error shrinks 10x, whereas an O(h^4) RK4 step error shrinks 10,000x!",
                "Little-o o(h) is even stronger: remainder / h -> 0 strictly.",
            ],
            dialect_equivalents: &[
                "Asymptotic equivalence: f(n) ~ g(n) means lim f(n)/g(n) = 1 (e.g. Stirling n! ~ sqrt(2*pi*n)*(n/e)^n)",
            ],
            common_pitfall: "Forgetting whether the limit context is h -> 0 (where O(h^4) << O(h^2)) versus n -> infinity (where O(n^2) << O(n^4))!",
        },
        NotationEntry {
            id: "sym-definite-integral-bar",
            stage: Stage::Calculus,
            symbol_display: "∫_{a}^{b} f(x) dx  =  [ F(x) ]_{x=a}^{x=b}  =  F(b) - F(a)",
            name: "Definite Integral & Evaluation Bar Notation",
            plain_text_form: "integral_{a..b} f(x) dx = [F(x)]_{a..b} = F(b) - F(a)",
            spoken_phrasing: "'the definite integral from a to b of f of x d-x equals F of x evaluated from a to b'",
            anatomy_points: &[
                "Elongated S '∫' (Leibniz 'summa') pairs with differential 'dx', which marks the dummy integration variable and carries width units.",
                "Bracket / vertical bar [F(x)]_a^b means top-bound substitution minus bottom-bound substitution.",
                "Indefinite ∫ f(x) dx (no bounds) represents a family of functions F(x) + C; definite ∫_a^b f(x) dx is a single number.",
            ],
            dialect_equivalents: &[
                "Single evaluation bar: F(x) |_{a}^{b}",
            ],
            common_pitfall: "During u-substitution u = g(x), plugging original x-bounds [a, b] into an expression written in u without converting bounds to [g(a), g(b)].",
        },

        // ==================== LINEAR ALGEBRA & VECTOR / PDE NOTATION ====================
        NotationEntry {
            id: "sym-inner-norm-transpose",
            stage: Stage::LinearAlgebra,
            symbol_display: "⟨u, v⟩ = uᵀ v,    ‖u‖₂ = √⟨u, u⟩",
            name: "Inner Product Brackets ⟨·,·⟩, Transpose (ᵀ) & Double-Bar Norm (‖·‖)",
            plain_text_form: "<u, v> = u^T * v = u . v;  ||u||_2 = sqrt(u . u)",
            spoken_phrasing: "'inner product of u and v equals u-transpose v; L-two norm of u equals root of u inner-product u'",
            anatomy_points: &[
                "Superscript T flips rows and columns: if u and v are column vectors (n x 1), u^T is (1 x n), making u^T v a (1 x 1) scalar!",
                "Note: outer product u * v^T (reversed order!) is an (n x n) rank-1 matrix, NOT a scalar.",
                "Angle brackets ⟨f, g⟩ generalize dot products to function spaces: ⟨f, g⟩ = ∫ f(x)*g(x) dx.",
            ],
            dialect_equivalents: &[
                "Dot notation: u . v",
                "Dirac bra-ket (quantum physics): <u | v>",
            ],
            common_pitfall: "Confusing single bars |x| (scalar absolute value or matrix determinant |A|) with double bars ||v|| (vector length / function norm).",
        },
        NotationEntry {
            id: "sym-nabla-operators",
            stage: Stage::AdvancedCalculus,
            symbol_display: "∇f (grad),   ∇ · F (div),   ∇ × F (curl),   ∇²f = Δf (Laplacian)",
            name: "The Four Faces of Nabla (∇)",
            plain_text_form: "grad f = <f_x, f_y>;  div F = P_x + Q_y;  curl F;  nabla^2 f = f_xx + f_yy",
            spoken_phrasing: "'grad f, divergence del-dot-F, curl del-cross-F, and Laplacian del-squared f'",
            anatomy_points: &[
                "Think of ∇ symbolically as the vector of partial operators <∂/∂x, ∂/∂y, ∂/∂z>.",
                "∇f (no dot, acting on SCALAR f) -> produces a VECTOR field pointing uphill.",
                "∇ · F (WITH dot, acting on VECTOR F) -> produces a SCALAR field measuring source/sink expansion.",
                "∇ × F (WITH cross, acting on VECTOR F) -> produces local rotational vorticity.",
                "∇²f = ∇ · (∇f) = f_xx + f_yy (+ f_zz) -> SCALAR curvature/diffusion operator (also written Δf in PDE papers!).",
            ],
            dialect_equivalents: &[
                "Explicit names: grad(f), div(F), curl(F), Δf (Laplace operator)",
            ],
            common_pitfall: "Writing ∇f when you mean scalar divergence ∇·F, or confusing PDE Laplacian Δu = u_xx + u_yy with finite increment Δx!",
        },
        NotationEntry {
            id: "sym-boundary-integrals",
            stage: Stage::AdvancedCalculus,
            symbol_display: "∮_{∂Ω} F · dr   =   ∬_{Ω} (∇ × F) · n dA",
            name: "Closed Contour Integral (∮) & Topological Boundary Operator (∂Ω)",
            plain_text_form: "oint_{d_Omega} F . dr = double_integral_{Omega} (curl F) . n dA",
            spoken_phrasing: "'the closed circulation integral around the boundary of Omega equals the double integral over Omega of...'",
            anatomy_points: &[
                "The circle on ∮ emphasizes the path/surface is CLOSED (starts and ends together with no boundary of its own).",
                "Applied to a geometric region Ω, curly ∂Ω means 'the oriented boundary of Ω' (e.g. circle bounding a disk, sphere bounding a ball).",
                "Unifying Stokes' notation: ∫_{∂M} ω = ∫_M dω—boundary ∂ on the domain swaps to exterior derivative d on the integrand!",
            ],
            dialect_equivalents: &[
                "Flux form: oint_{d_Omega} (F . n) ds = integral_{Omega} (div F) dA",
            ],
            common_pitfall: "Reversing boundary orientation: in 2D Green's theorem, positive orientation ∂Ω keeps the interior region Ω on your LEFT (counterclockwise).",
        },

        // ==================== ODE, PDE & PROBABILITY/STATS NOTATION ====================
        NotationEntry {
            id: "sym-pde-subscripts",
            stage: Stage::DifferentialEquations,
            symbol_display: "u_t = α u_{xx},     u_{tt} = c² u_{xx}",
            name: "Subscript Partial-Derivative Shorthand (u_t, u_xx, f_xy)",
            plain_text_form: "du/dt = alpha * d^2u/dx^2;  d^2u/dt^2 = c^2 * d^2u/dx^2",
            spoken_phrasing: "'u-sub-t equals alpha u-sub-x-x; u-sub-t-t equals c squared u-sub-x-x'",
            anatomy_points: &[
                "In PDEs and multivariable calculus, variable-letter subscripts (x, y, t) almost always denote partial differentiation, NOT sequence indexing!",
                "Repeated subscript u_{xx} means second partial ∂²u/∂x²; mixed u_{xy} means ∂/∂y(∂u/∂x).",
                "Tell index vs derivative apart by context: u_n or x_k (integer n, k) is an index; u_t or f_x (coordinate t, x) is a partial derivative.",
            ],
            dialect_equivalents: &[
                "Leibniz PDE form: ∂u/∂t = α * ∂²u/∂x²",
            ],
            common_pitfall: "Confusing initial condition u_t(x, 0) (initial velocity ∂u/∂t at t=0) with indexed sequence term u_0(x).",
        },
        NotationEntry {
            id: "sym-prob-conditioning-dist",
            stage: Stage::ProbabilityStats,
            symbol_display: "X ~ N(μ, σ²),    P(H | E),    E[g(X) | Y = y]",
            name: "Tilde Distribution (~), Conditioning Bar (|) & Expectation Brackets E[·]",
            plain_text_form: "X ~ Normal(mu, sigma^2);  P(H | E) = P(H and E) / P(E);  E[g(X)]",
            spoken_phrasing: "'X is distributed as Normal with mean mu and variance sigma squared; probability of H given E'",
            anatomy_points: &[
                "Tilde '~' here means 'follows probability law' (distinct from asymptotic '~' in limits!).",
                "Vertical bar '|' inside P(A | B) or f(x | θ) restricts the sample space to assuming everything to the right of '|' is GIVEN/KNOWN.",
                "Capital X is the random variable (a function on outcomes); lowercase x is a specific realized numerical value.",
            ],
            dialect_equivalents: &[
                "Likelihood notation: L(theta ; x) or f(x | theta) (same formula, viewed as a function of theta vs x)",
            ],
            common_pitfall: "Confusing P(A | B) with P(B | A) (the Prosecutor's / Base-Rate Fallacy—bridged only via Bayes' Theorem!), or mixing up σ vs σ² in N(μ, σ²).",
        },
        NotationEntry {
            id: "sym-hat-bar-argmax",
            stage: Stage::ProbabilityStats,
            symbol_display: "θ̂_{MLE} = argmax_{θ} ∑_{i=1}^{n} ln f(x_i; θ),    x̄ = (1/n) ∑ x_i",
            name: "Estimator Hat (θ̂), Sample-Mean Bar (x̄) & Argmax Operator",
            plain_text_form: "theta_hat = argmax_theta l(theta);  x_bar = (1/n) * sum_{i=1..n} x_i",
            spoken_phrasing: "'theta-hat M-L-E equals the arg-max over theta of the log-likelihood sum; x-bar is the sample mean'",
            anatomy_points: &[
                "True unknown population parameters wear plain Greek letters (θ, μ, σ, β); data-computed ESTIMATES wear hats (θ̂, σ̂, β̂) or bars (x̄).",
                "max_θ g(θ) returns the PEAK HEIGHT (output y-value); argmax_θ g(θ) returns the PARAMETER LOCATION θ where that peak occurs!",
                "Thus shifting or scaling l(θ) by positive constants changes max l(θ), but leaves argmax_θ l(θ) completely unchanged.",
            ],
            dialect_equivalents: &[
                "Unit vector hat: in vector calculus v_hat = v / ||v|| denotes a unit length-1 vector (check statistical vs geometric context!).",
            ],
            common_pitfall: "Reporting the maximum log-likelihood value l(θ̂) when asked for argmax_θ l(θ) (which asks for θ̂ itself!).",
        },
    ]
}

pub fn generate_notation_flash_question(seed: u64) -> NotationFlashQuestion {
    let bank: &[(&str, &str, &str, [&str; 4], usize, &str)] = &[
        (
            "sym-nabla-operators",
            "∇ · F   vs   ∇f   vs   ∇²u",
            "Which expression applies Nabla correctly to take a 2D vector field F = <P, Q> and return its SCALAR source/sink expansion rate?",
            [
                "∇f (gradient of scalar)",
                "∇ · F = ∂P/∂x + ∂Q/∂y (divergence)",
                "∇ × F (rotational curl)",
                "F · F (squared vector magnitude)",
            ],
            1,
            "The dot in ∇ · F contracts <∂/∂x, ∂/∂y> with vector <P, Q> to produce the scalar divergence ∂P/∂x + ∂Q/∂y.",
        ),
        (
            "sym-pi-product",
            "ln ( ∏_{i=1}^{n} L_i(θ) )",
            "Applying logarithm laws to the Capital-Pi product likelihood ∏_{i=1}^n L_i(θ) converts it into:",
            [
                "∏_{i=1}^{n} ln L_i(θ)",
                "∑_{i=1}^{n} ln L_i(θ)",
                "n · ln L_1(θ)",
                "( ∑_{i=1}^{n} L_i(θ) ) / n",
            ],
            1,
            "Because ln(A · B) = ln(A) + ln(B), the natural log converts any Capital-Pi product ∏ into a Capital-Sigma sum ∑ of logs.",
        ),
        (
            "sym-quantifiers",
            "∀ ε > 0,  ∃ δ > 0  :  0 < |x - a| < δ  ⇒  |f(x) - L| < ε",
            "In this rigorous limit statement, why does ∃ δ > 0 appear AFTER ∀ ε > 0?",
            [
                "So δ is allowed to depend on the target error tolerance ε",
                "Because δ must always be strictly larger than ε",
                "Because ε is a dummy summation variable",
                "Order of ∀ and ∃ never changes mathematical meaning",
            ],
            0,
            "Variables introduced by ∃ may depend on any universally quantified variables appearing to their left, so δ = δ(ε).",
        ),
        (
            "sym-hat-bar-argmax",
            "θ̂ = argmax_{θ ∈ Θ}  l(θ)",
            "What is the exact distinction between max_{θ} l(θ) and argmax_{θ} l(θ)?",
            [
                "They always evaluate to the same number",
                "argmax returns the parameter value θ̂ that achieves the peak, whereas max returns the peak height l(θ̂)",
                "argmax computes the second derivative of l(θ)",
                "argmax integrates l(θ) over Θ",
            ],
            1,
            "'argument of the maximum' (argmax) returns the input coordinate θ̂ where l(θ) is maximized, not the output value l(θ̂).",
        ),
        (
            "sym-derivative-dialects",
            "df/dx |_{x = 3}   vs   f^(3)(x)",
            "How do df/dx |_{x=3} and parenthesized superscript f^(3)(x) differ?",
            [
                "Both mean (f(3))^3",
                "df/dx |_{x=3} is the 1st derivative evaluated at x=3; f^(3)(x) is the 3rd derivative d³f/dx³",
                "f^(3)(x) means f(x)*f(x)*f(x)",
                "df/dx |_{x=3} means differentiate the constant number f(3)",
            ],
            1,
            "The vertical evaluation bar substitutes x=3 AFTER differentiating once; parenthesized superscript ^(n) denotes the n-th derivative.",
        ),
        (
            "sym-pde-subscripts",
            "u_t = α u_{xx}   with   u_t(x, 0) = 0",
            "In PDE subscript notation, what does u_{xx} represent?",
            [
                "The product u(x) * x * x",
                "Sequence element at index x squared",
                "The second partial derivative ∂²u / ∂x² holding t fixed",
                "The inverse function of u(x)",
            ],
            2,
            "Coordinate-letter subscripts denote partial derivatives: u_t = ∂u/∂t and u_{xx} = ∂²u/∂x².",
        ),
        (
            "sym-inner-norm-transpose",
            "uᵀ v   vs   u vᵀ   (for column vectors u, v ∈ ℝⁿ)",
            "For n × 1 column vectors u and v, what are the shapes of uᵀ v and u vᵀ?",
            [
                "Both are scalars",
                "uᵀ v is a 1×1 scalar (inner product); u vᵀ is an n×n rank-1 matrix (outer product)",
                "Both are n×n matrices",
                "uᵀ v is undefined unless n = 2",
            ],
            1,
            "(1×n)(n×1) collapses to a scalar inner product, whereas (n×1)(1×n) expands to an n×n outer-product matrix.",
        ),
        (
            "sym-prob-conditioning-dist",
            "P(H | +)   vs   P(+ | H)",
            "In diagnostic testing where H = 'trait present' and + = 'assay positive', what does P(H | +) denote?",
            [
                "Assay sensitivity (true positive rate among carriers)",
                "Joint probability that both H and + occur",
                "Posterior probability that H is present GIVEN an observed positive test +",
                "Prior prevalence before running the assay",
            ],
            2,
            "Everything to the right of '|' is the observed/assumed condition; P(H | +) is the posterior positive predictive value.",
        ),
    ];

    let pick = &bank[(seed as usize) % bank.len()];
    NotationFlashQuestion {
        entry_id: pick.0.to_string(),
        symbol_banner: pick.1.to_string(),
        question: pick.2.to_string(),
        choices: [
            pick.3[0].to_string(),
            pick.3[1].to_string(),
            pick.3[2].to_string(),
            pick.3[3].to_string(),
        ],
        correct_index: pick.4,
        explanation: pick.5.to_string(),
    }
}

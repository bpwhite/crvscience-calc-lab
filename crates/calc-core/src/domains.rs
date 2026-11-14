use crate::curriculum::Stage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScienceDomain {
    General,
    Biology,
    Physics,
    Chemistry,
    Ecology,
    Microbiology,
}

impl ScienceDomain {
    pub const ALL: [ScienceDomain; 6] = [
        ScienceDomain::General,
        ScienceDomain::Biology,
        ScienceDomain::Physics,
        ScienceDomain::Chemistry,
        ScienceDomain::Ecology,
        ScienceDomain::Microbiology,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General Mathematics",
            Self::Biology => "Biology & Physiology",
            Self::Physics => "Physics & Mechanics",
            Self::Chemistry => "Chemistry & Kinetics",
            Self::Ecology => "Ecology & Population Systems",
            Self::Microbiology => "Microbiology & Chemostats",
        }
    }

    pub fn short_tag(&self) -> &'static str {
        match self {
            Self::General => "GEN",
            Self::Biology => "BIO",
            Self::Physics => "PHY",
            Self::Chemistry => "CHM",
            Self::Ecology => "ECO",
            Self::Microbiology => "MIC",
        }
    }

    /// Provides an authentic scientific model framing note for a problem in `(domain, stage)` while keeping mathematical variable symbols crisp and unambiguous.
    pub fn context_banner(&self, stage: Stage, module_id: &str) -> Option<String> {
        if *self == ScienceDomain::General {
            return None;
        }

        let note = match (self, stage) {
            // ==================== BIOLOGY ====================
            (ScienceDomain::Biology, Stage::Notation) => "Biological Context: Reading multi-gene sum/product indices, stoichiometric state maps, and assay conditioning bars.",
            (ScienceDomain::Biology, Stage::Algebra) => match module_id {
                "alg-exp-log" => "Biological Context: Membrane Nernst potentials and tissue clearance follow base-e exponential/logarithmic scaling.",
                "alg-polynomial-rational" => "Biological Context: Enzyme saturation curves v([S]) = V_max*[S] / (K_m + [S]) behave as rational functions in substrate x.",
                _ => "Biological Context: Allometric organ scaling and physiological dose-response calibration.",
            },
            (ScienceDomain::Biology, Stage::Trigonometry) => {
                "Biological Context: Circadian gene-expression oscillators, cardiac ECG phase cycles, and limb biomechanics."
            }
            (ScienceDomain::Biology, Stage::Calculus) => match module_id {
                "calc-derivatives-rules" => "Biological Context: Instantaneous metabolic flux df/dx and receptor binding sensitivity.",
                "calc-integrals-ftc" | "calc-integration-tech" => "Biological Context: Pharmacokinetic Area Under the Curve (AUC = integral C(t) dt) in plasma.",
                _ => "Biological Context: Homeostatic feedback rates and physiological optimization.",
            },
            (ScienceDomain::Biology, Stage::LinearAlgebra) => {
                "Biological Context: Leslie age-structured projection matrices, morphometric PCA eigen-axes, and stoichiometry."
            }
            (ScienceDomain::Biology, Stage::AdvancedCalculus) => {
                "Biological Context: Morphogen concentration gradients grad c(x, y) and cellular Fickian membrane flux."
            }
            (ScienceDomain::Biology, Stage::DifferentialEquations) => {
                "Biological Context: FitzHugh-Nagumo excitable membrane dynamics and Turing reaction-diffusion morphogenesis."
            }
            (ScienceDomain::Biology, Stage::ProbabilityStats) => {
                "Biological Context: Clinical assay sensitivity/specificity (Bayes), quantitative trait distributions, and GWAS regression."
            }

            // ==================== PHYSICS ====================
            (ScienceDomain::Physics, Stage::Notation) => "Physics Context: Translating Newton dot derivatives, Einstein indices, Nabla field operators, and Big-O expansions.",
            (ScienceDomain::Physics, Stage::Algebra) => {
                "Physics Context: Kinematic conservation laws, resistor bridge networks, and dimensional scaling."
            }
            (ScienceDomain::Physics, Stage::Trigonometry) => {
                "Physics Context: Wave superposition, optical phase interference, and rigid-body torque projections."
            }
            (ScienceDomain::Physics, Stage::Calculus) => {
                "Physics Context: Instantaneous velocity/acceleration, mechanical work integral F(x) dx, and multipole Taylor expansions."
            }
            (ScienceDomain::Physics, Stage::LinearAlgebra) => {
                "Physics Context: Moment-of-inertia tensors, normal-mode eigenfrequencies, and Lorentz/orthogonal rotations."
            }
            (ScienceDomain::Physics, Stage::AdvancedCalculus) => {
                "Physics Context: Conservative gravitational/electrostatic potentials, Gauss flux div E, and fluid vorticity curl v."
            }
            (ScienceDomain::Physics, Stage::DifferentialEquations) => {
                "Physics Context: Damped harmonic oscillators, coupled pendula phase space, thermal conduction, and vibrating strings."
            }
            (ScienceDomain::Physics, Stage::ProbabilityStats) => {
                "Physics Context: Maxwell-Boltzmann thermal fluctuations, photon shot-noise Poisson counts, and detector error propagation."
            }

            // ==================== CHEMISTRY ====================
            (ScienceDomain::Chemistry, Stage::Notation) => "Chemistry Context: Decoding reaction-quotient Pi products, held-variable thermodynamic partials, and rate bars.",
            (ScienceDomain::Chemistry, Stage::Algebra) => match module_id {
                "alg-exp-log" => "Chemistry Context: Arrhenius activation rate k = A*exp(-E_a/(R*T)) and logarithmic pH / pKa equilibria.",
                "alg-quadratics" => "Chemistry Context: Weak-acid ICE table equilibrium quadratics K_a = x^2 / (C_0 - x).",
                _ => "Chemistry Context: Stoichiometric mass balance, Beer-Lambert calibration lines, and mole-fraction ratios.",
            },
            (ScienceDomain::Chemistry, Stage::Trigonometry) => {
                "Chemistry Context: Molecular orbital angular nodes, tetrahedral/octahedral bond angles, and X-ray Bragg diffraction phases."
            }
            (ScienceDomain::Chemistry, Stage::Calculus) => {
                "Chemistry Context: Instantaneous reaction rate -d[A]/dt, integrated rate laws, and reversible thermodynamic work."
            }
            (ScienceDomain::Chemistry, Stage::LinearAlgebra) => {
                "Chemistry Context: Chemical reaction stoichiometry matrices, Huckel MO secular eigenvalues, and spectral deconvolution."
            }
            (ScienceDomain::Chemistry, Stage::AdvancedCalculus) => {
                "Chemistry Context: Potential Energy Surfaces E(x, y), transition-state Hessian saddle points (D < 0), and Maxwell partials."
            }
            (ScienceDomain::Chemistry, Stage::DifferentialEquations) => {
                "Chemistry Context: Consecutive first-order kinetic chains A -> B -> C, Brusselator oscillations, and Fick's 2nd Law u_t = D*u_xx."
            }
            (ScienceDomain::Chemistry, Stage::ProbabilityStats) => {
                "Chemistry Context: Analytical titration replicate variance, partition functions, and linear calibration R^2."
            }

            // ==================== ECOLOGY ====================
            (ScienceDomain::Ecology, Stage::Notation) => "Ecology Context: Reading Shannon diversity Sigma sums, stage-matrix subscripts, and spatial dispersal Laplacians.",
            (ScienceDomain::Ecology, Stage::Algebra) => {
                "Ecology Context: Lincoln-Petersen mark-recapture ratios, metabolic scaling exponents, and resource supply lines."
            }
            (ScienceDomain::Ecology, Stage::Trigonometry) => {
                "Ecology Context: Seasonal photoperiod forcing, phenological cycles, and animal movement bearing vectors."
            }
            (ScienceDomain::Ecology, Stage::Calculus) => {
                "Ecology Context: Per-capita population growth (1/N)*dN/dt, Holling Type II functional responses, and net primary productivity integrals."
            }
            (ScienceDomain::Ecology, Stage::LinearAlgebra) => {
                "Ecology Context: Lefkovitch stage-classified matrices (dominant eigenvalue lambda_1 = asymptotic growth rate) and community food-web stability."
            }
            (ScienceDomain::Ecology, Stage::AdvancedCalculus) => {
                "Ecology Context: Landscape elevation/resource gradients, habitat suitability contours, and spatial dispersal flux."
            }
            (ScienceDomain::Ecology, Stage::DifferentialEquations) => {
                "Ecology Context: Logistic carrying capacity, Lotka-Volterra predator-prey phase cycles, and Fisher-KPP range expansion PDEs."
            }
            (ScienceDomain::Ecology, Stage::ProbabilityStats) => {
                "Ecology Context: Quadrat spatial count Poisson models, occupancy detection Bayes updates, and species-area log-linear regression."
            }

            // ==================== MICROBIOLOGY ====================
            (ScienceDomain::Microbiology, Stage::Notation) => "Microbiology Context: Unpacking Poisson MOI notation, chemostat normalized rates (1/X)*dX/dt, and replicate likelihood products.",
            (ScienceDomain::Microbiology, Stage::Algebra) => match module_id {
                "alg-exp-log" => "Microbiology Context: Exponential binary fission N(t) = N_0*exp(mu*t), generation doubling time ln(2)/mu, and thermal D-value log reduction.",
                _ => "Microbiology Context: Serial dilution factors, OD600 spectrophotometric curves, and Monod nutrient saturation.",
            },
            (ScienceDomain::Microbiology, Stage::Trigonometry) => {
                "Microbiology Context: Bacterial flagellar helical rotation kinematics and synchronized circadian cyanobacterial rhythms."
            }
            (ScienceDomain::Microbiology, Stage::Calculus) => {
                "Microbiology Context: Specific growth rate mu(S), antibiotic time-kill derivatives, and cumulative bioreactor product yield."
            }
            (ScienceDomain::Microbiology, Stage::LinearAlgebra) => {
                "Microbiology Context: Flux Balance Analysis (FBA) stoichiometric nullspaces S*v = 0 and multi-strain competition matrices."
            }
            (ScienceDomain::Microbiology, Stage::AdvancedCalculus) => {
                "Microbiology Context: Bacterial chemotaxis up attractant gradients grad c(x, y) and biofilm nutrient boundary layers."
            }
            (ScienceDomain::Microbiology, Stage::DifferentialEquations) => {
                "Microbiology Context: Chemostat washout ODEs (dX/dt = (mu(S) - D)*X), phage-bacteria lysis dynamics, and agar zone-of-inhibition diffusion PDEs."
            }
            (ScienceDomain::Microbiology, Stage::ProbabilityStats) => {
                "Microbiology Context: Luria-Delbruck mutation counts, Poisson Multiplicity of Infection (MOI) P(k = 0) = exp(-m), and MIC assay confidence bounds."
            }

            (ScienceDomain::General, _) => "",
        };

        Some(note.to_string())
    }
}

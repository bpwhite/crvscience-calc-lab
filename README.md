# CalcLab v0.5.0 — Native Rust Mathematics, Notation Literacy, PDEs & Statistical Inference Workbench

![Calc-Lab Headless Workbench](validation/headless_screenshot.png)

A standalone `#![forbid(unsafe_code)]`-core (`#![deny(unsafe_code)]` desktop shell outside localized Windows console attach) native Rust desktop application (`eframe` / `egui` + `egui_plot` + bundled `rusqlite`) featuring a **Main Menu Hub**, **8 Stage Portals with Focused 6-Module Drill-Down Views**, an interactive **Notation Studio**, **Timed Game Modes** (`Quick Blitz 90s`, `Medium 5m`, `Infinite`), **Step-by-Step Visual Method Walkthroughs**, **6 Scientific Domain Lenses**, and **8 Interactive Mathematical Laboratories** across **48 modules**.

- **User Documentation**: [`docs/USER_GUIDE.md`](docs/USER_GUIDE.md)
- **Numerical & Scientific Methods**: [`docs/SCIENTIFIC_METHODS.md`](docs/SCIENTIFIC_METHODS.md)
- **Headless Validation Telemetry**: [`validation/README.md`](validation/README.md) · [`validation/calc_lab_validation_summary.json`](validation/calc_lab_validation_summary.json)

---

## Quickstart (Build, Verify & Run)

```bash
# 1. Run all unit & scientific validation tests across the workspace
cargo test --workspace

# 2. Run the headless verification suite (48 modules x 6 domains = 288 problems, Notation Studio, Mat2/RK4/PDE/Stats, SQLite)
cargo run --release -p calc-app -- --self-test

# 3. Launch the native desktop GUI (opens into the Main Menu Hub)
cargo run --release -p calc-app
```

---

## Controls & Keyboard Shortcuts

| Shortcut / Control | Context | Action |
| :--- | :--- | :--- |
| `1` – `7` | Global *(when not typing in a text field)* | Switch across the 7 tabs: `1` Main Menu, `2` DAG Map, `3` Notation Studio, `4` Lesson, `5` Runs & Arena, `6` Visual Lab, `7` Leaderboard |
| `Space` | Visual Lab → 1D Spectral PDEs *(when not typing)* | Play / pause 1D PDE time evolution animation |
| `Ctrl+K` / `Cmd+K` | Global | Open Fuzzy Command Palette (jump across tabs, labs, ODE/PDE presets, integrators, exports) |
| `Ctrl+F` / `Cmd+F` | Global | Open the 48-module Curriculum Index sidebar and focus module search |
| `Ctrl+S` / `Cmd+S` | Global | Export SQLite progress & run history snapshot to JSON |
| `Enter` | Runs & Arena / Notation Flash Drill | Submit answer (1st press); advance to next problem in Learning Mode (2nd press) |
| `F1` | Global | Toggle *About Calc-Lab* modal |
| `Esc` | Global *(when not typing)* | Close *About* modal → dismiss reset prompt → return to *Main Menu* |
| **Mouse Click** | Multivariable Lab / ODE Phase Lab / OLS Scatter | Move gradient probe `(x0, y0)`, re-seed ODE `(x(0), y(0))`, or append `(x, y)` regression point |
| **Mouse Drag / Wheel / 2x-Click** | Multivariable 3D Surface Mesh (`z = f(x, y)`) | Orbit 3D yaw & pitch, zoom perspective scale, or double-click to reset camera |
| **Copy LaTeX** | Practice / Lesson / Notation Studio / Visual Lab | Copy clean rendered or parsed LaTeX expression source directly to clipboard |
| **Export ODE / PDE CSV...** | Visual Lab → ODE Phase & 1D PDE Sub-Labs | Save trajectory (`t,x_rk4,y_rk4[,x_euler,y_euler]`) or spatial PDE (`x,u_initial,u_m_t0,u_evolved`) curves to CSV |

---

## CLI Flags (`calc-app`)

| Flag | Description |
| :--- | :--- |
| *(none)* | Launch the native desktop GUI (`1460x910` viewport) |
| `-h`, `--help` | Print CLI usage instructions and exit |
| `-V`, `--version` | Print `calc-app` version and exit |
| `--self-test` | Run headless verification of 48 modules × 6 domains (`288` problems), Notation Studio, symbolic differentiation, ODE/PDE/Matrix/Stats solvers, and SQLite ledger |
| `--ode-benchmark` | Run Euler vs. Midpoint (RK2) vs. RK4 step-halving convergence benchmark (`--preset <harmonic\|damped\|lotka\|vanderpol>`, `--format <table\|json\|csv>`, `--out <PATH>`) |
| `--screenshot <PNG>` | Render a `1600x1000` offscreen RGBA PNG via `wgpu` and exit |
| `--screenshot-view <1\|2\|3>`, `--view <1\|2\|3>` | Select offscreen view for `--screenshot`: `1` = Practice Walkthrough, `2` = ODE Phase Lab, `3` = Notation Studio |

---

## Scientific Summary

- **Curriculum & Domain Framing**: 48 modules organized across 8 prerequisite-ordered stages (*0. Notation* through *VI. Prob & Stats*) with 6 domain lenses (*General Mathematics*, *Biology & Physiology*, *Physics & Mechanics*, *Chemistry & Kinetics*, *Ecology & Population Systems*, *Microbiology & Chemostats*).
- **Symbolic & Numerical Engine (`calc-core`)**:
  - Recursive-descent expression parser with symbolic differentiation, algebraic simplification, LaTeX formatting, and live syntax/variable validation (`validate_input_syntax`).
  - Adaptive Simpson quadrature with strict pole/domain `NaN` propagation (`adaptive_simpson`), Riemann & Darboux sums, and Taylor series expansion.
  - Scale-invariant $2 \times 2$ matrix diagnostics (`Mat2`): direct singular-value decomposition via $(a_{11} \pm a_{22})^2 + (a_{21} \mp a_{12})^2$ avoiding $A^T A$ condition-number squaring, plus Frobenius-scaled rank and spectral phase classification.
  - Classical 4th-order Runge-Kutta (`RK4`), 2nd-order Midpoint (`RK2`), and Explicit Euler ODE integrators with step-halving empirical convergence-order benchmarking (`JSON`/`CSV` export), multi-start damped Newton-Raphson planar equilibria detection, and marching-squares nullclines.
  - 1D Spectral Fourier-Galerkin PDE solver (`HeatDiffusion`, `WaveEquation` with full displacement + kinetic mechanical energy conservation, and periodic `AdvectionDiffusion`) with live mode-spectrum visualization.
  - $O(1)$ Lanczos $\ln\Gamma(z)$ / $\ln(k!)$ estimator (`ln_gamma`, `ln_factorial`), Normal/Exponential/Poisson/Binomial probability evaluators, Bayesian binary updating, and filtered OLS linear regression (`n >= 3`, $\text{df} = n - 2 \ge 1$).

---

## Validation Metrics

Verified deterministically by `cargo test --workspace --release` (`47 passed; 0 failed`: `calc-core` `20` unit + `4` validation, `calc-app` `23` unit) and logged in [`validation/calc_lab_validation_summary.json`](validation/calc_lab_validation_summary.json) & [`validation/rk4_convergence_table.csv`](validation/rk4_convergence_table.csv):

| Verification Suite | Key Measured Metric | Value |
| :--- | :--- | :--- |
| **Curriculum & Domain Sweep (`t1`)** | Verified `(module, domain)` problem pairs (`48 x 6`) | `288 / 288` pass (mean `2.67` steps/problem) |
| **ODE Step-Halving Convergence (`t2`)** | RK4 vs Euler empirical order on logistic IVP ($t \in [0, 2]$) | RK4 order `3.9909` (abs err `6.26e-11` at $\Delta t = 0.0125$) vs Euler `1.0026` (`1.88e-3`) |
| **1D Heat Diffusion PDE (`t3`)** | Max $L^2$ energy decay & Dirichlet boundary error | Max $L^2$ err `2.22e-16`; max boundary err `3.25e-16` |
| **1D Wave & Phase Equilibria (`t4`)** | Harmonic oscillator RK4 drift ($\Delta t = 0.05$) & Wave energy conservation | RK4 radius err `1.37e-8` (vs Euler `0.1704`); `WaveEquation` energy ratio `1.0` (`±1e-6` at $t = \pi/(2c)$) |

---

## Citation & CRV Science Apps Hub

- **CRV Science Apps Hub**: <https://crvscience.com/apps>
- **Companion Preprint (`APP-PREPRINT-03`)**: [Calc-Lab: Interactive Calculus, Linear Algebra, and Differential Equation Solver Workbench](https://crvscience.com/apps/preprints/calc-lab-scientific-calculus-ode)
- **Zenodo Concept DOI**: [10.5281/zenodo.24222372](https://doi.org/10.5281/zenodo.24222372)
- **Machine-Readable Metadata**: [`CITATION.cff`](CITATION.cff)

```bibtex
@article{White2026CalcLab,
  author  = {White, Bryan P.},
  title   = {Calc-Lab: Interactive Calculus, Linear Algebra, and Differential Equation Solver Workbench},
  journal = {CRV Science Apps Preprints},
  year    = {2026},
  doi     = {10.5281/zenodo.24222372},
  url     = {https://crvscience.com/apps/preprints/calc-lab-scientific-calculus-ode},
  note    = {Software v0.5.0; Preprint ID: APP-PREPRINT-03}
}
```

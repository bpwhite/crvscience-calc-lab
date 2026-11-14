# Calc-Lab Headless Validation Suite

This directory holds empirical verification datasets generated deterministically by running the `calc-core` integration suite in `crates/calc-core/tests/headless_validation.rs`.

## Running the Validation Suite

```bash
cargo test
# or target only the calc-core headless validation tests:
cargo test -p calc-core --test headless_validation
```

## Test Coverage & Generated Artifacts

1. `t1_48_modules_6_domains_verification_sweep`: Exhaustively generates, derives step-by-step solutions for, and self-grades all 48 curriculum modules across all 6 scientific domains (288 `(module, domain)` pairs), plus Notation flashcards, `Mat2` spectral phase diagnostics, Bayesian/OLS statistical estimators, and `ProfileStore` SQLite mastery persistence.
2. `t2_rk4_vs_euler_step_halving_convergence`: Audits global truncation error and empirical step-halving convergence order (`O(h^4)` RK4 vs. `O(h^1)` explicit Euler) on the nonlinear logistic IVP `dy/dt = 1.2 * y * (1 - y)`, `y(0) = 0.1` over `t in [0, 2]` -> writes [`rk4_convergence_table.csv`](rk4_convergence_table.csv).
3. `t3_1d_heat_pde_l2_decay_and_modal_stability`: Verifies 1D parabolic heat/diffusion PDE Fourier-Galerkin evolution (`u_t = alpha * u_xx`), exact analytical L2 energy decay, modal attenuation rates, Dirichlet boundary conditions (`u(0) = u(pi) = 0`), and maximum-principle dissipation across `TwoModeHarmonic`, `LocalizedGaussian`, and `SquarePulse` profiles -> consolidates all results into [`calc_lab_validation_summary.json`](calc_lab_validation_summary.json).

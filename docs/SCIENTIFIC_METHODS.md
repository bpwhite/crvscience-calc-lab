# Calc-Lab Scientific & Numerical Methods Reference

This document summarizes the mathematical formulations, numerical algorithms, conditioning guards, and singularity protections implemented in `calc-core`.

---

## 1. 1D Spectral Fourier-Galerkin PDEs & Wave Energy Conservation (`ode_pde.rs`)

### Modal Expansions

1. **1D Heat / Diffusion** on $x \in [0, \pi]$ with homogeneous Dirichlet boundary conditions $u(0, t) = u(\pi, t) = 0$:
   $$u_t = \alpha u_{xx}, \qquad u(x, t) = \sum_{n=1}^{M} b_n e^{-\alpha n^2 t} \sin(nx)$$
2. **1D Wave Equation** on $x \in [0, \pi]$ with fixed endpoints $u(0, t) = u(\pi, t) = 0$ and zero initial velocity $u_t(x, 0) = 0$:
   $$u_{tt} = c^2 u_{xx}, \qquad u(x, t) = \sum_{n=1}^{M} b_n \cos(c n t) \sin(nx)$$
3. **1D Periodic Advection-Diffusion** on $x \in [0, 2\pi]$:
   $$u_t + v u_x = \alpha u_{xx}, \qquad u(x, t) = a_0 + \sum_{n=1}^{M} e^{-\alpha n^2 t}\bigl[a_n \cos(n(x - vt)) + b_n \sin(n(x - vt))\bigr]$$

Fourier coefficients ($b_n$, or $a_0, a_n, b_n$) are projected from $u(x, 0)$ via $480$-point midpoint quadrature on the domain $[0, L]$, with relative $L^2$ truncation error $\|u(\cdot, 0) - u_M(\cdot, 0)\|_2 / \|u(\cdot, 0)\|_2$ reported alongside evolved profiles.

### Mechanical Energy Conservation in the Wave Equation

Evaluating only the displacement squared $\sum_{n=1}^{M} b_n^2 \cos^2(c n t)$ ignores the kinetic velocity field $u_t(x, t) = \sum_{n=1}^{M} (-c n b_n \sin(c n t)) \sin(nx)$, which causes apparent energy to drop to zero at quarter-period nodes $t = \pi / (2c)$ where all odd harmonics cross equilibrium simultaneously.

In `evaluate_1d_pde_with_fn`, each mode $k = n$ in `PdeKind::WaveEquation` sums both its displacement/potential contribution and its normalized kinetic energy term:

$$b_k(t) = b_k \cos(c k t), \qquad \kappa_k(t) = b_k \sin(c k t)$$

$$E_{\text{norm}}(t) = \sum_{k=1}^{M} \Bigl(b_k(t)^2 + \kappa_k(t)^2\Bigr) = \sum_{k=1}^{M} b_k^2 \bigl(\cos^2(ckt) + \sin^2(ckt)\bigr) = E_{\text{norm}}(0)$$

This preserves $E_{\text{norm}}(t) / E_{\text{norm}}(0) = 1.0$ to machine precision across all times $t$, including $t = \pi / (2c)$ (verified to within $\pm 10^{-6}$ across built-in and custom profiles).

---

## 2. Direct $2 \times 2$ SVD & Frobenius-Scaled Matrix Diagnostics (`linalg.rs`)

### Avoiding Condition-Number Squaring in Singular Values

Forming the normal matrix $A^T A$ squares the condition number $\kappa(A^T A) = \kappa(A)^2$. In IEEE-754 `f64` ($\varepsilon_{\text{mach}} \approx 2.22 \times 10^{-16}$), any matrix with $\kappa(A) \ge 10^8$ loses the smaller singular value $\sigma_2$ to zero via subtractive cancellation in $\operatorname{tr}(A^T A) - \sqrt{\operatorname{tr}(A^T A)^2 - 4\det(A^T A)}$.

`Mat2::singular_values` avoids $A^T A$ completely by scaling $A$ by $m = \max_{i,j}|a_{ij}|$ (preventing intermediate overflow/underflow) to $B = A / m$ and computing the direct $2 \times 2$ orthogonal invariants:

$$s_+ = \sqrt{(b_{11} + b_{22})^2 + (b_{21} - b_{12})^2}, \qquad s_- = \sqrt{(b_{11} - b_{22})^2 + (b_{21} + b_{12})^2}$$

evaluated via overflow-safe `.hypot(...)`. Since $\{s_+, s_-\} = \{\sigma_1/m + \sigma_2/m,\; |\sigma_1/m - \sigma_2/m|\}$:

$$\sigma_1 = \frac{m}{2}(s_+ + s_-), \qquad \sigma_2 = m \cdot \min\!\left(\frac{\sigma_1}{m},\; \frac{|\det(B)|}{\sigma_1/m}\right)$$

This recovers both singular values accurately even for extreme condition numbers such as $\kappa(A) = 10^9$ (and rank-1 singular matrices where $\det(B) = 0$). Numerical rank (`Mat2::rank`) and invertibility (`Mat2::inverse`) use the relative singular-value threshold $\sigma_2 > 10^{-12} \sigma_1$.

### Frobenius-Scaled Spectral & Phase Classification

In `Mat2::diagnostics`, fixed hard-coded thresholds (e.g. `1e-7`) misclassify well-conditioned small matrices such as $10^{-4} I$ (whose determinant is $10^{-8} < 10^{-7}$) as degenerate $\det(A) = 0$.

Instead, tolerances are dimensionally matched to the Frobenius norm $\|A\|_F = \sqrt{a_{11}^2 + a_{12}^2 + a_{21}^2 + a_{22}^2}$:
- **Quadratic invariants** ($\det(A)$ and $\Delta = \operatorname{tr}(A)^2 - 4\det(A)$): $\varepsilon_{\text{sq}} = 10^{-12} \|A\|_F^2$.
- **Linear invariants** ($\operatorname{tr}(A)$ and scalar-matrix check $|a_{11} - \lambda|, |a_{12}|, |a_{21}|, |a_{22} - \lambda|$): $\varepsilon_{\text{lin}} = 10^{-7} \|A\|_F$.

Under this scaling, `Mat2::new(1e-4, 0.0, 0.0, 1e-4)` preserves `rank = 2`, `geometric_mult = 2`, and is classified accurately as `"Unstable Star / Improper Node (repeated lambda > 0)"`.

---

## 3. Adaptive Simpson Quadrature & Pole `NaN` Propagation (`expr.rs`)

`adaptive_simpson(expr, var, a, b, eps, max_depth)` applies recursive Simpson's 1/3 rule with one-step Richardson extrapolation:

$$S(a, b) \approx S_L + S_R + \frac{S_L + S_R - S_{\text{whole}}}{15}$$

When integrating across domain violations (e.g. $\sqrt{x}$ on $[-1, 2]$) or singular poles (e.g. $1/(x - 0.5)$ on $[0, 2]$ or $1/x$ on $[-1, 3]$), endpoints or interior subdivision midpoints ($c$, $d$, $e$) evaluate to `NaN` or non-finite values. Rather than returning a stale coarse `whole` estimate from an outer interval, `adaptive_simpson` checks finiteness at the root and every recursive evaluation (`fa`, `fb`, `fc`, `fd`, `fe`, `left`, `right`) and immediately propagates `f64::NAN`.

---

## 4. Ordinary Least Squares Filtering & Degrees-of-Freedom Guard (`prob_stats.rs`)

`fit_ols_linear(points: &[[f64; 2]])` fits $\hat{y} = b_0 + b_1 x$ with correlation $r$, $R^2$, and residual standard error:
1. **Non-Finite Filtering**: Any pair with non-finite `x` or `y` (`NaN`, `±Inf`) is filtered out prior to accumulating sample means and second moments.
2. **Degrees-of-Freedom Guard ($n \le 2$)**: Linear regression estimates $2$ parameters ($b_0, b_1$), giving residual degrees of freedom $\text{df} = n - 2$. When $n \le 2$ finite points remain, $\text{df} \le 0$ leaves residual standard error $\sqrt{\text{SSE}/(n-2)}$ undefined, so `fit_ols_linear` returns `None`.
3. **Collinearity & Finiteness Check**: Vertical collinearity ($S_{xx} \le 10^{-12}$) or non-finite intermediate sums return `None`.

---

## 5. Fast $O(1)$ Lanczos $\ln\Gamma(z)$ & $\ln(k!)$ Approximation (`prob_stats.rs`)

To evaluate Poisson and Binomial log-likelihoods without $O(k)$ loop summation overhead or floating-point drift, `ln_gamma(z)` implements the 9-term Godfrey Lanczos approximation ($g = 7.0$) for $\operatorname{Re}(z) \ge 0.5$:

$$\ln\Gamma(z) = \frac{1}{2}\ln(2\pi) + \left(z - \frac{1}{2}\right)\ln(z + g - 0.5) - (z + g - 0.5) + \ln\!\left(c_0 + \sum_{i=1}^{8} \frac{c_i}{z - 1 + i}\right)$$

combined with Euler's reflection formula $\ln\Gamma(z) = \ln\pi - \ln|\sin(\pi z)| - \ln\Gamma(1 - z)$ for $z < 0.5$. `ln_factorial(k)` returns exact `0.0` for $k \in \{0, 1\}$ and delegates to `ln_gamma((k as f64) + 1.0)` for $k \ge 2$, achieving $< 10^{-13}$ absolute error on small integers and $< 10^{-14}$ relative error at $k = 10^5$ in constant $O(1)$ time.

---

## 6. Planar ODE Integration, Equilibria & Nullclines (`ode_pde.rs`)

- **Integrators**: Classical 4th-order Runge-Kutta (`solve_scalar_ode_rk4`, `solve_planar_system_rk4`, `solve_planar_system_rk4_timed`, $O(h^4)$ global truncation error) alongside Explicit Euler (`solve_planar_system_euler_timed`, $O(h^1)$) for live step-error comparison.
- **Equilibria & Nullclines**: `find_planar_equilibria` differentiates $P(x, y)$ and $Q(x, y)$ symbolically to form the exact $2 \times 2$ Jacobian $J(x, y)$ and runs multi-start damped Newton-Raphson with backtracking line search across the active viewport. `extract_zero_level_segments` extracts $P(x, y) = 0$ and $Q(x, y) = 0$ nullclines via marching squares.

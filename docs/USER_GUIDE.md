# Calc-Lab User Guide (`v0.5.0`)

Calc-Lab is a standalone native Rust desktop workbench (`eframe` / `egui` + `egui_plot` + bundled SQLite) for mastering mathematical notation, symbolic calculus, linear algebra, vector calculus, differential equations, and statistical inference across **48 modules**, **8 stages**, **6 scientific domain lenses**, and **8 interactive mathematical laboratories**.

---

## 1. The 7 Application Tabs

Switch between the 7 primary workspaces via the top navigation bar or using the number keys **`1`–`7`** (whenever a text input field is not focused):

| Hotkey | Tab | Purpose |
| :--- | :--- | :--- |
| `1` (or `Esc`) | **Main Menu** | Distraction-free launchpad showing Recommended Next-Step reviews, Scientific Domain pills, 1-click run launchers (*Learning*, *Quick Blitz 90s*, *Medium 5m*, *Infinite*), and 8 Stage Portals with focused 6-module drill-downs. |
| `2` | **DAG Map** | Interactive 48-module prerequisite dependency graph showing unlocked, review-due, and mastered modules across all 8 stages (`0. Notation` through `VI. Prob & Stats`). |
| `3` | **Notation Studio** | Side-by-side decoder of textbook mathematical symbols (`∀`, `∃`, `∑`, `∏`, `∫`, `∬`, `∮`, `∇`, `∂`, `θ̂`, etc.), plain-text Calc-Lab syntax, spoken phrasing, dialect equivalents, domain meanings, **Copy LaTeX**, and a rapid symbol recognition flash drill. |
| `4` | **Lesson** | First-principles concept synthesis, upstream prerequisite jump buttons, downstream scientific bridge, rendered 2D LaTeX core identities, and a complete worked reference example with **Copy LaTeX** buttons. |
| `5` | **Runs & Arena** | Guided step-by-step visual walkthroughs and timed calculation sprints with live syntax feedback, 2D LaTeX input preview, companion step plots, and an interactive symbolic derivative/numeric probe scratchpad. |
| `6` | **Visual Lab** | Eight interactive visualization laboratories covering Algebra, Trigonometry, Single-Variable Calculus, Linear Algebra & Eigen-Maps, Epsilon-Delta Proofs, Multivariable Fields (2D & 3D), ODE Phase Portraits / 1D Spectral PDEs, and Probability/Bayes/OLS Regression. |
| `7` | **Leaderboard** | High-score cards across all 4 run modes, a 48-module mastery bar chart, spaced-repetition review schedules, recent attempt audit table, SQLite profile reset, and JSON snapshot export (`Ctrl+S`). |

---

## 2. Sidebar Index, Scientific Domains & Copy LaTeX

- **Collapsible 48-Module Index (`Ctrl+F`)**: Press `Ctrl+F` (or `Cmd+F` on macOS) from any tab to open the curriculum index sidebar and focus the module filter input immediately. Search by module code (e.g. `DEQ-503`), title, stage, or topic keyword (e.g. `Fourier`).
- **6 Scientific Domain Lenses**: Use the **Domain** dropdown in the top bar to switch problem framing and contextual banners across *General Mathematics*, *Biology & Physiology*, *Physics & Mechanics*, *Chemistry & Kinetics*, *Ecology & Population Systems*, and *Microbiology & Chemostats*.
- **Themes & Unlock All**: Toggle between **Slate Dark** and **Paper Light** academic themes, or toggle **Unlock All** to bypass prerequisite gating and jump into any module freely.
- **Copy LaTeX Buttons**: Rendered problem equations, live answer previews, final solutions, Notation Studio symbol cards, Lesson worked examples, and Visual Lab equations each provide a **Copy LaTeX** button that copies clean LaTeX source to your clipboard with brief visual confirmation.

---

## 3. Expression Syntax & Practice Mode Live Parser

### Plain-Text Mathematical Syntax

Calc-Lab accepts standard mathematical plain text across **Runs & Arena**, **Symbolic Scratchpad**, and **Visual Lab** expression boxes:

- **Arithmetic & Powers**: `+`, `-`, `*`, `/`, `^` (right-associative powers, e.g. `e^(-x^2)`). Implicit multiplication such as `2*x`, `2x`, `3(x + 1)`, or `2 sin(x)` is supported; explicit `*` is recommended for unambiguous grouping.
- **Grouping**: Parentheses `(...)`, brackets `[...]`, and braces `{...}` are treated as grouping delimiters. Use `abs(x)` instead of vertical bars `|x|`.
- **Constants & Scientific Notation**: `pi` (or `π`), `e` (Euler's number), and scientific exponent literals such as `1e-5`, `1.5e2`, or `2.5E+3`.
- **Functions**: `sin`, `cos`, `tan`, `sec`, `csc`, `cot`, `asin` / `arcsin`, `acos` / `arccos`, `atan` / `arctan`, `sinh`, `cosh`, `tanh`, `exp`, `ln` / `log` (natural logarithm), `sqrt`, `abs`.
- **2D Vectors**: Enclose comma-separated components in angle brackets `<P, Q>` (or `[P, Q]` / `(P, Q)`), e.g., `<2*x*y, x^2 + 3>`.

### Practice Mode Live Syntax Helper & Parse Error Badge

- Directly beneath the format hint in **Runs & Arena**, an inline **Syntax Helper** bar summarizes supported operators, constants, functions, and 2D vector syntax.
- As you type into the **Answer** box, Calc-Lab validates your input immediately against the active problem's expected `AnswerFormat` (`Numeric`, `ScalarExpr`, or `Vector2Expr`):
  - **Valid input** displays a green **`Syntax OK`** pill, a live rendered 2D LaTeX preview, and a **Copy LaTeX** button.
  - **Invalid input** (such as unmatched parentheses, missing function parentheses, unknown/free variables on a numeric question, unbound variables on a symbolic expression, or undefined evaluation like `1/0`) displays a framed **`PARSE ERROR`** badge with an exact diagnostic message before you submit.
- **Keyboard Flow**: Press `Enter` once to grade your answer; in *Learning Mode*, press `Enter` a second time (without editing) to advance to the next problem. In timed modes (*Quick Blitz 90s*, *Medium Run 5m*), correct submissions auto-advance immediately.

---

## 4. Visual Lab Modes (`Tab 6`)

1. **Algebra & Difference Quotients**: Plot primary `f(x)` and secondary `g(x)`, scrub anchor `x0` and secant step `h`, inspect difference quotient convergence toward symbolic `f'(x0)`, and copy `f(x)` or `f'(x)` LaTeX.
2. **Trig Unit-Wave**: Coupled unit circle `(cos θ, sin θ, tan θ)` and sinusoidal wave `A sin(ωt + φ)` with continuous rotation animation.
3. **Single-Variable Calculus**: Toggle Secant & Exact Tangent, Riemann/Darboux partitions (`Left`, `Right`, `Midpoint`, `Trapezoid`, `Lower Darboux`, `Upper Darboux`) vs Adaptive Simpson quadrature, and Taylor polynomial `T_n(x)` overlays.
4. **Linear Algebra & Eigen-Maps**: Edit $2 \times 2$ matrix entries or load presets (*Rotation 45 deg*, *Horizontal Shear*, *Symmetric SPD*, *Hyperbolic Saddle*, *Damped Spiral*, *Rank-1 Projection*) to view unit-square distortion (`det A`), SVD ellipse `A(S^1)`, real eigen-axes, mapped vector `A*v`, and the `(tr A, det A)` stability plane.
5. **Epsilon-Delta Proofs**: Visually test whether candidate `δ > 0` keeps every `x` in `0 < |x - x0| < δ` strictly inside `(L - ε, L + ε)`, highlighting violating points in red.
6. **Multivariable Fields (2D Contours & Rotatable 3D Surface Mesh)**:
   - **Auto-Scaled Diverging Contours**: Marching-squares level curves `f(x, y) = c` automatically adapt levels to the `[z_min, z_max]` range on `[-3, 3]^2`, colored with a perceptual blue–neutral–coral diverging ramp and labeled with non-overlapping contour values and `x`/`y` axis labels.
   - **Vector Quiver & Gradient Ascent**: Click anywhere in the 2D plot to move probe `(x0, y0)`, inspect `∇f`, Hessian `H`, `div F`, and `curl_z F`, or click **Set F = grad f**.
   - **Toggleable 3D Surface Mesh `z = f(x, y)`**: Check **3D Surface Mesh z=f(x,y)** to display a side-by-side perspective surface mesh. Drag on the 3D canvas to rotate yaw/pitch, scroll the mouse wheel to zoom, double-click (or click **Reset 3D View**) to reset camera orientation, and watch the orange probe dot track `(x0, y0, f(x0, y0))` live.
7. **ODE Phase Portraits & 1D Spectral PDEs**:
   - **A. 2D ODE Systems & RK4 Phase Portraits**:
     - Enter autonomous `dx/dt = P(x, y)` and `dy/dt = Q(x, y)` or pick from 5 presets.
     - Direction field arrows render with **directional arrowheads** and **adaptive zoom/pan resampling**—panning or zooming the phase plot automatically recomputes the direction grid, nullclines (`dx/dt = 0`, `dy/dt = 0`), and damped Newton-Raphson equilibria `(x*, y*)` across the visible viewport. Click **Reset View** to return to `[-3.5, 3.5]^2`.
     - Click anywhere on the phase plane to re-seed `(x(0), y(0))`, compare 4th-order RK4 against 1st-order Explicit Euler across phase space and `x(t)`/`y(t)` time series, hover any control for tooltips, copy system LaTeX, or click **Export ODE Curves CSV...**.
   - **B. 1D Spectral PDEs (Heat, Wave & Advection)**:
     - Switch between *1D Heat / Diffusion*, *1D Wave Equation*, and *1D Advection-Diffusion* with built-in profiles or custom `u(x, 0)` expressions.
     - Press **`Space`** (when not focused in a text field) or toggle **Animate Evolution** to play/pause time evolution.
     - Inspect the spatial profile `u(x, t)` (`Position x` vs `Amplitude u(x, t)`) side-by-side with the **Fourier Mode Spectrum Bar Chart** comparing initial mode amplitudes `|c_n(0)|` against evolved active amplitudes `|c_n(t)|`.
     - Hover any PDE or Fourier parameter for details, copy the governing PDE or custom initial profile as LaTeX, or click **Export PDE Curves CSV...** to save `x,u_initial,u_m_t0,u_evolved`.
8. **Probability, Bayes & OLS Regression**: Explore Normal, Exponential, Poisson, and Binomial distributions with shaded interval probabilities alongside an interactive Bayesian diagnostic posterior updater or click-to-add OLS regression scatter plot (`n >= 3` finite points).

---

## 5. Keyboard Shortcuts & CLI Options

### Keyboard Shortcuts

| Shortcut | Scope | Action |
| :--- | :--- | :--- |
| `1` – `7` | Global (when not typing in a text box) | Switch directly to Tabs 1–7 (*Main Menu*, *DAG Map*, *Notation Studio*, *Lesson*, *Runs & Arena*, *Visual Lab*, *Leaderboard*) |
| `Space` | Visual Lab → 1D PDE Sub-Lab (when not typing) | Play / Pause 1D PDE time evolution animation |
| `Ctrl+F` / `Cmd+F` | Global | Open 48-module Curriculum Index sidebar and focus module search |
| `Ctrl+S` / `Cmd+S` | Global | Open native save dialog to export JSON progress & run history snapshot |
| `Enter` | Runs & Arena / Notation Flash Drill | Submit answer (1st press) / Advance to next question (2nd press) |
| `F1` | Global | Toggle *About Calc-Lab* modal |
| `Esc` | Global (when not typing) | Close *About* modal → Dismiss profile reset confirmation → Return to *Main Menu* |

### CLI Flags (`calc-app`)

```text
calc-app [OPTIONS]

  -h, --help                         Print help text and exit
  -V, --version                      Print version information and exit
      --self-test                    Run headless curriculum, symbolic, ODE/PDE, and SQLite verification
      --screenshot <PNG>             Render an offscreen 1600x1000 RGBA PNG screenshot and exit
      --screenshot-view <1|2|3>, --view <1|2|3>
                                     Select screenshot view (1 = Practice, 2 = ODE Phase Lab, 3 = Notation Studio)
```

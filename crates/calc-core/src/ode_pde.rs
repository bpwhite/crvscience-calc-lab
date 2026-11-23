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

use crate::expr::{EvalEnv, Expr};
use crate::linalg::{Mat2, Mat2Diagnostics};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Integrates a 1st-order scalar ODE dy/dt = f(t, y) via classical 4th-order Runge-Kutta (RK4).
pub fn solve_scalar_ode_rk4(
    rhs: &Expr,
    t0: f64,
    y0: f64,
    dt: f64,
    steps: usize,
) -> Vec<[f64; 2]> {
    let mut out = Vec::with_capacity(steps + 1);
    let mut t = t0;
    let mut y = y0;
    out.push([t, y]);

    let eval_ty = |tv: f64, yv: f64| -> f64 {
        let mut env = EvalEnv::new();
        env.insert("t".to_string(), tv);
        env.insert("x".to_string(), tv);
        env.insert("y".to_string(), yv);
        rhs.eval(&env)
    };

    for _ in 0..steps {
        let k1 = eval_ty(t, y);
        let k2 = eval_ty(t + 0.5 * dt, y + 0.5 * dt * k1);
        let k3 = eval_ty(t + 0.5 * dt, y + 0.5 * dt * k2);
        let k4 = eval_ty(t + dt, y + dt * k3);
        let next_y = y + (dt / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        if !next_y.is_finite() || next_y.abs() > 1e4 {
            break;
        }
        t += dt;
        y = next_y;
        out.push([t, y]);
    }
    out
}

/// Integrates a 2D autonomous planar system dx/dt = P(x, y), dy/dt = Q(x, y) via RK4.
pub fn solve_planar_system_rk4(
    rhs_p: &Expr,
    rhs_q: &Expr,
    x0: f64,
    y0: f64,
    dt: f64,
    steps: usize,
    bound: f64,
) -> Vec<[f64; 2]> {
    let mut traj = Vec::with_capacity(steps + 1);
    if !x0.is_finite() || !y0.is_finite() || x0.abs() > bound || y0.abs() > bound {
        return traj;
    }
    let mut x = x0;
    let mut y = y0;
    traj.push([x, y]);

    for _ in 0..steps {
        let k1x = rhs_p.eval_2d(x, y);
        let k1y = rhs_q.eval_2d(x, y);

        let k2x = rhs_p.eval_2d(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y);
        let k2y = rhs_q.eval_2d(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y);

        let k3x = rhs_p.eval_2d(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y);
        let k3y = rhs_q.eval_2d(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y);

        let k4x = rhs_p.eval_2d(x + dt * k3x, y + dt * k3y);
        let k4y = rhs_q.eval_2d(x + dt * k3x, y + dt * k3y);

        let nx = x + (dt / 6.0) * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        let ny = y + (dt / 6.0) * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);

        if !nx.is_finite() || !ny.is_finite() || nx.abs() > bound || ny.abs() > bound {
            break;
        }
        x = nx;
        y = ny;
        traj.push([x, y]);
    }
    traj
}

/// Integrates a 2D autonomous planar system dx/dt = P(x, y), dy/dt = Q(x, y) via RK4, returning `[t, x, y]`.
pub fn solve_planar_system_rk4_timed(
    rhs_p: &Expr,
    rhs_q: &Expr,
    x0: f64,
    y0: f64,
    dt: f64,
    steps: usize,
    bound: f64,
) -> Vec<[f64; 3]> {
    let mut traj = Vec::with_capacity(steps + 1);
    if !x0.is_finite() || !y0.is_finite() || x0.abs() > bound || y0.abs() > bound {
        return traj;
    }
    let mut t = 0.0_f64;
    let mut x = x0;
    let mut y = y0;
    traj.push([t, x, y]);

    for _ in 0..steps {
        let k1x = rhs_p.eval_2d(x, y);
        let k1y = rhs_q.eval_2d(x, y);

        let k2x = rhs_p.eval_2d(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y);
        let k2y = rhs_q.eval_2d(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y);

        let k3x = rhs_p.eval_2d(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y);
        let k3y = rhs_q.eval_2d(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y);

        let k4x = rhs_p.eval_2d(x + dt * k3x, y + dt * k3y);
        let k4y = rhs_q.eval_2d(x + dt * k3x, y + dt * k3y);

        let nx = x + (dt / 6.0) * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        let ny = y + (dt / 6.0) * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);

        if !nx.is_finite() || !ny.is_finite() || nx.abs() > bound || ny.abs() > bound {
            break;
        }
        t += dt;
        x = nx;
        y = ny;
        traj.push([t, x, y]);
    }
    traj
}

/// Integrates a 2D autonomous planar system dx/dt = P(x, y), dy/dt = Q(x, y) via Explicit Euler, returning `[t, x, y]`.
pub fn solve_planar_system_euler_timed(
    rhs_p: &Expr,
    rhs_q: &Expr,
    x0: f64,
    y0: f64,
    dt: f64,
    steps: usize,
    bound: f64,
) -> Vec<[f64; 3]> {
    let mut traj = Vec::with_capacity(steps + 1);
    if !x0.is_finite() || !y0.is_finite() || x0.abs() > bound || y0.abs() > bound {
        return traj;
    }
    let mut t = 0.0_f64;
    let mut x = x0;
    let mut y = y0;
    traj.push([t, x, y]);

    for _ in 0..steps {
        let vx = rhs_p.eval_2d(x, y);
        let vy = rhs_q.eval_2d(x, y);
        let nx = x + dt * vx;
        let ny = y + dt * vy;
        if !nx.is_finite() || !ny.is_finite() || nx.abs() > bound || ny.abs() > bound {
            break;
        }
        t += dt;
        x = nx;
        y = ny;
        traj.push([t, x, y]);
    }
    traj
}

/// Extracts zero-level contour segments `f(x, y) = 0` via marching squares on a `(grid_n + 1)^2` grid.
pub fn extract_zero_level_segments(
    expr: &Expr,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    grid_n: usize,
) -> Vec<([f64; 2], [f64; 2])> {
    let n = grid_n.max(8);
    let mut vals = vec![vec![f64::NAN; n + 1]; n + 1];
    let dx = (x_max - x_min) / (n as f64);
    let dy = (y_max - y_min) / (n as f64);

    for i in 0..=n {
        let x = x_min + (i as f64) * dx;
        for j in 0..=n {
            let y = y_min + (j as f64) * dy;
            vals[i][j] = expr.eval_2d(x, y);
        }
    }

    let interp = |p1: [f64; 2], p2: [f64; 2], v1: f64, v2: f64| -> [f64; 2] {
        let denom = v1 - v2;
        let t = if denom.abs() < 1e-15 {
            0.5
        } else {
            (v1 / denom).clamp(0.0, 1.0)
        };
        [p1[0] + t * (p2[0] - p1[0]), p1[1] + t * (p2[1] - p1[1])]
    };

    let mut segs = Vec::new();
    for i in 0..n {
        let x0 = x_min + (i as f64) * dx;
        let x1 = x_min + ((i + 1) as f64) * dx;
        for j in 0..n {
            let y0 = y_min + (j as f64) * dy;
            let y1 = y_min + ((j + 1) as f64) * dy;

            // Corners: 0=BL(x0,y0), 1=BR(x1,y0), 2=TR(x1,y1), 3=TL(x0,y1)
            let v0 = vals[i][j];
            let v1 = vals[i + 1][j];
            let v2 = vals[i + 1][j + 1];
            let v3 = vals[i][j + 1];
            if !v0.is_finite() || !v1.is_finite() || !v2.is_finite() || !v3.is_finite() {
                continue;
            }

            let p0 = [x0, y0];
            let p1 = [x1, y0];
            let p2 = [x1, y1];
            let p3 = [x0, y1];

            let mut crossings: Vec<(usize, [f64; 2])> = Vec::with_capacity(4);
            // Edge 0: BL -> BR (0..1)
            if (v0 >= 0.0) != (v1 >= 0.0) {
                crossings.push((0, interp(p0, p1, v0, v1)));
            }
            // Edge 1: BR -> TR (1..2)
            if (v1 >= 0.0) != (v2 >= 0.0) {
                crossings.push((1, interp(p1, p2, v1, v2)));
            }
            // Edge 2: TR -> TL (2..3)
            if (v2 >= 0.0) != (v3 >= 0.0) {
                crossings.push((2, interp(p2, p3, v2, v3)));
            }
            // Edge 3: TL -> BL (3..0)
            if (v3 >= 0.0) != (v0 >= 0.0) {
                crossings.push((3, interp(p3, p0, v3, v0)));
            }

            if crossings.len() == 2 {
                segs.push((crossings[0].1, crossings[1].1));
            } else if crossings.len() == 4 {
                let center = 0.25 * (v0 + v1 + v2 + v3);
                if (center >= 0.0) == (v0 >= 0.0) {
                    segs.push((crossings[0].1, crossings[3].1));
                    segs.push((crossings[1].1, crossings[2].1));
                } else {
                    segs.push((crossings[0].1, crossings[1].1));
                    segs.push((crossings[2].1, crossings[3].1));
                }
            }
        }
    }
    segs
}

#[derive(Debug, Clone)]
pub struct EquilibriumPoint {
    pub x: f64,
    pub y: f64,
    pub jacobian: Mat2,
    pub diagnostics: Mat2Diagnostics,
}

/// Finds planar equilibrium points `P(x, y) = 0, Q(x, y) = 0` inside `[x_min, x_max] x [y_min, y_max]`
/// using symbolic Jacobian partials and multi-start damped Newton-Raphson.
pub fn find_planar_equilibria(
    rhs_p: &Expr,
    rhs_q: &Expr,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> Vec<EquilibriumPoint> {
    let px_expr = rhs_p.differentiate("x");
    let py_expr = rhs_p.differentiate("y");
    let qx_expr = rhs_q.differentiate("x");
    let qy_expr = rhs_q.differentiate("y");

    let mut seeds = Vec::with_capacity(90);
    for ix in 0..9 {
        let sx = x_min + ((ix as f64 + 0.5) / 9.0) * (x_max - x_min);
        for iy in 0..9 {
            let sy = y_min + ((iy as f64 + 0.5) / 9.0) * (y_max - y_min);
            seeds.push([sx, sy]);
        }
    }
    seeds.extend_from_slice(&[
        [0.0, 0.0],
        [PI, 0.0],
        [-PI, 0.0],
        [1.5, 1.5],
        [1.0, 1.0],
    ]);

    let mut found: Vec<EquilibriumPoint> = Vec::new();
    let pad = 1e-4;

    for [sx, sy] in seeds {
        if sx < x_min - pad || sx > x_max + pad || sy < y_min - pad || sy > y_max + pad {
            continue;
        }
        let mut x = sx;
        let mut y = sy;
        let mut converged = false;

        for _ in 0..24 {
            let f1 = rhs_p.eval_2d(x, y);
            let f2 = rhs_q.eval_2d(x, y);
            if !f1.is_finite() || !f2.is_finite() {
                break;
            }
            let res = f1.hypot(f2);
            if res < 1e-10 {
                converged = true;
                break;
            }
            let j11 = px_expr.eval_2d(x, y);
            let j12 = py_expr.eval_2d(x, y);
            let j21 = qx_expr.eval_2d(x, y);
            let j22 = qy_expr.eval_2d(x, y);
            let det = j11 * j22 - j12 * j21;
            if !det.is_finite() || det.abs() < 1e-12 {
                break;
            }
            let dx = (-f1 * j22 + f2 * j12) / det;
            let dy = (f1 * j21 - f2 * j11) / det;
            if !dx.is_finite() || !dy.is_finite() {
                break;
            }

            // Backtracking line search
            let mut alpha = 1.0;
            let mut accepted = false;
            for _ in 0..8 {
                let nx = x + alpha * dx;
                let ny = y + alpha * dy;
                if nx < x_min - 0.5 || nx > x_max + 0.5 || ny < y_min - 0.5 || ny > y_max + 0.5 {
                    alpha *= 0.5;
                    continue;
                }
                let nf1 = rhs_p.eval_2d(nx, ny);
                let nf2 = rhs_q.eval_2d(nx, ny);
                if nf1.is_finite() && nf2.is_finite() && nf1.hypot(nf2) < res {
                    x = nx;
                    y = ny;
                    accepted = true;
                    break;
                }
                alpha *= 0.5;
            }
            if !accepted {
                break;
            }
        }

        if !converged {
            let f1 = rhs_p.eval_2d(x, y);
            let f2 = rhs_q.eval_2d(x, y);
            if f1.is_finite() && f2.is_finite() && f1.hypot(f2) < 1e-7 {
                converged = true;
            }
        }

        if converged && x >= x_min - pad && x <= x_max + pad && y >= y_min - pad && y <= y_max + pad
        {
            let x_clean = if x.abs() < 1e-11 { 0.0 } else { x };
            let y_clean = if y.abs() < 1e-11 { 0.0 } else { y };
            let already = found
                .iter()
                .any(|pt| (pt.x - x_clean).hypot(pt.y - y_clean) < 1e-4);
            if !already {
                let j11 = px_expr.eval_2d(x_clean, y_clean);
                let j12 = py_expr.eval_2d(x_clean, y_clean);
                let j21 = qx_expr.eval_2d(x_clean, y_clean);
                let j22 = qy_expr.eval_2d(x_clean, y_clean);
                if j11.is_finite() && j12.is_finite() && j21.is_finite() && j22.is_finite() {
                    let jacobian = Mat2::new(j11, j12, j21, j22);
                    let diagnostics = jacobian.diagnostics();
                    found.push(EquilibriumPoint {
                        x: x_clean,
                        y: y_clean,
                        jacobian,
                        diagnostics,
                    });
                }
            }
        }
    }

    found.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    found
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdeKind {
    HeatDiffusion,
    WaveEquation,
    AdvectionDiffusion,
}

impl PdeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::HeatDiffusion => "1D Heat / Diffusion: u_t = alpha * u_xx (Dirichlet [0, pi])",
            Self::WaveEquation => "1D Wave Equation: u_tt = c^2 * u_xx (Fixed Ends [0, pi])",
            Self::AdvectionDiffusion => {
                "1D Advection-Diffusion: u_t + v * u_x = alpha * u_xx (Periodic [0, 2*pi])"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdeInitialProfile {
    SquarePulse,
    PluckedTriangle,
    TwoModeHarmonic,
    LocalizedGaussian,
}

impl PdeInitialProfile {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SquarePulse => "Step / Square Pulse",
            Self::PluckedTriangle => "Plucked Triangle Hat",
            Self::TwoModeHarmonic => "Two-Mode Superposition (n=1 + n=3)",
            Self::LocalizedGaussian => "Localized Gaussian Packet",
        }
    }

    pub fn eval_on_unit(&self, xi: f64) -> f64 {
        // xi in [0.0, 1.0] normalized across domain length L
        match self {
            Self::SquarePulse => {
                if (0.28..=0.72).contains(&xi) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::PluckedTriangle => {
                if xi <= 0.5 {
                    2.0 * xi
                } else {
                    2.0 * (1.0 - xi)
                }
            }
            Self::TwoModeHarmonic => (PI * xi).sin() + 0.55 * (3.0 * PI * xi).sin(),
            Self::LocalizedGaussian => {
                let d = (xi - 0.38) / 0.10;
                (-0.5 * d * d).exp() * (PI * xi).sin()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PdeSnapshot {
    pub domain_len: f64,
    pub initial_curve: Vec<[f64; 2]>,
    pub reconstructed_t0_curve: Vec<[f64; 2]>,
    pub evolved_curve: Vec<[f64; 2]>,
    pub mode_energies: Vec<(usize, f64)>,
    pub a0: f64,
    pub fourier_coeffs: Vec<(usize, f64, f64)>,
    pub l2_energy_ratio: f64,
    pub l2_truncation_error: f64,
}

fn evaluate_1d_pde_with_fn<F>(
    kind: PdeKind,
    eval_u0: F,
    alpha: f64,
    speed: f64,
    time_t: f64,
    num_modes: usize,
    spatial_samples: usize,
) -> Result<PdeSnapshot, String>
where
    F: Fn(f64) -> f64,
{
    let modes = num_modes.clamp(1, 40);
    let n_pts = spatial_samples.max(64);
    let quad_m = 480usize;

    match kind {
        PdeKind::HeatDiffusion | PdeKind::WaveEquation => {
            let l = PI;
            let dx_q = l / (quad_m as f64);
            let mut quad_samples = Vec::with_capacity(quad_m);
            let mut finite_q = 0usize;
            for j in 0..quad_m {
                let x = (j as f64 + 0.5) * dx_q;
                let u0 = eval_u0(x);
                if u0.is_finite() {
                    finite_q += 1;
                    quad_samples.push((x, u0));
                } else {
                    quad_samples.push((x, 0.0));
                }
            }
            if finite_q * 100 < quad_m * 95 {
                return Err(
                    "u(x, 0) produced non-finite values on [0, pi]; check division by zero or domain restrictions."
                        .to_string(),
                );
            }

            // Fourier sine coefficients b_n = (2/pi) * integral_{0..pi} u0(x) * sin(n*x) dx
            let mut b_coeffs = Vec::with_capacity(modes);
            let mut fourier_coeffs = Vec::with_capacity(modes);
            let mut init_energy = 0.0_f64;

            for n in 1..=modes {
                let mut acc = 0.0;
                for &(x, u0) in &quad_samples {
                    acc += u0 * ((n as f64) * x).sin();
                }
                let bn = (2.0 / l) * acc * dx_q;
                if !bn.is_finite() {
                    return Err("Fourier quadrature produced non-finite mode coefficients.".to_string());
                }
                init_energy += bn * bn;
                b_coeffs.push(bn);
                fourier_coeffs.push((n, 0.0, bn));
            }

            let mut evolved_coeffs = Vec::with_capacity(modes);
            let mut mode_energies = Vec::with_capacity(modes);
            let mut curr_energy = 0.0_f64;

            for (idx, &bn) in b_coeffs.iter().enumerate() {
                let k = (idx + 1) as f64;
                let (bt, kinetic_norm) = match kind {
                    PdeKind::HeatDiffusion => (bn * (-alpha * k * k * time_t).exp(), 0.0),
                    PdeKind::WaveEquation => {
                        let phase = speed * k * time_t;
                        // Total mechanical energy combines potential/displacement (b_n cos(c*k*t))^2
                        // and normalized kinetic velocity (b_n sin(c*k*t))^2 so E(t) = E(0) is conserved.
                        (bn * phase.cos(), bn * phase.sin())
                    }
                    PdeKind::AdvectionDiffusion => (bn, 0.0),
                };
                curr_energy += bt * bt + kinetic_norm * kinetic_norm;
                evolved_coeffs.push(bt);
                mode_energies.push((idx + 1, bt.abs()));
            }

            let mut initial_curve = Vec::with_capacity(n_pts + 1);
            let mut reconstructed_t0_curve = Vec::with_capacity(n_pts + 1);
            let mut evolved_curve = Vec::with_capacity(n_pts + 1);
            for i in 0..=n_pts {
                let x = (i as f64 / n_pts as f64) * l;
                let u0_raw = eval_u0(x);
                let u0 = if u0_raw.is_finite() { u0_raw } else { 0.0 };
                let mut u0_rec = 0.0;
                let mut ut = 0.0;
                for (idx, (&bn, &bt)) in b_coeffs.iter().zip(evolved_coeffs.iter()).enumerate() {
                    let k = (idx + 1) as f64;
                    let s = (k * x).sin();
                    u0_rec += bn * s;
                    ut += bt * s;
                }
                initial_curve.push([x, u0]);
                reconstructed_t0_curve.push([x, u0_rec]);
                evolved_curve.push([x, ut]);
            }

            let mut diff_sq = 0.0_f64;
            let mut norm_sq = 0.0_f64;
            for &(x, u0) in &quad_samples {
                let mut u0_rec = 0.0;
                for (idx, &bn) in b_coeffs.iter().enumerate() {
                    let k = (idx + 1) as f64;
                    u0_rec += bn * (k * x).sin();
                }
                let d = u0 - u0_rec;
                diff_sq += d * d;
                norm_sq += u0 * u0;
            }
            let l2_truncation_error = if norm_sq > 1e-15 {
                (diff_sq / norm_sq).sqrt()
            } else {
                0.0
            };

            let ratio = if init_energy > 1e-12 {
                (curr_energy / init_energy).clamp(0.0, 2.0)
            } else {
                1.0
            };

            Ok(PdeSnapshot {
                domain_len: l,
                initial_curve,
                reconstructed_t0_curve,
                evolved_curve,
                mode_energies,
                a0: 0.0,
                fourier_coeffs,
                l2_energy_ratio: ratio,
                l2_truncation_error,
            })
        }

        PdeKind::AdvectionDiffusion => {
            let l = 2.0 * PI;
            let dx_q = l / (quad_m as f64);
            let mut quad_samples = Vec::with_capacity(quad_m);
            let mut finite_q = 0usize;
            for j in 0..quad_m {
                let x = (j as f64 + 0.5) * dx_q;
                let u0 = eval_u0(x);
                if u0.is_finite() {
                    finite_q += 1;
                    quad_samples.push((x, u0));
                } else {
                    quad_samples.push((x, 0.0));
                }
            }
            if finite_q * 100 < quad_m * 95 {
                return Err(
                    "u(x, 0) produced non-finite values on [0, 2*pi]; check division by zero or domain restrictions."
                        .to_string(),
                );
            }

            // Mean mode a0
            let mut a0_acc = 0.0;
            for &(_, u0) in &quad_samples {
                a0_acc += u0;
            }
            let a0 = (a0_acc * dx_q) / l;
            if !a0.is_finite() {
                return Err("Fourier mean mode a0 is non-finite.".to_string());
            }
            let mut init_energy = 2.0 * a0 * a0;
            let mut curr_energy = 2.0 * a0 * a0;

            // Harmonic pairs (a_n cos(n*x), b_n sin(n*x))
            let mut ab = Vec::with_capacity(modes);
            let mut fourier_coeffs = Vec::with_capacity(modes);
            let mut mode_energies = Vec::with_capacity(modes);

            for n in 1..=modes {
                let kf = n as f64;
                let mut ac = 0.0;
                let mut bs = 0.0;
                for &(x, u0) in &quad_samples {
                    ac += u0 * (kf * x).cos();
                    bs += u0 * (kf * x).sin();
                }
                let an = (ac * dx_q) / PI;
                let bn = (bs * dx_q) / PI;
                if !an.is_finite() || !bn.is_finite() {
                    return Err("Fourier quadrature produced non-finite mode coefficients.".to_string());
                }
                init_energy += an * an + bn * bn;

                let damp = (-alpha * kf * kf * time_t).exp();
                let amp = damp * (an * an + bn * bn).sqrt();
                curr_energy += amp * amp;
                ab.push((kf, an, bn, an * damp, bn * damp));
                fourier_coeffs.push((n, an, bn));
                mode_energies.push((n, amp));
            }

            let mut initial_curve = Vec::with_capacity(n_pts + 1);
            let mut reconstructed_t0_curve = Vec::with_capacity(n_pts + 1);
            let mut evolved_curve = Vec::with_capacity(n_pts + 1);
            for i in 0..=n_pts {
                let x = (i as f64 / n_pts as f64) * l;
                let u0_raw = eval_u0(x);
                let u0 = if u0_raw.is_finite() { u0_raw } else { 0.0 };
                let shifted = x - speed * time_t;
                let mut u0_rec = a0;
                let mut ut = a0;
                for &(kf, an, bn, an_d, bn_d) in &ab {
                    u0_rec += an * (kf * x).cos() + bn * (kf * x).sin();
                    ut += an_d * (kf * shifted).cos() + bn_d * (kf * shifted).sin();
                }
                initial_curve.push([x, u0]);
                reconstructed_t0_curve.push([x, u0_rec]);
                evolved_curve.push([x, ut]);
            }

            let mut diff_sq = 0.0_f64;
            let mut norm_sq = 0.0_f64;
            for &(x, u0) in &quad_samples {
                let mut u0_rec = a0;
                for &(kf, an, bn, _, _) in &ab {
                    u0_rec += an * (kf * x).cos() + bn * (kf * x).sin();
                }
                let d = u0 - u0_rec;
                diff_sq += d * d;
                norm_sq += u0 * u0;
            }
            let l2_truncation_error = if norm_sq > 1e-15 {
                (diff_sq / norm_sq).sqrt()
            } else {
                0.0
            };

            let ratio = if init_energy > 1e-12 {
                (curr_energy / init_energy).clamp(0.0, 2.0)
            } else {
                1.0
            };

            Ok(PdeSnapshot {
                domain_len: l,
                initial_curve,
                reconstructed_t0_curve,
                evolved_curve,
                mode_energies,
                a0,
                fourier_coeffs,
                l2_energy_ratio: ratio,
                l2_truncation_error,
            })
        }
    }
}

/// Evaluates spectral Fourier-Galerkin solution u(x, t) for Heat, Wave, or Periodic Advection-Diffusion PDEs.
pub fn evaluate_1d_pde(
    kind: PdeKind,
    profile: PdeInitialProfile,
    alpha: f64,
    speed: f64,
    time_t: f64,
    num_modes: usize,
    spatial_samples: usize,
) -> PdeSnapshot {
    let l = match kind {
        PdeKind::HeatDiffusion | PdeKind::WaveEquation => PI,
        PdeKind::AdvectionDiffusion => 2.0 * PI,
    };
    evaluate_1d_pde_with_fn(
        kind,
        |x| profile.eval_on_unit(x / l),
        alpha,
        speed,
        time_t,
        num_modes,
        spatial_samples,
    )
    .expect("built-in PDE initial profile should always evaluate finitely")
}

/// Evaluates spectral Fourier-Galerkin solution u(x, t) from a custom user-supplied `u(x, 0)` expression.
pub fn evaluate_1d_pde_custom(
    kind: PdeKind,
    expr_str: &str,
    alpha: f64,
    speed: f64,
    time_t: f64,
    num_modes: usize,
    spatial_samples: usize,
) -> Result<PdeSnapshot, String> {
    let expr = Expr::parse(expr_str)?;
    let unknown = expr.unknown_vars(&["x"]);
    if !unknown.is_empty() {
        return Err(format!(
            "Unknown variable(s) {}; express u(x, 0) in terms of x.",
            unknown.join(", ")
        ));
    }
    evaluate_1d_pde_with_fn(
        kind,
        |x| expr.eval_1d("x", x),
        alpha,
        speed,
        time_t,
        num_modes,
        spatial_samples,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_equilibria_and_nullclines_match_analytic_presets() {
        // 1. Lotka-Volterra: dx/dt = x*(1.2 - 0.8*y), dy/dt = y*(-0.9 + 0.6*x)
        let lv_p = Expr::parse("x * (1.2 - 0.8*y)").unwrap();
        let lv_q = Expr::parse("y * (-0.9 + 0.6*x)").unwrap();
        let eqs = find_planar_equilibria(&lv_p, &lv_q, -3.5, 3.5, -3.5, 3.5);
        assert!(
            eqs.iter()
                .any(|e| e.x.abs() < 1e-4 && e.y.abs() < 1e-4 && e.diagnostics.phase_portrait_class.contains("Saddle")),
            "Expected (0,0) Saddle in Lotka-Volterra, got: {:?}",
            eqs.iter().map(|e| (e.x, e.y, e.diagnostics.phase_portrait_class)).collect::<Vec<_>>()
        );
        assert!(
            eqs.iter()
                .any(|e| (e.x - 1.5).abs() < 1e-4 && (e.y - 1.5).abs() < 1e-4 && e.diagnostics.phase_portrait_class.contains("Center")),
            "Expected (1.5,1.5) Center in Lotka-Volterra, got: {:?}",
            eqs.iter().map(|e| (e.x, e.y, e.diagnostics.phase_portrait_class)).collect::<Vec<_>>()
        );

        // 2. Nonlinear Pendulum: dx/dt = y, dy/dt = -2*sin(x) - 0.15*y
        let np_p = Expr::parse("y").unwrap();
        let np_q = Expr::parse("-2.0*sin(x) - 0.15*y").unwrap();
        let np_eqs = find_planar_equilibria(&np_p, &np_q, -3.5, 3.5, -3.5, 3.5);
        assert!(np_eqs.iter().any(|e| e.x.abs() < 1e-4 && e.y.abs() < 1e-4 && e.diagnostics.phase_portrait_class.contains("Sink")));
        assert!(np_eqs.iter().any(|e| (e.x - PI).abs() < 1e-3 && e.y.abs() < 1e-4 && e.diagnostics.phase_portrait_class.contains("Saddle")));
        assert!(np_eqs.iter().any(|e| (e.x + PI).abs() < 1e-3 && e.y.abs() < 1e-4 && e.diagnostics.phase_portrait_class.contains("Saddle")));

        // 3. DEQ-501 autonomous time-like P=1 has 0 equilibria
        let p501 = Expr::parse("1.0").unwrap();
        let q501 = Expr::parse("y * (1.5 - y)").unwrap();
        assert!(find_planar_equilibria(&p501, &q501, -3.5, 3.5, -3.5, 3.5).is_empty());

        // 4. Nullcline extraction for x - 2 = 0
        let nc = Expr::parse("x - 2.0").unwrap();
        let segs = extract_zero_level_segments(&nc, -3.5, 3.5, -3.5, 3.5, 56);
        assert!(!segs.is_empty());
        for (a, b) in segs {
            assert!((a[0] - 2.0).abs() < 1e-6 && (b[0] - 2.0).abs() < 1e-6);
        }
    }

    #[test]
    fn planar_rk4_beats_euler_on_harmonic_oscillator() {
        let p = Expr::parse("y").unwrap();
        let q = Expr::parse("-x").unwrap();
        let dt = 0.05;
        let steps = ((2.0 * PI) / dt).round() as usize;
        let rk4 = solve_planar_system_rk4_timed(&p, &q, 1.0, 0.0, dt, steps, 10.0);
        let euler = solve_planar_system_euler_timed(&p, &q, 1.0, 0.0, dt, steps, 10.0);

        let rk4_last = *rk4.last().unwrap();
        let euler_last = *euler.last().unwrap();
        let rk4_r_err = (rk4_last[1].hypot(rk4_last[2]) - 1.0).abs();
        let euler_r_err = (euler_last[1].hypot(euler_last[2]) - 1.0).abs();
        assert!(rk4_r_err < 1e-4, "RK4 radius drift too large: {}", rk4_r_err);
        assert!(euler_r_err > 0.15, "Euler should noticeably drift outward: {}", euler_r_err);
    }

    #[test]
    fn custom_fourier_quadrature_recovers_exact_sine_and_surfaces_errors() {
        let heat = evaluate_1d_pde_custom(
            PdeKind::HeatDiffusion,
            "2*sin(x) - 0.5*sin(3*x)",
            0.2,
            1.0,
            0.0,
            6,
            128,
        )
        .expect("custom heat PDE");
        assert!((heat.fourier_coeffs[0].2 - 2.0).abs() < 1e-3);
        assert!(heat.fourier_coeffs[1].2.abs() < 1e-3);
        assert!((heat.fourier_coeffs[2].2 - (-0.5)).abs() < 1e-3);
        assert!(heat.l2_truncation_error < 1e-3);

        let adv = evaluate_1d_pde_custom(
            PdeKind::AdvectionDiffusion,
            "1.0 + sin(x) + 0.5*cos(2*x)",
            0.1,
            1.0,
            0.0,
            6,
            128,
        )
        .expect("custom advection PDE");
        assert!((adv.a0 - 1.0).abs() < 1e-3);
        assert!((adv.fourier_coeffs[0].2 - 1.0).abs() < 1e-3);
        assert!((adv.fourier_coeffs[1].1 - 0.5).abs() < 1e-3);

        let err_var = evaluate_1d_pde_custom(PdeKind::HeatDiffusion, "sin(t)", 0.1, 1.0, 0.0, 6, 64)
            .expect_err("unbound variable t must error");
        assert!(err_var.contains('t'));

        assert!(evaluate_1d_pde_custom(PdeKind::HeatDiffusion, "", 0.1, 1.0, 0.0, 6, 64).is_err());
        assert!(
            evaluate_1d_pde_custom(PdeKind::HeatDiffusion, "sqrt(-1 - x^2)", 0.1, 1.0, 0.0, 6, 64)
                .is_err()
        );
    }

    #[test]
    fn planar_solvers_reject_out_of_bound_or_non_finite_initial_seed() {
        let p = Expr::parse("y").unwrap();
        let q = Expr::parse("-x").unwrap();
        assert!(solve_planar_system_rk4(&p, &q, 10.0, 0.0, 0.05, 10, 4.8).is_empty());
        assert!(solve_planar_system_rk4_timed(&p, &q, 10.0, 0.0, 0.05, 10, 4.8).is_empty());
        assert!(solve_planar_system_euler_timed(&p, &q, 10.0, 0.0, 0.05, 10, 4.8).is_empty());

        assert!(solve_planar_system_rk4(&p, &q, f64::NAN, 0.0, 0.05, 10, 4.8).is_empty());
        assert!(solve_planar_system_rk4_timed(&p, &q, 0.0, f64::INFINITY, 0.05, 10, 4.8).is_empty());
        assert!(solve_planar_system_euler_timed(&p, &q, 0.0, f64::NEG_INFINITY, 0.05, 10, 4.8).is_empty());

        // In-bound finite seed still records initial state
        assert!(!solve_planar_system_rk4(&p, &q, 1.0, 0.0, 0.05, 10, 4.8).is_empty());
    }

    #[test]
    fn wave_equation_conserves_total_mechanical_energy_at_quarter_period() {
        for &speed in &[1.0_f64, 1.75_f64] {
            let t_quarter = PI / (2.0 * speed);
            for profile in [
                PdeInitialProfile::TwoModeHarmonic,
                PdeInitialProfile::PluckedTriangle,
                PdeInitialProfile::SquarePulse,
                PdeInitialProfile::LocalizedGaussian,
            ] {
                for &t in &[0.0, t_quarter, 0.73, 2.1] {
                    let snap = evaluate_1d_pde(
                        PdeKind::WaveEquation,
                        profile,
                        0.0,
                        speed,
                        t,
                        18,
                        128,
                    );
                    assert!(
                        (snap.l2_energy_ratio - 1.0).abs() < 1e-6,
                        "Wave energy conservation violated for {:?} at c={speed}, t={t}: ratio={}",
                        profile,
                        snap.l2_energy_ratio
                    );
                }
            }

            let custom = evaluate_1d_pde_custom(
                PdeKind::WaveEquation,
                "2*sin(x) - 0.5*sin(3*x)",
                0.0,
                speed,
                t_quarter,
                12,
                128,
            )
            .expect("custom wave PDE succeeds");
            assert!(
                (custom.l2_energy_ratio - 1.0).abs() < 1e-6,
                "Custom wave energy conservation failed at t=pi/(2c): {}",
                custom.l2_energy_ratio
            );
        }
    }
}

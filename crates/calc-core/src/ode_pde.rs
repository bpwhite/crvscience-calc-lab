use crate::expr::{EvalEnv, Expr};
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
    pub evolved_curve: Vec<[f64; 2]>,
    pub mode_energies: Vec<(usize, f64)>,
    pub l2_energy_ratio: f64,
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
    let modes = num_modes.clamp(1, 40);
    let n_pts = spatial_samples.max(64);

    match kind {
        PdeKind::HeatDiffusion | PdeKind::WaveEquation => {
            let l = PI;
            // Compute Fourier sine coefficients b_n = (2/pi) * integral_{0..pi} u0(x) * sin(n*x) dx via composite midpoint rule
            let quad_m = 240usize;
            let dx_q = l / (quad_m as f64);
            let mut b_coeffs = Vec::with_capacity(modes);
            let mut init_energy = 0.0_f64;

            for n in 1..=modes {
                let mut acc = 0.0;
                for j in 0..quad_m {
                    let x = (j as f64 + 0.5) * dx_q;
                    let u0 = profile.eval_on_unit(x / l);
                    acc += u0 * ((n as f64) * x).sin();
                }
                let bn = (2.0 / l) * acc * dx_q;
                init_energy += bn * bn;
                b_coeffs.push(bn);
            }

            let mut evolved_coeffs = Vec::with_capacity(modes);
            let mut mode_energies = Vec::with_capacity(modes);
            let mut curr_energy = 0.0_f64;

            for (idx, &bn) in b_coeffs.iter().enumerate() {
                let k = (idx + 1) as f64;
                let bt = match kind {
                    PdeKind::HeatDiffusion => bn * (-alpha * k * k * time_t).exp(),
                    PdeKind::WaveEquation => bn * (speed * k * time_t).cos(),
                    PdeKind::AdvectionDiffusion => bn,
                };
                curr_energy += bt * bt;
                evolved_coeffs.push(bt);
                mode_energies.push((idx + 1, bt.abs()));
            }

            let mut initial_curve = Vec::with_capacity(n_pts + 1);
            let mut evolved_curve = Vec::with_capacity(n_pts + 1);
            for i in 0..=n_pts {
                let x = (i as f64 / n_pts as f64) * l;
                let u0 = profile.eval_on_unit(x / l);
                let mut ut = 0.0;
                for (idx, &bt) in evolved_coeffs.iter().enumerate() {
                    let k = (idx + 1) as f64;
                    ut += bt * (k * x).sin();
                }
                initial_curve.push([x, u0]);
                evolved_curve.push([x, ut]);
            }

            let ratio = if init_energy > 1e-12 {
                (curr_energy / init_energy).clamp(0.0, 2.0)
            } else {
                1.0
            };

            PdeSnapshot {
                domain_len: l,
                initial_curve,
                evolved_curve,
                mode_energies,
                l2_energy_ratio: ratio,
            }
        }

        PdeKind::AdvectionDiffusion => {
            let l = 2.0 * PI;
            let quad_m = 280usize;
            let dx_q = l / (quad_m as f64);

            // Mean mode a0
            let mut a0_acc = 0.0;
            for j in 0..quad_m {
                let x = (j as f64 + 0.5) * dx_q;
                a0_acc += profile.eval_on_unit(x / l);
            }
            let a0 = (a0_acc * dx_q) / l;
            let mut init_energy = 2.0 * a0 * a0;
            let mut curr_energy = 2.0 * a0 * a0;

            // Harmonic pairs (a_n cos(n*x), b_n sin(n*x)) translated by x -> x - speed*t and damped by exp(-alpha*n^2*t)
            let mut ab = Vec::with_capacity(modes);
            let mut mode_energies = Vec::with_capacity(modes);

            for n in 1..=modes {
                let kf = n as f64;
                let mut ac = 0.0;
                let mut bs = 0.0;
                for j in 0..quad_m {
                    let x = (j as f64 + 0.5) * dx_q;
                    let u0 = profile.eval_on_unit(x / l);
                    ac += u0 * (kf * x).cos();
                    bs += u0 * (kf * x).sin();
                }
                let an = (ac * dx_q) / PI;
                let bn = (bs * dx_q) / PI;
                init_energy += an * an + bn * bn;

                let damp = (-alpha * kf * kf * time_t).exp();
                let amp = damp * (an * an + bn * bn).sqrt();
                curr_energy += amp * amp;
                ab.push((kf, an * damp, bn * damp));
                mode_energies.push((n, amp));
            }

            let mut initial_curve = Vec::with_capacity(n_pts + 1);
            let mut evolved_curve = Vec::with_capacity(n_pts + 1);
            for i in 0..=n_pts {
                let x = (i as f64 / n_pts as f64) * l;
                let u0 = profile.eval_on_unit(x / l);
                let shifted = x - speed * time_t;
                let mut ut = a0;
                for &(kf, an_d, bn_d) in &ab {
                    ut += an_d * (kf * shifted).cos() + bn_d * (kf * shifted).sin();
                }
                initial_curve.push([x, u0]);
                evolved_curve.push([x, ut]);
            }

            let ratio = if init_energy > 1e-12 {
                (curr_energy / init_energy).clamp(0.0, 2.0)
            } else {
                1.0
            };

            PdeSnapshot {
                domain_len: l,
                initial_curve,
                evolved_curve,
                mode_energies,
                l2_energy_ratio: ratio,
            }
        }
    }
}

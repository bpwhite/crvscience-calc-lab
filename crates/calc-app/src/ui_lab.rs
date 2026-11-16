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

use crate::theme::AcademicTheme;
use calc_core::{
    binomial_pmf, compute_bayes_binary, evaluate_1d_pde, evaluate_field_2d, exponential_cdf,
    exponential_pdf, fit_ols_linear, normal_cdf, normal_pdf, poisson_pmf, riemann_sum,
    solve_planar_system_rk4, taylor_polynomial_at, DistributionKind, EigenSpectrum2D, Expr,
    LabPreset, Mat2, PdeInitialProfile, PdeKind, RiemannMethod,
};
use egui::{
    Color32, ComboBox, Context, Frame, Margin, RichText, Rounding, Slider, TextEdit, TextStyle, Ui,
};
use egui_plot::{Bar, BarChart, HLine, Legend, Line, Plot, Points, Polygon, VLine};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabMode {
    AlgebraExplorer,
    TrigUnitWave,
    SingleVarCalculus,
    LinearAlgebraMap,
    EpsilonDelta,
    MultivariableField,
    OdePdeLab,
    ProbabilityStatsLab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsRightSubmode {
    BayesianDiagnostic,
    OlsRegression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdePdeSubmode {
    OdePhasePortrait,
    PdeEvolution1D,
}

pub struct LabState {
    pub mode: LabMode,
    // Mode 1: Algebra & Difference Quotients
    pub alg_primary: String,
    pub alg_secondary: String,
    pub alg_probe_x: f64,
    pub alg_step_h: f64,
    // Mode 2: Trigonometry Unit Circle <-> Wave
    pub trig_amp: f64,
    pub trig_omega: f64,
    pub trig_phase: f64,
    pub trig_angle: f64,
    pub trig_animate: bool,
    // Mode 3: Single-Variable Calculus
    pub calc_expr: String,
    pub calc_x0: f64,
    pub calc_h: f64,
    pub calc_int_a: f64,
    pub calc_int_b: f64,
    pub calc_partitions: usize,
    pub calc_riemann_method: RiemannMethod,
    pub calc_taylor_deg: usize,
    pub calc_show_secant_tangent: bool,
    pub calc_show_riemann: bool,
    pub calc_show_taylor: bool,
    // Mode 4: Linear Algebra & Eigen-Systems
    pub la_a11: f64,
    pub la_a12: f64,
    pub la_a21: f64,
    pub la_a22: f64,
    pub la_vx: f64,
    pub la_vy: f64,
    // Mode 5: Real Analysis Epsilon-Delta
    pub eps_expr: String,
    pub eps_x0: f64,
    pub eps_target: f64,
    pub eps_delta: f64,
    // Mode 6: Multivariable Scalar Contours & 2D Vector Field
    pub mv_scalar: String,
    pub mv_vec_p: String,
    pub mv_vec_q: String,
    pub mv_probe_x: f64,
    pub mv_probe_y: f64,
    pub mv_show_contours: bool,
    pub mv_show_quiver: bool,
    pub mv_show_grad_path: bool,
    // Mode 7: ODE Phase Portraits & 1D Spectral PDEs
    pub ode_submode: OdePdeSubmode,
    pub ode_rhs_p: String,
    pub ode_rhs_q: String,
    pub ode_seed_x: f64,
    pub ode_seed_y: f64,
    pub ode_show_ensemble: bool,
    pub pde_kind: PdeKind,
    pub pde_profile: PdeInitialProfile,
    pub pde_alpha: f64,
    pub pde_speed: f64,
    pub pde_time: f64,
    pub pde_modes: usize,
    pub pde_animate: bool,
    // Mode 8: Probability Distributions, Bayesian Updating & OLS Regression
    pub stats_dist: DistributionKind,
    pub stats_param_a: f64,
    pub stats_param_b: f64,
    pub stats_interval_a: f64,
    pub stats_interval_b: f64,
    pub stats_right_mode: StatsRightSubmode,
    pub stats_prior: f64,
    pub stats_sens: f64,
    pub stats_spec: f64,
    pub stats_points: Vec<[f64; 2]>,
}

impl Default for LabState {
    fn default() -> Self {
        Self {
            mode: LabMode::SingleVarCalculus,
            alg_primary: "x^2 - 3*x + 1".to_string(),
            alg_secondary: "x - 2".to_string(),
            alg_probe_x: 2.0,
            alg_step_h: 0.8,
            trig_amp: 1.0,
            trig_omega: 1.0,
            trig_phase: 0.0,
            trig_angle: PI / 3.0,
            trig_animate: false,
            calc_expr: "0.25*x^3 - x + 1".to_string(),
            calc_x0: 1.2,
            calc_h: 0.7,
            calc_int_a: -2.0,
            calc_int_b: 2.5,
            calc_partitions: 18,
            calc_riemann_method: RiemannMethod::Midpoint,
            calc_taylor_deg: 3,
            calc_show_secant_tangent: true,
            calc_show_riemann: true,
            calc_show_taylor: false,
            la_a11: 1.6,
            la_a12: 0.6,
            la_a21: 0.6,
            la_a22: 0.9,
            la_vx: 1.0,
            la_vy: 0.5,
            eps_expr: "0.5*x^2 + x".to_string(),
            eps_x0: 1.0,
            eps_target: 0.35,
            eps_delta: 0.15,
            mv_scalar: "x^2 - y^2".to_string(),
            mv_vec_p: "-y".to_string(),
            mv_vec_q: "x".to_string(),
            mv_probe_x: 0.9,
            mv_probe_y: 0.5,
            mv_show_contours: true,
            mv_show_quiver: true,
            mv_show_grad_path: true,
            ode_submode: OdePdeSubmode::OdePhasePortrait,
            ode_rhs_p: "y".to_string(),
            ode_rhs_q: "-2.0*x - 0.35*y".to_string(),
            ode_seed_x: 2.0,
            ode_seed_y: 0.0,
            ode_show_ensemble: true,
            pde_kind: PdeKind::HeatDiffusion,
            pde_profile: PdeInitialProfile::SquarePulse,
            pde_alpha: 0.16,
            pde_speed: 1.2,
            pde_time: 0.20,
            pde_modes: 18,
            pde_animate: false,
            stats_dist: DistributionKind::Normal,
            stats_param_a: 0.0,
            stats_param_b: 1.0,
            stats_interval_a: -1.0,
            stats_interval_b: 1.5,
            stats_right_mode: StatsRightSubmode::BayesianDiagnostic,
            stats_prior: 0.08,
            stats_sens: 0.95,
            stats_spec: 0.94,
            stats_points: vec![
                [0.5, 1.4],
                [1.2, 2.5],
                [2.0, 3.1],
                [2.8, 4.7],
                [3.6, 5.2],
                [4.4, 6.8],
            ],
        }
    }
}

impl LabState {
    pub fn apply_preset(&mut self, preset: &LabPreset) {
        match preset {
            LabPreset::AlgebraGraph {
                primary_expr,
                secondary_expr,
                probe_x,
            } => {
                self.mode = LabMode::AlgebraExplorer;
                self.alg_primary = (*primary_expr).to_string();
                self.alg_secondary = (*secondary_expr).to_string();
                self.alg_probe_x = *probe_x;
            }
            LabPreset::TrigUnitWave {
                amplitude,
                omega,
                phase,
                angle_rad,
            } => {
                self.mode = LabMode::TrigUnitWave;
                self.trig_amp = *amplitude;
                self.trig_omega = *omega;
                self.trig_phase = *phase;
                self.trig_angle = *angle_rad;
            }
            LabPreset::SingleVarCalculus {
                expr,
                x0,
                step_h,
                int_a,
                int_b,
                partitions,
                taylor_deg,
            } => {
                self.mode = LabMode::SingleVarCalculus;
                self.calc_expr = (*expr).to_string();
                self.calc_x0 = *x0;
                self.calc_h = *step_h;
                self.calc_int_a = *int_a;
                self.calc_int_b = *int_b;
                self.calc_partitions = *partitions;
                self.calc_taylor_deg = *taylor_deg;
                self.calc_show_secant_tangent = true;
                self.calc_show_riemann = true;
            }
            LabPreset::LinearAlgebraMap {
                a11,
                a12,
                a21,
                a22,
                vec_x,
                vec_y,
            } => {
                self.mode = LabMode::LinearAlgebraMap;
                self.la_a11 = *a11;
                self.la_a12 = *a12;
                self.la_a21 = *a21;
                self.la_a22 = *a22;
                self.la_vx = *vec_x;
                self.la_vy = *vec_y;
            }
            LabPreset::EpsilonDelta { expr, x0, epsilon } => {
                self.mode = LabMode::EpsilonDelta;
                self.eps_expr = (*expr).to_string();
                self.eps_x0 = *x0;
                self.eps_target = *epsilon;
                self.eps_delta = 0.20;
            }
            LabPreset::MultivariableField {
                scalar_expr,
                vec_p,
                vec_q,
                probe_x,
                probe_y,
            } => {
                self.mode = LabMode::MultivariableField;
                self.mv_scalar = (*scalar_expr).to_string();
                self.mv_vec_p = (*vec_p).to_string();
                self.mv_vec_q = (*vec_q).to_string();
                self.mv_probe_x = *probe_x;
                self.mv_probe_y = *probe_y;
            }
            LabPreset::OdePhaseSystem {
                rhs_p,
                rhs_q,
                seed_x,
                seed_y,
            } => {
                self.mode = LabMode::OdePdeLab;
                self.ode_submode = OdePdeSubmode::OdePhasePortrait;
                self.ode_rhs_p = (*rhs_p).to_string();
                self.ode_rhs_q = (*rhs_q).to_string();
                self.ode_seed_x = *seed_x;
                self.ode_seed_y = *seed_y;
            }
            LabPreset::PdeEvolution {
                kind,
                profile,
                alpha,
                speed,
                time_t,
            } => {
                self.mode = LabMode::OdePdeLab;
                self.ode_submode = OdePdeSubmode::PdeEvolution1D;
                self.pde_kind = *kind;
                self.pde_profile = *profile;
                self.pde_alpha = *alpha;
                self.pde_speed = *speed;
                self.pde_time = *time_t;
                self.pde_animate = false;
            }
            LabPreset::ProbabilityStats {
                dist,
                param_a,
                param_b,
                prior,
                sensitivity,
                specificity,
            } => {
                self.mode = LabMode::ProbabilityStatsLab;
                self.stats_dist = *dist;
                self.stats_param_a = *param_a;
                self.stats_param_b = *param_b;
                self.stats_prior = *prior;
                self.stats_sens = *sensitivity;
                self.stats_spec = *specificity;
            }
        }
    }
}

fn sample_curve_segments(expr: &Expr, x_min: f64, x_max: f64, samples: usize) -> Vec<Vec<[f64; 2]>> {
    let mut segments: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut current: Vec<[f64; 2]> = Vec::new();
    let dx = (x_max - x_min) / (samples.max(2) as f64);

    for i in 0..=samples {
        let x = x_min + (i as f64) * dx;
        let y = expr.eval_1d("x", x);
        if y.is_finite() && y.abs() <= 60.0 {
            if let Some(&prev) = current.last() {
                if (y - prev[1]).abs() > 14.0 {
                    if current.len() >= 2 {
                        segments.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                }
            }
            current.push([x, y]);
        } else if current.len() >= 2 {
            segments.push(std::mem::take(&mut current));
        } else {
            current.clear();
        }
    }
    if current.len() >= 2 {
        segments.push(current);
    }
    segments
}

pub fn render_lab(ctx: &Context, ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    ui.heading("Interactive Mathematical Laboratory (8 Modes)");
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(
            &mut state.mode,
            LabMode::AlgebraExplorer,
            "1. Algebra & DQ",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::TrigUnitWave,
            "2. Trig Unit-Wave",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::SingleVarCalculus,
            "3. Single-Var Calc",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::LinearAlgebraMap,
            "4. Linear Algebra & Eigen-Maps",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::EpsilonDelta,
            "5. Epsilon-Delta",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::MultivariableField,
            "6. Multivariable Fields",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::OdePdeLab,
            "7. ODE Phase & 1D PDEs",
        );
        ui.selectable_value(
            &mut state.mode,
            LabMode::ProbabilityStatsLab,
            "8. Prob, Bayes & Regression",
        );
    });
    ui.separator();

    match state.mode {
        LabMode::AlgebraExplorer => render_algebra_lab(ui, state, theme),
        LabMode::TrigUnitWave => render_trig_lab(ctx, ui, state, theme),
        LabMode::SingleVarCalculus => render_single_var_calc_lab(ui, state, theme),
        LabMode::LinearAlgebraMap => render_linear_algebra_lab(ui, state, theme),
        LabMode::EpsilonDelta => render_epsilon_delta_lab(ui, state, theme),
        LabMode::MultivariableField => render_multivariable_lab(ui, state, theme),
        LabMode::OdePdeLab => render_ode_pde_lab(ctx, ui, state, theme),
        LabMode::ProbabilityStatsLab => render_prob_stats_lab(ui, state, theme),
    }
}

fn render_algebra_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Primary f(x) =");
                ui.add(
                    TextEdit::singleline(&mut state.alg_primary)
                        .desired_width(220.0)
                        .font(TextStyle::Monospace),
                );
                ui.label("Secondary g(x) =");
                ui.add(
                    TextEdit::singleline(&mut state.alg_secondary)
                        .desired_width(220.0)
                        .font(TextStyle::Monospace),
                );
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.alg_probe_x, -6.0..=6.0).text("Anchor x0"));
                ui.add(Slider::new(&mut state.alg_step_h, -2.5..=2.5).text("Secant step h"));
            });
        });

    let f_parsed = Expr::parse(&state.alg_primary);
    let g_parsed = Expr::parse(&state.alg_secondary);

    if let Err(e) = &f_parsed {
        ui.label(RichText::new(format!("f(x) syntax error: {}", e)).color(theme.err_color()));
        return;
    }
    let f_expr = f_parsed.unwrap();

    Plot::new("algebra_explorer_plot")
        .height(380.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            for (idx, seg) in sample_curve_segments(&f_expr, -8.0, 8.0, 500)
                .into_iter()
                .enumerate()
            {
                let mut line = Line::new(seg)
                    .color(Color32::from_rgb(88, 166, 236))
                    .width(2.2);
                if idx == 0 {
                    line = line.name("f(x)");
                }
                plot_ui.line(line);
            }

            if let Ok(g_expr) = &g_parsed {
                for (idx, seg) in sample_curve_segments(g_expr, -8.0, 8.0, 500)
                    .into_iter()
                    .enumerate()
                {
                    let mut line = Line::new(seg)
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(1.8);
                    if idx == 0 {
                        line = line.name("g(x)");
                    }
                    plot_ui.line(line);
                }
            }

            let x0 = state.alg_probe_x;
            let h = if state.alg_step_h.abs() < 1e-4 {
                0.001
            } else {
                state.alg_step_h
            };
            let y0 = f_expr.eval_1d("x", x0);
            let y1 = f_expr.eval_1d("x", x0 + h);
            if y0.is_finite() && y1.is_finite() {
                let m_sec = (y1 - y0) / h;
                let sec_pts = vec![
                    [-8.0, y0 + m_sec * (-8.0 - x0)],
                    [8.0, y0 + m_sec * (8.0 - x0)],
                ];
                plot_ui.line(
                    Line::new(sec_pts)
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(1.5)
                        .name("Secant Line"),
                );
                plot_ui.points(
                    Points::new(vec![[x0, y0], [x0 + h, y1]])
                        .radius(5.0)
                        .color(Color32::from_rgb(232, 168, 72))
                        .name("Secant Anchor Points"),
                );
            }
        });

    let df_sym = f_expr.differentiate("x");
    let exact_slope = df_sym.eval_1d("x", state.alg_probe_x);
    let y0 = f_expr.eval_1d("x", state.alg_probe_x);
    let h_eff = if state.alg_step_h.abs() < 1e-6 {
        1e-6
    } else {
        state.alg_step_h
    };
    let dq_curr = (f_expr.eval_1d("x", state.alg_probe_x + h_eff) - y0) / h_eff;

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "At x0 = {:.3}: f(x0) = {:.4} | Difference Quotient (h = {:.3}) = {:.5} | Limit Slope f'(x0) = {:.5}  [symbolic f'(x) = {}]",
                    state.alg_probe_x,
                    y0,
                    h_eff,
                    dq_curr,
                    exact_slope,
                    df_sym.to_plain_string()
                ))
                .monospace(),
            );
        });
}

fn render_trig_lab(ctx: &Context, ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    if state.trig_animate {
        let dt = ctx.input(|i| i.stable_dt as f64).clamp(0.001, 0.05);
        state.trig_angle = (state.trig_angle + 0.9 * dt).rem_euclid(2.0 * PI);
        ctx.request_repaint();
    }

    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    Slider::new(&mut state.trig_angle, 0.0..=(2.0 * PI)).text("Angle theta (rad)"),
                );
                ui.checkbox(&mut state.trig_animate, "Animate Rotation");
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.trig_amp, 0.2..=3.0).text("Amplitude A"));
                ui.add(Slider::new(&mut state.trig_omega, 0.5..=5.0).text("Frequency omega"));
                ui.add(Slider::new(&mut state.trig_phase, -PI..=PI).text("Phase phi"));
            });
        });

    let theta = state.trig_angle;
    let cx = theta.cos();
    let sy = theta.sin();

    ui.columns(2, |cols| {
        Plot::new("unit_circle_plot")
            .height(360.0)
            .data_aspect(1.0)
            .legend(Legend::default())
            .show(&mut cols[0], |plot_ui| {
                let circle_pts: Vec<[f64; 2]> = (0..=180)
                    .map(|k| {
                        let a = (k as f64 / 180.0) * 2.0 * PI;
                        [a.cos(), a.sin()]
                    })
                    .collect();
                plot_ui.line(
                    Line::new(circle_pts)
                        .color(Color32::from_rgb(130, 145, 168))
                        .width(1.6)
                        .name("Unit Circle x^2 + y^2 = 1"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], [cx, 0.0]])
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(2.8)
                        .name("cos(theta)"),
                );
                plot_ui.line(
                    Line::new(vec![[cx, 0.0], [cx, sy]])
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.8)
                        .name("sin(theta)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], [cx, sy]])
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(2.2)
                        .name("Radius r = 1"),
                );
                if cx.abs() > 0.18 {
                    let tan_v = sy / cx;
                    if tan_v.abs() <= 3.0 {
                        plot_ui.line(
                            Line::new(vec![[1.0, 0.0], [1.0, tan_v]])
                                .color(Color32::from_rgb(206, 126, 218))
                                .width(2.0)
                                .name("tan(theta)"),
                        );
                    }
                }
                plot_ui.points(
                    Points::new(vec![[cx, sy]])
                        .radius(5.5)
                        .color(Color32::from_rgb(232, 168, 72)),
                );
            });

        Plot::new("trig_wave_plot")
            .height(360.0)
            .legend(Legend::default())
            .show(&mut cols[1], |plot_ui| {
                let sin_wave: Vec<[f64; 2]> = (0..=300)
                    .map(|k| {
                        let t = (k as f64 / 300.0) * 2.0 * PI;
                        let y = state.trig_amp * (state.trig_omega * t + state.trig_phase).sin();
                        [t, y]
                    })
                    .collect();
                let cos_wave: Vec<[f64; 2]> = (0..=300)
                    .map(|k| {
                        let t = (k as f64 / 300.0) * 2.0 * PI;
                        let y = state.trig_amp * (state.trig_omega * t + state.trig_phase).cos();
                        [t, y]
                    })
                    .collect();

                plot_ui.line(
                    Line::new(sin_wave)
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.2)
                        .name("A * sin(omega*t + phi)"),
                );
                plot_ui.line(
                    Line::new(cos_wave)
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(1.6)
                        .name("A * cos(omega*t + phi)"),
                );
                plot_ui.vline(VLine::new(theta).color(Color32::from_rgb(232, 168, 72)));

                let y_now = state.trig_amp * (state.trig_omega * theta + state.trig_phase).sin();
                plot_ui.points(
                    Points::new(vec![[theta, y_now]])
                        .radius(5.5)
                        .color(Color32::from_rgb(232, 168, 72))
                        .name("Active Phase Cursor"),
                );
            });
    });

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            let deg = theta.to_degrees();
            let period = 2.0 * PI / state.trig_omega;
            ui.label(
                RichText::new(format!(
                    "theta = {:.4} rad ({:.1} deg) | cos(theta) = {:.4}, sin(theta) = {:.4}, tan(theta) = {:.4} | Wave Period T = 2*pi/omega = {:.4}",
                    theta,
                    deg,
                    cx,
                    sy,
                    sy / cx,
                    period
                ))
                .monospace(),
            );
        });
}

fn render_single_var_calc_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("f(x) =");
                ui.add(
                    TextEdit::singleline(&mut state.calc_expr)
                        .desired_width(260.0)
                        .font(TextStyle::Monospace),
                );
                ui.checkbox(&mut state.calc_show_secant_tangent, "Secant & Tangent");
                ui.checkbox(&mut state.calc_show_riemann, "Riemann / Darboux Area");
                ui.checkbox(&mut state.calc_show_taylor, "Taylor Polynomial T_n(x)");
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.calc_x0, -4.0..=4.0).text("Center / Anchor x0"));
                ui.add(Slider::new(&mut state.calc_h, -2.0..=2.0).text("Secant h"));
                ui.add(Slider::new(&mut state.calc_taylor_deg, 0..=8).text("Taylor deg n"));
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.calc_int_a, -5.0..=4.0).text("Bound a"));
                ui.add(Slider::new(&mut state.calc_int_b, -4.0..=5.0).text("Bound b"));
                ui.add(Slider::new(&mut state.calc_partitions, 1..=100).text("Partitions N"));
                ComboBox::from_label("Sum Rule")
                    .selected_text(state.calc_riemann_method.label())
                    .show_ui(ui, |ui| {
                        for rule in [
                            RiemannMethod::Left,
                            RiemannMethod::Right,
                            RiemannMethod::Midpoint,
                            RiemannMethod::Trapezoid,
                            RiemannMethod::LowerDarboux,
                            RiemannMethod::UpperDarboux,
                        ] {
                            ui.selectable_value(&mut state.calc_riemann_method, rule, rule.label());
                        }
                    });
            });
        });

    let parsed = match Expr::parse(&state.calc_expr) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("Syntax error: {}", err)).color(theme.err_color()));
            return;
        }
    };

    let deriv = parsed.differentiate("x");
    let r_summary = riemann_sum(
        &parsed,
        "x",
        state.calc_int_a,
        state.calc_int_b,
        state.calc_partitions,
        state.calc_riemann_method,
    );
    let (taylor_coeffs, taylor_str) =
        taylor_polynomial_at(&parsed, "x", state.calc_x0, state.calc_taylor_deg);

    Plot::new("single_var_calc_plot")
        .height(390.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            if state.calc_show_riemann {
                let fill = Color32::from_rgba_unmultiplied(88, 166, 236, 48);
                let stroke =
                    egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(88, 166, 236, 150));
                for &(xl, xr, hl, hr) in &r_summary.subintervals {
                    let poly = Polygon::new(vec![[xl, 0.0], [xl, hl], [xr, hr], [xr, 0.0]])
                        .fill_color(fill)
                        .stroke(stroke);
                    plot_ui.polygon(poly);
                }
            }

            for (idx, seg) in sample_curve_segments(&parsed, -6.5, 6.5, 500)
                .into_iter()
                .enumerate()
            {
                let mut line = Line::new(seg)
                    .color(Color32::from_rgb(88, 166, 236))
                    .width(2.4);
                if idx == 0 {
                    line = line.name("f(x)");
                }
                plot_ui.line(line);
            }

            if state.calc_show_taylor {
                let t_pts: Vec<[f64; 2]> = (0..=320)
                    .filter_map(|k| {
                        let x = -6.5 + (k as f64 / 320.0) * 13.0;
                        let dx = x - state.calc_x0;
                        let mut y = 0.0;
                        let mut p = 1.0;
                        for &c in &taylor_coeffs {
                            y += c * p;
                            p *= dx;
                        }
                        if y.is_finite() && y.abs() <= 45.0 {
                            Some([x, y])
                        } else {
                            None
                        }
                    })
                    .collect();
                plot_ui.line(
                    Line::new(t_pts)
                        .color(Color32::from_rgb(206, 126, 218))
                        .width(1.9)
                        .name("Taylor T_n(x)"),
                );
            }

            if state.calc_show_secant_tangent {
                let x0 = state.calc_x0;
                let y0 = parsed.eval_1d("x", x0);
                let slope_tan = deriv.eval_1d("x", x0);
                if y0.is_finite() && slope_tan.is_finite() {
                    plot_ui.line(
                        Line::new(vec![
                            [-6.5, y0 + slope_tan * (-6.5 - x0)],
                            [6.5, y0 + slope_tan * (6.5 - x0)],
                        ])
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.0)
                        .name("Exact Tangent"),
                    );
                }

                let h = if state.calc_h.abs() < 1e-4 {
                    0.001
                } else {
                    state.calc_h
                };
                let y1 = parsed.eval_1d("x", x0 + h);
                if y0.is_finite() && y1.is_finite() {
                    let slope_sec = (y1 - y0) / h;
                    plot_ui.line(
                        Line::new(vec![
                            [-6.5, y0 + slope_sec * (-6.5 - x0)],
                            [6.5, y0 + slope_sec * (6.5 - x0)],
                        ])
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(1.5)
                        .name("Secant Line"),
                    );
                    plot_ui.points(
                        Points::new(vec![[x0, y0], [x0 + h, y1]])
                            .radius(5.0)
                            .color(Color32::from_rgb(232, 168, 72)),
                    );
                }
            }
        });

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            let y0 = parsed.eval_1d("x", state.calc_x0);
            let m_tan = deriv.eval_1d("x", state.calc_x0);
            ui.label(
                RichText::new(format!(
                    "Symbolic f'(x) = {}  |  At x0 = {:.2}: f(x0) = {:.4}, f'(x0) = {:.4}",
                    deriv.to_plain_string(),
                    state.calc_x0,
                    y0,
                    m_tan
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "Integral [{:.2}, {:.2}] ({}, N={}): {:.6}  |  Adaptive Simpson = {:.6}  |  |Error| = {:.2e}",
                    state.calc_int_a,
                    state.calc_int_b,
                    state.calc_riemann_method.label(),
                    state.calc_partitions,
                    r_summary.estimate,
                    r_summary.exact_simpson,
                    r_summary.abs_error
                ))
                .monospace(),
            );
            if state.calc_show_taylor {
                ui.label(
                    RichText::new(format!(
                        "T_{}(x) around {:.2}: {}",
                        state.calc_taylor_deg, state.calc_x0, taylor_str
                    ))
                    .monospace(),
                );
            }
        });
}

fn render_linear_algebra_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                if ui.small_button("Rotation 45 deg").clicked() {
                    let s = std::f64::consts::FRAC_1_SQRT_2;
                    state.la_a11 = s;
                    state.la_a12 = -s;
                    state.la_a21 = s;
                    state.la_a22 = s;
                }
                if ui.small_button("Horizontal Shear").clicked() {
                    state.la_a11 = 1.0;
                    state.la_a12 = 1.25;
                    state.la_a21 = 0.0;
                    state.la_a22 = 1.0;
                }
                if ui.small_button("Symmetric SPD").clicked() {
                    state.la_a11 = 1.8;
                    state.la_a12 = 0.7;
                    state.la_a21 = 0.7;
                    state.la_a22 = 1.1;
                }
                if ui.small_button("Hyperbolic Saddle").clicked() {
                    state.la_a11 = 1.6;
                    state.la_a12 = 0.8;
                    state.la_a21 = 0.8;
                    state.la_a22 = -1.2;
                }
                if ui.small_button("Damped Spiral").clicked() {
                    state.la_a11 = -0.5;
                    state.la_a12 = -1.4;
                    state.la_a21 = 1.4;
                    state.la_a22 = -0.5;
                }
                if ui.small_button("Rank-1 Projection").clicked() {
                    state.la_a11 = 0.8;
                    state.la_a12 = 0.4;
                    state.la_a21 = 0.4;
                    state.la_a22 = 0.2;
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.la_a11, -3.0..=3.0).text("a11"));
                ui.add(Slider::new(&mut state.la_a12, -3.0..=3.0).text("a12"));
                ui.add(Slider::new(&mut state.la_a21, -3.0..=3.0).text("a21"));
                ui.add(Slider::new(&mut state.la_a22, -3.0..=3.0).text("a22"));
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.la_vx, -2.5..=2.5).text("Input v_x"));
                ui.add(Slider::new(&mut state.la_vy, -2.5..=2.5).text("Input v_y"));
            });
        });

    let mat = Mat2::new(state.la_a11, state.la_a12, state.la_a21, state.la_a22);
    let diag = mat.diagnostics();
    let v_in = [state.la_vx, state.la_vy];
    let v_out = mat.apply(v_in);

    ui.columns(2, |cols| {
        // Left: Geometric Action of Matrix A on Unit Circle, Unit Square & Eigen-Axes
        Plot::new("linalg_geometry_plot")
            .height(370.0)
            .data_aspect(1.0)
            .legend(Legend::default())
            .show(&mut cols[0], |plot_ui| {
                // Oriented image parallelogram of unit square
                let c1 = mat.apply([1.0, 0.0]);
                let c12 = mat.apply([1.0, 1.0]);
                let c2 = mat.apply([0.0, 1.0]);
                let (fill_col, stroke_col) = if diag.det >= 0.0 {
                    (
                        Color32::from_rgba_unmultiplied(84, 196, 132, 42),
                        Color32::from_rgba_unmultiplied(84, 196, 132, 165),
                    )
                } else {
                    (
                        Color32::from_rgba_unmultiplied(224, 98, 98, 42),
                        Color32::from_rgba_unmultiplied(224, 98, 98, 165),
                    )
                };
                plot_ui.polygon(
                    Polygon::new(vec![[0.0, 0.0], c1, c12, c2])
                        .fill_color(fill_col)
                        .stroke(egui::Stroke::new(1.2, stroke_col))
                        .name("Image of Unit Square (det A)"),
                );

                // Reference unit circle & mapped SVD ellipse A(S^1)
                let unit_circle: Vec<[f64; 2]> = (0..=160)
                    .map(|k| {
                        let a = (k as f64 / 160.0) * 2.0 * PI;
                        [a.cos(), a.sin()]
                    })
                    .collect();
                let mapped_ellipse: Vec<[f64; 2]> =
                    unit_circle.iter().map(|&pt| mat.apply(pt)).collect();

                plot_ui.line(
                    Line::new(unit_circle)
                        .color(Color32::from_rgba_unmultiplied(140, 150, 168, 120))
                        .width(1.2)
                        .name("Unit Circle S^1"),
                );
                plot_ui.line(
                    Line::new(mapped_ellipse)
                        .color(Color32::from_rgb(76, 196, 208))
                        .width(2.2)
                        .name("Ellipse A(S^1)"),
                );

                // Invariant real eigenvector lines
                match &diag.spectrum {
                    EigenSpectrum2D::RealDistinct { vec1, vec2, .. } => {
                        plot_ui.line(
                            Line::new(vec![
                                [-3.5 * vec1[0], -3.5 * vec1[1]],
                                [3.5 * vec1[0], 3.5 * vec1[1]],
                            ])
                            .color(Color32::from_rgb(198, 126, 214))
                            .width(1.5)
                            .name("Eigen-axis v1"),
                        );
                        plot_ui.line(
                            Line::new(vec![
                                [-3.5 * vec2[0], -3.5 * vec2[1]],
                                [3.5 * vec2[0], 3.5 * vec2[1]],
                            ])
                            .color(Color32::from_rgb(230, 145, 220))
                            .width(1.5)
                            .name("Eigen-axis v2"),
                        );
                    }
                    EigenSpectrum2D::RealRepeated { primary_vec, .. } => {
                        plot_ui.line(
                            Line::new(vec![
                                [-3.5 * primary_vec[0], -3.5 * primary_vec[1]],
                                [3.5 * primary_vec[0], 3.5 * primary_vec[1]],
                            ])
                            .color(Color32::from_rgb(198, 126, 214))
                            .width(1.5)
                            .name("Eigen-axis v"),
                        );
                    }
                    EigenSpectrum2D::ComplexConjugate { .. } => {}
                }

                // Column vectors A*e1, A*e2 and user vector v -> A*v
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], c1])
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(2.6)
                        .name("Col 1 (A*e1)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], c2])
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.6)
                        .name("Col 2 (A*e2)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], v_in])
                        .color(Color32::from_rgba_unmultiplied(232, 168, 72, 140))
                        .width(1.8)
                        .name("Input v"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], v_out])
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(2.8)
                        .name("Mapped A*v"),
                );
                plot_ui.points(
                    Points::new(vec![v_out])
                        .radius(5.5)
                        .color(Color32::from_rgb(232, 168, 72)),
                );
            });

        // Right: Trace-Determinant Stability Plane (det = tr^2 / 4)
        Plot::new("trace_det_plane_plot")
            .height(370.0)
            .legend(Legend::default())
            .show(&mut cols[1], |plot_ui| {
                let parabola: Vec<[f64; 2]> = (0..=200)
                    .map(|k| {
                        let tr = -6.0 + (k as f64 / 200.0) * 12.0;
                        [tr, 0.25 * tr * tr]
                    })
                    .collect();
                plot_ui.line(
                    Line::new(parabola)
                        .color(Color32::from_rgb(198, 126, 214))
                        .width(2.0)
                        .name("Parabola det = tr^2 / 4 (Delta = 0)"),
                );
                plot_ui.hline(
                    HLine::new(0.0)
                        .color(Color32::from_rgb(140, 150, 168))
                        .name("Saddle Threshold det = 0"),
                );
                plot_ui.vline(VLine::new(0.0).color(Color32::from_rgb(140, 150, 168)));
                plot_ui.points(
                    Points::new(vec![[diag.trace, diag.det]])
                        .radius(6.5)
                        .color(Color32::from_rgb(232, 168, 72))
                        .name("Matrix A = (tr(A), det(A))"),
                );
            });
    });

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "tr(A) = {:.4}  |  det(A) = {:.4} (signed area factor)  |  rank(A) = {}  |  SVD stretches: sigma_1 = {:.4}, sigma_2 = {:.4}",
                    diag.trace,
                    diag.det,
                    diag.rank,
                    diag.singular_values.0,
                    diag.singular_values.1
                ))
                .monospace(),
            );

            let spec_str = match &diag.spectrum {
                EigenSpectrum2D::RealDistinct {
                    lambda1,
                    lambda2,
                    vec1,
                    vec2,
                } => format!(
                    "Real Eigenvalues: lambda_1 = {:.4} (v1 = <{:.3}, {:.3}>),  lambda_2 = {:.4} (v2 = <{:.3}, {:.3}>)",
                    lambda1, vec1[0], vec1[1], lambda2, vec2[0], vec2[1]
                ),
                EigenSpectrum2D::RealRepeated {
                    lambda,
                    primary_vec,
                    geometric_mult,
                } => format!(
                    "Repeated Eigenvalue: lambda = {:.4} (geom mult = {}, axis = <{:.3}, {:.3}>)",
                    lambda, geometric_mult, primary_vec[0], primary_vec[1]
                ),
                EigenSpectrum2D::ComplexConjugate {
                    real_part,
                    imag_part,
                } => format!(
                    "Complex Conjugate Eigenvalues: lambda = {:.4} +/- {:.4}*i (rotational spiral/center)",
                    real_part, imag_part
                ),
            };
            ui.label(RichText::new(spec_str).monospace());

            let inv_str = match diag.inverse {
                Some(inv) => format!(
                    "A^-1 = [[{:.3}, {:.3}], [{:.3}, {:.3}]]",
                    inv.a11, inv.a12, inv.a21, inv.a22
                ),
                None => "A^-1 does not exist (singular: det(A) = 0)".to_string(),
            };
            ui.label(
                RichText::new(format!(
                    "{}  |  x' = A*x Phase Portrait: {}",
                    inv_str, diag.phase_portrait_class
                ))
                .monospace(),
            );
        });
}

fn render_epsilon_delta_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                "Adjust delta > 0 so that EVERY point x in (x0 - delta, x0 + delta) maps strictly inside the target horizontal band (L - epsilon, L + epsilon):",
            );
            ui.horizontal_wrapped(|ui| {
                ui.label("f(x) =");
                ui.add(
                    TextEdit::singleline(&mut state.eps_expr)
                        .desired_width(220.0)
                        .font(TextStyle::Monospace),
                );
                ui.add(Slider::new(&mut state.eps_x0, -3.0..=3.0).text("Point x0"));
                ui.add(Slider::new(&mut state.eps_target, 0.05..=1.5).text("Target epsilon"));
                ui.add(Slider::new(&mut state.eps_delta, 0.005..=1.2).text("Candidate delta"));
            });
        });

    let expr = match Expr::parse(&state.eps_expr) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("Syntax error: {}", err)).color(theme.err_color()));
            return;
        }
    };

    let x0 = state.eps_x0;
    let l_val = 0.5 * (expr.eval_1d("x", x0 - 1e-7) + expr.eval_1d("x", x0 + 1e-7));
    let mut max_dev = 0.0_f64;
    let mut violations: Vec<[f64; 2]> = Vec::new();

    for k in 0..=300 {
        if k == 150 {
            continue;
        }
        let x = (x0 - state.eps_delta) + (k as f64 / 300.0) * (2.0 * state.eps_delta);
        let y = expr.eval_1d("x", x);
        let dev = (y - l_val).abs();
        if !dev.is_finite() || dev >= state.eps_target {
            if y.is_finite() {
                violations.push([x, y]);
            }
            max_dev = max_dev.max(dev);
        } else {
            max_dev = max_dev.max(dev);
        }
    }

    let valid = l_val.is_finite() && violations.is_empty() && max_dev < state.eps_target;

    Plot::new("epsilon_delta_plot")
        .height(370.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            let win = (state.eps_delta * 3.5).max(1.5);
            for (idx, seg) in sample_curve_segments(&expr, x0 - win, x0 + win, 450)
                .into_iter()
                .enumerate()
            {
                let mut line = Line::new(seg)
                    .color(Color32::from_rgb(88, 166, 236))
                    .width(2.2);
                if idx == 0 {
                    line = line.name("f(x)");
                }
                plot_ui.line(line);
            }

            plot_ui.hline(
                HLine::new(l_val + state.eps_target)
                    .color(Color32::from_rgb(92, 198, 148))
                    .name("L +/- epsilon"),
            );
            plot_ui.hline(
                HLine::new(l_val - state.eps_target).color(Color32::from_rgb(92, 198, 148)),
            );

            let delta_col = if valid {
                Color32::from_rgb(92, 198, 148)
            } else {
                Color32::from_rgb(224, 98, 98)
            };
            plot_ui.vline(
                VLine::new(x0 - state.eps_delta)
                    .color(delta_col)
                    .name("x0 +/- delta"),
            );
            plot_ui.vline(VLine::new(x0 + state.eps_delta).color(delta_col));

            if !violations.is_empty() {
                plot_ui.points(
                    Points::new(violations)
                        .radius(3.5)
                        .color(Color32::from_rgb(224, 98, 98))
                        .name("Points Exiting Epsilon Tube"),
                );
            }
        });

    ui.add_space(6.0);
    let badge_col = if valid {
        theme.ok_color()
    } else {
        theme.err_color()
    };
    Frame::none()
        .fill(theme.card_fill())
        .stroke(egui::Stroke::new(1.5, badge_col))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            let verdict = if valid {
                "VERIFIED: Every x in 0 < |x - x0| < delta satisfies |f(x) - L| < epsilon."
            } else {
                "BOUND VIOLATED: Shrink delta so the curve inside the vertical strip stays strictly between L - epsilon and L + epsilon."
            };
            ui.label(RichText::new(verdict).strong().color(badge_col));
            ui.label(
                RichText::new(format!(
                    "Limit L = {:.5}  |  Target epsilon = {:.4}  |  Candidate delta = {:.4}  |  sup |f(x) - L| on strip = {:.5}",
                    l_val, state.eps_target, state.eps_delta, max_dev
                ))
                .monospace(),
            );
        });
}

fn render_multivariable_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Scalar f(x, y) =");
                ui.add(
                    TextEdit::singleline(&mut state.mv_scalar)
                        .desired_width(180.0)
                        .font(TextStyle::Monospace),
                );
                ui.label("Vector F = <P, Q>:");
                ui.add(
                    TextEdit::singleline(&mut state.mv_vec_p)
                        .desired_width(120.0)
                        .font(TextStyle::Monospace),
                );
                ui.add(
                    TextEdit::singleline(&mut state.mv_vec_q)
                        .desired_width(120.0)
                        .font(TextStyle::Monospace),
                );
                if ui.small_button("Set F = grad f").clicked() {
                    if let Ok(sf) = Expr::parse(&state.mv_scalar) {
                        state.mv_vec_p = sf.differentiate("x").to_plain_string();
                        state.mv_vec_q = sf.differentiate("y").to_plain_string();
                    }
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.mv_probe_x, -2.8..=2.8).text("Probe x0"));
                ui.add(Slider::new(&mut state.mv_probe_y, -2.8..=2.8).text("Probe y0"));
                ui.checkbox(&mut state.mv_show_contours, "Level Contours");
                ui.checkbox(&mut state.mv_show_quiver, "Vector Quiver");
                ui.checkbox(&mut state.mv_show_grad_path, "Gradient Ascent Path");
                ui.label(
                    RichText::new("(Tip: click anywhere in the plot to move probe)")
                        .small()
                        .weak(),
                );
            });
        });

    let sf = match Expr::parse(&state.mv_scalar) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("f(x, y) error: {}", err)).color(theme.err_color()));
            return;
        }
    };
    let vp = match Expr::parse(&state.mv_vec_p) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("P(x, y) error: {}", err)).color(theme.err_color()));
            return;
        }
    };
    let vq = match Expr::parse(&state.mv_vec_q) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("Q(x, y) error: {}", err)).color(theme.err_color()));
            return;
        }
    };

    Plot::new("multivariable_field_plot")
        .height(390.0)
        .data_aspect(1.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            if plot_ui.response().clicked() {
                if let Some(pt) = plot_ui.pointer_coordinate() {
                    state.mv_probe_x = pt.x.clamp(-2.8, 2.8);
                    state.mv_probe_y = pt.y.clamp(-2.8, 2.8);
                }
            }

            if state.mv_show_contours {
                let grid_n = 36usize;
                let min_c = -3.0_f64;
                let max_c = 3.0_f64;
                let step = (max_c - min_c) / (grid_n as f64);
                let mut z_grid = vec![vec![0.0_f64; grid_n + 1]; grid_n + 1];
                for i in 0..=grid_n {
                    let x = min_c + (i as f64) * step;
                    for j in 0..=grid_n {
                        let y = min_c + (j as f64) * step;
                        z_grid[i][j] = sf.eval_2d(x, y);
                    }
                }

                let levels: &[f64] = &[-3.0, -2.0, -1.0, -0.4, 0.0, 0.4, 1.0, 2.0, 3.0];
                let mut first_seg = true;
                for &lvl in levels {
                    for i in 0..grid_n {
                        let x0 = min_c + (i as f64) * step;
                        let x1 = x0 + step;
                        for j in 0..grid_n {
                            let y0 = min_c + (j as f64) * step;
                            let y1 = y0 + step;
                            let z00 = z_grid[i][j];
                            let z10 = z_grid[i + 1][j];
                            let z11 = z_grid[i + 1][j + 1];
                            let z01 = z_grid[i][j + 1];
                            if !z00.is_finite()
                                || !z10.is_finite()
                                || !z11.is_finite()
                                || !z01.is_finite()
                            {
                                continue;
                            }

                            let mut hits: Vec<[f64; 2]> = Vec::with_capacity(4);
                            let edges = [
                                ([x0, y0], [x1, y0], z00, z10),
                                ([x1, y0], [x1, y1], z10, z11),
                                ([x1, y1], [x0, y1], z11, z01),
                                ([x0, y1], [x0, y0], z01, z00),
                            ];
                            for (pa, pb, za, zb) in edges {
                                if (za - lvl) * (zb - lvl) <= 0.0 && (zb - za).abs() > 1e-10 {
                                    let t = ((lvl - za) / (zb - za)).clamp(0.0, 1.0);
                                    hits.push([
                                        pa[0] + t * (pb[0] - pa[0]),
                                        pa[1] + t * (pb[1] - pa[1]),
                                    ]);
                                }
                            }
                            if hits.len() >= 2 {
                                let mut line = Line::new(vec![hits[0], hits[1]])
                                    .color(Color32::from_rgba_unmultiplied(120, 155, 200, 130))
                                    .width(1.1);
                                if first_seg {
                                    line = line.name("Level Curves f(x,y) = c");
                                    first_seg = false;
                                }
                                plot_ui.line(line);
                            }
                        }
                    }
                }
            }

            if state.mv_show_quiver {
                let mut first_q = true;
                for ix in -6..=6 {
                    for iy in -6..=6 {
                        let x = (ix as f64) * 0.45;
                        let y = (iy as f64) * 0.45;
                        let u = vp.eval_2d(x, y);
                        let v = vq.eval_2d(x, y);
                        let norm = (u * u + v * v).sqrt();
                        if norm.is_finite() && norm > 1e-6 {
                            let len = (0.14 + 0.06 * norm.ln_1p()).clamp(0.10, 0.30);
                            let ux = u / norm;
                            let uy = v / norm;
                            let tip = [x + len * ux, y + len * uy];
                            let mut shaft = Line::new(vec![[x, y], tip])
                                .color(Color32::from_rgb(92, 198, 148))
                                .width(1.4);
                            if first_q {
                                shaft = shaft.name("Vector Field F = <P, Q>");
                                first_q = false;
                            }
                            plot_ui.line(shaft);

                            let head = 0.06;
                            let wx1 = tip[0] - head * (ux * 0.85 - uy * 0.52);
                            let wy1 = tip[1] - head * (uy * 0.85 + ux * 0.52);
                            let wx2 = tip[0] - head * (ux * 0.85 + uy * 0.52);
                            let wy2 = tip[1] - head * (uy * 0.85 - ux * 0.52);
                            plot_ui.line(
                                Line::new(vec![[wx1, wy1], tip, [wx2, wy2]])
                                    .color(Color32::from_rgb(92, 198, 148))
                                    .width(1.3),
                            );
                        }
                    }
                }
            }

            if state.mv_show_grad_path {
                let fx = sf.differentiate("x");
                let fy = sf.differentiate("y");
                let mut path = Vec::with_capacity(65);
                let mut cx = state.mv_probe_x;
                let mut cy = state.mv_probe_y;
                path.push([cx, cy]);
                for _ in 0..60 {
                    let gx = fx.eval_2d(cx, cy);
                    let gy = fy.eval_2d(cx, cy);
                    let gn = (gx * gx + gy * gy).sqrt();
                    if !gn.is_finite() || gn < 1e-4 {
                        break;
                    }
                    cx += 0.06 * (gx / gn);
                    cy += 0.06 * (gy / gn);
                    if cx.abs() > 3.1 || cy.abs() > 3.1 {
                        break;
                    }
                    path.push([cx, cy]);
                }
                if path.len() >= 2 {
                    plot_ui.line(
                        Line::new(path)
                            .color(Color32::from_rgb(232, 168, 72))
                            .width(2.4)
                            .name("Gradient Ascent Trajectory"),
                    );
                }
            }

            plot_ui.points(
                Points::new(vec![[state.mv_probe_x, state.mv_probe_y]])
                    .radius(6.0)
                    .color(Color32::from_rgb(232, 168, 72))
                    .name("Probe (x0, y0)"),
            );
        });

    let diag = evaluate_field_2d(&sf, &vp, &vq, state.mv_probe_x, state.mv_probe_y);
    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "Probe ({:.2}, {:.2}):  f = {:.4}  |  grad f = <{:.4}, {:.4}>  (|grad f| = {:.4})",
                    state.mv_probe_x,
                    state.mv_probe_y,
                    diag.f_val,
                    diag.grad.0,
                    diag.grad.1,
                    diag.grad_norm
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "Hessian H = [[{:.3}, {:.3}], [{:.3}, {:.3}]]  |  det(H) = {:.4}  ->  {}",
                    diag.hessian[0][0],
                    diag.hessian[0][1],
                    diag.hessian[1][0],
                    diag.hessian[1][1],
                    diag.hessian_det,
                    diag.classification
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "Vector Operators at Probe:  div F (dP/dx + dQ/dy) = {:.4}  |  2D scalar curl_z F (dQ/dx - dP/dy) = {:.4}",
                    diag.divergence, diag.curl_z
                ))
                .monospace(),
            );
        });
}

fn render_ode_pde_lab(ctx: &Context, ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Differential Sub-Lab:").strong());
        ui.selectable_value(
            &mut state.ode_submode,
            OdePdeSubmode::OdePhasePortrait,
            "A. 2D ODE Systems & RK4 Phase Portraits",
        );
        ui.selectable_value(
            &mut state.ode_submode,
            OdePdeSubmode::PdeEvolution1D,
            "B. 1D Spectral PDEs (Heat, Wave & Advection)",
        );
    });
    ui.add_space(4.0);

    match state.ode_submode {
        OdePdeSubmode::OdePhasePortrait => render_ode_phase_sublab(ui, state, theme),
        OdePdeSubmode::PdeEvolution1D => render_pde_sublab(ctx, ui, state, theme),
    }
}

fn render_ode_phase_sublab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Presets:").strong());
                if ui.small_button("Damped Oscillator").clicked() {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "-2.0*x - 0.35*y".to_string();
                    state.ode_seed_x = 2.2;
                    state.ode_seed_y = 0.0;
                }
                if ui.small_button("Nonlinear Pendulum").clicked() {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "-2.0*sin(x) - 0.15*y".to_string();
                    state.ode_seed_x = 2.4;
                    state.ode_seed_y = 0.2;
                }
                if ui.small_button("Lotka-Volterra").clicked() {
                    state.ode_rhs_p = "x * (1.2 - 0.8*y)".to_string();
                    state.ode_rhs_q = "y * (-0.9 + 0.6*x)".to_string();
                    state.ode_seed_x = 1.8;
                    state.ode_seed_y = 0.8;
                }
                if ui.small_button("Van der Pol Cycle").clicked() {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "1.1*(1 - x^2)*y - x".to_string();
                    state.ode_seed_x = 0.35;
                    state.ode_seed_y = 0.25;
                }
                if ui.small_button("Saddle Portrait").clicked() {
                    state.ode_rhs_p = "x + 0.5*y".to_string();
                    state.ode_rhs_q = "0.5*x - y".to_string();
                    state.ode_seed_x = 0.4;
                    state.ode_seed_y = 2.2;
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("dx/dt = P(x, y) =");
                ui.add(
                    TextEdit::singleline(&mut state.ode_rhs_p)
                        .desired_width(185.0)
                        .font(TextStyle::Monospace),
                );
                ui.label("dy/dt = Q(x, y) =");
                ui.add(
                    TextEdit::singleline(&mut state.ode_rhs_q)
                        .desired_width(185.0)
                        .font(TextStyle::Monospace),
                );
                ui.checkbox(&mut state.ode_show_ensemble, "Show Background Portraits");
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.ode_seed_x, -3.2..=3.2).text("Initial x(0)"));
                ui.add(Slider::new(&mut state.ode_seed_y, -3.2..=3.2).text("Initial y(0)"));
                ui.label(
                    RichText::new("(Click anywhere in phase plane to re-seed x(0), y(0))")
                        .small()
                        .weak(),
                );
            });
        });

    let p_expr = match Expr::parse(&state.ode_rhs_p) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("dx/dt error: {}", err)).color(theme.err_color()));
            return;
        }
    };
    let q_expr = match Expr::parse(&state.ode_rhs_q) {
        Ok(e) => e,
        Err(err) => {
            ui.label(RichText::new(format!("dy/dt error: {}", err)).color(theme.err_color()));
            return;
        }
    };

    Plot::new("ode_phase_portrait_plot")
        .height(390.0)
        .data_aspect(1.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            if plot_ui.response().clicked() {
                if let Some(pt) = plot_ui.pointer_coordinate() {
                    state.ode_seed_x = pt.x.clamp(-3.2, 3.2);
                    state.ode_seed_y = pt.y.clamp(-3.2, 3.2);
                }
            }

            // Direction field quiver needles
            let mut first_q = true;
            for ix in -7..=7 {
                for iy in -7..=7 {
                    let x = (ix as f64) * 0.45;
                    let y = (iy as f64) * 0.45;
                    let u = p_expr.eval_2d(x, y);
                    let v = q_expr.eval_2d(x, y);
                    let norm = (u * u + v * v).sqrt();
                    if norm.is_finite() && norm > 1e-6 {
                        let len = 0.20;
                        let ux = u / norm;
                        let uy = v / norm;
                        let tip = [x + len * ux, y + len * uy];
                        let mut shaft = Line::new(vec![[x, y], tip])
                            .color(Color32::from_rgba_unmultiplied(125, 145, 175, 125))
                            .width(1.1);
                        if first_q {
                            shaft = shaft.name("Slope / Direction Field");
                            first_q = false;
                        }
                        plot_ui.line(shaft);
                    }
                }
            }

            // Background ensemble orbits
            if state.ode_show_ensemble {
                let seeds = [
                    (-2.2, -1.5),
                    (-2.2, 1.5),
                    (2.2, -1.5),
                    (2.2, 1.5),
                    (0.8, 2.0),
                    (-0.8, -2.0),
                    (1.2, 0.5),
                    (-1.2, -0.5),
                ];
                let mut first_bg = true;
                for (sx, sy) in seeds {
                    let bg_traj = solve_planar_system_rk4(&p_expr, &q_expr, sx, sy, 0.025, 450, 4.5);
                    if bg_traj.len() >= 2 {
                        let mut line = Line::new(bg_traj)
                            .color(Color32::from_rgba_unmultiplied(92, 180, 196, 120))
                            .width(1.3);
                        if first_bg {
                            line = line.name("Ensemble RK4 Orbits");
                            first_bg = false;
                        }
                        plot_ui.line(line);
                    }
                }
            }

            // Highlighted primary forward + backward RK4 trajectories from user seed
            let fwd = solve_planar_system_rk4(
                &p_expr,
                &q_expr,
                state.ode_seed_x,
                state.ode_seed_y,
                0.02,
                800,
                4.8,
            );
            let bwd = solve_planar_system_rk4(
                &p_expr,
                &q_expr,
                state.ode_seed_x,
                state.ode_seed_y,
                -0.02,
                350,
                4.8,
            );
            if bwd.len() >= 2 {
                plot_ui.line(
                    Line::new(bwd)
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(1.8)
                        .name("Backward Trajectory (t < 0)"),
                );
            }
            if fwd.len() >= 2 {
                plot_ui.line(
                    Line::new(fwd)
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(2.6)
                        .name("Forward RK4 Trajectory (t >= 0)"),
                );
            }
            plot_ui.points(
                Points::new(vec![[state.ode_seed_x, state.ode_seed_y]])
                    .radius(6.0)
                    .color(Color32::from_rgb(232, 168, 72))
                    .name("Initial State (x(0), y(0))"),
            );
        });

    // Symbolic Jacobian Linearization at (x(0), y(0))
    let px = p_expr.differentiate("x").eval_2d(state.ode_seed_x, state.ode_seed_y);
    let py = p_expr.differentiate("y").eval_2d(state.ode_seed_x, state.ode_seed_y);
    let qx = q_expr.differentiate("x").eval_2d(state.ode_seed_x, state.ode_seed_y);
    let qy = q_expr.differentiate("y").eval_2d(state.ode_seed_x, state.ode_seed_y);
    let jac = Mat2::new(px, py, qx, qy);
    let jdiag = jac.diagnostics();
    let vx = p_expr.eval_2d(state.ode_seed_x, state.ode_seed_y);
    let vy = q_expr.eval_2d(state.ode_seed_x, state.ode_seed_y);

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "At ({:.2}, {:.2}): Velocity <dx/dt, dy/dt> = <{:.4}, {:.4}>  |  Local Jacobian J = [[{:.3}, {:.3}], [{:.3}, {:.3}]]",
                    state.ode_seed_x, state.ode_seed_y, vx, vy, px, py, qx, qy
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "tr(J) = {:.4},  det(J) = {:.4},  Delta = {:.4}  |  Linearized Regime: {}",
                    jdiag.trace, jdiag.det, jdiag.discriminant, jdiag.phase_portrait_class
                ))
                .monospace(),
            );
        });
}

fn render_pde_sublab(ctx: &Context, ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    if state.pde_animate {
        let dt = ctx.input(|i| i.stable_dt as f64).clamp(0.001, 0.05);
        state.pde_time = (state.pde_time + 0.55 * dt) % 8.0;
        ctx.request_repaint();
    }

    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ComboBox::from_label("PDE Equation")
                    .selected_text(state.pde_kind.label())
                    .show_ui(ui, |ui| {
                        for k in [
                            PdeKind::HeatDiffusion,
                            PdeKind::WaveEquation,
                            PdeKind::AdvectionDiffusion,
                        ] {
                            ui.selectable_value(&mut state.pde_kind, k, k.label());
                        }
                    });

                ComboBox::from_label("Initial u(x, 0)")
                    .selected_text(state.pde_profile.label())
                    .show_ui(ui, |ui| {
                        for p in [
                            PdeInitialProfile::SquarePulse,
                            PdeInitialProfile::PluckedTriangle,
                            PdeInitialProfile::TwoModeHarmonic,
                            PdeInitialProfile::LocalizedGaussian,
                        ] {
                            ui.selectable_value(&mut state.pde_profile, p, p.label());
                        }
                    });
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.pde_time, 0.0..=8.0).text("Time t"));
                ui.checkbox(&mut state.pde_animate, "Animate Evolution");
                if ui.small_button("Reset t = 0").clicked() {
                    state.pde_time = 0.0;
                    state.pde_animate = false;
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.pde_alpha, 0.0..=0.5).text("Diffusivity alpha"));
                ui.add(Slider::new(&mut state.pde_speed, 0.2..=2.5).text("Speed c / v"));
                ui.add(Slider::new(&mut state.pde_modes, 1..=35).text("Fourier Modes M"));
            });
        });

    let snap = evaluate_1d_pde(
        state.pde_kind,
        state.pde_profile,
        state.pde_alpha,
        state.pde_speed,
        state.pde_time,
        state.pde_modes,
        240,
    );

    Plot::new("pde_1d_evolution_plot")
        .height(370.0)
        .include_y(-1.3)
        .include_y(1.3)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.line(
                Line::new(snap.initial_curve)
                    .color(Color32::from_rgba_unmultiplied(145, 160, 185, 145))
                    .width(1.6)
                    .name("Initial Profile u(x, 0)"),
            );
            plot_ui.line(
                Line::new(snap.evolved_curve)
                    .color(Color32::from_rgb(230, 122, 108))
                    .width(2.6)
                    .name("Evolved Solution u(x, t)"),
            );
        });

    let top_modes: Vec<String> = snap
        .mode_energies
        .iter()
        .take(5)
        .map(|(n, amp)| format!("|c_{}|={:.3}", n, amp))
        .collect();

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "Domain [0, {:.2}]  |  t = {:.3}  |  L^2 Energy Retention ||u(.,t)||^2 / ||u(.,0)||^2 = {:.4}",
                    snap.domain_len, state.pde_time, snap.l2_energy_ratio
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "Leading Harmonics (M = {}):  {}",
                    state.pde_modes,
                    top_modes.join("   ")
                ))
                .monospace(),
            );
        });
}


fn render_prob_stats_lab(ui: &mut Ui, state: &mut LabState, theme: AcademicTheme) {
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ComboBox::from_label("Distribution")
                    .selected_text(state.stats_dist.label())
                    .show_ui(ui, |ui| {
                        for d in [
                            DistributionKind::Normal,
                            DistributionKind::Exponential,
                            DistributionKind::Poisson,
                            DistributionKind::Binomial,
                        ] {
                            if ui
                                .selectable_value(&mut state.stats_dist, d, d.label())
                                .clicked()
                            {
                                match d {
                                    DistributionKind::Normal => {
                                        state.stats_param_a = 0.0;
                                        state.stats_param_b = 1.0;
                                        state.stats_interval_a = -1.0;
                                        state.stats_interval_b = 1.5;
                                    }
                                    DistributionKind::Exponential => {
                                        state.stats_param_a = 0.8;
                                        state.stats_interval_a = 0.2;
                                        state.stats_interval_b = 2.0;
                                    }
                                    DistributionKind::Poisson => {
                                        state.stats_param_a = 4.5;
                                        state.stats_interval_a = 2.0;
                                        state.stats_interval_b = 6.0;
                                    }
                                    DistributionKind::Binomial => {
                                        state.stats_param_a = 18.0;
                                        state.stats_param_b = 0.35;
                                        state.stats_interval_a = 4.0;
                                        state.stats_interval_b = 9.0;
                                    }
                                }
                            }
                        }
                    });

                ui.separator();
                ui.label(RichText::new("Right Panel:").strong());
                ui.selectable_value(
                    &mut state.stats_right_mode,
                    StatsRightSubmode::BayesianDiagnostic,
                    "Bayesian Updater",
                );
                ui.selectable_value(
                    &mut state.stats_right_mode,
                    StatsRightSubmode::OlsRegression,
                    "Interactive OLS Regression",
                );
            });

            ui.horizontal_wrapped(|ui| {
                match state.stats_dist {
                    DistributionKind::Normal => {
                        ui.add(Slider::new(&mut state.stats_param_a, -3.0..=3.0).text("Mean mu"));
                        ui.add(Slider::new(&mut state.stats_param_b, 0.25..=2.5).text("SD sigma"));
                        ui.add(Slider::new(&mut state.stats_interval_a, -5.0..=5.0).text("Lower a"));
                        ui.add(Slider::new(&mut state.stats_interval_b, -5.0..=5.0).text("Upper b"));
                    }
                    DistributionKind::Exponential => {
                        ui.add(Slider::new(&mut state.stats_param_a, 0.2..=3.0).text("Rate lambda"));
                        ui.add(Slider::new(&mut state.stats_interval_a, 0.0..=6.0).text("Lower a"));
                        ui.add(Slider::new(&mut state.stats_interval_b, 0.0..=8.0).text("Upper b"));
                    }
                    DistributionKind::Poisson => {
                        ui.add(Slider::new(&mut state.stats_param_a, 0.5..=14.0).text("Mean lambda"));
                        ui.add(Slider::new(&mut state.stats_interval_a, 0.0..=20.0).text("Min k"));
                        ui.add(Slider::new(&mut state.stats_interval_b, 0.0..=20.0).text("Max k"));
                    }
                    DistributionKind::Binomial => {
                        ui.add(Slider::new(&mut state.stats_param_a, 2.0..=30.0).text("Trials n"));
                        ui.add(Slider::new(&mut state.stats_param_b, 0.05..=0.95).text("Success p"));
                        ui.add(Slider::new(&mut state.stats_interval_a, 0.0..=30.0).text("Min k"));
                        ui.add(Slider::new(&mut state.stats_interval_b, 0.0..=30.0).text("Max k"));
                    }
                }
            });

            if state.stats_right_mode == StatsRightSubmode::BayesianDiagnostic {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Slider::new(&mut state.stats_prior, 0.005..=0.90).text("Base Prior P(H)"));
                    ui.add(Slider::new(&mut state.stats_sens, 0.50..=0.999).text("Sensitivity P(+|H)"));
                    ui.add(Slider::new(&mut state.stats_spec, 0.50..=0.999).text("Specificity P(-|~H)"));
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Click anywhere inside the right scatter plot to add an experimental (x, y) observation!")
                            .small()
                            .weak(),
                    );
                    if ui.small_button("Reset Sample Points").clicked() {
                        state.stats_points = vec![
                            [0.5, 1.4],
                            [1.2, 2.5],
                            [2.0, 3.1],
                            [2.8, 4.7],
                            [3.6, 5.2],
                            [4.4, 6.8],
                        ];
                    }
                });
            }
        });

    let lo = state.stats_interval_a.min(state.stats_interval_b);
    let hi = state.stats_interval_a.max(state.stats_interval_b);
    let mut prob_mass = 0.0_f64;
    let mut mean_val = 0.0_f64;
    let mut var_val = 0.0_f64;

    ui.columns(2, |cols| {
        // Left Plot: Continuous PDF + Shaded Area OR Discrete PMF Bars
        Plot::new("prob_dist_explorer_plot")
            .height(360.0)
            .include_y(0.0)
            .legend(Legend::default())
            .show(&mut cols[0], |plot_ui| match state.stats_dist {
                DistributionKind::Normal => {
                    let mu = state.stats_param_a;
                    let sig = state.stats_param_b.max(0.2);
                    mean_val = mu;
                    var_val = sig * sig;
                    prob_mass = (normal_cdf(hi, mu, sig) - normal_cdf(lo, mu, sig)).max(0.0);

                    let x_min = mu - 4.2 * sig;
                    let x_max = mu + 4.2 * sig;
                    let curve: Vec<[f64; 2]> = (0..=240)
                        .map(|i| {
                            let x = x_min + (i as f64 / 240.0) * (x_max - x_min);
                            [x, normal_pdf(x, mu, sig)]
                        })
                        .collect();

                    let shade_lo = lo.clamp(x_min, x_max);
                    let shade_hi = hi.clamp(x_min, x_max);
                    if shade_hi > shade_lo {
                        let mut poly = Vec::with_capacity(84);
                        poly.push([shade_lo, 0.0]);
                        for i in 0..=80 {
                            let x = shade_lo + (i as f64 / 80.0) * (shade_hi - shade_lo);
                            poly.push([x, normal_pdf(x, mu, sig)]);
                        }
                        poly.push([shade_hi, 0.0]);
                        plot_ui.polygon(
                            Polygon::new(poly)
                                .fill_color(Color32::from_rgba_unmultiplied(154, 208, 96, 65))
                                .stroke(egui::Stroke::new(
                                    1.2,
                                    Color32::from_rgb(154, 208, 96),
                                ))
                                .name("P(a <= X <= b)"),
                        );
                    }

                    plot_ui.line(
                        Line::new(curve)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(2.4)
                            .name("Normal PDF f(x)"),
                    );
                }

                DistributionKind::Exponential => {
                    let lam = state.stats_param_a.max(0.1);
                    mean_val = 1.0 / lam;
                    var_val = 1.0 / (lam * lam);
                    let slo = lo.max(0.0);
                    let shi = hi.max(slo);
                    prob_mass = (exponential_cdf(shi, lam) - exponential_cdf(slo, lam)).max(0.0);

                    let x_max = (5.0 / lam).clamp(3.0, 12.0);
                    let curve: Vec<[f64; 2]> = (0..=220)
                        .map(|i| {
                            let x = (i as f64 / 220.0) * x_max;
                            [x, exponential_pdf(x, lam)]
                        })
                        .collect();

                    let c_hi = shi.min(x_max);
                    if c_hi > slo {
                        let mut poly = Vec::with_capacity(74);
                        poly.push([slo, 0.0]);
                        for i in 0..=70 {
                            let x = slo + (i as f64 / 70.0) * (c_hi - slo);
                            poly.push([x, exponential_pdf(x, lam)]);
                        }
                        poly.push([c_hi, 0.0]);
                        plot_ui.polygon(
                            Polygon::new(poly)
                                .fill_color(Color32::from_rgba_unmultiplied(154, 208, 96, 65))
                                .stroke(egui::Stroke::new(
                                    1.2,
                                    Color32::from_rgb(154, 208, 96),
                                ))
                                .name("P(a <= T <= b)"),
                        );
                    }

                    plot_ui.line(
                        Line::new(curve)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(2.4)
                            .name("Exponential PDF f(t)"),
                    );
                }

                DistributionKind::Poisson => {
                    let lam = state.stats_param_a.max(0.1);
                    mean_val = lam;
                    var_val = lam;
                    let k_lo = lo.ceil().max(0.0) as u32;
                    let k_hi = hi.floor().max(0.0) as u32;
                    let max_k = ((lam + 4.5 * lam.sqrt()).ceil() as u32).clamp(12, 32);

                    let mut in_bars = Vec::new();
                    let mut out_bars = Vec::new();
                    for k in 0..=max_k {
                        let p = poisson_pmf(k, lam);
                        if k >= k_lo && k <= k_hi {
                            prob_mass += p;
                            in_bars.push(
                                Bar::new(k as f64, p)
                                    .width(0.68)
                                    .fill(Color32::from_rgb(154, 208, 96)),
                            );
                        } else {
                            out_bars.push(
                                Bar::new(k as f64, p)
                                    .width(0.68)
                                    .fill(Color32::from_rgba_unmultiplied(88, 166, 236, 135)),
                            );
                        }
                    }
                    plot_ui.bar_chart(BarChart::new(out_bars).name("Poisson PMF"));
                    plot_ui.bar_chart(BarChart::new(in_bars).name("Selected Range [k_min, k_max]"));
                }

                DistributionKind::Binomial => {
                    let n = (state.stats_param_a.round() as u32).clamp(2, 40);
                    let p_succ = state.stats_param_b.clamp(0.01, 0.99);
                    mean_val = (n as f64) * p_succ;
                    var_val = (n as f64) * p_succ * (1.0 - p_succ);
                    let k_lo = lo.ceil().max(0.0) as u32;
                    let k_hi = hi.floor().max(0.0) as u32;

                    let mut in_bars = Vec::new();
                    let mut out_bars = Vec::new();
                    for k in 0..=n {
                        let pm = binomial_pmf(k, n, p_succ);
                        if k >= k_lo && k <= k_hi {
                            prob_mass += pm;
                            in_bars.push(
                                Bar::new(k as f64, pm)
                                    .width(0.68)
                                    .fill(Color32::from_rgb(154, 208, 96)),
                            );
                        } else {
                            out_bars.push(
                                Bar::new(k as f64, pm)
                                    .width(0.68)
                                    .fill(Color32::from_rgba_unmultiplied(88, 166, 236, 135)),
                            );
                        }
                    }
                    plot_ui.bar_chart(BarChart::new(out_bars).name("Binomial PMF"));
                    plot_ui.bar_chart(BarChart::new(in_bars).name("Selected Range [k_min, k_max]"));
                }
            });

        // Right Plot: Bayesian Diagnostic Updater OR Interactive OLS Regression
        match state.stats_right_mode {
            StatsRightSubmode::BayesianDiagnostic => {
                let b = compute_bayes_binary(state.stats_prior, state.stats_sens, state.stats_spec);
                Plot::new("bayes_posterior_plot")
                    .height(360.0)
                    .include_y(0.0)
                    .include_y(1.05)
                    .legend(Legend::default())
                    .show(&mut cols[1], |plot_ui| {
                        let bars = vec![
                            Bar::new(1.0, b.prior)
                                .width(0.55)
                                .fill(Color32::from_rgb(88, 166, 236))
                                .name("1: Base Prior P(H)"),
                            Bar::new(2.0, b.marginal_positive)
                                .width(0.55)
                                .fill(Color32::from_rgb(232, 168, 72))
                                .name("2: Total Positive P(+)"),
                            Bar::new(3.0, b.posterior_given_pos)
                                .width(0.55)
                                .fill(Color32::from_rgb(92, 198, 148))
                                .name("3: Posterior P(H | +)"),
                            Bar::new(4.0, b.posterior_given_neg)
                                .width(0.55)
                                .fill(Color32::from_rgb(224, 98, 98))
                                .name("4: Posterior P(H | -)"),
                        ];
                        plot_ui.bar_chart(BarChart::new(bars).name("Probability"));
                    });
            }

            StatsRightSubmode::OlsRegression => {
                let ols = fit_ols_linear(&state.stats_points);
                Plot::new("ols_regression_plot")
                    .height(360.0)
                    .legend(Legend::default())
                    .show(&mut cols[1], |plot_ui| {
                        if plot_ui.response().clicked() {
                            if let Some(pt) = plot_ui.pointer_coordinate() {
                                state.stats_points.push([pt.x, pt.y]);
                            }
                        }

                        plot_ui.points(
                            Points::new(state.stats_points.clone())
                                .radius(5.5)
                                .color(Color32::from_rgb(88, 166, 236))
                                .name("Observed Data (x_i, y_i)"),
                        );

                        if let Some(fit) = ols {
                            let min_x = state
                                .stats_points
                                .iter()
                                .map(|p| p[0])
                                .fold(0.0_f64, f64::min)
                                - 0.5;
                            let max_x = state
                                .stats_points
                                .iter()
                                .map(|p| p[0])
                                .fold(5.0_f64, f64::max)
                                + 0.5;
                            plot_ui.line(
                                Line::new(vec![
                                    [min_x, fit.intercept + fit.slope * min_x],
                                    [max_x, fit.intercept + fit.slope * max_x],
                                ])
                                .color(Color32::from_rgb(154, 208, 96))
                                .width(2.5)
                                .name("OLS Fit y_hat = b0 + b1*x"),
                            );

                            // Draw vertical least-squares residual segments
                            for pt in &state.stats_points {
                                let yhat = fit.intercept + fit.slope * pt[0];
                                plot_ui.line(
                                    Line::new(vec![*pt, [pt[0], yhat]])
                                        .color(Color32::from_rgba_unmultiplied(232, 168, 72, 145))
                                        .width(1.2),
                                );
                            }
                        }
                    });
            }
        }
    });

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "Distribution Telemetry:  E[X] = {:.4}  |  Var(X) = {:.4} (SD = {:.4})  |  Selected Interval Probability P({:.2} <= X <= {:.2}) = {:.5}",
                    mean_val,
                    var_val,
                    var_val.sqrt(),
                    lo,
                    hi,
                    prob_mass
                ))
                .monospace(),
            );

            match state.stats_right_mode {
                StatsRightSubmode::BayesianDiagnostic => {
                    let b = compute_bayes_binary(
                        state.stats_prior,
                        state.stats_sens,
                        state.stats_spec,
                    );
                    ui.label(
                        RichText::new(format!(
                            "Bayes Telemetry:  Prior P(H) = {:.2}%  |  P(+) = {:.2}%  |  PPV Posterior P(H | +) = {:.2}%  |  False-Omission P(H | -) = {:.3}%",
                            b.prior * 100.0,
                            b.marginal_positive * 100.0,
                            b.posterior_given_pos * 100.0,
                            b.posterior_given_neg * 100.0
                        ))
                        .monospace(),
                    );
                }
                StatsRightSubmode::OlsRegression => {
                    if let Some(fit) = fit_ols_linear(&state.stats_points) {
                        ui.label(
                            RichText::new(format!(
                                "OLS Telemetry (n = {}):  y_hat = {:.4} + ({:.4})*x  |  r = {:.4},  R^2 = {:.4}  |  Residual SE = {:.4}",
                                fit.n, fit.intercept, fit.slope, fit.r_corr, fit.r_squared, fit.residual_std_err
                            ))
                            .monospace(),
                        );
                    }
                }
            }
        });
}

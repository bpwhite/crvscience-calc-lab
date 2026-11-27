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

use crate::math_render;
use crate::theme::AcademicTheme;
use calc_core::{
    benchmark_ode_integrator_convergence, binomial_pmf, compute_bayes_binary, evaluate_1d_pde,
    evaluate_1d_pde_custom, evaluate_field_2d, exponential_cdf, exponential_pdf,
    extract_zero_level_segments, find_planar_equilibria, fit_ols_linear, normal_cdf, normal_pdf,
    poisson_pmf, riemann_sum, solve_planar_system_euler_timed, solve_planar_system_midpoint_timed,
    solve_planar_system_rk4, solve_planar_system_rk4_timed, taylor_polynomial_at,
    DistributionKind, EigenSpectrum2D, EquilibriumPoint, Expr, LabPreset, Mat2,
    OdeConvergenceBenchmark, PdeInitialProfile, PdeKind, PdeSnapshot, RiemannMethod,
};
use egui::{
    Color32, ComboBox, Context, Frame, Margin, RichText, Rounding, Slider, Stroke, TextEdit,
    TextStyle, Ui,
};
use egui_plot::{
    Bar, BarChart, HLine, Legend, Line, LineStyle, MarkerShape, Plot, PlotPoint, Points, Polygon,
    Text as PlotText, VLine,
};
use std::f64::consts::PI;
use std::fmt::Write as _;

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
    pub mv_show_3d_surface: bool,
    pub mv_rot_yaw: f32,
    pub mv_rot_pitch: f32,
    pub mv_zoom_3d: f32,
    // Mode 7: ODE Phase Portraits & 1D Spectral PDEs
    pub ode_submode: OdePdeSubmode,
    pub ode_rhs_p: String,
    pub ode_rhs_q: String,
    pub ode_seed_x: f64,
    pub ode_seed_y: f64,
    pub ode_dt: f64,
    pub ode_t_max: f64,
    pub ode_show_ensemble: bool,
    pub ode_show_nullclines: bool,
    pub ode_show_equilibria: bool,
    pub ode_show_euler: bool,
    pub ode_show_midpoint: bool,
    pub ode_convergence_bench: Option<OdeConvergenceBenchmark>,
    pub ode_view_bounds: [f64; 4],
    pub ode_reset_view: bool,
    pub pde_kind: PdeKind,
    pub pde_profile: PdeInitialProfile,
    pub pde_use_custom: bool,
    pub pde_custom_expr: String,
    pub pde_show_truncation_t0: bool,
    pub pde_alpha: f64,
    pub pde_speed: f64,
    pub pde_time: f64,
    pub pde_modes: usize,
    pub pde_animate: bool,
    pub export_status_msg: Option<(bool, String)>,
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
            mv_show_3d_surface: false,
            mv_rot_yaw: 0.65,
            mv_rot_pitch: 0.48,
            mv_zoom_3d: 1.0,
            ode_submode: OdePdeSubmode::OdePhasePortrait,
            ode_rhs_p: "y".to_string(),
            ode_rhs_q: "-2.0*x - 0.35*y".to_string(),
            ode_seed_x: 2.0,
            ode_seed_y: 0.0,
            ode_dt: 0.05,
            ode_t_max: 10.0,
            ode_show_ensemble: true,
            ode_show_nullclines: true,
            ode_show_equilibria: true,
            ode_show_euler: true,
            ode_show_midpoint: true,
            ode_convergence_bench: None,
            ode_view_bounds: [-3.5, 3.5, -3.5, 3.5],
            ode_reset_view: false,
            pde_kind: PdeKind::HeatDiffusion,
            pde_profile: PdeInitialProfile::SquarePulse,
            pde_use_custom: false,
            pde_custom_expr: "x * (pi - x) * 0.4".to_string(),
            pde_show_truncation_t0: true,
            pde_alpha: 0.16,
            pde_speed: 1.2,
            pde_time: 0.20,
            pde_modes: 18,
            pde_animate: false,
            export_status_msg: None,
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
                self.pde_use_custom = false;
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
                if let Ok(fe) = Expr::parse(&state.alg_primary) {
                    math_render::copy_latex_button(
                        ui,
                        "lab_alg_f_latex",
                        &fe.to_latex(),
                        "Copy primary f(x) expression as LaTeX",
                    );
                }
            });
        });

    let f_expr = match Expr::parse(&state.alg_primary) {
        Ok(e) => e,
        Err(e) => {
            ui.label(RichText::new(format!("f(x) syntax error: {}", e)).color(theme.err_color()));
            return;
        }
    };
    let g_parsed = Expr::parse(&state.alg_secondary);

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
                    .width(2.2_f32);
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
                        .width(1.8_f32);
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
                        .width(1.5_f32)
                        .name("Secant Line"),
                );
                plot_ui.points(
                    Points::new(vec![[x0, y0], [x0 + h, y1]])
                        .radius(5.0_f32)
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
            ui.horizontal_wrapped(|ui| {
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
                math_render::copy_latex_button(
                    ui,
                    "lab_alg_df_latex",
                    &df_sym.to_latex(),
                    "Copy symbolic derivative f'(x) as LaTeX",
                );
            });
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
                        .width(1.6_f32)
                        .name("Unit Circle x^2 + y^2 = 1"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], [cx, 0.0]])
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(2.8_f32)
                        .name("cos(theta)"),
                );
                plot_ui.line(
                    Line::new(vec![[cx, 0.0], [cx, sy]])
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.8_f32)
                        .name("sin(theta)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], [cx, sy]])
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(2.2_f32)
                        .name("Radius r = 1"),
                );
                if cx.abs() > 0.18 {
                    let tan_v = sy / cx;
                    if tan_v.abs() <= 3.0 {
                        plot_ui.line(
                            Line::new(vec![[1.0, 0.0], [1.0, tan_v]])
                                .color(Color32::from_rgb(206, 126, 218))
                                .width(2.0_f32)
                                .name("tan(theta)"),
                        );
                    }
                }
                plot_ui.points(
                    Points::new(vec![[cx, sy]])
                        .radius(5.5_f32)
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
                        .width(2.2_f32)
                        .name("A * sin(omega*t + phi)"),
                );
                plot_ui.line(
                    Line::new(cos_wave)
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(1.6_f32)
                        .name("A * cos(omega*t + phi)"),
                );
                plot_ui.vline(VLine::new(theta).color(Color32::from_rgb(232, 168, 72)));

                let y_now = state.trig_amp * (state.trig_omega * theta + state.trig_phase).sin();
                plot_ui.points(
                    Points::new(vec![[theta, y_now]])
                        .radius(5.5_f32)
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
            let tan_str = format_tan_readout(sy, cx);
            ui.label(
                RichText::new(format!(
                    "theta = {:.4} rad ({:.1} deg) | cos(theta) = {:.4}, sin(theta) = {:.4}, tan(theta) = {} | Wave Period T = 2*pi/omega = {:.4}",
                    theta,
                    deg,
                    cx,
                    sy,
                    tan_str,
                    period
                ))
                .monospace(),
            );
        });
}

pub(crate) fn format_tan_readout(sy: f64, cx: f64) -> String {
    if cx.abs() < 1e-6 {
        "undefined (asymptote)".to_string()
    } else {
        format!("{:.4}", sy / cx)
    }
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
                if let Ok(ce) = Expr::parse(&state.calc_expr) {
                    math_render::copy_latex_button(
                        ui,
                        "lab_calc_f_latex",
                        &ce.to_latex(),
                        "Copy f(x) expression as LaTeX",
                    );
                }
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
                    egui::Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(88, 166, 236, 150));
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
                    .width(2.4_f32);
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
                        .width(1.9_f32)
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
                        .width(2.0_f32)
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
                        .width(1.5_f32)
                        .name("Secant Line"),
                    );
                    plot_ui.points(
                        Points::new(vec![[x0, y0], [x0 + h, y1]])
                            .radius(5.0_f32)
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
            ui.horizontal_wrapped(|ui| {
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
                math_render::copy_latex_button(
                    ui,
                    "lab_calc_df_latex",
                    &deriv.to_latex(),
                    "Copy symbolic derivative f'(x) as LaTeX",
                );
            });
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
                        .stroke(egui::Stroke::new(1.2_f32, stroke_col))
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
                        .width(1.2_f32)
                        .name("Unit Circle S^1"),
                );
                plot_ui.line(
                    Line::new(mapped_ellipse)
                        .color(Color32::from_rgb(76, 196, 208))
                        .width(2.2_f32)
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
                            .width(1.5_f32)
                            .name("Eigen-axis v1"),
                        );
                        plot_ui.line(
                            Line::new(vec![
                                [-3.5 * vec2[0], -3.5 * vec2[1]],
                                [3.5 * vec2[0], 3.5 * vec2[1]],
                            ])
                            .color(Color32::from_rgb(230, 145, 220))
                            .width(1.5_f32)
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
                            .width(1.5_f32)
                            .name("Eigen-axis v"),
                        );
                    }
                    EigenSpectrum2D::ComplexConjugate { .. } => {}
                }

                // Column vectors A*e1, A*e2 and user vector v -> A*v
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], c1])
                        .color(Color32::from_rgb(88, 166, 236))
                        .width(2.6_f32)
                        .name("Col 1 (A*e1)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], c2])
                        .color(Color32::from_rgb(92, 198, 148))
                        .width(2.6_f32)
                        .name("Col 2 (A*e2)"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], v_in])
                        .color(Color32::from_rgba_unmultiplied(232, 168, 72, 140))
                        .width(1.8_f32)
                        .name("Input v"),
                );
                plot_ui.line(
                    Line::new(vec![[0.0, 0.0], v_out])
                        .color(Color32::from_rgb(232, 168, 72))
                        .width(2.8_f32)
                        .name("Mapped A*v"),
                );
                plot_ui.points(
                    Points::new(vec![v_out])
                        .radius(5.5_f32)
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
                        .width(2.0_f32)
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
                        .radius(6.5_f32)
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
                if let Ok(ee) = Expr::parse(&state.eps_expr) {
                    math_render::copy_latex_button(
                        ui,
                        "lab_eps_f_latex",
                        &ee.to_latex(),
                        "Copy epsilon-delta f(x) expression as LaTeX",
                    );
                }
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
                    .width(2.2_f32);
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
                        .radius(3.5_f32)
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
        .stroke(egui::Stroke::new(1.5_f32, badge_col))
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

pub(crate) fn diverging_perceptual_color(t_raw: f64) -> Color32 {
    let t = t_raw.clamp(-1.0, 1.0);
    let mid = [176.0_f64, 184.0, 198.0];
    let (r, g, b) = if t < 0.0 {
        let low = [59.0_f64, 102.0, 204.0];
        let u = t + 1.0;
        (
            low[0] + u * (mid[0] - low[0]),
            low[1] + u * (mid[1] - low[1]),
            low[2] + u * (mid[2] - low[2]),
        )
    } else {
        let high = [218.0_f64, 72.0, 58.0];
        let u = t;
        (
            mid[0] + u * (high[0] - mid[0]),
            mid[1] + u * (high[1] - mid[1]),
            mid[2] + u * (high[2] - mid[2]),
        )
    };
    Color32::from_rgb(r.round() as u8, g.round() as u8, b.round() as u8)
}

pub(crate) fn nice_contour_levels(z_min: f64, z_max: f64, target_count: usize) -> Vec<f64> {
    let span = z_max - z_min;
    if !z_min.is_finite() || !z_max.is_finite() || !span.is_finite() || span < 1e-9 {
        return Vec::new();
    }
    let count = target_count.clamp(3, 15);
    let raw_step = span / (count as f64);
    let mag = 10.0_f64.powf(raw_step.log10().floor());
    let norm = raw_step / mag;
    let nice_mult = if norm <= 1.25 {
        1.0
    } else if norm <= 2.25 {
        2.0
    } else if norm <= 3.5 {
        2.5
    } else if norm <= 7.0 {
        5.0
    } else {
        10.0
    };
    let step = nice_mult * mag;
    let margin = 0.02 * span;
    let k_start = ((z_min + margin) / step).ceil() as i64;
    let k_end = ((z_max - margin) / step).floor() as i64;
    let mut levels = Vec::new();
    for k in k_start..=k_end {
        let mut lvl = (k as f64) * step;
        if lvl.abs() < 1e-12 * span.max(1.0) {
            lvl = 0.0;
        }
        levels.push(lvl);
    }
    if levels.len() < 3 {
        levels.clear();
        for idx in 1..=7 {
            levels.push(z_min + (idx as f64 / 8.0) * span);
        }
    }
    levels
}

pub(crate) fn format_contour_label(val: f64) -> String {
    if val.abs() < 1e-9 {
        "0".to_string()
    } else if (val - val.round()).abs() < 1e-6 && val.abs() < 1e6 {
        format!("{}", val.round() as i64)
    } else if val.abs() >= 100.0 || val.abs() < 0.01 {
        format!("{:.2e}", val)
    } else {
        let s = format!("{:.2}", val);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub(crate) fn project_3d_surface_point(
    x: f64,
    y: f64,
    z_norm: f64,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    center: egui::Pos2,
    scale_px: f32,
) -> (egui::Pos2, f32) {
    let (sy, cy) = (yaw as f64).sin_cos();
    let (sp, cp) = (pitch as f64).sin_cos();
    let xr = x * cy - y * sy;
    let yr = x * sy + y * cy;
    let up = z_norm * cp + yr * sp;
    let depth = yr * cp - z_norm * sp;
    let cam_dist = 14.0_f64;
    let persp = (cam_dist / (cam_dist + depth).max(2.0)) * (zoom as f64);
    let sx = center.x + (xr * persp) as f32 * scale_px;
    let sy_screen = center.y - (up * persp) as f32 * scale_px;
    (egui::pos2(sx, sy_screen), depth as f32)
}

fn contour_level_param(lvl: f64, z_min: f64, z_max: f64) -> f64 {
    if z_min < 0.0 && z_max > 0.0 {
        let abs_max = z_min.abs().max(z_max.abs()).max(1e-9);
        (lvl / abs_max).clamp(-1.0, 1.0)
    } else {
        let span = (z_max - z_min).max(1e-9);
        (2.0 * (lvl - z_min) / span - 1.0).clamp(-1.0, 1.0)
    }
}

fn render_multivariable_2d_plot(
    ui: &mut Ui,
    state: &mut LabState,
    sf: &Expr,
    vp: &Expr,
    vq: &Expr,
    z_min: f64,
    z_max: f64,
    levels: &[f64],
    z_grid: &[Vec<f64>],
    grid_n: usize,
) {
    let min_c = -3.0_f64;
    let max_c = 3.0_f64;
    let step = (max_c - min_c) / (grid_n as f64);

    Plot::new("multivariable_field_plot")
        .height(390.0)
        .data_aspect(1.0)
        .x_axis_label("x")
        .y_axis_label("y")
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            if plot_ui.response().clicked() {
                if let Some(pt) = plot_ui.pointer_coordinate() {
                    state.mv_probe_x = pt.x.clamp(-2.8, 2.8);
                    state.mv_probe_y = pt.y.clamp(-2.8, 2.8);
                }
            }

            if state.mv_show_contours {
                let mut first_seg = true;
                let mut placed_labels: Vec<[f64; 2]> = Vec::new();
                for &lvl in levels {
                    let t_col = contour_level_param(lvl, z_min, z_max);
                    let c_rgb = diverging_perceptual_color(t_col);
                    let is_zero = lvl.abs() < 1e-12;
                    let stroke_w = if is_zero { 1.8_f32 } else { 1.25_f32 };
                    let mut seg_midpoints: Vec<[f64; 2]> = Vec::new();

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
                                seg_midpoints.push([
                                    0.5 * (hits[0][0] + hits[1][0]),
                                    0.5 * (hits[0][1] + hits[1][1]),
                                ]);
                                let mut line = Line::new(vec![hits[0], hits[1]])
                                    .color(c_rgb)
                                    .width(stroke_w);
                                if first_seg {
                                    line = line.name("Level Contours f(x,y) = c");
                                    first_seg = false;
                                }
                                plot_ui.line(line);
                            }
                        }
                    }

                    // Place at most one non-overlapping level label per contour value near domain interior
                    if !seg_midpoints.is_empty() {
                        let candidate = seg_midpoints
                            .iter()
                            .copied()
                            .filter(|pt| pt[0].abs() <= 2.4 && pt[1].abs() <= 2.4)
                            .find(|pt| {
                                placed_labels
                                    .iter()
                                    .all(|prev| (prev[0] - pt[0]).hypot(prev[1] - pt[1]) >= 0.55)
                            })
                            .or_else(|| {
                                let mid = seg_midpoints[seg_midpoints.len() / 2];
                                if placed_labels
                                    .iter()
                                    .all(|prev| (prev[0] - mid[0]).hypot(prev[1] - mid[1]) >= 0.45)
                                {
                                    Some(mid)
                                } else {
                                    None
                                }
                            });
                        if let Some(lbl_pt) = candidate {
                            placed_labels.push(lbl_pt);
                            plot_ui.text(
                                PlotText::new(
                                    PlotPoint::new(lbl_pt[0], lbl_pt[1]),
                                    format_contour_label(lvl),
                                )
                                .color(c_rgb),
                            );
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
                                .width(1.4_f32);
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
                                    .width(1.3_f32),
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
                            .width(2.4_f32)
                            .name("Gradient Ascent Trajectory"),
                    );
                }
            }

            plot_ui.points(
                Points::new(vec![[state.mv_probe_x, state.mv_probe_y]])
                    .radius(6.0_f32)
                    .color(Color32::from_rgb(232, 168, 72))
                    .name("Probe (x0, y0)"),
            );
        });
}

fn render_3d_surface_mesh(
    ui: &mut Ui,
    state: &mut LabState,
    sf: &Expr,
    z_min: f64,
    z_max: f64,
    theme: AcademicTheme,
) {
    let desired = egui::vec2(ui.available_width().max(260.0), 390.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
    if response.dragged() {
        let delta = response.drag_delta();
        state.mv_rot_yaw += delta.x * 0.012;
        state.mv_rot_pitch = (state.mv_rot_pitch + delta.y * 0.012).clamp(-1.45, 1.45);
    }
    if response.hovered() {
        let scroll = ui.ctx().input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.1 {
            state.mv_zoom_3d = (state.mv_zoom_3d * (1.0 + scroll * 0.002)).clamp(0.55, 2.2);
        }
    }
    if response.double_clicked() {
        state.mv_rot_yaw = 0.65;
        state.mv_rot_pitch = 0.48;
        state.mv_zoom_3d = 1.0;
    }

    let bg = match theme {
        AcademicTheme::SlateDark => Color32::from_rgb(16, 20, 27),
        AcademicTheme::PaperLight => Color32::from_rgb(250, 249, 244),
    };
    let axis_col = match theme {
        AcademicTheme::SlateDark => Color32::from_rgb(150, 162, 182),
        AcademicTheme::PaperLight => Color32::from_rgb(75, 85, 102),
    };
    let wire_col = match theme {
        AcademicTheme::SlateDark => Color32::from_rgba_unmultiplied(20, 25, 34, 130),
        AcademicTheme::PaperLight => Color32::from_rgba_unmultiplied(40, 48, 60, 95),
    };

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, Rounding::same(6.0), bg);
    painter.rect_stroke(rect, Rounding::same(6.0), theme.card_stroke());

    let center = rect.center();
    let scale_px = rect.width().min(rect.height()) * 0.115;
    let z_span = (z_max - z_min).max(1e-6);
    let z_mid = 0.5 * (z_min + z_max);
    let normalize_z = |zv: f64| -> f64 { ((zv - z_mid) / z_span) * 3.6 };

    let mesh_n = 28usize;
    let min_c = -3.0_f64;
    let max_c = 3.0_f64;
    let step = (max_c - min_c) / (mesh_n as f64);

    let mut pts_2d = vec![vec![(egui::Pos2::ZERO, 0.0_f32, f64::NAN); mesh_n + 1]; mesh_n + 1];
    for i in 0..=mesh_n {
        let x = min_c + (i as f64) * step;
        for j in 0..=mesh_n {
            let y = min_c + (j as f64) * step;
            let zv = sf.eval_2d(x, y);
            if zv.is_finite() {
                let zn = normalize_z(zv.clamp(z_min - 0.5 * z_span, z_max + 0.5 * z_span));
                let (sp, d) = project_3d_surface_point(
                    x,
                    y,
                    zn,
                    state.mv_rot_yaw,
                    state.mv_rot_pitch,
                    state.mv_zoom_3d,
                    center,
                    scale_px,
                );
                pts_2d[i][j] = (sp, d, zv);
            }
        }
    }

    struct QuadCell {
        depth: f32,
        corners: [egui::Pos2; 4],
        fill: Color32,
    }
    let mut quads = Vec::with_capacity(mesh_n * mesh_n);
    for i in 0..mesh_n {
        for j in 0..mesh_n {
            let c00 = pts_2d[i][j];
            let c10 = pts_2d[i + 1][j];
            let c11 = pts_2d[i + 1][j + 1];
            let c01 = pts_2d[i][j + 1];
            if !c00.2.is_finite() || !c10.2.is_finite() || !c11.2.is_finite() || !c01.2.is_finite()
            {
                continue;
            }
            let avg_z = 0.25 * (c00.2 + c10.2 + c11.2 + c01.2);
            let avg_depth = 0.25 * (c00.1 + c10.1 + c11.1 + c01.1);
            let t_col = contour_level_param(avg_z, z_min, z_max);
            let base = diverging_perceptual_color(t_col);
            let fill = Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), 220);
            quads.push(QuadCell {
                depth: avg_depth,
                corners: [c00.0, c10.0, c11.0, c01.0],
                fill,
            });
        }
    }
    quads.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for q in quads {
        painter.add(egui::Shape::convex_polygon(
            q.corners.to_vec(),
            q.fill,
            Stroke::new(0.6_f32, wire_col),
        ));
    }

    // 3D bounding axes (x, y, z)
    let z_lo = -1.8_f64;
    let z_hi = 1.8_f64;
    let proj = |x: f64, y: f64, zn: f64| -> egui::Pos2 {
        project_3d_surface_point(
            x,
            y,
            zn,
            state.mv_rot_yaw,
            state.mv_rot_pitch,
            state.mv_zoom_3d,
            center,
            scale_px,
        )
        .0
    };
    let o = proj(-3.0, -3.0, z_lo);
    let px = proj(3.2, -3.0, z_lo);
    let py = proj(-3.0, 3.2, z_lo);
    let pz = proj(-3.0, -3.0, z_hi + 0.35);
    painter.line_segment([o, px], Stroke::new(1.5_f32, axis_col));
    painter.line_segment([o, py], Stroke::new(1.5_f32, axis_col));
    painter.line_segment([o, pz], Stroke::new(1.5_f32, axis_col));
    let font = egui::FontId::monospace(11.5);
    painter.text(px, egui::Align2::LEFT_CENTER, "x (+3)", font.clone(), axis_col);
    painter.text(py, egui::Align2::LEFT_CENTER, "y (+3)", font.clone(), axis_col);
    painter.text(
        pz,
        egui::Align2::LEFT_BOTTOM,
        format!("z max={}", format_contour_label(z_max)),
        font.clone(),
        axis_col,
    );
    painter.text(
        o,
        egui::Align2::RIGHT_TOP,
        format!("z min={}", format_contour_label(z_min)),
        font.clone(),
        axis_col,
    );

    // Active probe dot on 3D surface
    let probe_z = sf.eval_2d(state.mv_probe_x, state.mv_probe_y);
    if probe_z.is_finite() {
        let p_screen = proj(
            state.mv_probe_x,
            state.mv_probe_y,
            normalize_z(probe_z.clamp(z_min - 0.5 * z_span, z_max + 0.5 * z_span)),
        );
        painter.circle_filled(p_screen, 5.5_f32, Color32::from_rgb(232, 168, 72));
        painter.circle_stroke(p_screen, 5.5_f32, Stroke::new(1.4_f32, Color32::WHITE));
    }

    painter.text(
        rect.min + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        "3D Surface z = f(x, y)  [Drag to orbit | Wheel to zoom | Double-click reset]",
        egui::FontId::proportional(11.5),
        axis_col,
    );
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
                if let Ok(fe) = Expr::parse(&state.mv_scalar) {
                    math_render::copy_latex_button(
                        ui,
                        "lab_mv_scalar_latex",
                        &fe.to_latex(),
                        "Copy scalar surface f(x, y) as LaTeX",
                    );
                }
                ui.label("Vector F = <P, Q>:");
                ui.add(
                    TextEdit::singleline(&mut state.mv_vec_p)
                        .desired_width(115.0)
                        .font(TextStyle::Monospace),
                );
                ui.add(
                    TextEdit::singleline(&mut state.mv_vec_q)
                        .desired_width(115.0)
                        .font(TextStyle::Monospace),
                );
                if ui
                    .small_button("Set F = grad f")
                    .on_hover_text("Populate F = <df/dx, df/dy> symbolically from f(x, y)")
                    .clicked()
                {
                    if let Ok(sf) = Expr::parse(&state.mv_scalar) {
                        state.mv_vec_p = sf.differentiate("x").to_plain_string();
                        state.mv_vec_q = sf.differentiate("y").to_plain_string();
                    }
                }
                if let (Ok(pe), Ok(qe)) = (Expr::parse(&state.mv_vec_p), Expr::parse(&state.mv_vec_q))
                {
                    let vec_tex =
                        format!(r"\left\langle {},\, {} \right\rangle", pe.to_latex(), qe.to_latex());
                    math_render::copy_latex_button(
                        ui,
                        "lab_mv_vec_latex",
                        &vec_tex,
                        "Copy vector field F = <P, Q> as LaTeX",
                    );
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.mv_probe_x, -2.8..=2.8).text("Probe x0"));
                ui.add(Slider::new(&mut state.mv_probe_y, -2.8..=2.8).text("Probe y0"));
                ui.checkbox(&mut state.mv_show_contours, "Level Contours")
                    .on_hover_text("Draw auto-scaled diverging perceptual contours f(x, y) = c");
                ui.checkbox(&mut state.mv_show_quiver, "Vector Quiver")
                    .on_hover_text("Overlay vector field arrows F(x, y) = <P, Q>");
                ui.checkbox(&mut state.mv_show_grad_path, "Gradient Ascent Path")
                    .on_hover_text("Integrate steepest ascent path along +grad f from probe");
                ui.checkbox(&mut state.mv_show_3d_surface, "3D Surface Mesh z=f(x,y)")
                    .on_hover_text("Toggle rotatable 3D perspective surface mesh alongside 2D contours");
            });
            if state.mv_show_3d_surface {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Slider::new(&mut state.mv_rot_yaw, -3.14..=3.14).text("3D Yaw"));
                    ui.add(Slider::new(&mut state.mv_rot_pitch, -1.45..=1.45).text("3D Pitch"));
                    ui.add(Slider::new(&mut state.mv_zoom_3d, 0.55..=2.2).text("3D Zoom"));
                    if ui.small_button("Reset 3D View").clicked() {
                        state.mv_rot_yaw = 0.65;
                        state.mv_rot_pitch = 0.48;
                        state.mv_zoom_3d = 1.0;
                    }
                });
            }
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

    // Sample scalar field for auto-scaled contours & 3D color ramp
    let grid_n = 44usize;
    let min_c = -3.0_f64;
    let max_c = 3.0_f64;
    let step = (max_c - min_c) / (grid_n as f64);
    let mut z_grid = vec![vec![0.0_f64; grid_n + 1]; grid_n + 1];
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for i in 0..=grid_n {
        let x = min_c + (i as f64) * step;
        for j in 0..=grid_n {
            let y = min_c + (j as f64) * step;
            let zv = sf.eval_2d(x, y);
            z_grid[i][j] = zv;
            if zv.is_finite() {
                z_min = z_min.min(zv);
                z_max = z_max.max(zv);
            }
        }
    }
    if !z_min.is_finite() || !z_max.is_finite() {
        z_min = -1.0;
        z_max = 1.0;
    }
    let levels = nice_contour_levels(z_min, z_max, 11);

    if state.mv_show_contours && !levels.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Contour Color Ramp:").small().strong());
            for (idx, &t) in [-1.0, -0.5, 0.0, 0.5, 1.0].iter().enumerate() {
                let c = diverging_perceptual_color(t);
                let (r_sw, _) = ui.allocate_exact_size(egui::vec2(14.0, 12.0), egui::Sense::hover());
                ui.painter().rect_filled(r_sw, Rounding::same(2.0), c);
                if idx == 0 {
                    ui.label(RichText::new(format!("min {}", format_contour_label(z_min))).small().monospace());
                } else if idx == 2 && z_min < 0.0 && z_max > 0.0 {
                    ui.label(RichText::new("0").small().monospace());
                } else if idx == 4 {
                    ui.label(RichText::new(format!("max {}", format_contour_label(z_max))).small().monospace());
                }
            }
            ui.label(RichText::new("(Click 2D plot to move probe)").small().weak());
        });
    }

    if state.mv_show_3d_surface {
        ui.columns(2, |cols| {
            render_multivariable_2d_plot(
                &mut cols[0],
                state,
                &sf,
                &vp,
                &vq,
                z_min,
                z_max,
                &levels,
                &z_grid,
                grid_n,
            );
            render_3d_surface_mesh(&mut cols[1], state, &sf, z_min, z_max, theme);
        });
    } else {
        render_multivariable_2d_plot(
            ui,
            state,
            &sf,
            &vp,
            &vq,
            z_min,
            z_max,
            &levels,
            &z_grid,
            grid_n,
        );
    }

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

pub(crate) fn compute_adaptive_quiver_grid(bounds: [f64; 4], grid_n: usize) -> (Vec<[f64; 2]>, f64) {
    let [mut x_min, mut x_max, mut y_min, mut y_max] = bounds;
    if !x_min.is_finite()
        || !x_max.is_finite()
        || !y_min.is_finite()
        || !y_max.is_finite()
        || (x_max - x_min) <= 1e-6
        || (y_max - y_min) <= 1e-6
    {
        x_min = -3.5;
        x_max = 3.5;
        y_min = -3.5;
        y_max = 3.5;
    }
    let n = grid_n.clamp(5, 30);
    let dx = (x_max - x_min) / (n as f64);
    let dy = (y_max - y_min) / (n as f64);
    let cell = dx.min(dy);
    let mut pts = Vec::with_capacity(n * n);
    for ix in 0..n {
        let x = x_min + (ix as f64 + 0.5) * dx;
        for iy in 0..n {
            let y = y_min + (iy as f64 + 0.5) * dy;
            pts.push([x, y]);
        }
    }
    (pts, cell)
}

pub(crate) fn direction_arrow_geometry(
    x: f64,
    y: f64,
    ux: f64,
    uy: f64,
    cell_size: f64,
) -> ([f64; 2], [f64; 2], [[f64; 2]; 3]) {
    let len = (0.52 * cell_size).max(1e-6);
    let tail = [x - 0.35 * len * ux, y - 0.35 * len * uy];
    let tip = [x + 0.65 * len * ux, y + 0.65 * len * uy];
    let head = 0.30 * len;
    let wing1 = [
        tip[0] - head * (ux * 0.85 - uy * 0.52),
        tip[1] - head * (uy * 0.85 + ux * 0.52),
    ];
    let wing2 = [
        tip[0] - head * (ux * 0.85 + uy * 0.52),
        tip[1] - head * (uy * 0.85 - ux * 0.52),
    ];
    (tail, tip, [wing1, tip, wing2])
}

pub(crate) fn format_ode_curves_csv(rk4_timed: &[[f64; 3]], euler_timed: &[[f64; 3]]) -> String {
    let mut csv = String::new();
    if euler_timed.is_empty() {
        csv.push_str("t,x_rk4,y_rk4\n");
        for pt in rk4_timed {
            let _ = writeln!(csv, "{:.8},{:.8},{:.8}", pt[0], pt[1], pt[2]);
        }
    } else {
        csv.push_str("t,x_rk4,y_rk4,x_euler,y_euler\n");
        let n = rk4_timed.len().max(euler_timed.len());
        for idx in 0..n {
            let t_val = rk4_timed
                .get(idx)
                .map(|p| p[0])
                .or_else(|| euler_timed.get(idx).map(|p| p[0]))
                .unwrap_or(0.0);
            let rk4_cols = rk4_timed
                .get(idx)
                .map(|p| format!("{:.8},{:.8}", p[1], p[2]))
                .unwrap_or_else(|| ",".to_string());
            let euler_cols = euler_timed
                .get(idx)
                .map(|p| format!("{:.8},{:.8}", p[1], p[2]))
                .unwrap_or_else(|| ",".to_string());
            let _ = writeln!(csv, "{:.8},{},{}", t_val, rk4_cols, euler_cols);
        }
    }
    csv
}

pub(crate) fn format_pde_curves_csv(snap: &PdeSnapshot) -> String {
    let mut csv = String::from("x,u_initial,u_m_t0,u_evolved\n");
    for (idx, pt) in snap.initial_curve.iter().enumerate() {
        let u_rec = snap
            .reconstructed_t0_curve
            .get(idx)
            .map(|p| p[1])
            .unwrap_or(f64::NAN);
        let u_ev = snap
            .evolved_curve
            .get(idx)
            .map(|p| p[1])
            .unwrap_or(f64::NAN);
        let _ = writeln!(csv, "{:.8},{:.8},{:.8},{:.8}", pt[0], pt[1], u_rec, u_ev);
    }
    csv
}

fn save_text_via_dialog(
    default_name: &str,
    filter_label: &str,
    ext: &str,
    content: &str,
) -> (bool, String) {
    if let Some(out_path) = rfd::FileDialog::new()
        .set_file_name(default_name)
        .add_filter(filter_label, &[ext])
        .save_file()
    {
        match std::fs::write(&out_path, content) {
            Ok(()) => (false, format!("Exported {} to {}", ext.to_uppercase(), out_path.display())),
            Err(e) => (true, format!("{} export write error: {}", ext.to_uppercase(), e)),
        }
    } else {
        (false, format!("{} export cancelled.", ext.to_uppercase()))
    }
}

fn save_csv_via_dialog(default_name: &str, csv_content: &str) -> (bool, String) {
    save_text_via_dialog(default_name, "CSV Data", "csv", csv_content)
}

impl LabState {
    pub fn run_current_ode_benchmark(&mut self) -> Result<(), String> {
        let p_expr = Expr::parse(&self.ode_rhs_p)?;
        let q_expr = Expr::parse(&self.ode_rhs_q)?;
        let label = format!("dx/dt = {}, dy/dt = {}", self.ode_rhs_p, self.ode_rhs_q);
        let bench = benchmark_ode_integrator_convergence(
            &p_expr,
            &q_expr,
            &label,
            self.ode_seed_x,
            self.ode_seed_y,
            2.0,
        );
        self.ode_convergence_bench = Some(bench);
        Ok(())
    }

    pub fn export_ode_benchmark_dialog(&mut self, is_csv: bool) {
        if self.ode_convergence_bench.is_none() {
            if let Err(e) = self.run_current_ode_benchmark() {
                self.export_status_msg = Some((true, format!("Benchmark failed: {e}")));
                return;
            }
        }
        let Some(bench) = &self.ode_convergence_bench else {
            return;
        };
        let res = if is_csv {
            save_text_via_dialog(
                "calclab_ode_convergence.csv",
                "CSV Table",
                "csv",
                &bench.to_csv(),
            )
        } else {
            match bench.to_json_pretty() {
                Ok(json) => save_text_via_dialog(
                    "calclab_ode_convergence.json",
                    "JSON Benchmark",
                    "json",
                    &json,
                ),
                Err(e) => (true, format!("JSON serialize error: {e}")),
            }
        };
        self.export_status_msg = Some(res);
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
                if ui
                    .small_button("Damped Oscillator")
                    .on_hover_text("Load damped harmonic oscillator: x' = y, y' = -2x - 0.35y")
                    .clicked()
                {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "-2.0*x - 0.35*y".to_string();
                    state.ode_seed_x = 2.2;
                    state.ode_seed_y = 0.0;
                }
                if ui
                    .small_button("Nonlinear Pendulum")
                    .on_hover_text("Load damped nonlinear pendulum: x' = y, y' = -2 sin(x) - 0.15y")
                    .clicked()
                {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "-2.0*sin(x) - 0.15*y".to_string();
                    state.ode_seed_x = 2.4;
                    state.ode_seed_y = 0.2;
                }
                if ui
                    .small_button("Lotka-Volterra")
                    .on_hover_text("Load predator-prey Lotka-Volterra system with center at (1.5, 1.5)")
                    .clicked()
                {
                    state.ode_rhs_p = "x * (1.2 - 0.8*y)".to_string();
                    state.ode_rhs_q = "y * (-0.9 + 0.6*x)".to_string();
                    state.ode_seed_x = 1.8;
                    state.ode_seed_y = 0.8;
                }
                if ui
                    .small_button("Van der Pol Cycle")
                    .on_hover_text("Load Van der Pol nonlinear limit-cycle oscillator (mu = 1.1)")
                    .clicked()
                {
                    state.ode_rhs_p = "y".to_string();
                    state.ode_rhs_q = "1.1*(1 - x^2)*y - x".to_string();
                    state.ode_seed_x = 0.35;
                    state.ode_seed_y = 0.25;
                }
                if ui
                    .small_button("Saddle Portrait")
                    .on_hover_text("Load hyperbolic saddle portrait with stable and unstable manifolds")
                    .clicked()
                {
                    state.ode_rhs_p = "x + 0.5*y".to_string();
                    state.ode_rhs_q = "0.5*x - y".to_string();
                    state.ode_seed_x = 0.4;
                    state.ode_seed_y = 2.2;
                }
                if ui
                    .small_button("Reset View")
                    .on_hover_text("Reset phase-plane viewport bounds back to [-3.5, 3.5]^2")
                    .clicked()
                {
                    state.ode_view_bounds = [-3.5, 3.5, -3.5, 3.5];
                    state.ode_reset_view = true;
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("dx/dt = P(x, y) =");
                ui.add(
                    TextEdit::singleline(&mut state.ode_rhs_p)
                        .desired_width(180.0)
                        .font(TextStyle::Monospace),
                )
                .on_hover_text("Autonomous x'(t) = P(x, y) expression in variables x and y");
                ui.label("dy/dt = Q(x, y) =");
                ui.add(
                    TextEdit::singleline(&mut state.ode_rhs_q)
                        .desired_width(180.0)
                        .font(TextStyle::Monospace),
                )
                .on_hover_text("Autonomous y'(t) = Q(x, y) expression in variables x and y");
                if let (Ok(pe), Ok(qe)) = (Expr::parse(&state.ode_rhs_p), Expr::parse(&state.ode_rhs_q))
                {
                    let ode_tex = format!(
                        r"\frac{{dx}}{{dt}} = {},\quad \frac{{dy}}{{dt}} = {}",
                        pe.to_latex(),
                        qe.to_latex()
                    );
                    math_render::copy_latex_button(
                        ui,
                        "lab_ode_system_latex",
                        &ode_tex,
                        "Copy planar ODE system as LaTeX",
                    );
                }
                ui.checkbox(&mut state.ode_show_ensemble, "Show Background Portraits")
                    .on_hover_text("Plot 8 surrounding RK4 phase trajectories across the plane");
                ui.checkbox(&mut state.ode_show_nullclines, "Nullclines")
                    .on_hover_text("Draw zero-level contours where dx/dt = 0 and dy/dt = 0");
                ui.checkbox(&mut state.ode_show_equilibria, "Equilibria")
                    .on_hover_text("Locate fixed points (x*, y*) via damped Newton-Raphson and classify Jacobians");
                ui.checkbox(&mut state.ode_show_euler, "Compare Euler")
                    .on_hover_text("Overlay 1st-order Explicit Euler trajectory against 4th-order RK4");
                ui.checkbox(&mut state.ode_show_midpoint, "Compare Midpoint (RK2)")
                    .on_hover_text("Overlay 2nd-order Midpoint (RK2) trajectory");
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.ode_seed_x, -3.2..=3.2).text("Initial x(0)"))
                    .on_hover_text("Initial x-coordinate x(0) for forward/backward trajectory integration");
                ui.add(Slider::new(&mut state.ode_seed_y, -3.2..=3.2).text("Initial y(0)"))
                    .on_hover_text("Initial y-coordinate y(0) for forward/backward trajectory integration");
                ui.add(Slider::new(&mut state.ode_dt, 0.01..=0.25).text("Step dt"))
                    .on_hover_text("Fixed integrator step size dt (smaller dt reduces truncation error)");
                ui.add(Slider::new(&mut state.ode_t_max, 2.0..=20.0).text("T_max"))
                    .on_hover_text("Total forward integration horizon T_max");
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
    let p_unknown = p_expr.unknown_vars(&["x", "y"]);
    if !p_unknown.is_empty() {
        ui.label(
            RichText::new(format!(
                "dx/dt unknown variable(s) {}; express P(x, y) in terms of x and y.",
                p_unknown.join(", ")
            ))
            .color(theme.err_color()),
        );
        return;
    }
    let q_unknown = q_expr.unknown_vars(&["x", "y"]);
    if !q_unknown.is_empty() {
        ui.label(
            RichText::new(format!(
                "dy/dt unknown variable(s) {}; express Q(x, y) in terms of x and y.",
                q_unknown.join(", ")
            ))
            .color(theme.err_color()),
        );
        return;
    }

    let dt = state.ode_dt.clamp(0.01, 0.25);
    let t_max = state.ode_t_max.clamp(2.0, 20.0);
    let steps = ((t_max / dt).round() as usize).clamp(1, 2000);
    let bwd_steps = ((t_max * 0.4 / dt).round() as usize).clamp(1, 800);
    let [bx_min, bx_max, by_min, by_max] = state.ode_view_bounds;
    let integ_bound = (bx_min
        .abs()
        .max(bx_max.abs())
        .max(by_min.abs())
        .max(by_max.abs())
        * 1.4)
        .clamp(4.8, 120.0);

    let rk4_timed = solve_planar_system_rk4_timed(
        &p_expr,
        &q_expr,
        state.ode_seed_x,
        state.ode_seed_y,
        dt,
        steps,
        integ_bound,
    );
    let euler_timed = if state.ode_show_euler {
        solve_planar_system_euler_timed(
            &p_expr,
            &q_expr,
            state.ode_seed_x,
            state.ode_seed_y,
            dt,
            steps,
            integ_bound,
        )
    } else {
        Vec::new()
    };
    let midpoint_timed = if state.ode_show_midpoint {
        solve_planar_system_midpoint_timed(
            &p_expr,
            &q_expr,
            state.ode_seed_x,
            state.ode_seed_y,
            dt,
            steps,
            integ_bound,
        )
    } else {
        Vec::new()
    };
    let bwd = solve_planar_system_rk4(
        &p_expr,
        &q_expr,
        state.ode_seed_x,
        state.ode_seed_y,
        -dt,
        bwd_steps,
        integ_bound,
    );
    let p_nullcline = if state.ode_show_nullclines {
        extract_zero_level_segments(&p_expr, bx_min, bx_max, by_min, by_max, 56)
    } else {
        Vec::new()
    };
    let q_nullcline = if state.ode_show_nullclines {
        extract_zero_level_segments(&q_expr, bx_min, bx_max, by_min, by_max, 56)
    } else {
        Vec::new()
    };
    let equilibria: Vec<EquilibriumPoint> = if state.ode_show_equilibria {
        find_planar_equilibria(&p_expr, &q_expr, bx_min, bx_max, by_min, by_max)
    } else {
        Vec::new()
    };

    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Export ODE Curves CSV...")
            .on_hover_text("Export RK4 (and Euler if enabled) trajectory curves t, x(t), y(t) to CSV")
            .clicked()
        {
            let csv = format_ode_curves_csv(&rk4_timed, &euler_timed);
            state.export_status_msg = Some(save_csv_via_dialog("calclab_ode_curves.csv", &csv));
        }
        if ui
            .button("Run Convergence Benchmark (Euler vs RK2 vs RK4)")
            .on_hover_text("Compare step-halving empirical convergence orders across Euler, Midpoint (RK2), and RK4")
            .clicked()
        {
            match state.run_current_ode_benchmark() {
                Ok(()) => {
                    state.export_status_msg = Some((
                        false,
                        "Computed step-halving convergence orders for Euler, Midpoint (RK2), and RK4.".to_string(),
                    ));
                }
                Err(e) => {
                    state.export_status_msg = Some((true, format!("Benchmark error: {e}")));
                }
            }
        }
        if ui
            .button("Export Benchmark JSON...")
            .on_hover_text("Export ODE step-halving convergence benchmark to JSON")
            .clicked()
        {
            state.export_ode_benchmark_dialog(false);
        }
        if ui
            .button("Export Benchmark CSV...")
            .on_hover_text("Export ODE step-halving convergence benchmark to CSV")
            .clicked()
        {
            state.export_ode_benchmark_dialog(true);
        }
        if let Some((is_err, msg)) = &state.export_status_msg {
            let col = if *is_err {
                theme.err_color()
            } else {
                theme.ok_color()
            };
            ui.label(RichText::new(msg).small().monospace().color(col));
        }
    });

    ui.columns(2, |cols| {
        let side = cols[0].available_width().clamp(300.0, 440.0);

        let mut phase_plot = Plot::new("ode_phase_portrait_plot")
            .width(side)
            .height(side)
            .data_aspect(1.0)
            .view_aspect(1.0)
            .include_x(-3.5)
            .include_x(3.5)
            .include_y(-3.5)
            .include_y(3.5)
            .x_axis_label("x")
            .y_axis_label("y")
            .legend(Legend::default());
        if state.ode_reset_view {
            phase_plot = phase_plot.reset();
            state.ode_reset_view = false;
        }
        phase_plot.show(&mut cols[0], |plot_ui| {
                let b = plot_ui.plot_bounds();
                if b.is_valid() {
                    let bmin = b.min();
                    let bmax = b.max();
                    if bmin[0].is_finite()
                        && bmax[0].is_finite()
                        && bmin[1].is_finite()
                        && bmax[1].is_finite()
                        && (bmax[0] - bmin[0]) > 1e-4
                        && (bmax[1] - bmin[1]) > 1e-4
                    {
                        state.ode_view_bounds = [bmin[0], bmax[0], bmin[1], bmax[1]];
                    }
                }

                if plot_ui.response().clicked() {
                    if let Some(pt) = plot_ui.pointer_coordinate() {
                        state.ode_seed_x = pt.x.clamp(-3.2, 3.2);
                        state.ode_seed_y = pt.y.clamp(-3.2, 3.2);
                    }
                }

                // 1. Adaptive zoom/pan direction field quiver with directional arrowheads
                let (grid_pts, cell_size) = compute_adaptive_quiver_grid(state.ode_view_bounds, 15);
                let mut first_q = true;
                for [x, y] in grid_pts {
                    let u = p_expr.eval_2d(x, y);
                    let v = q_expr.eval_2d(x, y);
                    let norm = u.hypot(v);
                    if norm.is_finite() && norm > 1e-8 {
                        let ux = u / norm;
                        let uy = v / norm;
                        let (tail, tip, head_pts) =
                            direction_arrow_geometry(x, y, ux, uy, cell_size);
                        let arrow_col = Color32::from_rgba_unmultiplied(125, 155, 192, 155);
                        let mut shaft = Line::new(vec![tail, tip])
                            .color(arrow_col)
                            .width(1.15_f32);
                        if first_q {
                            shaft = shaft.name("Slope / Direction Field");
                            first_q = false;
                        }
                        plot_ui.line(shaft);
                        plot_ui.line(
                            Line::new(head_pts.to_vec())
                                .color(arrow_col)
                                .width(1.15_f32),
                        );
                    }
                }

                // 2. Nullclines: dx/dt = 0 and dy/dt = 0
                if state.ode_show_nullclines {
                    let mut first_pn = true;
                    for &(a, b) in &p_nullcline {
                        let mut line = Line::new(vec![a, b])
                            .color(Color32::from_rgb(76, 196, 208))
                            .width(1.6_f32)
                            .style(LineStyle::dashed_dense());
                        if first_pn {
                            line = line.name("dx/dt = 0 Nullcline");
                            first_pn = false;
                        }
                        plot_ui.line(line);
                    }

                    let mut first_qn = true;
                    for &(a, b) in &q_nullcline {
                        let mut line = Line::new(vec![a, b])
                            .color(Color32::from_rgb(198, 126, 214))
                            .width(1.6_f32)
                            .style(LineStyle::dashed_dense());
                        if first_qn {
                            line = line.name("dy/dt = 0 Nullcline");
                            first_qn = false;
                        }
                        plot_ui.line(line);
                    }
                }

                // 3. Background ensemble RK4 orbits
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
                        let bg_traj =
                            solve_planar_system_rk4(&p_expr, &q_expr, sx, sy, 0.025, 450, 4.5);
                        if bg_traj.len() >= 2 {
                            let mut line = Line::new(bg_traj)
                                .color(Color32::from_rgba_unmultiplied(92, 180, 196, 120))
                                .width(1.3_f32);
                            if first_bg {
                                line = line.name("Ensemble RK4 Orbits");
                                first_bg = false;
                            }
                            plot_ui.line(line);
                        }
                    }
                }

                // 4. Explicit Euler & Midpoint (RK2) comparison trajectories
                if state.ode_show_euler && euler_timed.len() >= 2 {
                    let euler_xy: Vec<[f64; 2]> =
                        euler_timed.iter().map(|pt| [pt[1], pt[2]]).collect();
                    plot_ui.line(
                        Line::new(euler_xy)
                            .color(Color32::from_rgb(224, 98, 98))
                            .width(1.8_f32)
                            .style(LineStyle::dashed_loose())
                            .name("Explicit Euler (t >= 0)"),
                    );
                }
                if state.ode_show_midpoint && midpoint_timed.len() >= 2 {
                    let mid_xy: Vec<[f64; 2]> =
                        midpoint_timed.iter().map(|pt| [pt[1], pt[2]]).collect();
                    plot_ui.line(
                        Line::new(mid_xy)
                            .color(Color32::from_rgb(168, 85, 247))
                            .width(1.9_f32)
                            .style(LineStyle::dotted_dense())
                            .name("Midpoint RK2 (t >= 0)"),
                    );
                }

                // 5. Backward & Forward RK4 trajectories
                if bwd.len() >= 2 {
                    plot_ui.line(
                        Line::new(bwd)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(1.8_f32)
                            .name("Backward RK4 (t < 0)"),
                    );
                }
                if rk4_timed.len() >= 2 {
                    let rk4_xy: Vec<[f64; 2]> =
                        rk4_timed.iter().map(|pt| [pt[1], pt[2]]).collect();
                    plot_ui.line(
                        Line::new(rk4_xy)
                            .color(Color32::from_rgb(232, 168, 72))
                            .width(2.6_f32)
                            .name("Forward RK4 (t >= 0)"),
                    );
                }

                // 6. Equilibrium points (x*, y*)
                if state.ode_show_equilibria && !equilibria.is_empty() {
                    let eq_pts: Vec<[f64; 2]> = equilibria.iter().map(|e| [e.x, e.y]).collect();
                    plot_ui.points(
                        Points::new(eq_pts)
                            .shape(MarkerShape::Diamond)
                            .radius(6.5_f32)
                            .color(Color32::from_rgb(92, 198, 148))
                            .name("Equilibria (x*, y*)"),
                    );
                }

                // 7. Initial seed point (x(0), y(0))
                plot_ui.points(
                    Points::new(vec![[state.ode_seed_x, state.ode_seed_y]])
                        .radius(6.0_f32)
                        .color(Color32::from_rgb(232, 168, 72))
                        .name("Initial State (x(0), y(0))"),
                );
            });

        // Right Column: Paired x(t) and y(t) Time-Series (RK4 vs Explicit Euler)
        let half_h = (side - 8.0) * 0.5;
        let rk4_xt: Vec<[f64; 2]> = rk4_timed.iter().map(|pt| [pt[0], pt[1]]).collect();
        let rk4_yt: Vec<[f64; 2]> = rk4_timed.iter().map(|pt| [pt[0], pt[2]]).collect();
        let euler_xt: Vec<[f64; 2]> = euler_timed.iter().map(|pt| [pt[0], pt[1]]).collect();
        let euler_yt: Vec<[f64; 2]> = euler_timed.iter().map(|pt| [pt[0], pt[2]]).collect();

        Plot::new("ode_xt_plot")
            .height(half_h)
            .include_x(0.0)
            .include_x(t_max)
            .x_axis_label("t")
            .y_axis_label("x(t)")
            .legend(Legend::default())
            .show(&mut cols[1], |plot_ui| {
                if euler_xt.len() >= 2 {
                    plot_ui.line(
                        Line::new(euler_xt)
                            .color(Color32::from_rgb(224, 98, 98))
                            .width(1.8_f32)
                            .style(LineStyle::dashed_loose())
                            .name("x_Euler(t)"),
                    );
                }
                if rk4_xt.len() >= 2 {
                    plot_ui.line(
                        Line::new(rk4_xt)
                            .color(Color32::from_rgb(232, 168, 72))
                            .width(2.2_f32)
                            .name("x_RK4(t)"),
                    );
                }
            });

        cols[1].add_space(8.0);

        Plot::new("ode_yt_plot")
            .height(half_h)
            .include_x(0.0)
            .include_x(t_max)
            .x_axis_label("t")
            .y_axis_label("y(t)")
            .legend(Legend::default())
            .show(&mut cols[1], |plot_ui| {
                if euler_yt.len() >= 2 {
                    plot_ui.line(
                        Line::new(euler_yt)
                            .color(Color32::from_rgb(224, 98, 98))
                            .width(1.8_f32)
                            .style(LineStyle::dashed_loose())
                            .name("y_Euler(t)"),
                    );
                }
                if rk4_yt.len() >= 2 {
                    plot_ui.line(
                        Line::new(rk4_yt)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(2.2_f32)
                            .name("y_RK4(t)"),
                    );
                }
            });
    });

    // Symbolic Jacobian Linearization at (x(0), y(0)) + RK4 vs Euler + Equilibria Telemetry
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
            if state.ode_show_euler {
                if let (Some(r_last), Some(e_last)) = (rk4_timed.last(), euler_timed.last()) {
                    let sep = (r_last[1] - e_last[1]).hypot(r_last[2] - e_last[2]);
                    ui.label(
                        RichText::new(format!(
                            "RK4 vs. Explicit Euler (dt = {:.3}, T = {:.1}):  RK4({:.2}) = ({:.4}, {:.4})  |  Euler({:.2}) = ({:.4}, {:.4})  |  ||Delta X(T)||_2 = {:.4}",
                            dt,
                            t_max,
                            r_last[0],
                            r_last[1],
                            r_last[2],
                            e_last[0],
                            e_last[1],
                            e_last[2],
                            sep
                        ))
                        .monospace(),
                    );
                }
            }
            if state.ode_show_equilibria {
                if equilibria.is_empty() {
                    ui.label(
                        RichText::new(format!(
                            "Detected Equilibria in [{:.1}, {:.1}] x [{:.1}, {:.1}]: None (no P(x*, y*) = Q(x*, y*) = 0 roots found in window)",
                            bx_min, bx_max, by_min, by_max
                        ))
                        .monospace()
                        .weak(),
                    );
                } else {
                    ui.label(
                        RichText::new(format!(
                            "Detected Equilibria in [{:.1}, {:.1}] x [{:.1}, {:.1}] ({} found):",
                            bx_min,
                            bx_max,
                            by_min,
                            by_max,
                            equilibria.len()
                        ))
                        .strong()
                        .small(),
                    );
                    for (idx, eq) in equilibria.iter().enumerate() {
                        ui.label(
                            RichText::new(format!(
                                "  E{}: (x*, y*) = ({:.4}, {:.4})  |  tr(J) = {:.4}, det(J) = {:.4}, Delta = {:.4}  |  {}",
                                idx + 1,
                                eq.x,
                                eq.y,
                                eq.diagnostics.trace,
                                eq.diagnostics.det,
                                eq.diagnostics.discriminant,
                                eq.diagnostics.phase_portrait_class
                            ))
                            .monospace(),
                        );
                    }
                }
            }
            if let Some(bench) = &state.ode_convergence_bench {
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Step-Halving Convergence Comparator (T={:.1}) — Mean Empirical Orders: Euler={:.3}, Midpoint(RK2)={:.3}, RK4={:.3}",
                        bench.t_end,
                        bench.mean_order_euler,
                        bench.mean_order_midpoint_rk2,
                        bench.mean_order_rk4
                    ))
                    .strong()
                    .small(),
                );
                egui::Grid::new("ode_convergence_bench_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Method");
                        ui.strong("Theory p");
                        ui.strong("dt");
                        ui.strong("Steps");
                        ui.strong("Abs Error");
                        ui.strong("Empirical Order");
                        ui.end_row();
                        for r in &bench.rows {
                            ui.label(&r.method);
                            ui.label(format!("O(dt^{})", r.theoretical_order));
                            ui.label(format!("{:.4}", r.dt));
                            ui.label(format!("{}", r.steps));
                            ui.label(format!("{:.3e}", r.abs_error));
                            ui.label(
                                r.empirical_order
                                    .map(|o| format!("{o:.3}"))
                                    .unwrap_or_else(|| "—".to_string()),
                            );
                            ui.end_row();
                        }
                    });
            }
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
                    })
                    .response
                    .on_hover_text("Select 1D Heat Diffusion, Standing Wave, or Periodic Advection-Diffusion PDE");

                ui.separator();
                ui.selectable_value(&mut state.pde_use_custom, false, "Preset Profile")
                    .on_hover_text("Use one of the built-in canonical initial profiles u(x, 0)");
                ui.selectable_value(&mut state.pde_use_custom, true, "Custom Expr u(x, 0)")
                    .on_hover_text("Project an arbitrary expression u(x, 0) onto M Fourier modes via quadrature");
                ui.checkbox(&mut state.pde_show_truncation_t0, "Show M-Mode t=0 Sum")
                    .on_hover_text("Overlay the truncated M-mode Fourier reconstruction at t = 0 to inspect Gibbs ringing");

                let pde_tex = match (state.pde_kind, state.pde_use_custom) {
                    (PdeKind::HeatDiffusion, false) => r"\frac{\partial u}{\partial t} = \alpha \frac{\partial^{2} u}{\partial x^{2}}",
                    (PdeKind::WaveEquation, false) => r"\frac{\partial^{2} u}{\partial t^{2}} = c^{2} \frac{\partial^{2} u}{\partial x^{2}}",
                    (PdeKind::AdvectionDiffusion, false) => r"\frac{\partial u}{\partial t} + v \frac{\partial u}{\partial x} = \alpha \frac{\partial^{2} u}{\partial x^{2}}",
                    _ => "",
                };
                if state.pde_use_custom {
                    if let Ok(ue) = Expr::parse(&state.pde_custom_expr) {
                        let ctex = format!("u(x, 0) = {}", ue.to_latex());
                        math_render::copy_latex_button(
                            ui,
                            "lab_pde_custom_latex",
                            &ctex,
                            "Copy custom u(x, 0) initial condition as LaTeX",
                        );
                    }
                } else {
                    math_render::copy_latex_button(
                        ui,
                        "lab_pde_eq_latex",
                        pde_tex,
                        "Copy governing PDE equation as LaTeX",
                    );
                }
            });

            ui.horizontal_wrapped(|ui| {
                if !state.pde_use_custom {
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
                        })
                        .response
                        .on_hover_text("Select initial condition profile u(x, 0) across [0, L]");
                } else {
                    ui.label("u(x, 0) =");
                    ui.add(
                        TextEdit::singleline(&mut state.pde_custom_expr)
                            .desired_width(250.0)
                            .font(TextStyle::Monospace),
                    )
                    .on_hover_text("Custom initial profile u(x, 0) in terms of x over [0, L]");
                    ui.label(
                        RichText::new("(Fourier quadrature over x in [0, L])")
                            .small()
                            .weak(),
                    );
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.pde_time, 0.0..=8.0).text("Time t"))
                    .on_hover_text("Scrub simulation time t (cycles 0..8 when animated)");
                ui.checkbox(&mut state.pde_animate, "Animate Evolution")
                    .on_hover_text("Play or pause time-stepping animation (Shortcut: Space)");
                if ui
                    .small_button("Reset t = 0")
                    .on_hover_text("Pause animation and reset time t back to 0.0")
                    .clicked()
                {
                    state.pde_time = 0.0;
                    state.pde_animate = false;
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.add(Slider::new(&mut state.pde_alpha, 0.0..=0.5).text("Diffusivity alpha"))
                    .on_hover_text("Thermal/viscous diffusivity alpha (governs exp(-alpha * n^2 * t) modal damping)");
                ui.add(Slider::new(&mut state.pde_speed, 0.2..=2.5).text("Speed c / v"))
                    .on_hover_text("Wave propagation speed c (Wave) or drift velocity v (Advection-Diffusion)");
                ui.add(Slider::new(&mut state.pde_modes, 1..=35).text("Fourier Modes M"))
                    .on_hover_text("Number of truncated Fourier harmonics M included in the spectral Galerkin expansion");
            });
        });

    let snap_res: Result<PdeSnapshot, String> = if state.pde_use_custom {
        evaluate_1d_pde_custom(
            state.pde_kind,
            &state.pde_custom_expr,
            state.pde_alpha,
            state.pde_speed,
            state.pde_time,
            state.pde_modes,
            240,
        )
    } else {
        Ok(evaluate_1d_pde(
            state.pde_kind,
            state.pde_profile,
            state.pde_alpha,
            state.pde_speed,
            state.pde_time,
            state.pde_modes,
            240,
        ))
    };

    let snap = match snap_res {
        Ok(s) => s,
        Err(msg) => {
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("Custom u(x, 0) error: {}", msg))
                    .color(theme.err_color())
                    .monospace(),
            );
            return;
        }
    };

    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Export PDE Curves CSV...")
            .on_hover_text("Export x, u(x,0), u_M(x,0), and u(x,t) curves to a CSV file")
            .clicked()
        {
            let csv = format_pde_curves_csv(&snap);
            state.export_status_msg = Some(save_csv_via_dialog("calclab_pde_curves.csv", &csv));
        }
        if let Some((is_err, msg)) = &state.export_status_msg {
            let col = if *is_err {
                theme.err_color()
            } else {
                theme.ok_color()
            };
            ui.label(RichText::new(msg).small().monospace().color(col));
        }
    });

    let x_axis_lbl = match state.pde_kind {
        PdeKind::AdvectionDiffusion => "Position x in [0, 2*pi]",
        _ => "Position x in [0, pi]",
    };

    ui.columns(2, |cols| {
        Plot::new("pde_1d_evolution_plot")
            .height(360.0)
            .include_y(-1.3)
            .include_y(1.3)
            .x_axis_label(x_axis_lbl)
            .y_axis_label("Amplitude u(x, t)")
            .legend(Legend::default())
            .show(&mut cols[0], |plot_ui| {
                plot_ui.line(
                    Line::new(snap.initial_curve.clone())
                        .color(Color32::from_rgba_unmultiplied(145, 160, 185, 145))
                        .width(1.6_f32)
                        .name("Initial Profile u(x, 0)"),
                );
                if state.pde_show_truncation_t0 {
                    plot_ui.line(
                        Line::new(snap.reconstructed_t0_curve.clone())
                            .color(Color32::from_rgb(76, 196, 208))
                            .width(1.8_f32)
                            .style(LineStyle::dashed_dense())
                            .name("M-Mode Truncation u_M(x, 0)"),
                    );
                }
                plot_ui.line(
                    Line::new(snap.evolved_curve.clone())
                        .color(Color32::from_rgb(230, 122, 108))
                        .width(2.6_f32)
                        .name("Evolved Solution u(x, t)"),
                );
            });

        // Right Plot: Fourier Mode Spectrum Bar Chart
        let mut init_bars = Vec::with_capacity(snap.fourier_coeffs.len() + 1);
        let mut evolved_bars = Vec::with_capacity(snap.mode_energies.len() + 1);
        if matches!(state.pde_kind, PdeKind::AdvectionDiffusion) {
            let a0_abs = snap.a0.abs();
            init_bars.push(
                Bar::new(0.0, a0_abs)
                    .width(0.66)
                    .fill(Color32::from_rgba_unmultiplied(88, 166, 236, 110)),
            );
            evolved_bars.push(
                Bar::new(0.0, a0_abs)
                    .width(0.40)
                    .fill(Color32::from_rgb(230, 122, 108)),
            );
        }
        for (&(n, an, bn), &(_, amp_t)) in
            snap.fourier_coeffs.iter().zip(snap.mode_energies.iter())
        {
            let amp_0 = an.hypot(bn);
            init_bars.push(
                Bar::new(n as f64, amp_0)
                    .width(0.66)
                    .fill(Color32::from_rgba_unmultiplied(88, 166, 236, 110)),
            );
            evolved_bars.push(
                Bar::new(n as f64, amp_t)
                    .width(0.40)
                    .fill(Color32::from_rgb(230, 122, 108)),
            );
        }

        Plot::new("pde_fourier_spectrum_plot")
            .height(360.0)
            .include_y(0.0)
            .x_axis_label("Fourier Mode n")
            .y_axis_label("Modal Amplitude |c_n|")
            .legend(Legend::default())
            .show(&mut cols[1], |plot_ui| {
                plot_ui.bar_chart(BarChart::new(init_bars).name("Initial |c_n(0)|"));
                plot_ui.bar_chart(BarChart::new(evolved_bars).name("Active |c_n(t)|"));
            });
    });

    let top_modes: Vec<String> = snap
        .mode_energies
        .iter()
        .take(5)
        .map(|(n, amp)| format!("|c_{}|={:.3}", n, amp))
        .collect();
    let coeff_str: Vec<String> = snap
        .fourier_coeffs
        .iter()
        .take(5)
        .map(|(n, an, bn)| match state.pde_kind {
            PdeKind::AdvectionDiffusion => format!("n={}: (a={:.3}, b={:.3})", n, an, bn),
            _ => format!("b_{}={:.4}", n, bn),
        })
        .collect();

    ui.add_space(6.0);
    Frame::none()
        .fill(theme.card_fill())
        .stroke(theme.card_stroke())
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            let energy_label = match state.pde_kind {
                PdeKind::WaveEquation => "Total Mechanical Energy Ratio E(t) / E(0)",
                _ => "L^2 Energy Retention ||u(.,t)||^2 / ||u(.,0)||^2",
            };
            ui.label(
                RichText::new(format!(
                    "Domain [0, {:.2}]  |  t = {:.3}  |  {} = {:.4}  |  t=0 L^2 Truncation Error = {:.4}",
                    snap.domain_len,
                    state.pde_time,
                    energy_label,
                    snap.l2_energy_ratio,
                    snap.l2_truncation_error
                ))
                .monospace(),
            );
            let mean_prefix = if matches!(state.pde_kind, PdeKind::AdvectionDiffusion) {
                format!("a_0 = {:.4}  |  ", snap.a0)
            } else {
                String::new()
            };
            ui.label(
                RichText::new(format!(
                    "Fourier Quadrature (M = {}):  {}{}",
                    state.pde_modes,
                    mean_prefix,
                    coeff_str.join("   ")
                ))
                .monospace(),
            );
            ui.label(
                RichText::new(format!(
                    "Leading Evolved Harmonics (M = {}):  {}",
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
                    if ui
                        .add_enabled(!state.stats_points.is_empty(), egui::Button::new("Undo Last").small())
                        .on_hover_text("Remove the most recently added (x, y) observation")
                        .clicked()
                    {
                        state.stats_points.pop();
                    }
                    if ui
                        .add_enabled(!state.stats_points.is_empty(), egui::Button::new("Clear Points").small())
                        .on_hover_text("Clear all scatter plot points to start a fresh regression dataset")
                        .clicked()
                    {
                        state.stats_points.clear();
                    }
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
                                    1.2_f32,
                                    Color32::from_rgb(154, 208, 96),
                                ))
                                .name("P(a <= X <= b)"),
                        );
                    }

                    plot_ui.line(
                        Line::new(curve)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(2.4_f32)
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
                                    1.2_f32,
                                    Color32::from_rgb(154, 208, 96),
                                ))
                                .name("P(a <= T <= b)"),
                        );
                    }

                    plot_ui.line(
                        Line::new(curve)
                            .color(Color32::from_rgb(88, 166, 236))
                            .width(2.4_f32)
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
                                .radius(5.5_f32)
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
                                .width(2.5_f32)
                                .name("OLS Fit y_hat = b0 + b1*x"),
                            );

                            // Draw vertical least-squares residual segments
                            for pt in &state.stats_points {
                                let yhat = fit.intercept + fit.slope * pt[0];
                                plot_ui.line(
                                    Line::new(vec![*pt, [pt[0], yhat]])
                                        .color(Color32::from_rgba_unmultiplied(232, 168, 72, 145))
                                        .width(1.2_f32),
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
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "OLS Telemetry (n = {}): Add at least 3 finite points with distinct x-coordinates to fit y_hat = b0 + b1*x (df = n - 2 >= 1; click inside plot or Reset Sample Points).",
                                state.stats_points.len()
                            ))
                            .monospace()
                            .color(theme.warn_color()),
                        );
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tan_readout_handles_asymptotes_at_half_pi() {
        let theta = PI / 2.0;
        assert_eq!(
            format_tan_readout(theta.sin(), theta.cos()),
            "undefined (asymptote)"
        );
        assert_eq!(format_tan_readout(1.0, 0.0), "undefined (asymptote)");
        assert_eq!(format_tan_readout(0.0, 1.0), "0.0000");
        assert_eq!(format_tan_readout((PI / 4.0).sin(), (PI / 4.0).cos()), "1.0000");
    }

    #[test]
    fn diverging_ramp_and_auto_contour_levels_scale_cleanly() {
        let low = diverging_perceptual_color(-1.0);
        let mid = diverging_perceptual_color(0.0);
        let high = diverging_perceptual_color(1.0);
        assert!(low.b() > low.r());
        assert!(high.r() > high.b());
        assert!((mid.r() as i16 - mid.b() as i16).abs() < 30);

        let levels = nice_contour_levels(-9.0, 9.0, 11);
        assert!(levels.len() >= 5);
        assert!(levels.iter().any(|&v| v.abs() < 1e-12));
        assert!(nice_contour_levels(2.0, 2.0, 11).is_empty());

        let (p_pt, d_front) = project_3d_surface_point(
            0.0,
            -2.0,
            1.0,
            0.0,
            0.3,
            1.0,
            egui::pos2(200.0, 200.0),
            40.0,
        );
        let (_, d_back) = project_3d_surface_point(
            0.0,
            2.0,
            1.0,
            0.0,
            0.3,
            1.0,
            egui::pos2(200.0, 200.0),
            40.0,
        );
        assert!(p_pt.x.is_finite() && p_pt.y.is_finite());
        assert!(d_front < d_back);
    }

    #[test]
    fn ode_adaptive_quiver_and_csv_exporters_format_cleanly() {
        let (pts, cell) = compute_adaptive_quiver_grid([-1.0, 1.0, -2.0, 2.0], 10);
        assert_eq!(pts.len(), 100);
        assert!((cell - 0.2).abs() < 1e-12);
        let (tail, tip, wings) = direction_arrow_geometry(0.0, 0.0, 1.0, 0.0, cell);
        assert!(tip[0] > tail[0]);
        assert!(wings[0][0] < tip[0] && wings[2][0] < tip[0]);
        assert!(wings[0][1] < tip[1] && wings[2][1] > tip[1]);

        let rk4 = vec![[0.0, 1.0, 0.0], [0.1, 0.99, -0.1]];
        let euler = vec![[0.0, 1.0, 0.0]];
        let csv_no_euler = format_ode_curves_csv(&rk4, &[]);
        assert!(csv_no_euler.starts_with("t,x_rk4,y_rk4\n"));
        assert_eq!(csv_no_euler.lines().count(), 3);

        let csv_with_euler = format_ode_curves_csv(&rk4, &euler);
        assert!(csv_with_euler.starts_with("t,x_rk4,y_rk4,x_euler,y_euler\n"));
        assert_eq!(csv_with_euler.lines().count(), 3);
        assert!(csv_with_euler.lines().last().unwrap().ends_with(",,"));

        let snap = evaluate_1d_pde(
            PdeKind::WaveEquation,
            PdeInitialProfile::TwoModeHarmonic,
            0.0,
            1.0,
            0.5,
            8,
            64,
        );
        let pde_csv = format_pde_curves_csv(&snap);
        assert!(pde_csv.starts_with("x,u_initial,u_m_t0,u_evolved\n"));
        assert_eq!(pde_csv.lines().count(), snap.initial_curve.len() + 1);
    }

    #[test]
    fn lab_state_runs_ode_convergence_benchmark_for_default_system() {
        let mut st = LabState::default();
        st.run_current_ode_benchmark().expect("run benchmark");
        let bench = st.ode_convergence_bench.as_ref().expect("bench populated");
        assert_eq!(bench.rows.len(), 12);
        assert!(bench.mean_order_rk4 > bench.mean_order_midpoint_rk2);
        assert!(bench.mean_order_midpoint_rk2 > bench.mean_order_euler);
    }
}

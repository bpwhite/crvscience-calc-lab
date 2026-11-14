use calc_core::{
    build_curriculum, build_notation_lexicon, compute_bayes_binary, evaluate_1d_pde, fit_ols_linear,
    generate_notation_flash_question, solve_scalar_ode_rk4, EvalEnv, Expr, Mat2, ModuleMastery,
    PdeInitialProfile, PdeKind, ProblemGenerator, ProfileStore, ScienceDomain, Stage,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static SUMMARY_LOCK: Mutex<()> = Mutex::new(());

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir")
        .parent()
        .expect("calc-lab repo root")
        .to_path_buf()
}

fn validation_dir() -> PathBuf {
    let dir = repo_root().join("validation");
    fs::create_dir_all(&dir).expect("create validation directory");
    dir
}

fn write_summary_section(section_key: &str, section_value: Value) {
    let _guard = SUMMARY_LOCK.lock().expect("lock summary mutex");
    let local_path = validation_dir().join("calc_lab_validation_summary.json");

    let mut root = if local_path.exists() {
        fs::read_to_string(&local_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };

    if !root.is_object() {
        root = json!({});
    }
    let obj = root.as_object_mut().expect("root JSON object");
    obj.insert("schema_version".into(), json!("1.0"));
    obj.insert("repo".into(), json!("crvscience-calc-lab"));
    obj.insert("version".into(), json!("0.5.0"));
    obj.insert("preprint_id".into(), json!("APP-PREPRINT-03"));
    obj.insert("concept_doi".into(), json!("10.5281/zenodo.24222372"));
    obj.insert(
        "preprint_url".into(),
        json!("https://crvscience.com/apps/preprints/calc-lab-scientific-calculus-ode"),
    );

    let tests_entry = obj
        .entry("tests".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !tests_entry.is_object() {
        *tests_entry = Value::Object(Map::new());
    }
    tests_entry
        .as_object_mut()
        .expect("tests JSON map")
        .insert(section_key.to_string(), section_value);

    let serialized = serde_json::to_string_pretty(&root).expect("serialize calc-lab summary");
    fs::write(&local_path, format!("{serialized}\n")).expect("write local summary JSON");

    if let Some(hub_root) = repo_root()
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    {
        let hub_releases = hub_root.join("app-preprints").join("releases");
        if hub_releases.is_dir() {
            let hub_path =
                hub_releases.join("calc-lab-scientific-calculus-ode-validation-telemetry.json");
            fs::write(&hub_path, format!("{serialized}\n"))
                .expect("write hub calc-lab telemetry copy");
        }
    }
}

#[test]
fn t1_48_modules_6_domains_verification_sweep() {
    let catalog = build_curriculum();
    assert_eq!(
        catalog.modules.len(),
        48,
        "Curriculum catalog must expose all 48 interactive modules"
    );
    assert_eq!(Stage::ALL.len(), 8);
    assert_eq!(ScienceDomain::ALL.len(), 6);

    let mut stage_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut domain_verified: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_problem_checks = 0usize;
    let mut total_step_count = 0usize;

    for (m_idx, spec) in catalog.modules.iter().enumerate() {
        *stage_counts
            .entry(format!("{:?}", spec.stage))
            .or_insert(0) += 1;

        for (d_idx, &domain) in ScienceDomain::ALL.iter().enumerate() {
            let seed = 10_000_u64 + (m_idx as u64) * 97 + (d_idx as u64) * 13;
            let prob = ProblemGenerator::generate_with_domain(spec.id, seed, domain);

            assert!(
                !prob.prompt.trim().is_empty(),
                "Empty prompt for module '{}' domain '{:?}'",
                spec.id,
                domain
            );
            assert!(
                !prob.steps.is_empty(),
                "Missing derivation steps for module '{}' domain '{:?}'",
                spec.id,
                domain
            );

            let canonical = prob.answer.canonical();
            assert!(
                !canonical.trim().is_empty(),
                "Empty canonical answer for module '{}'",
                spec.id
            );

            let outcome = prob.grade(&canonical);
            assert!(
                outcome.correct,
                "Canonical self-grading failed for module '{}' domain '{:?}' canonical='{}': {}",
                spec.id,
                domain,
                canonical,
                outcome.feedback
            );

            let wrong_outcome = prob.grade("999999.12345 + 0.0001");
            assert!(
                !wrong_outcome.correct,
                "Wrong answer unexpectedly accepted for module '{}'",
                spec.id
            );

            *domain_verified
                .entry(domain.label().to_string())
                .or_insert(0) += 1;
            total_problem_checks += 1;
            total_step_count += prob.steps.len();
        }
    }

    assert_eq!(total_problem_checks, 288);
    assert_eq!(stage_counts.len(), 8);
    assert!(stage_counts.values().all(|&c| c == 6));

    // Verify Notation Lexicon & Flashcard Quiz
    let lexicon = build_notation_lexicon();
    assert!(!lexicon.is_empty());
    let flash = generate_notation_flash_question(42);
    assert!(flash.correct_index < 4 && !flash.question.is_empty());

    // Verify Linear Algebra 2x2 Spectral Diagnostics
    let diag = Mat2::new(-1.5, 0.8, -0.4, -0.9).diagnostics();
    assert_eq!(diag.rank, 2);
    assert!((diag.trace - (-2.4)).abs() < 1e-12);
    assert!((diag.det - (1.35 + 0.32)).abs() < 1e-12);
    assert!(diag.inverse.is_some());

    // Verify Probability & Statistics (Bayesian Diagnostic + OLS Regression)
    let bayes = compute_bayes_binary(0.02, 0.94, 0.96);
    assert!(bayes.posterior_given_pos > bayes.prior);
    let ols = fit_ols_linear(&[[1.0, 3.1], [2.0, 5.2], [3.0, 6.9], [4.0, 9.1], [5.0, 11.0]])
        .expect("OLS linear fit");
    assert!((ols.slope - 1.97).abs() < 0.05 && ols.r_squared > 0.99);

    // Verify SQLite ProfileStore In-Memory Mastery Persistence Roundtrip
    let store = ProfileStore::open_in_memory().expect("open in-memory ProfileStore");
    let target_mod_id = catalog
        .modules
        .iter()
        .find(|m| m.id.contains("rk4") || m.id.contains("ode"))
        .map(|m| m.id)
        .unwrap_or(catalog.modules[0].id);
    let mut m = ModuleMastery::new(target_mod_id);
    m.mastery_score = 0.92;
    m.streak = 5;
    m.attempts = 6;
    m.correct_count = 6;
    store.save_mastery(&m).expect("save module mastery");
    let loaded = store.load_all_mastery(&catalog).expect("load mastery map");
    assert_eq!(loaded.len(), 48);
    assert!((loaded[target_mod_id].mastery_score - 0.92).abs() < 1e-9);

    write_summary_section(
        "t1_curriculum_domain_sweep",
        json!({
            "module_count": catalog.modules.len(),
            "stage_count": Stage::ALL.len(),
            "domain_count": ScienceDomain::ALL.len(),
            "verified_module_domain_pairs": total_problem_checks,
            "mean_steps_per_problem": (total_step_count as f64) / (total_problem_checks as f64),
            "notation_lexicon_entries": lexicon.len(),
            "mat2_trace": diag.trace,
            "mat2_det": diag.det,
            "mat2_rank": diag.rank,
            "mat2_phase_class": diag.phase_portrait_class,
            "bayes_posterior_given_pos": bayes.posterior_given_pos,
            "ols_slope": ols.slope,
            "ols_r_squared": ols.r_squared,
            "sqlite_mastery_records": loaded.len()
        }),
    );
}

fn logistic_exact(t: f64, y0: f64, r: f64) -> f64 {
    1.0 / (1.0 + ((1.0 - y0) / y0) * (-r * t).exp())
}

fn solve_scalar_ode_euler(rhs: &Expr, t0: f64, y0: f64, dt: f64, steps: usize) -> f64 {
    let mut t = t0;
    let mut y = y0;
    let mut env = EvalEnv::new();
    for _ in 0..steps {
        env.insert("t".to_string(), t);
        env.insert("y".to_string(), y);
        let slope = rhs.eval(&env);
        y += dt * slope;
        t += dt;
    }
    y
}

#[test]
fn t2_rk4_vs_euler_step_halving_convergence() {
    let r = 1.2_f64;
    let y0 = 0.1_f64;
    let t_end = 2.0_f64;
    let y_exact = logistic_exact(t_end, y0, r);
    let rhs = Expr::parse("1.2 * y * (1 - y)").expect("parse nonlinear logistic ODE RHS");

    let grids = [
        (0.4_f64, 5usize),
        (0.2_f64, 10usize),
        (0.1_f64, 20usize),
        (0.05_f64, 40usize),
        (0.025_f64, 80usize),
        (0.0125_f64, 160usize),
    ];

    let mut csv = String::from(
        "dt,steps,t_end,y_exact,y_rk4,rk4_abs_error,rk4_empirical_order,y_euler,euler_abs_error,euler_empirical_order\n",
    );

    let mut prev_rk4_err: Option<f64> = None;
    let mut prev_euler_err: Option<f64> = None;
    let mut rk4_orders = Vec::new();
    let mut euler_orders = Vec::new();
    let mut rows_json = Vec::new();

    for &(dt, steps) in &grids {
        let traj = solve_scalar_ode_rk4(&rhs, 0.0, y0, dt, steps);
        let last_pt = *traj.last().expect("RK4 terminal point");
        assert!((last_pt[0] - t_end).abs() < 1e-12);

        let y_rk4 = last_pt[1];
        let rk4_err = (y_rk4 - y_exact).abs();
        let y_euler = solve_scalar_ode_euler(&rhs, 0.0, y0, dt, steps);
        let euler_err = (y_euler - y_exact).abs();

        let rk4_order = prev_rk4_err.map(|prev| (prev / rk4_err).log2());
        let euler_order = prev_euler_err.map(|prev| (prev / euler_err).log2());

        if let Some(o) = rk4_order {
            rk4_orders.push(o);
        }
        if let Some(o) = euler_order {
            euler_orders.push(o);
        }

        let rk4_order_str = rk4_order
            .map(|o| format!("{o:.4}"))
            .unwrap_or_else(|| "NA".to_string());
        let euler_order_str = euler_order
            .map(|o| format!("{o:.4}"))
            .unwrap_or_else(|| "NA".to_string());

        let _ = writeln!(
            csv,
            "{dt:.4},{steps},{t_end:.1},{y_exact:.12},{y_rk4:.12},{rk4_err:.6e},{rk4_order_str},{y_euler:.12},{euler_err:.6e},{euler_order_str}"
        );

        rows_json.push(json!({
            "dt": dt,
            "steps": steps,
            "y_exact": y_exact,
            "y_rk4": y_rk4,
            "rk4_abs_error": rk4_err,
            "rk4_empirical_order": rk4_order,
            "y_euler": y_euler,
            "euler_abs_error": euler_err,
            "euler_empirical_order": euler_order
        }));

        prev_rk4_err = Some(rk4_err);
        prev_euler_err = Some(euler_err);
    }

    let finest_rk4_err = prev_rk4_err.expect("finest RK4 error");
    let finest_euler_err = prev_euler_err.expect("finest Euler error");
    let mean_rk4_order = rk4_orders.iter().sum::<f64>() / (rk4_orders.len() as f64);
    let mean_euler_order = euler_orders.iter().sum::<f64>() / (euler_orders.len() as f64);
    let final_rk4_order = *rk4_orders.last().expect("final RK4 order");
    let final_euler_order = *euler_orders.last().expect("final Euler order");

    assert!(
        (3.85..=4.15).contains(&final_rk4_order) && (3.85..=4.15).contains(&mean_rk4_order),
        "RK4 must demonstrate 4th-order convergence O(h^4) (mean={mean_rk4_order:.4}, final={final_rk4_order:.4})"
    );
    assert!(
        (0.90..=1.15).contains(&final_euler_order) && (0.85..=1.15).contains(&mean_euler_order),
        "Explicit Euler must demonstrate 1st-order convergence O(h^1) (mean={mean_euler_order:.4}, final={final_euler_order:.4})"
    );
    assert!(
        finest_rk4_err < 1e-8 && finest_rk4_err * 1000.0 < finest_euler_err,
        "At dt=0.0125 RK4 error ({finest_rk4_err:.3e}) must beat Euler ({finest_euler_err:.3e}) by >3 orders of magnitude"
    );

    let csv_path = validation_dir().join("rk4_convergence_table.csv");
    fs::write(&csv_path, &csv).expect("write rk4_convergence_table.csv");

    write_summary_section(
        "t2_rk4_vs_euler_convergence",
        json!({
            "ode": "dy/dt = 1.2 * y * (1 - y)",
            "initial_condition_y0": y0,
            "t_end": t_end,
            "y_exact_t_end": y_exact,
            "mean_rk4_order": mean_rk4_order,
            "final_rk4_order": final_rk4_order,
            "finest_rk4_abs_error": finest_rk4_err,
            "mean_euler_order": mean_euler_order,
            "final_euler_order": final_euler_order,
            "finest_euler_abs_error": finest_euler_err,
            "csv_artifact": "validation/rk4_convergence_table.csv",
            "refinement_rows": rows_json
        }),
    );
}

fn integrate_trapezoid(curve: &[[f64; 2]]) -> f64 {
    curve
        .windows(2)
        .map(|w| 0.5 * (w[0][1] + w[1][1]) * (w[1][0] - w[0][0]))
        .sum()
}

#[test]
fn t3_1d_heat_pde_l2_decay_and_modal_stability() {
    let alpha = 0.45_f64;
    let time_grid = [0.0_f64, 0.05, 0.15, 0.35, 0.70, 1.20];
    let mut prev_l2 = f64::INFINITY;
    let mut max_l2_rel_err = 0.0_f64;
    let mut max_boundary_err = 0.0_f64;
    let mut snapshots_json = Vec::new();

    for &t in &time_grid {
        let snap = evaluate_1d_pde(
            PdeKind::HeatDiffusion,
            PdeInitialProfile::TwoModeHarmonic,
            alpha,
            1.0,
            t,
            12,
            129,
        );
        assert!((snap.domain_len - PI).abs() < 1e-12);
        assert_eq!(snap.evolved_curve.len(), 130);
        assert!(snap
            .evolved_curve
            .iter()
            .all(|pt| pt[0].is_finite() && pt[1].is_finite()));

        // Verify Dirichlet boundary conditions u(0,t) = 0, u(pi,t) = 0
        let left_bc = snap.evolved_curve.first().unwrap()[1].abs();
        let right_bc = snap.evolved_curve.last().unwrap()[1].abs();
        max_boundary_err = max_boundary_err.max(left_bc).max(right_bc);
        assert!(left_bc < 1e-12 && right_bc < 1e-12);

        // Exact analytical TwoModeHarmonic L2 energy ratio: (e^{-2*alpha*t} + 0.55^2 * e^{-18*alpha*t}) / (1 + 0.55^2)
        let exact_l2_ratio =
            ((-2.0 * alpha * t).exp() + 0.55 * 0.55 * (-18.0 * alpha * t).exp())
                / (1.0 + 0.55 * 0.55);
        let l2_err = (snap.l2_energy_ratio - exact_l2_ratio).abs();
        if l2_err > max_l2_rel_err {
            max_l2_rel_err = l2_err;
        }
        assert!(
            l2_err < 1e-9,
            "Heat PDE L2 energy ratio mismatch at t={t}: got {}, exact {}, err={l2_err:.3e}",
            snap.l2_energy_ratio,
            exact_l2_ratio
        );
        assert!(
            snap.l2_energy_ratio <= prev_l2 + 1e-12,
            "Heat PDE L2 energy must decay monotonically"
        );
        prev_l2 = snap.l2_energy_ratio;

        // Verify exact mode 1 vs mode 3 exponential amplitude attenuation (|b_n(t)| = |b_n(0)| * exp(-alpha * n^2 * t))
        let mode1_e = snap.mode_energies[0].1;
        let mode3_e = snap.mode_energies[2].1;
        let exact_m1 = (-alpha * 1.0 * t).exp();
        let exact_m3 = 0.55 * (-alpha * 9.0 * t).exp();
        assert!((mode1_e - exact_m1).abs() < 1e-9);
        assert!((mode3_e - exact_m3).abs() < 1e-9);

        let max_u = snap
            .evolved_curve
            .iter()
            .map(|p| p[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let spatial_mass = integrate_trapezoid(&snap.evolved_curve);

        snapshots_json.push(json!({
            "time_t": t,
            "l2_energy_ratio": snap.l2_energy_ratio,
            "exact_l2_energy_ratio": exact_l2_ratio,
            "l2_abs_error": l2_err,
            "mode1_energy": mode1_e,
            "mode3_energy": mode3_e,
            "peak_amplitude": max_u,
            "spatial_integral": spatial_mass
        }));
    }

    // Verify maximum principle and smooth parabolic dissipation on LocalizedGaussian & SquarePulse
    let gauss_t0 = evaluate_1d_pde(
        PdeKind::HeatDiffusion,
        PdeInitialProfile::LocalizedGaussian,
        alpha,
        1.0,
        0.0,
        24,
        129,
    );
    let gauss_t1 = evaluate_1d_pde(
        PdeKind::HeatDiffusion,
        PdeInitialProfile::LocalizedGaussian,
        alpha,
        1.0,
        0.4,
        24,
        129,
    );
    let gauss_peak_0 = gauss_t0
        .evolved_curve
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let gauss_peak_1 = gauss_t1
        .evolved_curve
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max);
    let gauss_mass_0 = integrate_trapezoid(&gauss_t0.evolved_curve);
    let gauss_mass_1 = integrate_trapezoid(&gauss_t1.evolved_curve);
    assert!(gauss_peak_1 < gauss_peak_0);
    assert!(gauss_mass_1 > 0.0 && gauss_mass_1 <= gauss_mass_0 + 1e-9);
    assert!(gauss_t1.l2_energy_ratio < 1.0);

    let square_t1 = evaluate_1d_pde(
        PdeKind::HeatDiffusion,
        PdeInitialProfile::SquarePulse,
        alpha,
        1.0,
        0.4,
        24,
        129,
    );
    assert!(square_t1.l2_energy_ratio > 0.0 && square_t1.l2_energy_ratio < 1.0);

    write_summary_section(
        "t3_1d_heat_pde_stability",
        json!({
            "alpha": alpha,
            "domain_len": PI,
            "num_modes": 12,
            "spatial_samples": 129,
            "max_l2_energy_error": max_l2_rel_err,
            "max_dirichlet_boundary_error": max_boundary_err,
            "gaussian_peak_t0": gauss_peak_0,
            "gaussian_peak_t0_4": gauss_peak_1,
            "gaussian_mass_t0": gauss_mass_0,
            "gaussian_mass_t0_4": gauss_mass_1,
            "gaussian_l2_ratio_t0_4": gauss_t1.l2_energy_ratio,
            "square_pulse_l2_ratio_t0_4": square_t1.l2_energy_ratio,
            "two_mode_snapshots": snapshots_json
        }),
    );
}

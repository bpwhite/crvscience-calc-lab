mod app;
mod theme;
mod ui_analytics;
mod ui_lab;
mod ui_lesson;
mod ui_menu;
mod ui_notation;
mod ui_practice;
mod ui_roadmap;

use app::CalcLabApp;
use calc_core::{
    build_curriculum, build_notation_lexicon, check_scalar_equivalence, compute_bayes_binary,
    evaluate_1d_pde, fit_ols_linear, generate_notation_flash_question, normal_cdf, score_attempt,
    solve_planar_system_rk4, Expr, LearningRunRecord, Mat2, PdeInitialProfile, PdeKind,
    ProblemGenerator, ProfileStore, RunMode, ScienceDomain,
};
use chrono::Utc;

fn run_headless_self_test() -> Result<(), String> {
    println!("Running CalcLab v0.5.0 headless verification suite...");
    let catalog = build_curriculum();
    if catalog.modules.len() != 48 {
        return Err(format!(
            "Expected 48 curriculum modules across 8 stages, found {}",
            catalog.modules.len()
        ));
    }

    // 1. Notation Lexicon & Flash Recognition Verification
    let lexicon = build_notation_lexicon();
    if lexicon.len() < 14 {
        return Err("Notation lexicon entry count below expected minimum.".to_string());
    }
    for s in 0..8_u64 {
        let fq = generate_notation_flash_question(s);
        if fq.correct_index >= 4 || fq.choices[fq.correct_index].is_empty() {
            return Err("Invalid notation flash question choice index.".to_string());
        }
    }

    // 2. Symbolic Differentiation Verification
    let cases = [
        ("x^4 - 3*x^2 + 7", "4*x^3 - 6*x"),
        ("exp(2*x) * cos(3*x)", "2*exp(2*x)*cos(3*x) - 3*exp(2*x)*sin(3*x)"),
        ("ln(1 + x^2)", "(2*x)/(1 + x^2)"),
        ("atan(2*x)", "2/(1 + 4*x^2)"),
    ];
    for (src, expected_deriv) in cases {
        let parsed = Expr::parse(src)?;
        let deriv_str = parsed.differentiate("x").to_plain_string();
        let ok = check_scalar_equivalence(&deriv_str, expected_deriv, &["x"])?;
        if !ok {
            return Err(format!(
                "Derivative mismatch on {}: got {}, expected {}",
                src, deriv_str, expected_deriv
            ));
        }
    }

    // 3. Linear Algebra, ODE/PDE & Probability/Statistics Engines
    let mat = Mat2::new(2.0, 1.0, 1.0, 2.0);
    let diag = mat.diagnostics();
    if (diag.trace - 4.0).abs() > 1e-9 || (diag.det - 3.0).abs() > 1e-9 {
        return Err("Mat2 trace/determinant check failed.".to_string());
    }

    let p = Expr::parse("y")?;
    let q = Expr::parse("-x")?;
    let orbit = solve_planar_system_rk4(&p, &q, 1.0, 0.0, 0.01, 200, 5.0);
    if orbit.len() != 201 {
        return Err("RK4 planar harmonic orbit step count mismatch.".to_string());
    }

    let heat = evaluate_1d_pde(
        PdeKind::HeatDiffusion,
        PdeInitialProfile::TwoModeHarmonic,
        0.2,
        1.0,
        0.5,
        15,
        100,
    );
    if !(heat.l2_energy_ratio > 0.0 && heat.l2_energy_ratio < 1.0) {
        return Err("1D Heat PDE modal dissipation check failed.".to_string());
    }

    if (normal_cdf(0.0, 0.0, 1.0) - 0.5).abs() > 1e-6 {
        return Err("Normal CDF symmetry check failed.".to_string());
    }
    let bayes = compute_bayes_binary(0.05, 0.95, 0.95);
    if !(bayes.posterior_given_pos > 0.49 && bayes.posterior_given_pos < 0.51) {
        return Err("Bayesian binary update check failed.".to_string());
    }
    let ols = fit_ols_linear(&[[0.0, 1.0], [1.0, 3.0], [2.0, 5.0]])
        .ok_or_else(|| "OLS fit returned None".to_string())?;
    if (ols.slope - 2.0).abs() > 1e-9 || (ols.r_squared - 1.0).abs() > 1e-9 {
        return Err("OLS exact collinear test failed.".to_string());
    }

    // 4. Procedural Generator, Named Method Steps & Domain Framing across 48 modules x 6 domains (288 problems)
    let mut verified_problems = 0usize;
    for spec in &catalog.modules {
        for (d_idx, &dom) in ScienceDomain::ALL.iter().enumerate() {
            let prob = ProblemGenerator::generate_with_domain(
                spec.id,
                (d_idx as u64 + 1) * 37,
                dom,
            );
            if prob.steps.is_empty() {
                return Err(format!("No SolutionSteps generated for {}", spec.id));
            }
            let canon = prob.answer.canonical();
            let res = prob.grade(&canon);
            if !res.correct {
                return Err(format!(
                    "Generator self-check failed for {} ({:?}): canonical '{}' -> {}",
                    spec.id, dom, canon, res.feedback
                ));
            }
            verified_problems += 1;
        }
    }

    // 5. Timed Scoring & SQLite Run History Persistence Roundtrip
    let delta = score_attempt(RunMode::QuickBlitz90s, 3, true, 3, 6.0, 0);
    if delta.points_awarded <= 0 || delta.new_streak != 4 {
        return Err("Run scoring multiplier check failed.".to_string());
    }

    let store = ProfileStore::open_in_memory()?;
    store.append_learning_run(&LearningRunRecord {
        id: 0,
        finished_at_utc: Utc::now(),
        mode: RunMode::QuickBlitz90s.short_code().to_string(),
        domain: ScienceDomain::Microbiology.label().to_string(),
        stage_scope: "All 8 Stages".to_string(),
        score: 1850,
        solved_count: 8,
        attempted_count: 9,
        max_streak: 6,
        elapsed_secs: 90.0,
    })?;
    let loaded_runs = store.load_learning_runs(10)?;
    if loaded_runs.len() != 1 || loaded_runs[0].score != 1850 {
        return Err("SQLite learning_runs persistence check failed.".to_string());
    }

    println!(
        "[PASS] Verified Main Menu v0.5.0 build, 48 modules (8 stages), Notation Studio, 6 scientific domains, {} method-step problems, and SQLite run ledger.",
        verified_problems
    );
    Ok(())
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--self-test") {
        if let Err(err) = run_headless_self_test() {
            eprintln!("[FAIL] {}", err);
            std::process::exit(1);
        }
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CalcLab v0.5 — Main Menu | Notation -> Algebra -> Trig -> Calc -> LinAlg -> AdvCalc -> ODE/PDE -> Prob & Stats")
            .with_inner_size([1460.0, 910.0])
            .with_min_inner_size([1060.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CalcLab",
        native_options,
        Box::new(|cc| Ok(Box::new(CalcLabApp::new(cc)))),
    )
}

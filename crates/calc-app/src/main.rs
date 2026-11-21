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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod math_render;
mod theme;
mod ui_analytics;
mod ui_lab;
mod ui_lesson;
mod ui_menu;
mod ui_notation;
mod ui_practice;
mod ui_roadmap;

use app::{AppTab, CalcLabApp, NavCommand};
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

fn block_on_future<F: std::future::Future>(fut: F) -> F::Output {
    struct ThreadUnparker(std::thread::Thread);
    impl std::task::Wake for ThreadUnparker {
        fn wake(self: std::sync::Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = std::task::Waker::from(std::sync::Arc::new(ThreadUnparker(
        std::thread::current(),
    )));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => return val,
            std::task::Poll::Pending => std::thread::park(),
        }
    }
}

fn write_rgba_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let row_bytes = (width as usize) * 4;
    if rgba.len() != row_bytes * (height as usize) {
        return Err("RGBA buffer length mismatch".to_string());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }
    let mut encoded = Vec::with_capacity((width as usize) * (height as usize));
    {
        let mut encoder = png::Encoder::new(&mut encoded, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::High);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("Failed to encode PNG header: {e}"))?;
        writer
            .write_image_data(rgba)
            .map_err(|e| format!("Failed to write PNG image data: {e}"))?;
    }
    std::fs::write(path, &encoded).map_err(|e| format!("Failed to write PNG: {e}"))
}

fn calclab_window_icon() -> eframe::egui::IconData {
    const ICO_BYTES: &[u8] = include_bytes!("../assets/calclab.ico");
    // Parse ICO directory and decode the largest embedded PNG entry (256x256).
    if ICO_BYTES.len() >= 6 {
        let count = u16::from_le_bytes([ICO_BYTES[4], ICO_BYTES[5]]) as usize;
        let mut best_slice: Option<&[u8]> = None;
        let mut best_area = 0usize;
        for i in 0..count {
            let off = 6 + i * 16;
            if off + 16 > ICO_BYTES.len() {
                break;
            }
            let w = if ICO_BYTES[off] == 0 { 256 } else { ICO_BYTES[off] as usize };
            let h = if ICO_BYTES[off + 1] == 0 { 256 } else { ICO_BYTES[off + 1] as usize };
            let size = u32::from_le_bytes([
                ICO_BYTES[off + 8],
                ICO_BYTES[off + 9],
                ICO_BYTES[off + 10],
                ICO_BYTES[off + 11],
            ]) as usize;
            let img_off = u32::from_le_bytes([
                ICO_BYTES[off + 12],
                ICO_BYTES[off + 13],
                ICO_BYTES[off + 14],
                ICO_BYTES[off + 15],
            ]) as usize;
            if img_off + size <= ICO_BYTES.len() && ICO_BYTES[img_off..].starts_with(b"\x89PNG\r\n\x1a\n") {
                let area = w * h;
                if area >= best_area {
                    best_area = area;
                    best_slice = Some(&ICO_BYTES[img_off..img_off + size]);
                }
            }
        }
        if let Some(png_slice) = best_slice {
            let decoder = png::Decoder::new(std::io::Cursor::new(png_slice));
            if let Ok(mut reader) = decoder.read_info() {
                let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(256 * 256 * 4)];
                if let Ok(info) = reader.next_frame(&mut buf) {
                    buf.truncate(info.buffer_size());
                    if info.color_type == png::ColorType::Rgba && info.width > 0 && info.height > 0 {
                        return eframe::egui::IconData {
                            rgba: buf,
                            width: info.width,
                            height: info.height,
                        };
                    }
                }
            }
        }
    }
    // Fallback solid emblem if decoding ever fails
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for px in rgba.chunks_exact_mut(4) {
        px[0] = 20;
        px[1] = 30;
        px[2] = 51;
        px[3] = 255;
    }
    eframe::egui::IconData {
        rgba,
        width: 32,
        height: 32,
    }
}

fn cli_usage_text() -> String {
    format!(
        "CalcLab v{} — Mathematical Mastery & Scientific Calculus Workbench\n\
         Copyright CRV Science Network (https://crvscience.com) | License: GPL-3.0-or-later\n\n\
         USAGE:\n\
           calc-app [OPTIONS]\n\n\
         OPTIONS:\n\
           -h, --help                         Print this help message and exit\n\
           -V, --version                      Print version information and exit\n\
               --self-test                    Run headless curriculum, symbolic, ODE/PDE, and SQLite verification\n\
               --screenshot <PNG>             Render an offscreen 1600x1000 screenshot to <PNG> and exit\n\
               --screenshot-view <1|2|3>, --view <1|2|3>\n\
                                              Select screenshot view for --screenshot (1 = Practice, 2 = ODE Phase Lab, 3 = Notation Studio)",
        env!("CARGO_PKG_VERSION")
    )
}

fn print_cli_help() {
    println!("{}", cli_usage_text());
}

fn run_headless_screenshot(out_path: &std::path::Path, view: u8) -> Result<(), String> {
    use eframe::egui;
    use eframe::egui_wgpu::{self, wgpu};

    const WIDTH: u32 = 1600;
    const HEIGHT: u32 = 1000;
    const BYTES_PER_ROW: u32 = WIDTH * 4;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = block_on_future(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .or_else(|| {
        block_on_future(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: true,
        }))
    })
    .ok_or_else(|| "No wgpu adapter available for offscreen screenshot.".to_string())?;

    let (device, queue) =
        block_on_future(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .map_err(|e| format!("wgpu request_device failed: {e}"))?;

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

    let ctx = egui::Context::default();
    let mut app = CalcLabApp::new_with_mode(true);
    app.show_sidebar = true;
    match view {
        2 => {
            app.apply_nav_command(NavCommand::OpenModule {
                module_id: "deq-linear-systems-phase".to_string(),
                tab: AppTab::Lab,
            });
            app.apply_nav_command(NavCommand::OpenLabPreset(
                "deq-linear-systems-phase".to_string(),
            ));
        }
        3 => {
            app.notation_studio.selected_idx = 11; // Boundary Integrals & Curl/Flux Operators
            app.apply_nav_command(NavCommand::OpenTab(AppTab::Notation));
        }
        _ => {
            app.apply_nav_command(NavCommand::OpenModule {
                module_id: "calc-integration-tech".to_string(),
                tab: AppTab::Practice,
            });
            // Select the definite u-substitution integral variant to showcase tall integrals & fractions
            for s in 1..40u64 {
                let p = ProblemGenerator::generate_with_domain(
                    "calc-integration-tech",
                    s,
                    ScienceDomain::General,
                );
                if p.target_equation_latex.contains(r"\int_{0}^{1}") {
                    app.practice.active_problem = p;
                    break;
                }
            }
            app.practice.answer_input = app.practice.active_problem.answer.canonical();
            app.practice.last_outcome =
                Some(app.practice.active_problem.grade(&app.practice.answer_input));
            app.practice.last_points_delta = 120;
            app.practice.hints_revealed = 0;
            app.practice.step_cursor = app.practice.active_problem.steps.len();
        }
    }

    let extent = wgpu::Extent3d {
        width: WIDTH,
        height: HEIGHT,
        depth_or_array_layers: 1,
    };
    let render_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("calclab_headless_target"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let render_view = render_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let staging_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("calclab_headless_readback"),
        size: (BYTES_PER_ROW as u64) * (HEIGHT as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    for frame_idx in 0..4_u32 {
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::Vec2::new(WIDTH as f32, HEIGHT as f32),
            )),
            predicted_dt: 1.0 / 30.0,
            time: Some((frame_idx as f64) * (1.0 / 30.0)),
            ..Default::default()
        };

        let full_output = ctx.run(raw_input, |ctx| {
            app.render_ui(ctx);
        });
        let paint_jobs = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [WIDTH, HEIGHT],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in &full_output.textures_delta.set {
            renderer.update_texture(&device, &queue, *id, delta);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("calclab_headless_encoder"),
        });
        let user_cmds =
            renderer.update_buffers(&device, &queue, &mut encoder, &paint_jobs, &screen_descriptor);

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("calclab_headless_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 15.0 / 255.0,
                            g: 23.0 / 255.0,
                            b: 42.0 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut rpass = rpass.forget_lifetime();
            renderer.render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        if frame_idx == 3 {
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &render_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &staging_buf,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(BYTES_PER_ROW),
                        rows_per_image: Some(HEIGHT),
                    },
                },
                extent,
            );
        }

        queue.submit(user_cmds.into_iter().chain(std::iter::once(encoder.finish())));
        device.poll(wgpu::Maintain::Wait);

        for id in &full_output.textures_delta.free {
            renderer.free_texture(id);
        }
    }

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|e| format!("Readback channel failed: {e}"))?
        .map_err(|e| format!("Staging buffer map failed: {e}"))?;

    let rgba = slice.get_mapped_range().to_vec();
    let _ = slice;
    staging_buf.unmap();

    write_rgba_png(out_path, WIDTH, HEIGHT, &rgba)
}

#[derive(Debug, PartialEq, Eq)]
enum CliAction {
    Help,
    Version,
    SelfTest,
    Screenshot { path: String, view: u8 },
    Gui,
}

fn parse_cli_args(args: &[String]) -> Result<CliAction, String> {
    if args.len() <= 1 {
        return Ok(CliAction::Gui);
    }

    let mut help = false;
    let mut version = false;
    let mut self_test = false;
    let mut screenshot_path: Option<String> = None;
    let mut view: Option<u8> = None;

    let mut idx = 1usize;
    while idx < args.len() {
        let arg = args[idx].as_str();
        match arg {
            "--help" | "-h" => {
                if help {
                    return Err(format!("Duplicate option '{arg}'."));
                }
                help = true;
                idx += 1;
            }
            "--version" | "-V" => {
                if version {
                    return Err(format!("Duplicate option '{arg}'."));
                }
                version = true;
                idx += 1;
            }
            "--self-test" => {
                if self_test {
                    return Err("Duplicate option '--self-test'.".to_string());
                }
                self_test = true;
                idx += 1;
            }
            "--screenshot" => {
                if screenshot_path.is_some() {
                    return Err("Duplicate option '--screenshot'.".to_string());
                }
                let Some(raw) = args.get(idx + 1).map(|s| s.trim()).filter(|s| !s.is_empty()) else {
                    return Err("Missing output PNG path after '--screenshot'.".to_string());
                };
                if raw.starts_with('-') {
                    return Err(format!(
                        "Invalid output PNG path '{raw}' after '--screenshot' (expected a filename, not an option)."
                    ));
                }
                screenshot_path = Some(raw.to_string());
                idx += 2;
            }
            "--screenshot-view" | "--view" => {
                if view.is_some() {
                    return Err(format!("Duplicate screenshot view option '{arg}'."));
                }
                let Some(raw) = args.get(idx + 1).map(|s| s.trim()).filter(|s| !s.is_empty()) else {
                    return Err(format!(
                        "Missing view index after '{arg}' (expected 1, 2, or 3)."
                    ));
                };
                if raw.starts_with('-') {
                    return Err(format!(
                        "Invalid view index '{raw}' after '{arg}' (expected 1, 2, or 3)."
                    ));
                }
                let parsed = raw.parse::<u8>().ok().filter(|v| (1..=3).contains(v));
                let Some(v) = parsed else {
                    return Err(format!(
                        "Unsupported screenshot view '{raw}' after '{arg}' (expected 1, 2, or 3)."
                    ));
                };
                view = Some(v);
                idx += 2;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option '{arg}'."));
            }
            _ => {
                return Err(format!("Unexpected positional argument '{arg}'."));
            }
        }
    }

    if help {
        if version || self_test || screenshot_path.is_some() || view.is_some() {
            return Err("Option '--help' cannot be combined with other options.".to_string());
        }
        return Ok(CliAction::Help);
    }

    if version {
        if self_test || screenshot_path.is_some() || view.is_some() {
            return Err("Option '--version' cannot be combined with other options.".to_string());
        }
        return Ok(CliAction::Version);
    }

    if self_test {
        if screenshot_path.is_some() || view.is_some() {
            return Err(
                "Option '--self-test' cannot be combined with screenshot options.".to_string(),
            );
        }
        return Ok(CliAction::SelfTest);
    }

    match (screenshot_path, view) {
        (Some(path), v) => Ok(CliAction::Screenshot {
            path,
            view: v.unwrap_or(1),
        }),
        (None, Some(_)) => {
            Err("Option '--screenshot-view' / '--view' requires '--screenshot <PNG>'.".to_string())
        }
        (None, None) => Ok(CliAction::Gui),
    }
}

fn main() -> eframe::Result<()> {
    #[cfg(windows)]
    if std::env::args_os().len() > 1 {
        unsafe {
            windows_sys::Win32::System::Console::AttachConsole(
                windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
            );
        }
    }

    let args: Vec<String> = std::env::args().collect();
    let action = match parse_cli_args(&args) {
        Ok(action) => action,
        Err(msg) => {
            eprintln!("[CalcLab CLI Error] {}\n\n{}", msg, cli_usage_text());
            std::process::exit(2);
        }
    };

    match action {
        CliAction::Help => {
            print_cli_help();
            return Ok(());
        }
        CliAction::Version => {
            println!("calc-app {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        CliAction::SelfTest => {
            if let Err(err) = run_headless_self_test() {
                eprintln!("[FAIL] {}", err);
                std::process::exit(1);
            }
            return Ok(());
        }
        CliAction::Screenshot { path, view } => {
            let out_path = std::path::PathBuf::from(&path);
            if let Err(err) = run_headless_screenshot(&out_path, view) {
                eprintln!("[CalcLab Screenshot Error] {}", err);
                std::process::exit(1);
            }
            println!(
                "Saved {} (1600x1000 RGBA offscreen screenshot, view {view})",
                out_path.display()
            );
            return Ok(());
        }
        CliAction::Gui => {}
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CalcLab v0.5 — Main Menu | Notation -> Algebra -> Trig -> Calc -> LinAlg -> AdvCalc -> ODE/PDE -> Prob & Stats")
            .with_inner_size([1460.0, 910.0])
            .with_min_inner_size([1060.0, 680.0])
            .with_icon(std::sync::Arc::new(calclab_window_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "CalcLab",
        native_options,
        Box::new(|cc| Ok(Box::new(CalcLabApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_rgba_png_roundtrip_and_compression() {
        let w = 160u32;
        let h = 100u32;
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                rgba.push(((x * 255) / w) as u8);
                rgba.push(((y * 255) / h) as u8);
                rgba.push(64);
                rgba.push(255);
            }
        }
        let tmp = std::env::temp_dir().join("calclab_png_roundtrip_test.png");
        write_rgba_png(&tmp, w, h, &rgba).expect("write compressed PNG");
        let bytes = std::fs::read(&tmp).expect("read PNG back");
        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(
            bytes.len() < rgba.len() / 2,
            "Expected DEFLATE compression well under raw size, got {} vs {}",
            bytes.len(),
            rgba.len()
        );
        let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
        let mut reader = decoder.read_info().expect("decode PNG header");
        let mut out = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut out).expect("decode PNG frame");
        assert_eq!(info.width, w);
        assert_eq!(info.height, h);
        assert_eq!(&out[..info.buffer_size()], rgba.as_slice());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn calclab_window_icon_has_valid_rgba_dimensions() {
        let icon = calclab_window_icon();
        assert!(icon.width >= 32 && icon.height >= 32);
        assert_eq!(
            icon.rgba.len(),
            (icon.width as usize) * (icon.height as usize) * 4
        );
        assert!(icon.rgba.chunks_exact(4).any(|px| px[3] > 0));
    }

    #[test]
    fn parse_cli_args_handles_valid_modes_and_views() {
        let s = |v: &[&str]| v.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        assert_eq!(parse_cli_args(&s(&[])), Ok(CliAction::Gui));
        assert_eq!(parse_cli_args(&s(&["calc-app"])), Ok(CliAction::Gui));
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "--help"])),
            Ok(CliAction::Help)
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "-h"])),
            Ok(CliAction::Help)
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "--version"])),
            Ok(CliAction::Version)
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "-V"])),
            Ok(CliAction::Version)
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "--self-test"])),
            Ok(CliAction::SelfTest)
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "--screenshot", "out.png"])),
            Ok(CliAction::Screenshot {
                path: "out.png".to_string(),
                view: 1,
            })
        );
        assert_eq!(
            parse_cli_args(&s(&[
                "calc-app",
                "--screenshot",
                "out.png",
                "--screenshot-view",
                "2"
            ])),
            Ok(CliAction::Screenshot {
                path: "out.png".to_string(),
                view: 2,
            })
        );
        assert_eq!(
            parse_cli_args(&s(&["calc-app", "--view", "3", "--screenshot", "out.png"])),
            Ok(CliAction::Screenshot {
                path: "out.png".to_string(),
                view: 3,
            })
        );
    }

    #[test]
    fn parse_cli_args_rejects_invalid_flags_values_and_combinations() {
        let s = |v: &[&str]| v.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        let invalid_cases: Vec<&[&str]> = vec![
            &["calc-app", "--bogus-flag"],
            &["calc-app", "--screenshot-view", "2"],
            &["calc-app", "--view", "2"],
            &["calc-app", "bogus_positional"],
            &["calc-app", "--screenshot"],
            &["calc-app", "--screenshot", ""],
            &["calc-app", "--screenshot", "   "],
            &["calc-app", "--screenshot", "--self-test"],
            &["calc-app", "--screenshot", "--help"],
            &["calc-app", "--screenshot", "-h"],
            &["calc-app", "--screenshot", "-badname.png"],
            &["calc-app", "--help", "--bogus-flag"],
            &["calc-app", "-h", "--bogus-flag"],
            &["calc-app", "--help", "bogus_positional"],
            &["calc-app", "--help", "--self-test"],
            &["calc-app", "--self-test", "--bogus-flag"],
            &["calc-app", "--self-test", "--screenshot", "out.png"],
            &["calc-app", "--self-test", "--view", "2"],
            &["calc-app", "--screenshot", "out.png", "--view", "abc"],
            &["calc-app", "--screenshot", "out.png", "--view", "9"],
            &["calc-app", "--screenshot", "out.png", "--view", "0"],
            &["calc-app", "--screenshot", "out.png", "--view", "4"],
            &["calc-app", "--screenshot", "out.png", "--screenshot-view"],
            &["calc-app", "--screenshot", "out.png", "--view", "--self-test"],
            &["calc-app", "--screenshot", "out.png", "-z"],
            &["calc-app", "--screenshot", "out.png", "extra_positional"],
            &["calc-app", "--screenshot", "a.png", "--screenshot", "b.png"],
            &["calc-app", "--screenshot", "a.png", "--view", "1", "--view", "2"],
            &["calc-app", "--version", "--help"],
            &["calc-app", "-V", "--self-test"],
            &["calc-app", "--version", "-V"],
        ];
        for case in invalid_cases {
            assert!(
                parse_cli_args(&s(case)).is_err(),
                "Expected error for CLI args {:?}",
                case
            );
        }
    }

    #[test]
    fn cli_usage_text_documents_view_and_screenshot_view() {
        let usage = cli_usage_text();
        assert!(usage.contains("-V, --version"));
        assert!(usage.contains("--screenshot-view <1|2|3>"));
        assert!(usage.contains("--view <1|2|3>"));
    }
}

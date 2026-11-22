#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

mod app;

use app::GraphCraftApp;
use graphcraft_core::{dispatch, Command, GraphEngine};

fn run_smoke() -> i32 {
    let mut engine = GraphEngine::new_with_demo();
    let checks = [
        Command::Status,
        Command::ComputeStats,
        Command::ComputePagerank {
            damping: Some(0.85),
            iterations: Some(25),
        },
        Command::FindShortestPath {
            source: "boomy".into(),
            target: "storytold".into(),
            directed: Some(false),
        },
        Command::RunLayout {
            mode: "force".into(),
            steps: Some(10),
        },
        Command::Export {
            format: "mermaid".into(),
        },
    ];

    for (idx, cmd) in checks.into_iter().enumerate() {
        let res = dispatch(&mut engine, cmd);
        if !res["ok"].as_bool().unwrap_or(false) {
            eprintln!("Smoke check {idx} failed: {res}");
            return 1;
        }
    }

    let _app = GraphCraftApp::new_headless();
    eprintln!("GraphCraft --smoke passed (6/6 core dispatches + headless app init ok).");
    0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--smoke" || a == "--smoke-test") {
        std::process::exit(run_smoke());
    }

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([960.0, 620.0])
            .with_title("GraphCraft - Pure-Rust Knowledge & Network Studio"),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "GraphCraft",
        native_options,
        Box::new(|cc| Ok(Box::new(GraphCraftApp::new(cc)))),
    ) {
        eprintln!("Failed to launch GraphCraft UI: {err}");
        std::process::exit(1);
    }
}

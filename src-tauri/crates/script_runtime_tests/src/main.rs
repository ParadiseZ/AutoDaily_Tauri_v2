use script_runtime_tests::runner::{RunMode, run_config_path};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let mut args = std::env::args_os().skip(1);
    let mode = match args.next().and_then(|value| value.into_string().ok()) {
        Some(value) if value.eq_ignore_ascii_case("record") => RunMode::Record,
        Some(value) if value.eq_ignore_ascii_case("verify") => RunMode::Verify,
        _ => {
            eprintln!(
                "用法: cargo run -p script-runtime-tests -- <record|verify> <config.json> [more.json]"
            );
            std::process::exit(2);
        }
    };
    let paths = args.map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!(
            "用法: cargo run -p script-runtime-tests -- <record|verify> <config.json> [more.json]"
        );
        std::process::exit(2);
    }

    let mut passed = true;
    for path in paths {
        match run_config_path(&path, mode).await {
            Ok(report) => {
                passed &= report.passed;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("serialize suite report")
                );
            }
            Err(error) => {
                passed = false;
                eprintln!("{}: {}", path.display(), error);
            }
        }
    }

    if !passed {
        std::process::exit(1);
    }
}

use script_runtime_tests::runner::run_config_path;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("用法: cargo run -p script-runtime-tests -- <config.json> [more.json]");
        std::process::exit(2);
    }

    let mut passed = true;
    for path in paths {
        match run_config_path(&path).await {
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

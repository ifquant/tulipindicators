use std::process::ExitCode;

use tulipindicators::benchmark::{
    render_markdown, run_registry_benchmarks, write_report, BenchmarkConfig,
};

fn main() -> ExitCode {
    let config = BenchmarkConfig::from_env();

    match run_registry_benchmarks(&config) {
        Ok(results) => {
            print!("{}", render_markdown(&results));

            match write_report(&config, &results) {
                Ok((markdown_path, tsv_path)) => {
                    println!();
                    println!("Saved reports:");
                    println!("- {}", markdown_path.display());
                    println!("- {}", tsv_path.display());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("failed to write benchmark reports: {error}");
                    ExitCode::from(1)
                }
            }
        }
        Err(error) => {
            eprintln!("benchmark failed: {error}");
            ExitCode::from(1)
        }
    }
}

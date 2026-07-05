// gitb log [-n N]: show recent commits across repos

use crate::core::{executor, git, GlobalOpts, Repo};
use colored::*;

pub fn run(repos: &[Repo], opts: &GlobalOpts, number: usize) -> anyhow::Result<()> {
    executor::print_header(opts, &format!("Log ({} commits per repo)", number));

    let n_str = number.to_string();

    let results = executor::execute_parallel(repos, opts, "Log", |repo| {
        let log_output = if opts.dry_run {
            String::new()
        } else {
            git::run_git_capture(&repo.path, &["log", "--oneline", "-n", &n_str])
                .unwrap_or_default()
        };

        let count = log_output.lines().count();

        crate::core::GitResult {
            repo_name: repo.name.clone(),
            success: true,
            exit_code: 0,
            stdout: log_output,
            stderr: String::new(),
            message: format!("{} commit(s)", count),
        }
    });

    if opts.output == crate::core::OutputFormat::Json {
        let json_data: Vec<_> = results
            .iter()
            .map(|r| {
                let commits: Vec<&str> = r.stdout.lines().collect();
                serde_json::json!({
                    "repo": r.repo_name,
                    "commits": commits,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
        return Ok(());
    }

    if opts.quiet {
        return Ok(());
    }

    for r in &results {
        println!("\n  {} ({})", r.repo_name.cyan(), r.message.dimmed());
        if r.stdout.is_empty() {
            println!("    (no commits)");
        } else {
            for line in r.stdout.lines() {
                println!("    {}", line);
            }
        }
    }
    println!();
    Ok(())
}

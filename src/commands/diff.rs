// gitb diff: show diff across all repos

use crate::core::{executor, git, GlobalOpts, Repo};
use colored::*;

pub fn run(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Diff across repos");

    if opts.dry_run {
        println!(" [DRY-RUN] Would show diff across {} repos", repos.len());
        return Ok(());
    }

    let results = executor::execute_parallel(repos, opts, "Diff", |repo| {
        let diff_output = git::run_git_capture(&repo.path, &["diff"]).unwrap_or_default();
        let staged_output =
            git::run_git_capture(&repo.path, &["diff", "--cached"]).unwrap_or_default();

        let has_changes = !diff_output.is_empty() || !staged_output.is_empty();
        let total_lines = diff_output.lines().count() + staged_output.lines().count();

        crate::core::GitResult {
            repo_name: repo.name.clone(),
            success: true,
            exit_code: 0,
            stdout: format!("{}\n{}", staged_output, diff_output),
            stderr: String::new(),
            message: if has_changes {
                format!("{} diff line(s)", total_lines)
            } else {
                "No changes".to_string()
            },
        }
    });

    if opts.output == crate::core::OutputFormat::Json {
        let json_data: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo": r.repo_name,
                    "has_changes": !r.stdout.trim().is_empty(),
                    "diff": r.stdout,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
        return Ok(());
    }

    if opts.quiet {
        return Ok(());
    }

    // Print diffs per repo
    for r in &results {
        if !r.stdout.trim().is_empty() {
            println!("\n--- {} ---", r.repo_name.cyan());
            print!("{}", r.stdout);
        } else {
            println!("{} {} - no changes", "  ".normal(), r.repo_name.dimmed());
        }
    }
    println!();
    Ok(())
}

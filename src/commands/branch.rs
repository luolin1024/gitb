// gitb branch list / gitb branch delete <name>: cross-repo branch operations

use crate::cli::BranchAction;
use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, git, output, GlobalOpts, Repo};
use colored::*;

pub fn run(repos: &[Repo], opts: &GlobalOpts, action: BranchAction) -> anyhow::Result<()> {
    match action {
        BranchAction::List => run_list(repos, opts),
        BranchAction::Delete {
            name,
            force,
            remote,
        } => run_delete(repos, opts, &name, force, remote),
    }
}

fn run_list(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Branch listing");

    if opts.dry_run {
        println!(
            " [DRY-RUN] Would list branches across {} repos",
            repos.len()
        );
        return Ok(());
    }

    let results = executor::execute_parallel(repos, opts, "Branches", |repo| {
        let branches = git::run_git_capture(&repo.path, &["branch", "--format=%(refname:short)"])
            .unwrap_or_default();

        let _current = git::get_current_branch(&repo.path).unwrap_or_default();

        crate::core::GitResult {
            repo_name: repo.name.clone(),
            success: true,
            exit_code: 0,
            stdout: branches.clone(),
            stderr: String::new(),
            message: format!("{} branch(es)", branches.lines().count()),
        }
    });

    if opts.output == crate::core::OutputFormat::Json {
        let json_data: Vec<_> = results
            .iter()
            .map(|r| {
                let branches: Vec<&str> = r.stdout.lines().collect();
                serde_json::json!({
                    "repo": r.repo_name,
                    "branches": branches,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
        return Ok(());
    }

    if opts.quiet {
        return Ok(());
    }

    // Print branches per repo
    for r in &results {
        let current = if let Some(repo) = repos.iter().find(|repo| repo.name == r.repo_name) {
            git::get_current_branch(&repo.path).unwrap_or_default()
        } else {
            String::new()
        };

        println!("\n  {} ({})", r.repo_name.cyan(), r.message.dimmed());
        for line in r.stdout.lines() {
            let branch_name = line.trim();
            if branch_name == current {
                println!("    {} {}", "*".green(), branch_name.green().bold());
            } else {
                println!("     {}", branch_name);
            }
        }
    }
    println!();
    Ok(())
}

fn run_delete(
    repos: &[Repo],
    opts: &GlobalOpts,
    name: &str,
    force: bool,
    remote: bool,
) -> anyhow::Result<()> {
    let flag = if force { "-D" } else { "-d" };
    let title = format!(
        "Deleting branch: {} {}",
        if remote { "(local+remote)" } else { "(local)" },
        name
    );
    executor::print_header(opts, &title);
    executor::print_skip_info(opts, &opts.skip);

    let name_local = name.to_string();
    let name_remote = name.to_string();

    let results = executor::execute_parallel(repos, opts, "Delete", move |repo| {
        // Delete local branch
        let local_result = exec_git_on_repo(
            repo,
            opts,
            &["branch", flag, &name_local],
            &format!("Deleted local branch {}", name_local),
            &format!("git branch {} {}", flag, name_local),
        );

        if !local_result.success && !opts.dry_run {
            // Branch might not exist in this repo - not an error
            if git::run_git_capture(
                &repo.path,
                &[
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/heads/{}", name_local),
                ],
            )
            .is_none()
            {
                return crate::core::GitResult::ok(
                    &repo.name,
                    &format!("No local branch '{}' to delete", name_local),
                );
            }
        }

        // Delete remote branch if requested
        if remote {
            let remote_result = exec_git_on_repo(
                repo,
                opts,
                &["push", "origin", "--delete", &name_remote],
                "",
                &format!("git push origin --delete {}", name_remote),
            );

            if remote_result.success {
                return crate::core::GitResult {
                    message: format!("Deleted {} (local + remote)", name_local),
                    ..local_result
                };
            }
        }

        local_result
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

// gitb stash [push|pop|list|clear]: batch stash operations

use crate::cli::StashAction;
use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, git, output, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts, action: Option<StashAction>) -> anyhow::Result<()> {
    let action = action.unwrap_or(StashAction::Push);

    match action {
        StashAction::Push => run_push(repos, opts),
        StashAction::Pop => run_pop(repos, opts),
        StashAction::List => run_list(repos, opts),
        StashAction::Clear => run_clear(repos, opts),
    }
}

fn run_push(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Stashing");

    let results = executor::execute_parallel(repos, opts, "Stash", |repo| {
        // Check if there's anything to stash
        if !opts.dry_run && !git::is_dirty(&repo.path) && !git::has_untracked(&repo.path) {
            return crate::core::GitResult::ok(&repo.name, "Nothing to stash (clean)");
        }

        exec_git_on_repo(
            repo,
            opts,
            &["stash", "push", "-u"],
            "Stashed changes",
            "git stash push -u",
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

fn run_pop(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Popping stash");

    let results = executor::execute_parallel(repos, opts, "Pop", |repo| {
        // Check if there's a stash to pop
        if !opts.dry_run {
            let stash_list =
                git::run_git_capture(&repo.path, &["stash", "list"]).unwrap_or_default();
            if stash_list.is_empty() {
                return crate::core::GitResult::ok(&repo.name, "No stash to pop");
            }
        }

        exec_git_on_repo(
            repo,
            opts,
            &["stash", "pop"],
            "Popped stash",
            "git stash pop",
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

fn run_list(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Stash list");

    let results = executor::execute_parallel(repos, opts, "Stash list", |repo| {
        let stash_list = if opts.dry_run {
            String::new()
        } else {
            git::run_git_capture(&repo.path, &["stash", "list"]).unwrap_or_default()
        };

        let count = if stash_list.is_empty() {
            0
        } else {
            stash_list.lines().count()
        };

        crate::core::GitResult {
            repo_name: repo.name.clone(),
            success: true,
            exit_code: 0,
            stdout: stash_list,
            stderr: String::new(),
            message: format!("{} stash(es)", count),
        }
    });

    if opts.output == crate::core::OutputFormat::Json {
        let json_data: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo": r.repo_name,
                    "stash_count": r.stdout.lines().count(),
                    "stashes": r.stdout.lines().collect::<Vec<_>>(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
        return Ok(());
    }

    if !opts.quiet {
        for r in &results {
            if !r.stdout.is_empty() {
                println!("\n  {} ({}):", r.repo_name, r.message);
                for line in r.stdout.lines() {
                    println!("    {}", line);
                }
            } else if !opts.quiet {
                println!("  {} - no stashes", r.repo_name);
            }
        }
        println!();
    }

    Ok(())
}

fn run_clear(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Clearing stashes");

    let results = executor::execute_parallel(repos, opts, "Clear", |repo| {
        if !opts.dry_run {
            let stash_list =
                git::run_git_capture(&repo.path, &["stash", "list"]).unwrap_or_default();
            if stash_list.is_empty() {
                return crate::core::GitResult::ok(&repo.name, "No stash to clear");
            }
        }

        exec_git_on_repo(
            repo,
            opts,
            &["stash", "clear"],
            "Cleared stashes",
            "git stash clear",
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

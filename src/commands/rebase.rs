// gitb rebase: smart rebase (stash → rebase → pop) across repos

use crate::core::{executor, git, output, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts, branch: Option<&str>) -> anyhow::Result<()> {
    let target = branch.unwrap_or("@{u}").to_string();
    let title = format!("Smart rebasing onto: {}", target);
    executor::print_header(opts, &title);
    executor::print_skip_info(opts, &opts.skip);

    let target_clone = target.clone();

    let results = executor::execute_parallel(repos, opts, "Rebase", move |repo| {
        if opts.dry_run {
            return crate::core::GitResult {
                repo_name: repo.name.clone(),
                success: true,
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                message: format!(
                    "[DRY-RUN] Would stash → rebase {} → pop in {}",
                    target_clone, repo.name
                ),
            };
        }

        // Step 1: Check if dirty, stash if needed
        let was_dirty = git::is_dirty(&repo.path) || git::has_untracked(&repo.path);
        let stashed = if was_dirty {
            let stash_result = git::run_git(&repo.name, &repo.path, &["stash", "push", "-u"]);
            if stash_result.success {
                true
            } else {
                return crate::core::GitResult::fail(
                    &repo.name,
                    "Failed to stash changes before rebase",
                );
            }
        } else {
            false
        };

        // Step 2: Rebase
        let rebase_result = git::run_git(&repo.name, &repo.path, &["rebase", &target_clone]);

        if !rebase_result.success {
            // Rebase failed - try to abort and restore
            git::run_git(&repo.name, &repo.path, &["rebase", "--abort"]);
            if stashed {
                git::run_git(&repo.name, &repo.path, &["stash", "pop"]);
            }
            return crate::core::GitResult {
                message: format!(
                    "Rebase failed (aborted, changes restored): {}",
                    rebase_result.stderr.lines().last().unwrap_or("")
                ),
                ..rebase_result
            };
        }

        // Step 3: Pop stash if we stashed
        if stashed {
            let pop_result = git::run_git(&repo.name, &repo.path, &["stash", "pop"]);
            if !pop_result.success {
                return crate::core::GitResult {
                    message: format!(
                        "Rebased, but stash pop failed: {}",
                        pop_result.stderr.lines().last().unwrap_or("")
                    ),
                    success: false,
                    ..pop_result
                };
            }
        }

        crate::core::GitResult::ok(
            &repo.name,
            &format!(
                "Rebased onto {}{}",
                target_clone,
                if stashed { " (stash restored)" } else { "" }
            ),
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

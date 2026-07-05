// gitb create <branch>: batch create+switch branch across repos

use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, git, output, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts, branch: &str) -> anyhow::Result<()> {
    executor::print_header(opts, &format!("Creating branch: {}", branch));
    executor::print_skip_info(opts, &opts.skip);

    let branch = branch.to_string();

    let results = executor::execute_parallel(repos, opts, "Create", |repo| {
        // Fetch all first
        if !opts.dry_run {
            git::run_git(&repo.name, &repo.path, &["fetch", "--all"]);
        }

        // Check if branch already exists locally
        if !opts.dry_run && git::local_branch_exists(&repo.path, &branch) {
            // Switch to existing branch
            return exec_git_on_repo(
                repo,
                opts,
                &["checkout", &branch],
                &format!("Branch exists, switched to {}", branch),
                &format!("git checkout {} (already exists)", branch),
            );
        }

        // Create and switch to new branch
        let result = exec_git_on_repo(
            repo,
            opts,
            &["checkout", "-b", &branch],
            &format!("Created and switched to {}", branch),
            &format!("git checkout -b {}", branch),
        );

        if result.success && !opts.dry_run {
            // Try to push with -u (don't fail if push fails)
            let push_result =
                git::run_git(&repo.name, &repo.path, &["push", "-u", "origin", &branch]);
            if push_result.success {
                return crate::core::GitResult {
                    message: format!("Created {} and pushed to remote", branch),
                    ..result
                };
            }
            // Push failed is not critical - branch was still created locally
            return crate::core::GitResult {
                message: format!("Created {} (not pushed to remote)", branch),
                ..result
            };
        }

        result
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

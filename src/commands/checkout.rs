// gitb checkout <branch>: batch switch branch across repos

use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, git, output, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts, branch: &str) -> anyhow::Result<()> {
    executor::print_header(opts, &format!("Checking out branch: {}", branch));
    executor::print_skip_info(opts, &opts.skip);

    let force = opts.force;
    let branch = branch.to_string();

    let results = executor::execute_parallel(repos, opts, "Checkout", |repo| {
        // Force mode: discard all changes first
        if force && !opts.dry_run {
            git::run_git(&repo.name, &repo.path, &["reset", "--hard", "HEAD"]);
            git::run_git(&repo.name, &repo.path, &["clean", "-fd"]);
        }

        // Fetch all remotes first
        if !opts.dry_run {
            git::run_git(&repo.name, &repo.path, &["fetch", "--all"]);
        }

        // Try exact branch checkout
        let checkout_result = exec_git_on_repo(
            repo,
            opts,
            &["checkout", &branch],
            &format!("Switched to {}", branch),
            &format!("git checkout {}", branch),
        );

        if checkout_result.success {
            return checkout_result;
        }

        // Fuzzy match: find a branch that contains the pattern
        if !opts.dry_run {
            if let Some(branches) = git::run_git_capture(
                &repo.path,
                &["branch", "--list", "--all", "--format=%(refname:short)"],
            ) {
                let matched: Vec<&str> = branches.lines().filter(|b| b.contains(&branch)).collect();
                if matched.len() == 1 {
                    let matched_branch = matched[0].trim_start_matches("origin/");
                    return exec_git_on_repo(
                        repo,
                        opts,
                        &["checkout", matched_branch],
                        &format!("Switched to {} (fuzzy match)", matched_branch),
                        &format!("git checkout {} (fuzzy)", matched_branch),
                    );
                }
            }
        }

        checkout_result
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

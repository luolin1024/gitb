// gitb commit -m <msg> [-a]: batch commit across repos

use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, git, output, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts, message: &str, all: bool) -> anyhow::Result<()> {
    let title = format!("Committing: \"{}\"", message);
    executor::print_header(opts, &title);
    executor::print_skip_info(opts, &opts.skip);

    let message = message.to_string();

    let results = executor::execute_parallel(repos, opts, "Commit", |repo| {
        // Optionally stage all changes
        if all && !opts.dry_run {
            git::run_git(&repo.name, &repo.path, &["add", "-A"]);
        }

        // Check if there's anything to commit
        if !opts.dry_run {
            let has_staged = std::process::Command::new("git")
                .args(["diff", "--cached", "--quiet"])
                .current_dir(&repo.path)
                .status()
                .map(|s| !s.success())
                .unwrap_or(false);

            if !has_staged {
                if !all {
                    if !git::is_dirty(&repo.path) {
                        return crate::core::GitResult::ok(&repo.name, "Nothing to commit (clean)");
                    }
                    return crate::core::GitResult::ok(
                        &repo.name,
                        "Nothing staged (use -a to stage all)",
                    );
                }
                // all=true but nothing staged after git add -A = nothing to commit
                return crate::core::GitResult::ok(&repo.name, "Nothing to commit (clean)");
            }
        }

        exec_git_on_repo(
            repo,
            opts,
            &["commit", "-m", &message],
            &format!("Committed: {}", message),
            &format!("git commit -m \"{}\"", message),
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

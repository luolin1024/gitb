// gitb pull / gitb fetch / gitb push: batch sync operations

use crate::core::executor::exec_git_on_repo;
use crate::core::{executor, output, GlobalOpts, Repo};

pub fn run_pull(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Pulling");
    executor::print_skip_info(opts, &opts.skip);

    let results = executor::execute_parallel(repos, opts, "Pull", |repo| {
        exec_git_on_repo(repo, opts, &["pull"], "Pulled", "git pull")
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

pub fn run_fetch(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Fetching");
    executor::print_skip_info(opts, &opts.skip);

    let results = executor::execute_parallel(repos, opts, "Fetch", |repo| {
        exec_git_on_repo(
            repo,
            opts,
            &["fetch", "--all", "--prune"],
            "Fetched",
            "git fetch --all --prune",
        )
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

pub fn run_push(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    executor::print_header(opts, "Pushing");
    executor::print_skip_info(opts, &opts.skip);

    let results = executor::execute_parallel(repos, opts, "Push", |repo| {
        exec_git_on_repo(repo, opts, &["push"], "Pushed", "git push")
    });

    output::print_results(&results, opts.output, opts.quiet);
    Ok(())
}

// gitb status: colored multi-repo overview (gita-ll style)

use crate::core::output::{print_status_table, StatusRow};
use crate::core::{git, GlobalOpts, Repo};
use colored::*;

pub fn run(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    if opts.dry_run {
        println!(
            "\n{} [DRY-RUN] Would show status of {} repos",
            "Status".bold(),
            repos.len()
        );
        return Ok(());
    }

    // Build StatusRows in parallel using rayon
    use rayon::prelude::*;
    let status_rows: Vec<StatusRow> = repos
        .par_iter()
        .map(|repo| {
            let branch = git::get_branch_display(&repo.path);
            let dirty = git::is_dirty(&repo.path);
            let staged = std::process::Command::new("git")
                .args(["diff", "--cached", "--quiet"])
                .current_dir(&repo.path)
                .status()
                .map(|s| !s.success())
                .unwrap_or(false);
            let untracked = git::has_untracked(&repo.path);
            let stashed = git::run_git_capture(&repo.path, &["stash", "list"])
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let (ahead, behind) = git::get_ahead_behind(&repo.path);
            let has_upstream = git::run_git_capture(&repo.path, &["rev-parse", "@{u}"]).is_some();
            let last_msg = git::last_commit_message(&repo.path).unwrap_or_default();
            let last_time = git::last_commit_time(&repo.path).unwrap_or_default();

            StatusRow {
                repo_name: repo.name.clone(),
                branch,
                dirty,
                staged,
                untracked,
                stashed,
                ahead,
                behind,
                last_msg,
                last_time,
                has_upstream,
            }
        })
        .collect();

    if opts.output == crate::core::OutputFormat::Json {
        let json_rows: Vec<_> = status_rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo": r.repo_name,
                    "branch": r.branch,
                    "dirty": r.dirty,
                    "staged": r.staged,
                    "untracked": r.untracked,
                    "stashed": r.stashed,
                    "ahead": r.ahead,
                    "behind": r.behind,
                    "has_upstream": r.has_upstream,
                    "last_commit": r.last_msg,
                    "last_commit_time": r.last_time,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_rows).unwrap());
    } else {
        print_status_table(&status_rows);
    }

    Ok(())
}

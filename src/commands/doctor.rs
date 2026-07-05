// gitb doctor: health check across repos

use crate::core::output::{print_doctor_report, DoctorRow};
use crate::core::{executor, git, GlobalOpts, Repo};

pub fn run(repos: &[Repo], opts: &GlobalOpts) -> anyhow::Result<()> {
    if opts.dry_run {
        println!(
            "\n[DRY-RUN] Would run health check on {} repos",
            repos.len()
        );
        return Ok(());
    }

    executor::print_header(opts, "Health check");

    let _results = executor::execute_parallel(repos, opts, "Doctor", |repo| {
        // Just warm up parallel execution
        crate::core::GitResult::ok(&repo.name, "")
    });

    // Build doctor rows
    let mut rows: Vec<DoctorRow> = repos
        .iter()
        .map(|repo| {
            let branch = git::get_branch_display(&repo.path);
            let dirty = git::is_dirty(&repo.path);
            let (ahead, behind) = git::get_ahead_behind(&repo.path);
            let has_upstream = git::run_git_capture(&repo.path, &["rev-parse", "@{u}"]).is_some();
            let stashed = git::run_git_capture(&repo.path, &["stash", "list"])
                .map(|s| !s.is_empty())
                .unwrap_or(false);

            let mut issues = Vec::new();

            if !has_upstream {
                issues.push("No upstream tracking branch".to_string());
            }
            if dirty {
                issues.push("Uncommitted changes".to_string());
            }
            if ahead > 0 {
                issues.push(format!("{} unpushed commit(s)", ahead));
            }
            if behind > 0 {
                issues.push(format!("{} commit(s) behind upstream", behind));
            }
            if stashed {
                issues.push("Has stashed changes".to_string());
            }

            DoctorRow {
                repo_name: repo.name.clone(),
                branch,
                dirty,
                ahead,
                behind,
                has_upstream,
                stashed,
                issues,
            }
        })
        .collect();

    // Sort: repos with issues first
    rows.sort_by(|a, b| {
        let a_issues = !a.issues.is_empty();
        let b_issues = !b.issues.is_empty();
        b_issues.cmp(&a_issues).then(a.repo_name.cmp(&b.repo_name))
    });

    if opts.output == crate::core::OutputFormat::Json {
        let json_data: Vec<_> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo": r.repo_name,
                    "branch": r.branch,
                    "dirty": r.dirty,
                    "ahead": r.ahead,
                    "behind": r.behind,
                    "has_upstream": r.has_upstream,
                    "stashed": r.stashed,
                    "issues": r.issues,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_data).unwrap());
        return Ok(());
    }

    if !opts.quiet {
        print_doctor_report(&rows);
    }

    Ok(())
}

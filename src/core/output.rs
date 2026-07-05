// Output formatting: table, json, quiet

use crate::core::{GitResult, OutputFormat};
use colored::*;

/// Print git command results in the configured format
pub fn print_results(results: &[GitResult], format: OutputFormat, quiet: bool) {
    match format {
        OutputFormat::Json => print_json(results),
        OutputFormat::Quiet => print_quiet(results),
        OutputFormat::Table => print_table(results, quiet),
    }
}

fn print_table(results: &[GitResult], _quiet: bool) {
    if results.is_empty() {
        println!("\nNo repositories found.");
        return;
    }

    for r in results {
        let status_icon = if r.success {
            "✓".green()
        } else {
            "✗".red()
        };
        let repo_colored = if r.success {
            r.repo_name.normal()
        } else {
            r.repo_name.red()
        };
        let msg = if r.message.is_empty() {
            String::new()
        } else if r.success {
            r.message.green().to_string()
        } else {
            r.message.red().to_string()
        };

        println!("  {} {:<20} {}", status_icon, repo_colored, msg);

        // Print stderr/stdout in verbose context if non-empty and failed
        if !r.success && !r.stderr.is_empty() {
            for line in r.stderr.lines().take(3) {
                println!("    {}", line.dimmed());
            }
        }
    }

    // Summary
    let total = results.len();
    let success = results.iter().filter(|r| r.success).count();
    let failed = total - success;
    println!();
    if failed == 0 {
        println!("  {} {}/{} repos succeeded", "✓".green(), success, total);
    } else {
        println!(
            "  {} {}/{} repos succeeded, {} failed",
            "✗".red(),
            success,
            total,
            failed
        );
    }
}

fn print_json(results: &[GitResult]) {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "repo": r.repo_name,
                "success": r.success,
                "exit_code": r.exit_code,
                "stdout": r.stdout,
                "stderr": r.stderr,
                "message": r.message,
            })
        })
        .collect();

    let summary = serde_json::json!({
        "total": results.len(),
        "success": results.iter().filter(|r| r.success).count(),
        "failed": results.iter().filter(|r| !r.success).count(),
    });

    let output = serde_json::json!({
        "results": json_results,
        "summary": summary,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_quiet(results: &[GitResult]) {
    let failed = results.iter().filter(|r| !r.success);
    for r in failed {
        eprintln!("{}: {}", r.repo_name, r.message);
    }
}

/// Status table row data (for `gitb status`)
#[derive(Debug, Clone)]
pub struct StatusRow {
    pub repo_name: String,
    pub branch: String,
    pub dirty: bool,       // unstaged changes
    pub staged: bool,      // staged changes
    pub untracked: bool,   // untracked files
    pub stashed: bool,     // has stashes
    pub ahead: usize,      // commits ahead of upstream
    pub behind: usize,     // commits behind upstream
    pub last_msg: String,  // last commit message
    pub last_time: String, // last commit relative time
    pub has_upstream: bool,
}

/// Determine branch state for color coding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchState {
    NoRemote, // no upstream tracking
    Synced,   // up to date with upstream
    Ahead,    // local commits not pushed
    Behind,   // remote commits not pulled
    Diverged, // both ahead and behind
}

impl StatusRow {
    pub fn branch_state(&self) -> BranchState {
        if !self.has_upstream {
            return BranchState::NoRemote;
        }
        match (self.ahead, self.behind) {
            (0, 0) => BranchState::Synced,
            (a, 0) if a > 0 => BranchState::Ahead,
            (0, b) if b > 0 => BranchState::Behind,
            _ => BranchState::Diverged,
        }
    }
}

/// Print the multi-repo status overview (gita-ll style)
pub fn print_status_table(rows: &[StatusRow]) {
    if rows.is_empty() {
        println!("\nNo repositories found.");
        return;
    }

    println!();

    // Calculate column widths for alignment
    let name_width = rows
        .iter()
        .map(|r| r.repo_name.chars().count())
        .max()
        .unwrap_or(10)
        .max(10);

    let branch_width = rows
        .iter()
        .map(|r| r.branch.chars().count())
        .max()
        .unwrap_or(6)
        .clamp(6, 30);

    // Header
    println!(
        "  {:<nw$}  {:<bw$}  {:<8}  {:<8}  LAST COMMIT",
        "REPO",
        "BRANCH",
        "SYNC",
        "CHANGES",
        nw = name_width,
        bw = branch_width,
    );
    println!("  {}", "-".repeat(name_width + branch_width + 40));

    for row in rows {
        let state = row.branch_state();

        // Sync status (upstream relationship)
        let sync_text = match state {
            BranchState::NoRemote => "no remote".to_string(),
            BranchState::Synced => "synced".to_string(),
            BranchState::Ahead => format!("ahead {}", row.ahead),
            BranchState::Behind => format!("behind {}", row.behind),
            BranchState::Diverged => format!("↑{}↓{}", row.ahead, row.behind),
        };
        let sync_colored = match state {
            BranchState::NoRemote => sync_text.dimmed(),
            BranchState::Synced => sync_text.green(),
            BranchState::Ahead => sync_text.purple(),
            BranchState::Behind => sync_text.yellow(),
            BranchState::Diverged => sync_text.red(),
        };

        // Changes status — explicit text instead of cryptic symbols
        let changes_text = if !row.dirty && !row.staged && !row.untracked && !row.stashed {
            "clean".to_string()
        } else {
            let mut parts = Vec::new();
            if row.staged {
                parts.push("staged");
            }
            if row.dirty {
                parts.push("dirty");
            }
            if row.untracked {
                parts.push("untracked");
            }
            if row.stashed {
                parts.push("stash");
            }
            parts.join("+")
        };
        let changes_colored = if changes_text == "clean" {
            changes_text.green()
        } else {
            changes_text.yellow()
        };

        // Truncate branch name if too long
        let branch_display: String = if row.branch.chars().count() > branch_width {
            row.branch
                .chars()
                .take(branch_width - 1)
                .collect::<String>()
                + "…"
        } else {
            row.branch.clone()
        };
        let branch_padded = format!("{:<bw$}", branch_display, bw = branch_width);
        let branch_final = match state {
            BranchState::NoRemote => branch_padded.normal(),
            BranchState::Synced => branch_padded.green(),
            BranchState::Ahead => branch_padded.purple(),
            BranchState::Behind => branch_padded.yellow(),
            BranchState::Diverged => branch_padded.red(),
        };

        // Last commit message (truncated)
        let last_msg: String = row.last_msg.chars().take(45).collect::<String>();

        println!(
            "  {:<nw$}  {}  {:<8}  {:<8}  {}",
            row.repo_name.cyan(),
            branch_final,
            sync_colored,
            changes_colored,
            last_msg.dimmed(),
            nw = name_width,
        );
    }

    println!();
    println!(
        "  {} repos | CHANGES: clean / staged+dirty+untracked+stash | SYNC: synced / ahead N / behind N / no remote",
        rows.len()
    );
}

/// Print doctor report (health check)
#[derive(Debug, Clone)]
pub struct DoctorRow {
    pub repo_name: String,
    pub branch: String,
    pub dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub has_upstream: bool,
    pub stashed: bool,
    pub issues: Vec<String>,
}

pub fn print_doctor_report(rows: &[DoctorRow]) {
    if rows.is_empty() {
        println!("\nNo repositories found.");
        return;
    }

    println!("\n{}", "Health Check Report".bold());
    println!("==================");

    let mut total_issues = 0;

    for row in rows {
        let icon = if row.issues.is_empty() {
            "✓".green()
        } else {
            "⚠".yellow()
        };

        println!("\n  {} {} ({})", icon, row.repo_name.cyan(), row.branch);

        if row.issues.is_empty() {
            println!("    {}", "clean".green());
        } else {
            for issue in &row.issues {
                println!("    {}", issue.yellow());
                total_issues += 1;
            }
        }
    }

    println!("\n{}", "-".repeat(40));
    let clean = rows.iter().filter(|r| r.issues.is_empty()).count();
    let with_issues = rows.len() - clean;
    if total_issues == 0 {
        println!("  {} All {} repos are healthy", "✓".green(), rows.len());
    } else {
        println!(
            "  {} {} issue(s) across {} repo(s), {} clean",
            "⚠".yellow(),
            total_issues,
            with_issues,
            clean
        );
    }
}

// Git command wrapper: execute git commands in repo directories

use crate::core::GitResult;
use std::path::Path;
use std::process::Command;

/// Execute a git command in a repo directory.
///
/// Returns a GitResult with captured stdout/stderr/exit_code.
pub fn run_git(repo_name: &str, repo_path: &Path, args: &[&str]) -> GitResult {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let success = out.status.success();
            let exit_code = out.status.code().unwrap_or(-1);

            // Build a concise message from stdout or stderr
            let message = if success {
                stdout.lines().last().unwrap_or("").to_string()
            } else {
                stderr
                    .lines()
                    .last()
                    .unwrap_or("git command failed")
                    .to_string()
            };

            GitResult {
                repo_name: repo_name.to_string(),
                success,
                exit_code,
                stdout,
                stderr,
                message,
            }
        }
        Err(e) => GitResult {
            repo_name: repo_name.to_string(),
            success: false,
            exit_code: -1,
            stdout: String::new(),
            stderr: e.to_string(),
            message: format!("failed to execute git: {}", e),
        },
    }
}

/// Run git and return the trimmed stdout on success, or empty string on failure.
pub fn run_git_capture(repo_path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Check if a remote branch exists: `git show-ref --verify --quiet refs/remotes/origin/<branch>`
#[allow(dead_code)]
pub fn remote_branch_exists(repo_path: &Path, branch: &str) -> bool {
    let ref_name = format!("refs/remotes/origin/{}", branch);
    Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &ref_name])
        .current_dir(repo_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a local branch exists: `git show-ref --verify --quiet refs/heads/<branch>`
pub fn local_branch_exists(repo_path: &Path, branch: &str) -> bool {
    let ref_name = format!("refs/heads/{}", branch);
    Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &ref_name])
        .current_dir(repo_path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the current branch name via `git rev-parse --abbrev-ref HEAD`
pub fn get_current_branch(repo_path: &Path) -> Option<String> {
    run_git_capture(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
}

/// Get the current branch name, or "(detached)" if detached HEAD
pub fn get_branch_display(repo_path: &Path) -> String {
    match get_current_branch(repo_path) {
        Some(b) if b != "HEAD" => b,
        Some(_) => "(detached)".to_string(),
        None => "(unknown)".to_string(),
    }
}

/// Check if the working tree has uncommitted changes (staged or unstaged)
pub fn is_dirty(repo_path: &Path) -> bool {
    Command::new("git")
        .args(["diff", "--quiet"])
        .current_dir(repo_path)
        .status()
        .map(|s| !s.success())
        .unwrap_or(true)
        || Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(repo_path)
            .status()
            .map(|s| !s.success())
            .unwrap_or(true)
}

/// Check if there are untracked files
pub fn has_untracked(repo_path: &Path) -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().any(|line| line.starts_with("??"))
        }
        Err(_) => false,
    }
}

/// Get ahead/behind count vs upstream: returns (ahead, behind)
pub fn get_ahead_behind(repo_path: &Path) -> (usize, usize) {
    let upstream = match run_git_capture(repo_path, &["rev-parse", "@{u}"]) {
        Some(u) => u,
        None => return (0, 0),
    };
    let local = match run_git_capture(repo_path, &["rev-parse", "HEAD"]) {
        Some(l) => l,
        None => return (0, 0),
    };
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{}...{}", &local, &upstream),
        ])
        .current_dir(repo_path)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() == 2 {
                let behind = parts[0].parse().unwrap_or(0);
                let ahead = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}

/// Get the last commit message (subject line)
pub fn last_commit_message(repo_path: &Path) -> Option<String> {
    run_git_capture(repo_path, &["log", "-1", "--format=%s"])
}

/// Get the last commit relative time
pub fn last_commit_time(repo_path: &Path) -> Option<String> {
    run_git_capture(repo_path, &["log", "-1", "--format=%cr"])
}

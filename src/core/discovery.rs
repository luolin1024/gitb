// Repository discovery: scan directories for git repos

use crate::core::Repo;
use std::fs;
use std::path::{Path, PathBuf};

/// Discover git repositories under the given root directory.
///
/// - `root`: starting directory
/// - `depth`: max recursion depth (0 = only check root's immediate children,
///   1 = also check grandchildren, etc.)
/// - `skip`: directory names to skip
///
/// Returns a sorted list of Repo objects.
pub fn discover_repos(root: &Path, depth: usize, skip: &[String]) -> Vec<Repo> {
    let skip_set: std::collections::HashSet<&str> = skip.iter().map(|s| s.as_str()).collect();

    let mut repos = Vec::new();
    scan_dir(root, depth, &skip_set, &mut repos);

    // Sort by name for stable output
    repos.sort_by(|a, b| a.name.cmp(&b.name));
    repos
}

/// Recursively scan a directory for git repos.
fn scan_dir(
    dir: &Path,
    remaining_depth: usize,
    skip_set: &std::collections::HashSet<&str>,
    repos: &mut Vec<Repo>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only process directories
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip hidden directories and explicitly skipped dirs
        if name.starts_with('.') || skip_set.contains(name.as_str()) {
            continue;
        }

        // Check if this directory is a git repo
        if is_git_repo(&path) {
            repos.push(Repo {
                path: path.clone(),
                name,
            });
        } else if remaining_depth > 0 {
            // Recurse into subdirectories
            scan_dir(&path, remaining_depth - 1, skip_set, repos);
        }
    }
}

/// Check if a directory contains a `.git` (directory or file, for worktrees).
pub fn is_git_repo(path: &Path) -> bool {
    let git_path = path.join(".git");
    git_path.exists()
}

/// Get the absolute path of the current working directory.
pub fn current_dir() -> anyhow::Result<PathBuf> {
    std::env::current_dir().map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))
}

// Workspace configuration: gitb.toml parser

use crate::core::Repo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Workspace configuration loaded from gitb.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub workspace: Workspace,
    #[serde(default)]
    pub groups: HashMap<String, GroupDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    /// Default branch name for checkout/create operations
    pub default_branch: Option<String>,
    /// Directories to always skip
    #[serde(default)]
    pub default_skip: Vec<String>,
    /// Default discovery depth
    pub default_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupDef {
    /// Repo names in this group
    #[serde(default)]
    pub repos: Vec<String>,
}

impl WorkspaceConfig {
    /// Load config from the current directory or parent directories.
    /// Searches for `gitb.toml` starting from CWD going up to root.
    pub fn load() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::load_from(&cwd)
    }

    /// Load config from a specific directory, searching upward.
    pub fn load_from(start: &Path) -> anyhow::Result<Self> {
        let mut current = start.to_path_buf();
        loop {
            let config_path = current.join("gitb.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: WorkspaceConfig = toml::from_str(&content).map_err(|e| {
                    anyhow::anyhow!("Failed to parse {}: {}", config_path.display(), e)
                })?;
                return Ok(config);
            }
            if !current.pop() {
                // Reached root without finding config
                return Ok(WorkspaceConfig::default());
            }
        }
    }

    /// Save config to gitb.toml in the current directory.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = PathBuf::from("gitb.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get the list of repos for a group, filtering the discovered repos.
    pub fn filter_repos_by_group(&self, repos: &[Repo], group_name: &str) -> Vec<Repo> {
        match self.groups.get(group_name) {
            Some(group) => {
                let name_set: std::collections::HashSet<&str> =
                    group.repos.iter().map(|s| s.as_str()).collect();
                repos
                    .iter()
                    .filter(|r| name_set.contains(r.name.as_str()))
                    .cloned()
                    .collect()
            }
            None => {
                eprintln!("Warning: group '{}' not found in gitb.toml", group_name);
                repos.to_vec()
            }
        }
    }

    /// Add a group to the config.
    pub fn add_group(&mut self, name: &str, repos: Vec<String>) {
        self.groups.insert(name.to_string(), GroupDef { repos });
    }

    /// Remove a group from the config.
    pub fn remove_group(&mut self, name: &str) -> bool {
        self.groups.remove(name).is_some()
    }

    /// Check if a config file exists in the current directory.
    pub fn exists_in_cwd() -> bool {
        Path::new("gitb.toml").exists()
    }
}

// gitb init: interactive workspace setup wizard

use crate::config::{GroupDef, Workspace, WorkspaceConfig};
use crate::core::{discovery, GlobalOpts};
use colored::*;
use dialoguer::{Confirm, Input, MultiSelect};

pub fn run(opts: &GlobalOpts) -> anyhow::Result<()> {
    println!("\n{}", "gitb workspace initialization".bold());
    println!("===========================\n");

    // Check if gitb.toml already exists
    if WorkspaceConfig::exists_in_cwd()
        && !Confirm::new()
            .with_prompt("gitb.toml already exists. Overwrite?")
            .default(false)
            .interact()?
    {
        println!("Aborted.");
        return Ok(());
    }

    // Discover repos in current directory
    let cwd = discovery::current_dir()?;
    println!(
        "Scanning for git repos in {} (depth={})...\n",
        cwd.display(),
        opts.depth
    );
    let repos = discovery::discover_repos(&cwd, opts.depth, &[]);

    if repos.is_empty() {
        eprintln!(
            "{} No git repositories found in current directory.",
            "✗".red()
        );
        eprintln!("Try running gitb init in a directory containing git repos, or use -d to increase depth.");
        std::process::exit(1);
    }

    println!("{} Found {} git repos:", "✓".green(), repos.len());
    for repo in &repos {
        println!("  - {}", repo.name);
    }
    println!();

    // Ask for default branch
    let default_branch: String = Input::new()
        .with_prompt("Default branch name (leave empty for none)")
        .allow_empty(true)
        .interact()?;

    // Ask for default skip dirs
    let default_skip: String = Input::new()
        .with_prompt("Default skip directories (comma-separated, leave empty for none)")
        .allow_empty(true)
        .interact()?;

    let skip_dirs: Vec<String> = if default_skip.is_empty() {
        Vec::new()
    } else {
        default_skip
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    // Ask about creating groups
    let mut groups = std::collections::HashMap::new();

    if Confirm::new()
        .with_prompt("Create a repo group?")
        .default(false)
        .interact()?
    {
        loop {
            let group_name: String = Input::new().with_prompt("Group name").interact()?;

            let repo_names: Vec<String> = repos.iter().map(|r| r.name.clone()).collect();
            let selections = MultiSelect::new()
                .with_prompt(format!("Select repos for group '{}'", group_name))
                .items(&repo_names)
                .interact()?;

            let selected_repos: Vec<String> =
                selections.iter().map(|i| repo_names[*i].clone()).collect();
            groups.insert(
                group_name.clone(),
                GroupDef {
                    repos: selected_repos,
                },
            );

            println!();

            if !Confirm::new()
                .with_prompt("Create another group?")
                .default(false)
                .interact()?
            {
                break;
            }
        }
    }

    // Build config
    let config = WorkspaceConfig {
        workspace: Workspace {
            default_branch: if default_branch.is_empty() {
                None
            } else {
                Some(default_branch)
            },
            default_skip: skip_dirs,
            default_depth: Some(opts.depth),
        },
        groups,
    };

    // Save
    config.save()?;
    println!("\n{} Created gitb.toml in current directory", "✓".green());
    println!("\nYou can now use:");
    println!("  gitb status              - view repo status");
    println!("  gitb checkout main       - switch all repos to main");
    println!("  gitb pull                - pull all repos");
    println!("  gitb doctor              - health check");
    if !config.groups.is_empty() {
        println!("  gitb -g <group> status   - run on a specific group");
    }
    println!();

    Ok(())
}

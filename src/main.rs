mod cli;
mod commands;
mod config;
mod core;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use cli::{Cli, Command};
use config::WorkspaceConfig;
use core::{discovery, GlobalOpts};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Build global options from CLI flags + config defaults
    let config = WorkspaceConfig::load().unwrap_or_default();
    let opts = build_global_opts(&cli, &config);

    // Special: completion doesn't need repo discovery
    if let Command::Completion { shell } = &cli.command {
        return run_completion(shell);
    }

    // Special: init is interactive, doesn't need repos first
    if let Command::Init = &cli.command {
        return commands::init::run(&opts);
    }

    // Special: group management doesn't need parallel execution
    if let Command::Group { action } = &cli.command {
        return commands::group::run(action.clone(), &config, &opts);
    }

    // Discover repos
    let cwd = discovery::current_dir()?;
    let mut skip_dirs = opts.skip.clone();
    // Merge config default_skip
    for s in &config.workspace.default_skip {
        if !skip_dirs.contains(s) {
            skip_dirs.push(s.clone());
        }
    }

    let mut repos = discovery::discover_repos(&cwd, opts.depth, &skip_dirs);

    if repos.is_empty() {
        eprintln!(
            "No git repositories found in current directory (depth={}).",
            opts.depth
        );
        eprintln!("Try increasing depth with -d <N> or run in a different directory.");
        std::process::exit(1);
    }

    // Filter by group if specified
    if let Some(ref group_name) = cli.group {
        repos = config.filter_repos_by_group(&repos, group_name);
        if repos.is_empty() {
            eprintln!("No repos match group '{}' in gitb.toml.", group_name);
            std::process::exit(1);
        }
    }

    // Dispatch to command handler
    match cli.command {
        Command::Checkout { branch } => commands::checkout::run(&repos, &opts, &branch),
        Command::Create { branch } => commands::create::run(&repos, &opts, &branch),
        Command::Status => commands::status::run(&repos, &opts),
        Command::Pull => commands::sync::run_pull(&repos, &opts),
        Command::Fetch => commands::sync::run_fetch(&repos, &opts),
        Command::Push => commands::sync::run_push(&repos, &opts),
        Command::Exec { args } => commands::exec::run(&repos, &opts, &args),
        Command::Branch { action } => commands::branch::run(&repos, &opts, action),
        Command::Commit { message, all } => commands::commit::run(&repos, &opts, &message, all),
        Command::Stash { action } => commands::stash::run(&repos, &opts, action),
        Command::Rebase { branch } => commands::rebase::run(&repos, &opts, branch.as_deref()),
        Command::Diff => commands::diff::run(&repos, &opts),
        Command::Log { number } => commands::log::run(&repos, &opts, number),
        Command::Doctor => commands::doctor::run(&repos, &opts),
        Command::Group { .. } => unreachable!("handled above"),
        Command::Init => unreachable!("handled above"),
        Command::Completion { .. } => unreachable!("handled above"),
    }
}

fn build_global_opts(cli: &Cli, config: &WorkspaceConfig) -> GlobalOpts {
    let depth = if cli.depth == 1 {
        config.workspace.default_depth.unwrap_or(cli.depth)
    } else {
        cli.depth
    };

    GlobalOpts {
        jobs: cli.jobs,
        dry_run: cli.dry_run,
        skip: cli.skip.clone(),
        depth,
        output: cli.output,
        verbose: cli.verbose,
        quiet: cli.quiet,
        force: cli.force,
    }
}

fn run_completion(shell: &str) -> anyhow::Result<()> {
    let shell = shell.to_lowercase();
    let shell = match shell.as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            eprintln!(
                "Unsupported shell: {}. Supported: bash, zsh, fish, powershell, elvish",
                shell
            );
            std::process::exit(1);
        }
    };

    let mut cmd = cli::Cli::command();
    generate(shell, &mut cmd, "gitb", &mut std::io::stdout());
    Ok(())
}

# gitb — Blazing-fast multi-repo git batch tool

[![CI](https://github.com/luolin1024/git-batch/actions/workflows/ci.yml/badge.svg)](https://github.com/luolin1024/git-batch/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gitb.svg)](https://crates.io/crates/gitb)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![rust: 1.74+](https://img.shields.io/badge/rust-1.74%2B-orange.svg)](https://www.rust-lang.org)

> Run git across 100+ repos in parallel. Rust-powered, 3-4× faster than `gita` / `mr`.
> 用 Rust 写的多仓库 Git 批量工具，比 gita / mr 快 3-4 倍。

![demo](docs/assets/gitb-demo.png)

## Why gitb?

- **Fast** — parallel execution via rayon. 100 repos `status` in ~2s. [See benchmark →](#performance)
- **Zero-config** — works out of the box. `cd` into a folder of repos, run `gitb status`.
- **Cross-platform** — macOS, Linux, Windows. Shell completion for bash / zsh / fish / powershell.

## Install

**macOS Homebrew:**

```bash
brew install luolin1024/git-batch/gitb
```

**Linux / macOS Cargo:**

```bash
cargo install gitb
```

**Windows PowerShell:**

```powershell
irm https://github.com/luolin1024/git-batch/raw/main/install.ps1 | iex
```

**Windows Scoop:**

```powershell
scoop bucket add gitb https://github.com/luolin1024/git-batch
scoop install gitb
```

Verify:

```bash
gitb --version
```

<details>
<summary>Build from source</summary>

```bash
git clone https://github.com/luolin1024/git-batch.git
cd git-batch
cargo build --release
# Binary at ./target/release/gitb
```
</details>

<details>
<summary>Manual download</summary>

Download the right file from [GitHub Releases](https://github.com/luolin1024/git-batch/releases/latest):

| Platform | File |
|---|---|
| Windows x86_64 | `gitb.exe` |

Windows: place `gitb.exe` anywhere in your `PATH`.
</details>

## Quick start (30 seconds)

```bash
cd ~/Work          # a folder containing many git repos
gitb status        # see all repos at a glance
gitb pull -j 8     # pull all in parallel
gitb doctor        # health check: who's behind / dirty / unpushed
```

## Features

| Command      | Description (EN)                               | 中文描述                          |
|-------------|-----------------------------------------------|-----------------------------------|
| `checkout`  | Switch to a branch across all repos (fuzzy)   | 在所有仓库中切换分支（支持模糊匹配）|
| `create`    | Create and switch to a new branch             | 创建并切换到新分支                 |
| `status`    | Show colored multi-repo status overview       | 显示多仓库状态概览（彩色）         |
| `pull`      | Pull from remote across all repos             | 拉取远程更新                       |
| `fetch`     | Fetch from remote across all repos            | 获取远程引用                       |
| `push`      | Push to remote across all repos               | 推送至远程                         |
| `exec`      | Execute arbitrary git commands                 | 执行任意 git 命令                  |
| `branch`    | List, delete branches across repos            | 列出/删除分支                      |
| `commit`    | Commit changes across all repos               | 提交更改                           |
| `stash`     | Stash/pop/list/clear operations               | 暂存/弹出/列出/清除               |
| `rebase`    | Smart rebase (stash -> rebase -> unstash)     | 智能变基                           |
| `diff`      | Show diff across all repos                    | 显示差异                           |
| `log`       | Show commit log across all repos              | 显示提交日志                       |
| `doctor`    | Health check (ahead/behind/dirty/unpushed)    | 健康检查                           |
| `group`     | Manage repo groups                            | 管理仓库分组                       |
| `init`      | Initialize workspace config interactively      | 交互式初始化工作区配置             |
| `completion`| Generate shell completion scripts             | 生成 Shell 补全脚本                |

## Usage

### Basic Commands

```bash
gitb status                    # show status of all repos
gitb pull -j 8                 # pull all in parallel (8 jobs)
gitb checkout main             # switch to branch (fuzzy match)
gitb create feature/new        # create and switch to new branch
gitb fetch                     # fetch from remote
gitb push                      # push to remote
gitb exec log --oneline -5     # run any git command
gitb commit -m "fix: update"   # commit changes
gitb stash push                 # stash changes
gitb stash pop                  # pop stash
gitb rebase                     # smart rebase (auto-stash)
gitb rebase -b main             # rebase onto specific branch
gitb diff                       # show diff
gitb log -n 10                  # show last 10 commits per repo
gitb branch list                # list branches
gitb branch delete old-feature  # delete branch (-f force, --remote also remote)
gitb doctor                     # health check
```

### Group Management

```bash
gitb group add frontend repo-a,repo-b,repo-c
gitb group list
gitb group show frontend
gitb group remove frontend
gitb status -g frontend          # filter by group
gitb pull -g frontend
```

### Workspace Initialization

```bash
gitb init    # interactive setup, generates gitb.toml
```

### Shell Completion

```bash
gitb completion bash > ~/.bash_completion.d/gitb
gitb completion zsh > /usr/local/share/zsh/site-functions/_gitb
gitb completion fish > ~/.config/fish/completions/gitb.fish
gitb completion powershell >> $PROFILE
```

## Global Options

| Flag            | Description (EN)                            | 中文描述                    |
|----------------|--------------------------------------------|----------------------------|
| `-j N`         | Number of parallel jobs (0 = auto-detect)  | 并行任务数（0 为自动检测）  |
| `--dry-run`    | Show what would happen without executing   | 模拟运行，不实际执行        |
| `-s <dirs>`    | Skip directories (comma-separated)         | 跳过指定目录（逗号分隔）    |
| `-d <depth>`   | Max recursion depth for repo discovery     | 仓库发现最大递归深度        |
| `-o <format>`  | Output format: table, json, quiet          | 输出格式                    |
| `-f`           | Force operation (discard uncommitted)      | 强制操作（丢弃未提交更改）  |
| `-v`           | Verbose output                             | 详细输出                    |
| `-q`           | Quiet mode (only show errors)              | 静默模式（仅显示错误）      |
| `-g <group>`   | Filter repos by group name                 | 按分组过滤仓库              |

## Configuration (gitb.toml)

Optional — gitb works with zero config. Place `gitb.toml` in your workspace root:

```toml
[workspace]
default_branch = "main"
default_skip = ["node_modules", "target"]
default_depth = 2

[groups.frontend]
repos = ["web-app", "mobile-app", "ui-kit"]

[groups.backend]
repos = ["api-gateway", "user-service", "payment-service"]
```

## Performance

| Tool          | Language | 50 repos (pull) | 100 repos (status) |
|---------------|----------|-----------------|--------------------|
| gitb          | Rust     | ~1.2s           | ~2.1s              |
| gita          | Python   | ~4.5s           | ~8.9s              |
| myrepos (mr)  | Perl     | ~3.8s           | ~7.2s              |

*Benchmarks measured on an 8-core machine with SSD. Your results may vary.*

## Comparison with alternatives

| Tool | Language | Parallel | Zero-config | Cross-platform | Shell completion | Smart rebase |
|------|----------|----------|-------------|----------------|-----------------|--------------|
| **gitb** | Rust | ✅ rayon | ✅ | ✅ macOS/Linux/Windows | ✅ bash/zsh/fish/pwsh | ✅ stash→rebase→unstash |
| [gita](https://github.com/nosarthur/gita) | Python | ✅ | ✅ | ✅ | partial | ❌ |
| [myrepos (mr)](https://myrepos.branchable.com/) | Perl | ✅ | ❌ config file | ✅ | ❌ | ❌ |
| [mu-repo](https://github.com/fabioz/mu-repo) | Python | ✅ | ✅ | ✅ | ❌ | ❌ |

## FAQ

**Q: `gitb` not found after install?**
A: Restart your terminal so PATH changes take effect.

**Q: Can't find my repos?**
A: Default scan depth is 1. Use `-d 2` or `-d 3` for nested directories.

**Q: Not sure what a command will do?**
A: Add `--dry-run` to preview without executing.

**Q: Skip certain directories?**
A: `-s node_modules,target,.vscode` (comma-separated).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). PRs welcome — `cargo fmt` and `cargo clippy` must pass.

## License

MIT

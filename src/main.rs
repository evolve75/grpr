/*
 * grpr - A CLI tool for recursively executing git commands.
 *
 * Copyright (c) 2025 Anupam Sengupta
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use clap::Parser;
use rayon::prelude::*;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};

mod grpgit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// CLI represents the command-line arguments for grpr.
#[derive(Parser, Debug)]
#[command(author, version = VERSION, about, long_about = None)]
struct Cli {
    /// The number of threads to use for concurrent processing. When omitted,
    /// grpr scans and processes repositories sequentially for predictable
    /// output and compatibility with grp.
    #[arg(
        short,
        long,
        help = "Opt in to parallel execution with the given number of worker threads"
    )]
    threads: Option<usize>,

    /// The git command and its arguments to execute (e.g., "pull", "status",
    /// etc.). Defaults to "status" if not provided.
    #[arg(required = false, num_args = 1.., trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

/// Extracts the git command from the CLI arguments.
fn git_command_from_cli(cli: &Cli) -> Vec<String> {
    if cli.command.is_empty() {
        vec!["status".to_string()]
    } else {
        cli.command.clone()
    }
}

/// Executes a git command across the discovered repositories. Processing is
/// sequential by default and becomes parallel only when a thread count is
/// provided.
fn execute_repositories(
    repositories: &[PathBuf],
    git_args: &[String],
    threads: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    if let Some(thread_count) = threads.filter(|count| *count > 1) {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()?;

        pool.install(|| {
            repositories.par_iter().for_each(|repo_path| {
                if let Err(err) = grpgit::process_repository(repo_path, git_args) {
                    eprintln!("{err}");
                }
            });
        });
    } else {
        for repo_path in repositories {
            if let Err(err) = grpgit::process_repository(repo_path, git_args) {
                eprintln!("{err}");
            }
        }
    }

    Ok(())
}

fn discover_repositories_from(current_dir: &Path) -> Vec<PathBuf> {
    grpgit::discover_repositories(current_dir)
}

/// Main function initializes the program, parses CLI arguments, discovers git
/// repositories, and executes the requested git command in each one.
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let git_args = git_command_from_cli(&cli);
    let current_dir = env::current_dir()?;
    let repositories = discover_repositories_from(current_dir.as_path());

    execute_repositories(&repositories, &git_args, cli.threads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};
    use std::fs;
    use tempfile::tempdir;

    fn create_regular_repo(path: &Path) {
        let git_dir = path.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "[core]\n").unwrap();
    }

    #[test]
    fn git_command_defaults_to_status() {
        let cli = Cli::parse_from(["grpr"]);

        assert_eq!(git_command_from_cli(&cli), vec!["status"]);
    }

    #[test]
    fn git_command_preserves_multiple_arguments() {
        let cli = Cli::parse_from(["grpr", "log", "--oneline", "--graph"]);

        assert_eq!(
            git_command_from_cli(&cli),
            vec!["log", "--oneline", "--graph"]
        );
    }

    #[test]
    fn cli_version_matches_cargo_package_version() {
        assert_eq!(VERSION, "2.0.0");
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn clap_renders_expected_version_string() {
        let rendered = Cli::command().render_version().to_string();

        assert_eq!(rendered.trim(), "grpr 2.0.0");
    }

    #[test]
    fn discover_repositories_from_finds_root_level_repositories() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();
        create_regular_repo(&repo_dir);

        let repositories = discover_repositories_from(dir.path());

        assert_eq!(repositories, vec![repo_dir]);
    }

    #[test]
    fn execute_repositories_succeeds_with_sequential_processing() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        let status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&repo_dir)
            .status()
            .unwrap();
        assert!(status.success());

        let repositories = vec![repo_dir];
        let git_args = vec!["status".to_string()];

        assert!(execute_repositories(&repositories, &git_args, None).is_ok());
    }
}

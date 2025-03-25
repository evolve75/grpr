/*
 * grpr - A CLI tool for recursively executing git commands.
 *
 * Copyright (c) 2025 Anupam Sengupta
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use clap::Parser;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use std::env;
use std::error::Error;
use std::path::Path;
use walkdir::WalkDir;

mod grpgit;

/// CLI represents the command-line arguments for grpr.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The number of threads to use for concurrent processing (default: system default,
    /// i.e. number of logical CPUs).
    #[arg(
        short,
        long,
        help = "The number of threads to use for concurrent processing (default: system default, i.e. number of logical CPUs)"
    )]
    threads: Option<usize>,

    /// The git command and its arguments to execute (e.g., "pull", "status", etc.).
    /// Defaults to "status" if not provided.
    #[arg(required = false, num_args = 1..)]
    command: Vec<String>,
}

/// Sets up the Rayon thread pool if a thread count is provided.
fn setup_thread_pool(threads: Option<usize>) -> Result<(), Box<dyn Error>> {
    if let Some(t) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()?;
    }
    Ok(())
}

/// Extracts the git command from the CLI arguments.
/// Defaults to "status" if no command is provided.
fn get_command_from_cli(cli: &Cli) -> String {
    if cli.command.is_empty() {
        "status".to_string()
    } else {
        cli.command.join(" ")
    }
}

/// Processes repositories found under `current_dir` using the provided `git_processor`
/// function concurrently.
///
/// The function uses a generic parameter `F` to ensure that `git_processor` implements
/// both `Fn(&Path) -> Result<(), String>` and `Sync`.
fn process_repositories<F>(current_dir: &Path, git_processor: &F)
where
    F: Fn(&Path) -> Result<(), String> + Sync,
{
    WalkDir::new(current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| grpgit::is_git_repo(entry.path()))
        .par_bridge()
        .for_each(|entry| {
            let path = entry.path();
            println!("Processing Git repository: {}", path.display());
            if let Err(err) = grpgit::process_git_dir(path, git_processor) {
                eprintln!("Error processing {}: {}", path.display(), err);
            }
        });
}

/// Main function initializes the program, parses CLI arguments, sets up the thread pool,
/// and concurrently processes directories to execute a Git command in each Git repository.
fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments using Clap.
    let cli = Cli::parse();

    // Set up the Rayon thread pool if needed.
    setup_thread_pool(cli.threads)?;

    // Determine the git command from CLI.
    let command = get_command_from_cli(&cli);

    // Get the current working directory.
    let current_dir = env::current_dir()?;

    // Create a processor closure that will run the Git command.
    let git_processor = grpgit::create_git_processor(command);

    // Process repositories concurrently.
    process_repositories(current_dir.as_path(), &git_processor);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_command_from_cli_default() {
        let cli = Cli::parse_from(&["grpr"]);
        let cmd = get_command_from_cli(&cli);
        assert_eq!(cmd, "status");
    }

    #[test]
    fn test_get_command_from_cli_join() {
        let cli = Cli::parse_from(&["grpr", "pull", "origin", "master"]);
        let cmd = get_command_from_cli(&cli);
        assert_eq!(cmd, "pull origin master");
    }

    #[test]
    fn test_setup_thread_pool_default() {
        // If no thread count is provided, the setup should succeed without error.
        assert!(setup_thread_pool(None).is_ok());
    }

    #[test]
    fn test_setup_thread_pool_custom() {
        // Providing a thread count should also succeed.
        assert!(setup_thread_pool(Some(1)).is_ok());
    }

    #[test]
    fn test_process_repositories() {
        // Create a temporary directory structure with a fake git repository.
        let temp_dir = tempdir().unwrap();
        let repo_dir = temp_dir.path().join("fake_repo");
        fs::create_dir_all(&repo_dir).unwrap();
        // Create a dummy .git directory inside fake_repo.
        fs::create_dir_all(repo_dir.join(".git")).unwrap();

        // Create a dummy git_processor that always returns Ok.
        let dummy_processor = |_: &Path| -> Result<(), String> { Ok(()) };

        // Run process_repositories; if no panic occurs, assume success.
        process_repositories(temp_dir.path(), &dummy_processor);
    }
}

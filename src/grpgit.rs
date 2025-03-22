/*
 * grpr - A CLI tool for recursively executing git commands.
 *
 * Copyright (c) 2025 Anupam Sengupta
 *
 * This source code is licensed under the MIT license found in the LICENSE file
 * in the root directory of this source tree.
 *
 * Summary:
 * This file (grpgit.rs) contains helper functions and type definitions for
 * interacting with Git repositories. It provides functionality to check if a
 * directory is a Git repository, execute Git commands within a repository,
 * process directories based on whether they are Git repositories, and create
 * closures to process Git commands in a modular fashion.
 */

use std::path::Path;
use std::process::{Command, Stdio};

/// Type alias for a Git command, represented as a string. This can be
/// something like "status", "pull", etc.
pub type GitCommand = String;

/// Checks whether the given path is a Git repository by verifying the existence
/// of a ".git" directory.
///
/// # Arguments
///
/// * `path` - The path to check.
///
/// # Returns
///
/// * `true` if the ".git" directory exists in the given path.
/// * `false` otherwise.
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").is_dir()
}

/// Executes a Git command in the provided repository path.
///
/// The function splits the command into the Git subcommand and its arguments,
/// executes it in the given directory, and prints the output to stdout and
/// stderr.
///
/// # Arguments
///
/// * `repo_path` - The path of the Git repository.
/// * `command` - The Git command to execute (e.g., "pull", "status").
///
/// # Returns
///
/// * `Ok(())` if the command executed successfully.
/// * `Err(String)` if there was an error.
pub fn run_git_command(repo_path: &Path, command: &str) -> Result<(), String> {
    // Split the command string into the subcommand and arguments.
    let mut parts = command.split_whitespace();
    let subcommand = parts.next().ok_or("Empty git command")?;
    let args: Vec<&str> = parts.collect();

    // Execute the git command in the specified repository directory.
    let output = Command::new("git")
        .arg(subcommand)
        .args(&args)
        .current_dir(repo_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    // Check if the command executed successfully.
    if !output.status.success() {
        return Err(format!("Git command failed in {}", repo_path.display()));
    }

    Ok(())
}

/// Processes a directory: if it is a Git repository, the provided processor
/// function is executed.
///
/// # Arguments
///
/// * `path` - The directory path to process.
/// * `processor` - A function that takes a path and returns a result.
///
/// # Returns
///
/// * `Ok(())` if processing was successful or if the directory is not a Git
///   repository.
/// * `Err(String)` if there was an error during processing.
pub fn process_git_dir(
    path: &Path,
    processor: &impl Fn(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if is_git_repo(path) {
        processor(path)
    } else {
        Ok(())
    }
}

/// Creates and returns a closure that executes the provided Git command in a
/// given repository path.
///
/// # Arguments
///
/// * `command` - The Git command to execute.
///
/// # Returns
///
/// * A closure that takes a path and returns a result after executing the Git
///   command.
pub fn create_git_processor(command: GitCommand) -> impl Fn(&Path) -> Result<(), String> {
    move |repo_path: &Path| -> Result<(), String> { run_git_command(repo_path, &command) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_is_git_repo() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        // Initially, no .git directory exists.
        assert!(!is_git_repo(&path));

        // Create a .git directory and test again.
        fs::create_dir_all(path.join(".git")).unwrap();
        assert!(is_git_repo(&path));
    }

    #[test]
    fn test_process_git_dir_without_git() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        // Dummy processor that always returns Ok.
        let processor = |_: &Path| -> Result<(), String> { Ok(()) };
        // Since no .git directory exists, process_git_dir should simply return Ok.
        assert!(process_git_dir(&path, &processor).is_ok());
    }

    #[test]
    fn test_process_git_dir_with_git() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        // Create a .git directory.
        fs::create_dir_all(path.join(".git")).unwrap();

        // Dummy processor that returns Ok.
        let processor = |_: &Path| -> Result<(), String> { Ok(()) };
        assert!(process_git_dir(&path, &processor).is_ok());
    }

    #[test]
    fn test_create_git_processor_runs_command() {
        // We use a known git command. `git --version` should work in any directory.
        let processor = create_git_processor("--version".to_string());
        // Even though current directory might not be a git repo, `git --version`
        // works globally.
        let result = processor(Path::new("."));
        assert!(result.is_ok());
    }
}

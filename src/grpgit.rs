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
        return Err(format!("Git command failed in {}",
                           repo_path.display()));
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
    move |repo_path: &Path| -> Result<(), String> {
        run_git_command(repo_path, &command)
    }
}

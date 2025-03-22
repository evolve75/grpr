/*
 * grpr - A CLI tool for recursively executing git commands.
 *
 * Copyright (c) 2025 Anupam Sengupta
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod grpgit;

use std::env;
use walkdir::WalkDir;

/// Main function that initializes the program, parses the command-line arguments,
/// and recursively processes directories to execute a Git command in each Git
/// repository.
fn main() {
    // Collect command-line arguments.
    let args: Vec<String> = env::args().collect();

    // If -h or --help is provided as the first argument, display the help message.
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_help();
        return;
    }

    // Parse the Git command from the provided arguments.
    let git_command = parse_git_command(&args);

    // Get the current working directory.
    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Create a processor closure to run the Git command.
    let git_processor = grpgit::create_git_processor(git_command);

    // Recursively walk through directories starting at the current directory.
    for entry in WalkDir::new(current_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let path = entry.path();
        // Process the directory if it is a Git repository.
        if let Err(err) = grpgit::process_git_dir(path, &git_processor) {
            eprintln!("Error processing {}: {}", path.display(), err);
        }
    }
}

/// Parses the Git command from the command-line arguments. If no command is
/// provided, it defaults to "status".
///
/// # Arguments
///
/// * `args` - The list of command-line arguments.
///
/// # Returns
///
/// * A string representing the Git command to execute.
fn parse_git_command(args: &[String]) -> String {
    if args.len() > 1 {
        // Join all arguments after the executable name.
        args[1..].join(" ")
    } else {
        "status".to_string()
    }
}

/// Prints a short help message for grpr.
fn print_help() {
    println!(
        r#"grpr - A CLI tool for recursively executing git commands.

Usage:
    grpr [git-command]

If no git-command is provided, 'status' is used by default.

Arguments:
    -h, --help   Show this help message.

Example:
    grpr pull

For a list of available git sub-commands, please visit:
    https://git-scm.com/docs
"#
    );
}

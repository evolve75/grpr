/*
 * grpr - A CLI tool for recursively executing git commands.
 *
 * Copyright (c) 2025 Anupam Sengupta
 *
 * This source code is licensed under the MIT license found in the LICENSE file
 * in the root directory of this source tree.
 */

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

const GIT_PATH_NAME: &str = ".git";
const GIT_CONFIG_NAME: &str = "config";
const GITDIR_PREFIX: &str = "gitdir:";

/// Classifies the git repository type discovered at a directory path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryKind {
    Regular,
    Worktree,
}

/// Detects whether `path` is a supported git repository root.
///
/// Regular repositories must contain a `.git/config` file. Worktrees are
/// identified by a `.git` file whose trimmed contents start with `gitdir:`.
pub fn detect_repository(path: &Path) -> Option<RepositoryKind> {
    if !path.is_dir() {
        return None;
    }

    let git_path = path.join(GIT_PATH_NAME);
    let git_metadata = fs::metadata(&git_path).ok()?;

    if git_metadata.is_dir() {
        let config_path = git_path.join(GIT_CONFIG_NAME);
        return config_path.is_file().then_some(RepositoryKind::Regular);
    }

    if git_metadata.is_file() {
        let contents = fs::read_to_string(&git_path).ok()?;
        return contents
            .trim_start()
            .starts_with(GITDIR_PREFIX)
            .then_some(RepositoryKind::Worktree);
    }

    None
}

/// Discovers git repositories under `root`, skipping descendants of any
/// repository that is found.
pub fn discover_repositories(root: &Path) -> Vec<PathBuf> {
    let mut repositories = Vec::new();
    let mut walker = WalkDir::new(root).into_iter();

    while let Some(entry_result) = walker.next() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Error walking directory tree: {err}");
                continue;
            }
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        if detect_repository(entry.path()).is_some() {
            repositories.push(entry.into_path());
            walker.skip_current_dir();
        }
    }

    repositories
}

/// Executes a git command in the provided repository path.
pub fn run_git_command(repo_path: &Path, args: &[String]) -> Result<(), io::Error> {
    let status = Command::new("git")
        .args(args.iter().map(OsStr::new))
        .current_dir(repo_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "git command failed in {} with status {status}",
            repo_path.display()
        )))
    }
}

/// Prints the repository being processed and runs the git command in it.
pub fn process_repository(repo_path: &Path, args: &[String]) -> Result<(), io::Error> {
    println!("Inside git repo: {}", repo_path.display());
    run_git_command(repo_path, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn create_regular_repo(path: &Path) {
        let git_dir = path.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "[core]\n").unwrap();
    }

    #[test]
    fn detect_repository_identifies_valid_regular_repo() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();
        create_regular_repo(&repo_dir);

        assert_eq!(detect_repository(&repo_dir), Some(RepositoryKind::Regular));
    }

    #[test]
    fn detect_repository_rejects_missing_config() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("repo");
        fs::create_dir_all(repo_dir.join(".git")).unwrap();

        assert_eq!(detect_repository(&repo_dir), None);
    }

    #[test]
    fn detect_repository_identifies_valid_worktree() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("worktree");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join(".git"),
            "gitdir: /path/to/repo/.git/worktrees/topic\n",
        )
        .unwrap();

        assert_eq!(detect_repository(&repo_dir), Some(RepositoryKind::Worktree));
    }

    #[test]
    fn detect_repository_rejects_invalid_worktree_file() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("worktree");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join(".git"), "not a gitdir reference\n").unwrap();

        assert_eq!(detect_repository(&repo_dir), None);
    }

    #[test]
    fn detect_repository_rejects_empty_worktree_file() {
        let dir = tempdir().unwrap();
        let repo_dir = dir.path().join("worktree");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(repo_dir.join(".git"), "").unwrap();

        assert_eq!(detect_repository(&repo_dir), None);
    }

    #[test]
    fn detect_repository_rejects_file_paths() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not-a-dir");
        fs::write(&file_path, "test").unwrap();

        assert_eq!(detect_repository(&file_path), None);
    }

    #[test]
    fn detect_repository_rejects_missing_paths() {
        let dir = tempdir().unwrap();
        let missing_path = dir.path().join("missing");

        assert_eq!(detect_repository(&missing_path), None);
    }

    #[test]
    fn discover_repositories_skips_descendants_of_found_repositories() {
        let dir = tempdir().unwrap();
        let parent_repo = dir.path().join("parent");
        let nested_repo = parent_repo.join("nested");
        let sibling_repo = dir.path().join("sibling");

        fs::create_dir_all(&nested_repo).unwrap();
        fs::create_dir_all(&sibling_repo).unwrap();
        create_regular_repo(&parent_repo);
        create_regular_repo(&nested_repo);
        create_regular_repo(&sibling_repo);

        let mut discovered = discover_repositories(dir.path());
        discovered.sort();

        assert_eq!(discovered, vec![parent_repo, sibling_repo]);
    }

    #[test]
    fn discover_repositories_handles_root_repository() {
        let dir = tempdir().unwrap();
        create_regular_repo(dir.path());
        let nested_repo = dir.path().join("nested");
        fs::create_dir_all(&nested_repo).unwrap();
        create_regular_repo(&nested_repo);

        let discovered = discover_repositories(dir.path());

        assert_eq!(discovered, vec![dir.path().to_path_buf()]);
    }

    #[test]
    fn run_git_command_accepts_multi_argument_commands() {
        let dir = tempdir().unwrap();
        let status = Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success());

        let args = vec!["status".to_string(), "--short".to_string()];
        assert!(run_git_command(dir.path(), &args).is_ok());
    }
}

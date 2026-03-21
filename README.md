# grpr: The Recursive git CLI

`grpr` (_git recursive program, Rust edition_) is a CLI tool for executing
`git` commands recursively across the current directory and nested repositories.
It is a Rust port of the Go-based [`grp`](../grp), with the goal of matching
its repository discovery and traversal behavior while keeping the implementation
idiomatic to Rust.

Like `grp`, `grpr` is a thin wrapper around the real `git` executable. It
handles directory traversal and repository detection, then delegates the actual
VCS operation to `git`.

## Requirements

- Rust 1.56 or later for building from source
- `git` must be available in your `PATH`

## Installation

### Build from source

```bash
cargo build --release
```

The executable will be available at:

```bash
target/release/grpr
```

### Install with Cargo

```bash
cargo install --path . --root ~
```

This installs `grpr` into `~/bin/grpr`. Make sure `~/bin` is in your `PATH`:

```bash
export PATH="$HOME/bin:$PATH"
```

## Usage

`grpr` is designed to be a drop-in replacement for many `git` commands. For
example:

```bash
grpr pull
```

instead of:

```bash
git pull
```

If the current directory is itself a git repository, `grpr` runs the command
there. Otherwise, it recursively searches child directories for repositories
and runs the command in each one it finds.

If no command is specified, `grpr status` is executed by default.

### Examples

```bash
# Check the status of all repositories in the current directory tree
grpr status

# Pull latest changes for all repositories
grpr pull

# Fetch from all remotes for all repositories
grpr fetch

# Show the last commit for all repositories
grpr log -1

# Pass through git flags and arguments directly
grpr log --oneline --graph
```

### Parallel execution

By default, `grpr` processes repositories sequentially to match `grp`'s
behavior and keep output predictable.

`grpr` also provides one Rust-specific extension:

```bash
grpr --threads 8 status
```

This enables parallel execution after repository discovery has completed. The
same repository detection and traversal rules still apply.

## Git Worktree Support

`grpr` supports both standard git repositories and git worktrees.

- Regular repositories are recognized when `.git/config` exists.
- Git worktrees are recognized when `.git` is a file whose contents begin with
  `gitdir:`.

This allows `grpr` to operate correctly in directories containing linked
worktrees, not just traditional repository roots.

## Git Submodules and Nested Repositories

`grpr` stops descending once it finds a repository root. This matches the
current `grp` behavior and has two important consequences:

- Git submodules inside a discovered repository are skipped automatically.
- Nested repositories inside a discovered repository are not processed during
  that same walk.

This avoids duplicate operations and respects repository boundaries. To operate
on a submodule directly, run `grpr` from inside that submodule directory.

## How It Works

`grpr` works in three stages:

1. Start from the current working directory.
2. Walk the directory tree and identify repository roots.
3. Execute the requested `git` command in each discovered repository.

Once a repository is found, `grpr` skips its descendants. This is what keeps
submodules and nested repositories from being visited during the same traversal.

If a git command fails in one repository, `grpr` reports the error and
continues processing other repositories.

## Development

### Build and test

```bash
# Build a development binary
cargo build

# Run the CLI without a separate install step
cargo run -- status

# Run the test suite
cargo test
```

The CLI version reported by `grpr --version` comes from `Cargo.toml`
`package.version`. Release tags should mirror that version using the existing
`v<major>.<minor>.<patch>` format.

### Project structure

- `src/main.rs`: CLI parsing and repository execution orchestration
- `src/grpgit.rs`: Repository detection, traversal, and git command execution

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE)
file for details.

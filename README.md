# grpr: The Recursive git CLI

`grpr` (_git recursive program_) is a CLI program for executing `git` VCS
operations recursively on the current _and_ nested directories from the shell
prompt. It is a simple wrapper around the actual `git` executable (which _must_
be present) and primarily handles the directory tree navigation, leaving the
execution of the commands to the `git` program.

The rationale for `grpr` is to quickly apply the same `git` commands across a
directory tree of `git` repositories (for example: a set of related code
directories in a micro-services application). While this can certainly be
achieved using shell programming such as `for` loops, `grpr` makes the process
more consistent and repeatable, and is also usable in larger scripts (including
CI/CD pipelines).

## Build and Installation

You have two main options to build and install `grpr`:

### 1. Using Cargo Install with a Custom Root

You can build and install `grpr` directly to a specific directory (such as 
`~/bin`) by running the following command from the project root:

```bash
cargo install --path . --root ~
```

This command will place the executable in `~/bin/grpr`. Ensure that your
`~/bin` directory is included in your PATH. For example, add the following line
to your shell configuration file (e.g., `~/.bashrc` or `~/.zshrc`):

```bash
export PATH="$HOME/bin:$PATH"
```

### 2. Build and Copy Manually

Alternatively, you can build the project in release mode and then copy the
executable to your desired directory:

```bash
cargo build --release
cp target/release/grpr ~/bin
```

## Usage

`grpr` is easy to use and can basically be used as a drop-in replacement to
`git` (which does need to _still_ exist on the machine!). For example, one can
use:

    grpr pull

instead of

    git pull

with similar effect. The **main** difference is that `grpr` will execute the
command in the current directory (if it is a `git` repository), **or** attempt
to execute the command in any child directories under the current directory (if
it is **not** a `git` repository).

In addition, this behavior is recursive: `grpr` will recursively navigate the
directory tree under the current directory and execute the `git` command in all
directories which are version controlled using `git`.

[//]: # (TODO: Define the behavior for git repos with modules. `grpr` probably
[//]: # should ignore the module)

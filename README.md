# grpr: The Recursive git CLI

`grpr` (_git recursive program_) is a CLI program for executing `git`
VCS operations recursively on the current _and_ nested directories from the
shell prompt. It is a simple wrapper around the actual `git` executable
(which _must_ be present) and primarily handles the directory tree
navigation, leaving the execution of the commands to the `git` program.

The rationale for `grpr` is to quickly apply the same `git` commands across a
directory tree of `git` repositories (for example: a set of related code
directories in a micro-services application). While this can certainly be
achieved using shell programming such as `for` loops, `grpr` makes the
process more consistent and repeatable, and is also usable in larger scripts
(including CI/CD pipelines).

## Usage

`grpr` is easy to use and can basically be used as a drop-in replacement to
`git` (which does need to _still_ exist on the machine!). For example, one can
use:

    grpr pull

instead of

    git pull

with similar effect. The **main** difference is that `grpr` will execute the
command in the current directory (if it is a `git` repository), **or**
attempt to execute the command in any child directories under the current
directory (if it is **not** a `git` repository).

In addition, this behavior is recursive: `grpr` will recursively navigate the
directory tree under the current directory and execute the `git` command in all
directories which are version controlled using `git`.

[//]: # (TODO: Define the behavior for git repos with modules. `grpr`
probably _should_ ignore the module)

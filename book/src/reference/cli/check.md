---
editLink: false
lastUpdated: false
---

# mabo check

- Aliases: `c`

Check that a project or set of files are valid schemas.

This involves first checking each schema individually to be parseable, then run various lints over it and finally resolve any external schema types.  In case any of the steps fail an error will be returned.

## Arguments

### `FILES`

Loose list of glob patterns for files that should be checked instead of a project.

Using this will disable the loading of a project and instead locate the files from the glob patterns, then treat them as one single set. The files will be treated as a single project but the `Mabo.toml` file is fully ignored.

## Options

### `--project-dir`

Alternative location of the project directory containing a `Mabo.toml` file.

By default, the current directory is assumed to be the project directory. This is the root from where the command operates. Therefore, using it has the same effect as moving to the project directory and executing the command without it.

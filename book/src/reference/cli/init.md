---
editLink: false
lastUpdated: false
---

# mabo init

- Aliases: `i`, `initialize`

Initialize a new project.

## Arguments

### `PATH`

Alternative path were the project should be generated.

By default, the current directory is assumed to be the to-be project directory. This is where the `Mabo.toml` file will be placed to mark it as project. Therefore, the path must point to a directory, not a file.

## Options

### `--name`

Name of the project. If omitted, the name is derived from the current working directory.

This is used as the project name in the `Mabo.toml` file to give it a unique identifier. It must only be unique within the project (in case it has multiple projects).

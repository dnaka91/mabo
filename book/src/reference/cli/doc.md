---
editLink: false
lastUpdated: false
---

# mabo doc

- Aliases: `d`, `document`

Generate documentation for a project.

## Arguments

### `OUT_DIR`

Directory where the documentation files are written to.

Note that this directory will be overwritten without confirmation. Any existing files will be replaced, but the directory is not fully cleared beforehand. That means any unrelated files not having the same name as any documentation file, will remain.

## Options

### `--project-dir`

Alternative location of the project directory containing a `Mabo.toml` file.

By default, the current directory is assumed to be the project directory. This is the root from where the command operates. Therefore, using it has the same effect as moving to the project directory and executing the command without it.

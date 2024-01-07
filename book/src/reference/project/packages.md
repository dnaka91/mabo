# Packages

The `[package]` section defines a single collection of Mabo schema files. It defines additional metadata that will be useful in future features that involve sharing and consumption of schema collections.

## `name`

- Type `string`
- **Required**

The package name is an identifier used to refer to the package.

## `description`

- Type `string`
- Optional

The description is a short explanation about content of the package.

Usually not too extensive, as the schema files can contain their own root-level description which will be carried over into the generated source code.

## `license`

- Type `string`
- Optional

The license under which a package is distributed. This is either a single [SPDX](https://spdx.org/) license or an expression that combines multiple licenses.

Expressions are usually `AND` and `OR` which define how multiple licenses are to be combined. Further details about all the features of this expression language are available in the [SPDX Specification](https://spdx.github.io/spdx-spec/v2.3/).

This is a simple example of the common combination of the `MIT` and `Apache-2.0` license:

```toml
[package]
# ...
license = "MIT OR Apache-2.0"
```

## `files`

- Type `array<string>`
- **Required**

The list of files that form the collection of schemas making up the package content. These are [Glob patterns](https://en.wikipedia.org/wiki/Glob_(programming)) which allow for easy inclusion of whole file trees.

For example, the likely most common pattern would be `schemas/**/*.mabo`. This pattern recurses into all the sub-directories of the `schemas` folder look for all files with the `.mabo` extension.

::: info
Regardless of the glob patterns defined, files are filtered by the `.mabo` extension in the end, as these are the only files considered valid schema files.
:::

This is an example described above as part of the `Mabo.toml` file:

```toml
[package]
# ...
files = ["schemas/**/*.mabo"]
```

---
order: 8
---

# Imports

When splitting data structures into multiple Schema files, imports are used to reference the types defined within. Imports are declared with the `use` statement.

These come in two flavors:

- Import a specific type.

  <<< imports/struct.mabo

- Import only a module.

  <<< imports/module.mabo

Individual elements forming the import path are separated by a double-colon `::`. The first element is name of the external schema, and all intermediate elements are modules.

The last element is either omitted, which results in bringing the whole module into scope, or a specific type in that module or root schema.

Importing a module can help to reduce repetition if the module path is deep. Another use case is the avoidance of duplicate type names. For example:

## Scoping example

A simple example with a somewhat deeply nested module structure, which we'll use in the next step. Let's consider this file to be named `other.mabo`.

<<< imports/other.mabo

When importing the above schema, we bring the `addresses` module into scope and can reference all the contained types:

<<< imports/scoping.mabo

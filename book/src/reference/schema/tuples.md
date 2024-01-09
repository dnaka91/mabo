# Tuples

Tuples allow for the definition of a set of types, without having to define an explicit struct for it. These do not have any associated [ID](index.md#identifiers), meaning the order of declaration matters, and any modification to the type definition is generally incompatible.

The minimum amount of types in a tuple are **2** and the maximum are **12**. Reasons for this choice are:

- A tuple with 0 types is empty and can't carry any data.
- Having only 1 type is equivalent to the contained type itself and is redundant.
- Up to 12 types seems arbitrary, but having more than 3 or 4 types often calls for defining an explicit struct with field names for better clarity.

## Schema

In the schema, tuples are declared with parenthesis `(` and `)`, each type separated by a comma `,`, forming a definition like `(T1, T2, TN...)`.

<<<tuples/basic.mabo

## Languages

::: code-group
<<< tuples/basic.rs#snippet [Rust]
<<< tuples/basic.go#snippet [Go]
<<< tuples/basic.kt#snippet [Kotlin]
<<< tuples/basic.ts#snippet [TypeScript]
<<< tuples/basic.py#snippet [Python]
:::

### Rust

In Rust we have tuples and they look the same as in the schema, minus the ID.

### Go

There is no native support for tuples. Thus, types for tuples should be provided as part of the encompassing library, for up to N types.

### Kotlin

Again, no native support for tuples. The `Map.Entry` interface exists, but seems not to be meant as tuple replacement. Therefore, generic types should be provided together with the library, for up to N types.

### TypeScript

Typescript has support for tuples, defined as `[T1, T2, TN...]`.

### Python

Python has support for tuples, defined as `tuple[T1, T2, TN...]`.

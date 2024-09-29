---
order: 1
---

# Structs

Structs are a short naming for _structures_ and define a series of data elements with their respective type. Those individual elements can be named or unnamed.

In schema they are declared with the `struct` keyword followed by the name and the types contained.

The name must start with an uppercase ASCII character (`A-Z`), and _may_ be followed by zero or more upper- and lowercase ASCII characters and digits (`A-Z`, `a-z`, `0-9`).

Note that acronyms should be written in strict _CamelCase_, meaning `Html` instead of `HTML` or `Api` instead of `API`.

Individual fields in both named and unnamed form are separated by a comma `,`, and it's recommended to even give the last field a trailing comma. This allows for simpler diffs in version control systems.

## Named

The likely most common form is a named struct. Named means that each element is represented as a field with a name to identify it.

To declare the struct as named the content is contained in curly braces `{...}`.

A single field is defined as `name: type @id`, the name, its type and [ID]. Field names must start with a lowercase ASCII character (`a-z`) and _may_ be followed by zero or more lowercase ASCII characters, digits and underscores (`a-z`, `0-9`, `_`).

[ID]: ./#identifiers

### Schema {#named-schema}

Here is a basic named schema with two fields `field1` and `field2`. The first one is a 32-bit unsigned integer and assigned the ID 1. The second one is a 16-bit unsigned integer and assigned the ID 2.

<<< structs/named.mabo

### Languages {#named-lang}

These samples describe how the schema would be defined in each language, when generating the code for it.

::: code-group
<<< structs/named.rs#snippet [Rust]
<<< structs/named.go#snippet [Go]
<<< structs/named.kt#snippet [Kotlin]
<<< structs/named.ts#snippet [TypeScript]
<<< structs/named.py#snippet [Python]
:::

#### Rust {#named-lang-rs}

#### Go {#named-lang-go}

As Go allows to create new instances without declaring a value for each field (they default to the zero value), an additional constructor is created.

#### Kotlin {#named-lang-kt}

#### TypeScript {#named-lang-ts}

An additional constructor is required, to ensure all fields are properly initialized when creating new instances.

#### Python {#named-lang-py}

In Python the `@dataclass` attribute is used to define the fields of a class.

## Unnamed

This variant is very similar to named structs, but in contrast lack a field name. They can be convenient if the data type is rather compact and explicit field names aren't needed. For example a position with the horizontal and vertical offset.

To declare the struct as unnamed the content is contained in parenthesis `(...)`.

A single field is defined as `type @id`, the name and [ID].

### Schema {#unnamed-schema}

<<< structs/unnamed.mabo

### Languages {#unnamed-lang}

These samples describe how the schema would be defined in each language, when generating the code for it.

::: code-group
<<< structs/unnamed.rs#snippet [Rust]
<<< structs/unnamed.go#snippet [Go]
<<< structs/unnamed.kt#snippet [Kotlin]
<<< structs/unnamed.ts#snippet [TypeScript]
<<< structs/unnamed.py#snippet [Python]
:::

## Unit

In addition to the above, a struct can completely omit field definitions. That is call a unit struct and doesn't carry any data. It doesn't take any space in encoded form either.

For most languages, this type doesn't take up any memory either. Creating some vector or list of said type would require zero bytes.

Instead, it's only the type that carries information.

### Schema {#unit-schema}

<<< structs/unit.mabo

### Languages {#unit-lang}

These samples describe how the schema would be defined in each language, when generating the code for it.

::: code-group
<<< structs/unit.rs#snippet [Rust]
<<< structs/unit.go#snippet [Go]
<<< structs/unit.kt#snippet [Kotlin]
<<< structs/unit.ts#snippet [TypeScript]
<<< structs/unit.py#snippet [Python]
:::

## Generics

### Schema {#generics-schema}

<<< structs/generics.mabo

### Languages {#generics-lang}

These samples describe how the schema would be defined in each language, when generating the code for it.

::: code-group
<<< structs/generics.rs#snippet [Rust]
<<< structs/generics.go#snippet [Go]
<<< structs/generics.kt#snippet [Kotlin]
<<< structs/generics.ts#snippet [TypeScript]
<<< structs/generics.py#snippet [Python]
:::

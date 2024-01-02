# Creating schemas

[[toc]]

## Create the first schema

Lets jump right in and discover further details along the way.

In the following sample we define a simple data structure that describes some details about a user. It holds information about a user's name and age.

<<< creating/basic.mabo

Step by step we will disect this schema and explain each part:

1. The `/// ...` parts are comments. Everything following the three slashes is understood as comment and attached to the element it's defined on. Multiple lines can be written by repeating the slashes on each line.
2. Next the `struct User { ... }` element defines a data structure indicated by the keyword `struct`, followed by its name. The curly braces denote start and end of the declaration, which contains the _named_ fields.
3. Each line inside the struct declaration is either a comment (`/// ...`) or a field. The `name: string @1` defines the first field titled `name` and the data type `string`, meaning it can hodl text content. The `@1` describes the unique identifier for the field. This is simply a number but must be unique within each struct.
4. Lastly there is another field declaration `age: u16 @2`, which is a field named `age` with a 16-bit unsigned integer as type and identifier 2.

For Rust developers this might look very familiar. That is, because Mabo is inspired by Rust and will look much alike in many ways. The generated code for the struct definition would actually be almost the same, but with a few visibility modifiers and minus the identifiers.

## Data types

It is important to understand built-in data types, as these are used constantly to define other data structures.

As the name might imply, Mabo tries to have a very strong typing system. Therefore, it has a rather large variety of types available.

A full list of all available data types and more specifics can be found in the [schema reference](../reference/schema/).

### Basic types

The following list contains the most essential data types that are likely to be seen regularly in other schema definitions:

- Signed integers `i8`, `i16`, `i32`, `i64` and `i128`: These are numbers that can be positive or negative and have no fraction. Depending on the programming languages that one might know this might look familiar.

  The `i` describes that the integer is signed, and the number denotes the amount of bits used to represent the integer. The bit size defines the possible range of values that can be represented.

- Unsigned integers `u8`, `u16`, `u32`, `u64` and `u128`: Numbers again, but unsigned, meaning they can only represent positive numbers. Due to how they're stored, it means the maximum possible value is double as large as that of a signed integer.

- Floating point numbers `f32` and `f64`: Numbers with fractions. In some programming languages known as `float`/`double`.

- Strings `string`: Any form of text value. Probably most common throughout programming languages, but they differ often in the encoding chosen.

  They are encoded in UTF-8 and never represent an invalid encoded string (or otherwise a payload is considered invalid or corrupted).

- Bytes `bytes`: Raw bytes of an arbitrary length. These can contain literally anything, images, binaries, structured data, and so on. The interpretation is up to the application.

### Colletions

To bundle multiple values together, they can be put into collections. These all have in common that they can hold zero or more elements of the same type.

The following list describes available collection types. Note that the `T`, `K` and `V` are type parameters, meaning they can be replaced with any other type:

- Vectors `vec<T>`: A list of values, sometimes called a dynamic array or list as well.

- Hash maps `hash_map<K, V>`: Mapping from a keys to values. Each entry is unique and inserting a new entry will take place of any previously existing entry.

- Hash sets `hash_set<T>`: The same as a hash map, but without an associated value. This enforces that all elements are unique, in contrast to a vector.

### Special types

Lastly there are a few special types and not all of them are mentioned here. They have very specific use cases and only the most common ones are shown:

- Optionals `option<T>`: Values that might not always be present. There can be some value or none at all.

- Non-zero `non_zero<T>`: Some type that is guaranteed to not be empty. What empty exactly means, depends on the type itself. For example, an integer might never be zero, or a string might never have zero characters.

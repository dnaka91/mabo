# Attributes

The attributes allow to further apply configuration to elements, while not affecting the schema itself. These rather provide either extra information or customize how language specific code is generated.

For example, they allow to mark elements as _deprecated_ or define how a certain value is represented in code, off the default variant.

<!-- toc -->

## Schema

Attributes can come in 3 forms.

- Unit, just having the name of the attribute itself.
- Singe-value, with a literal assignment (can be optional).
- Multi-value, as an object that can have multiple sub-attributes.

### Unit

```stef
#[deprecated]
struct Sample {}
```

### Single-value

```stef
#[deprecated = "Don't use anymore"]
struct Sample {}
```

### Multi-value

```stef
struct Sample {
    #[validate(min = 1, max = 100)]
    age: u8 @1,
}
```

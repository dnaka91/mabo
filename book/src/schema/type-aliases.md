# Type aliases

<!-- toc -->
<!-- toc:max-level = 2 -->

| Language      | Example            |
| ------------- | ------------------ |
| Schema / Rust | `type A = T;`      |
| Go            | `type A = T`       |
| Kotlin        | `typealias A = T`  |
| TypeScript    | `type A = T;`      |
| Python        | `A: TypeAlias = T` |

## Schema

```stef
{{#include type-aliases/basic.stef}}
```

## Languages

### Rust

```rust
{{#include type-aliases/basic.rs}}
```

### Go

```go
{{#include type-aliases/basic.go:5:}}
```

### Kotlin

```kotlin
{{#include type-aliases/basic.kt:3:}}
```

### TypeScript

```typescript
{{#include type-aliases/basic.ts}}
```

### Python

```python
{{#include type-aliases/basic.py:3:}}
```

# Constants

<!-- toc -->
<!-- toc:max-level = 2 -->

| Language      | Example                         |
| ------------- | ------------------------------- |
| Schema / Rust | `const NAME: T = <literal>;`    |
| Go            | `const Name T = <literal>`      |
| Kotlin        | `const val NAME: T = <literal>` |
| TypeScript    | `const name: T = <literal>;`    |
| Python        | `NAME: T = <literal>`           |

## Schema

```rust,ignore
{{#include constants/basic.stef}}
```

## Languages

### Rust

```rust
{{#include constants/basic.rs}}
```

### Go

```go
{{#include constants/basic.go:7:}}
```

### Kotlin

```kotlin
{{#include constants/basic.kt:3:}}
```

### TypeScript

```typescript
{{#include constants/basic.ts}}
```

### Python

```python
{{#include constants/basic.py}}
```

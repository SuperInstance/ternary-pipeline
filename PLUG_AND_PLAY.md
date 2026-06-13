# PLUG_AND_PLAY — Pipeline

> Composable processing pipelines with ternary state propagation

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-pipeline = { git = "https://github.com/SuperInstance/ternary-pipeline" }
```

Use in your code:

```rust
use ternary_pipeline::{Pipeline, stage};

let result = Pipeline::new()
    .then(stage(|x| x + 1))
    .then(stage(|x| x * 2))
    .execute(5);
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
